use crate::error::ServiceError;

use crate::middleware::validation::validate_positive_amount;
use rust_decimal::Decimal;
use serde::Deserialize;
use sqlx::{PgPool, Row};
use validator::Validate;

use crate::config::Config;
use crate::models::withdrawal::Withdrawal;
use crate::payment::models::CryptoType;
use crate::services::price_service::PriceService;
use crate::services::volume_tracking_service::VolumeTrackingService;
use rust_decimal::prelude::FromPrimitive;
use std::sync::Arc;

#[derive(Debug, Deserialize, Validate)]
pub struct WithdrawalRequest {
    pub crypto_type: String,
    #[validate(custom(function = "validate_positive_amount"))]
    pub amount: Decimal,
    #[validate(length(min = 10, max = 200))]
    pub destination_address: String,
    #[validate(length(min = 4, max = 10))]
    pub pin: String,
}

pub struct WithdrawalService {
    db_pool: PgPool,
    price_service: Arc<PriceService>,
    volume_tracking: Arc<VolumeTrackingService>,
    config: Config,
}

impl WithdrawalService {
    pub fn new(
        db_pool: PgPool,
        price_service: Arc<PriceService>,
        volume_tracking: Arc<VolumeTrackingService>,
        config: Config,
    ) -> Self {
        Self {
            db_pool,
            price_service,
            volume_tracking,
            config,
        }
    }

    pub async fn create_withdrawal(
        &self,
        merchant_id: i64,
        request: WithdrawalRequest,
        sandbox_mode: bool,
    ) -> Result<Withdrawal, ServiceError> {
        let withdrawal_id = format!("wd_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));

        // Calculate USD amount (Requirement 20)
        let crypto_type_enum = CryptoType::from_string(&request.crypto_type)?;
        let price = self
            .price_service
            .get_price(crypto_type_enum)
            .await
            .unwrap_or(0.0);
        let amount_usd = request.amount * Decimal::from_f64(price).unwrap_or(Decimal::ZERO);
        let amount_usd = amount_usd.round_dp(2);

        let mut tx = self.db_pool.begin().await?;

        // 0. Fetch Merchant KYC status and limit
        let merchant_row =
            sqlx::query("SELECT kyc_verified, daily_limit_usd FROM merchants WHERE id = $1")
                .bind(merchant_id)
                .fetch_one(&mut *tx)
                .await?;
        let kyc_verified: bool = merchant_row.get("kyc_verified");
        let daily_limit_usd: Option<Decimal> = merchant_row.get("daily_limit_usd");

        let default_limit = if kyc_verified {
            self.config.daily_volume_limit_verified_usd
        } else {
            self.config.daily_volume_limit_non_kyc_usd
        };

        let limit = daily_limit_usd.unwrap_or(default_limit);
        let remaining = self
            .volume_tracking
            .get_remaining_daily_volume(merchant_id, limit)
            .await?
            .unwrap_or(Decimal::ZERO);

        if amount_usd > remaining {
            let status_msg = if kyc_verified {
                "Daily volume limit reached. Contact support to increase your enterprise limit."
                    .to_string()
            } else {
                "Daily volume limit exceeded. Please complete KYC to increase your limit."
                    .to_string()
            };

            return Err(ServiceError::Forbidden(format!(
                "{} Requested: ${}, Remaining: ${}.",
                status_msg, amount_usd, remaining
            )));
        }

        // 1. Lock and check merchant balance (Requirement: Ledger Security)
        let balance_row = sqlx::query(
            "SELECT available_balance FROM merchant_balances 
             WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 
             FOR UPDATE",
        )
        .bind(merchant_id)
        .bind(&request.crypto_type)
        .bind(sandbox_mode)
        .fetch_optional(&mut *tx)
        .await?;

        let available: Decimal = match balance_row {
            Some(row) => row.get("available_balance"),
            None => {
                return Err(ServiceError::ValidationError(
                    "No balance found for this currency".to_string(),
                ))
            }
        };

        if available < request.amount {
            return Err(ServiceError::ValidationError(format!(
                "Insufficient ledger balance. Available: {}, Requested: {}",
                available, request.amount
            )));
        }

        // 2. Deduct balance from ledger immediately (Requirement: Prevent double-spending)
        sqlx::query(
            "UPDATE merchant_balances 
             SET available_balance = available_balance - $1, last_updated = NOW()
             WHERE merchant_id = $2 AND crypto_type = $3 AND sandbox_mode = $4",
        )
        .bind(request.amount)
        .bind(merchant_id)
        .bind(&request.crypto_type)
        .bind(sandbox_mode)
        .execute(&mut *tx)
        .await?;

        // 2. Calculate Fee and Net Amount (Percentage based, e.g. 0.75 = 0.75%)
        let fee_percentage = self.config.withdrawal_fee_percentage;
        let fee = (request.amount * fee_percentage / Decimal::from(100)).round_dp(8);
        let net_amount = request.amount - fee;

        // 3. Create the withdrawal record
        let withdrawal_res: Result<Withdrawal, sqlx::Error> = sqlx::query_as::<_, Withdrawal>(
            r#"
            INSERT INTO withdrawals (
                withdrawal_id, merchant_id, crypto_type, amount, amount_usd, destination_address,
                status, fee, net_amount, created_at, updated_at, sandbox_mode
            )
            VALUES ($1, $2, $3, $4, $5, $6, 'PENDING', $7, $8, NOW(), NOW(), $9)
            RETURNING id, withdrawal_id, merchant_id, crypto_type, 
                     amount, amount_usd, destination_address, status, fee, net_amount, transaction_hash,
                     rejection_reason, requires_approval, approved_by, approved_at, 
                     completed_at, created_at, updated_at
            "#
        )
        .bind(&withdrawal_id)
        .bind(merchant_id)
        .bind(&request.crypto_type)
        .bind(request.amount)
        .bind(amount_usd)
        .bind(&request.destination_address)
        .bind(fee) // fee
        .bind(net_amount) // net_amount
        .bind(sandbox_mode)
        .fetch_one(&mut *tx)
        .await;

        let withdrawal = withdrawal_res?;
        tx.commit().await?;

        tracing::info!(
            "Withdrawal created: id={}, merchant_id={}, crypto_type={}, amount={}, target={}, sandbox={}",
            withdrawal.withdrawal_id,
            withdrawal.merchant_id,
            withdrawal.crypto_type,
            withdrawal.amount,
            withdrawal.destination_address,
            sandbox_mode
        );

        Ok(withdrawal)
    }

    pub async fn get_withdrawal(
        &self,
        merchant_id: i64,
        withdrawal_id: &str,
    ) -> Result<Withdrawal, ServiceError> {
        let withdrawal_res: Result<Option<Withdrawal>, sqlx::Error> = sqlx::query_as::<_, Withdrawal>(
            r#"
            SELECT id, withdrawal_id, merchant_id, crypto_type, 
                   amount, amount_usd, destination_address, status, fee, net_amount, transaction_hash,
                   rejection_reason, requires_approval, approved_by, approved_at, 
                   completed_at, created_at, updated_at
            FROM withdrawals 
            WHERE withdrawal_id = $1 AND merchant_id = $2
            "#
        )
        .bind(withdrawal_id)
        .bind(merchant_id)
        .fetch_optional(&self.db_pool)
        .await;

        let withdrawal = withdrawal_res?;

        withdrawal.ok_or(ServiceError::PaymentNotFound)
    }

    pub async fn list_withdrawals(
        &self,
        merchant_id: i64,
        sandbox_mode: bool,
    ) -> Result<Vec<Withdrawal>, ServiceError> {
        let withdrawals_res: Result<Vec<Withdrawal>, sqlx::Error> = sqlx::query_as::<_, Withdrawal>(
            r#"
            SELECT id, withdrawal_id, merchant_id, crypto_type, 
                   amount, amount_usd, destination_address, status, fee, net_amount, transaction_hash,
                   rejection_reason, requires_approval, approved_by, approved_at, 
                   completed_at, created_at, updated_at
            FROM withdrawals 
            WHERE merchant_id = $1 AND sandbox_mode = $2
            ORDER BY created_at DESC
            "#
        )
        .bind(merchant_id)
        .bind(sandbox_mode)
        .fetch_all(&self.db_pool)
        .await;

        let withdrawals = withdrawals_res?;

        Ok(withdrawals)
    }

    pub async fn cancel_withdrawal(
        &self,
        merchant_id: i64,
        withdrawal_id: &str,
    ) -> Result<Withdrawal, ServiceError> {
        let withdrawal_res: Result<Option<Withdrawal>, sqlx::Error> = sqlx::query_as::<_, Withdrawal>(
            r#"
            UPDATE withdrawals 
            SET status = 'CANCELLED', updated_at = NOW()
            WHERE withdrawal_id = $1 AND merchant_id = $2 AND status = 'PENDING'
            RETURNING id, withdrawal_id, merchant_id, crypto_type, 
                     amount, amount_usd, destination_address, status, fee, net_amount, transaction_hash,
                     rejection_reason, requires_approval, approved_by, approved_at, 
                     completed_at, created_at, updated_at
            "#
        )
        .bind(withdrawal_id)
        .bind(merchant_id)
        .fetch_optional(&self.db_pool)
        .await;

        let withdrawal = withdrawal_res?;

        withdrawal.ok_or(ServiceError::PaymentNotFound)
    }
}
