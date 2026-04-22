// Pay (Ecosystem Interoperability) Service
// Handles universal identifier resolution and payment previews

use crate::error::ServiceError;
use crate::models::merchant::Merchant;
use crate::services::trust_score_service::{TrustScore, TrustScoreService};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use rust_decimal::Decimal;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolvedMerchantProfile {
    pub merchant_id: i64,
    pub business_name: String,
    pub email: String,
    pub username: Option<String>,
    pub pay_id: Option<String>,
    pub kyc_tier: i32,
    pub trust_score: TrustScore,
}

pub struct PayService {
    db_pool: PgPool,
}

impl PayService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Resolve an identifier (Email, Username, or PayID) to a Merchant profile
    pub async fn resolve_merchant(
        &self,
        identifier: &str,
    ) -> Result<ResolvedMerchantProfile, ServiceError> {
        let identifier = identifier.trim();

        // Normalize username identifier (remove @ if present)
        let clean_id = identifier.strip_prefix('@').unwrap_or(identifier);

        // Try to find by PayID, Email, or Username
        let merchant = sqlx::query_as::<_, Merchant>(
            r#"
            SELECT * FROM merchants 
            WHERE (pay_id = $1 OR email = $1 OR username = $1)
            AND is_active = true
            LIMIT 1
            "#,
        )
        .bind(clean_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(ServiceError::Database)?
        .ok_or_else(|| {
            ServiceError::ValidationError("Recipient not found or inactive".to_string())
        })?;

        // Calculate real-time trust score for the preview
        let trust_score =
            TrustScoreService::calculate_score(merchant.kyc_tier, &merchant.social_handles);

        Ok(ResolvedMerchantProfile {
            merchant_id: merchant.id,
            business_name: merchant.business_name,
            email: merchant.email,
            username: merchant.username,
            pay_id: merchant.pay_id,
            kyc_tier: merchant.kyc_tier,
            trust_score,
        })
    }

    /// Execute a zero-fee P2P transfer between two merchant balances
    pub async fn execute_transfer(
        &self,
        sender_id: i64,
        recipient_id: i64,
        crypto_type: &str,
        amount: Decimal,
        is_sandbox: bool,
    ) -> Result<String, ServiceError> {
        if sender_id == recipient_id {
            return Err(ServiceError::ValidationError(
                "Cannot send funds to yourself".into(),
            ));
        }

        if amount <= Decimal::ZERO {
            return Err(ServiceError::ValidationError(
                "Amount must be greater than zero".into(),
            ));
        }

        let mut tx = self.db_pool.begin().await.map_err(ServiceError::Database)?;

        // 1. Check sender's balance and lock it
        let sender_balance_row = sqlx::query(
            r#"
            SELECT available_balance FROM merchant_balances 
            WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 
            FOR UPDATE
            "#,
        )
        .bind(sender_id)
        .bind(crypto_type)
        .bind(is_sandbox)
        .fetch_optional(&mut *tx)
        .await
        .map_err(ServiceError::Database)?
        .ok_or_else(|| {
            ServiceError::ValidationError(format!("No {} wallet found for sender", crypto_type))
        })?;

        use sqlx::Row;
        let available_balance: Decimal = sender_balance_row.get("available_balance");

        if available_balance < amount {
            return Err(ServiceError::ValidationError(
                "Insufficient available balance".into(),
            ));
        }

        // 2. Deduct from sender
        sqlx::query(
            r#"
            UPDATE merchant_balances 
            SET available_balance = available_balance - $1,
                last_updated = NOW()
            WHERE merchant_id = $2 AND crypto_type = $3 AND sandbox_mode = $4
            "#,
        )
        .bind(amount)
        .bind(sender_id)
        .bind(crypto_type)
        .bind(is_sandbox)
        .execute(&mut *tx)
        .await
        .map_err(ServiceError::Database)?;

        // 3. Add to recipient (create balance record if it doesn't exist)
        sqlx::query(
            r#"
            INSERT INTO merchant_balances (merchant_id, crypto_type, available_balance, reserved_balance, sandbox_mode, last_updated)
            VALUES ($1, $2, $3, 0, $4, NOW())
            ON CONFLICT (merchant_id, crypto_type, sandbox_mode) 
            DO UPDATE SET 
                available_balance = merchant_balances.available_balance + $3,
                last_updated = NOW()
            "#,
        )
        .bind(recipient_id)
        .bind(crypto_type)
        .bind(amount)
        .bind(is_sandbox)
        .execute(&mut *tx)
        .await
        .map_err(ServiceError::Database)?;

        // 4. Record the P2P transaction
        let transaction_id = format!(
            "P2P-{}",
            uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
        );

        sqlx::query(
            r#"
            INSERT INTO payment_transactions (
                payment_id, merchant_id, amount, amount_usd, crypto_type, 
                status, sandbox_mode, description, created_at
            )
            VALUES ($1, $2, $3, $3, $4, 'CONFIRMED', $5, $6, NOW())
            "#,
        )
        .bind(&transaction_id)
        .bind(sender_id)
        .bind(amount)
        .bind(crypto_type)
        .bind(is_sandbox)
        .bind(format!("P2P Transfer to RECP-{}", recipient_id))
        .execute(&mut *tx)
        .await
        .map_err(ServiceError::Database)?;

        tx.commit().await.map_err(ServiceError::Database)?;

        Ok(transaction_id)
    }
}
