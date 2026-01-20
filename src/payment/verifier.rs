// Payment Verification Service
// Verifies cryptocurrency payments and updates payment status

use chrono::Utc;
use rust_decimal::Decimal;
use std::str::FromStr;
use sqlx::{PgPool, Postgres, Transaction};
use tracing::{info, warn, error};

use super::models::{PaymentTransaction, PaymentStatus, CryptoType, BlockchainTransaction};
use super::blockchain_monitor::get_blockchain_monitor;

pub struct PaymentVerifier {
    db_pool: PgPool,
}

impl PaymentVerifier {
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool,
        }
    }

    /// Verify a payment using public payment_id and transaction hash
    /// 
    /// This is the public API method that accepts the payment_id string (e.g., "pay_abc123")
    /// and verifies merchant ownership before delegating to the internal verification method.
    /// 
    /// # Arguments
    /// * `payment_id` - Public-facing payment ID (e.g., "pay_abc123")
    /// * `transaction_hash` - Blockchain transaction hash
    /// * `merchant_id` - ID of the merchant requesting verification
    /// 
    /// # Returns
    /// * `Ok(true)` if payment is confirmed
    /// * `Ok(false)` if payment is pending more confirmations
    /// * `Err` if verification fails
    pub async fn verify_payment(
        &self,
        payment_id: &str,
        transaction_hash: &str,
        merchant_id: i64,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Look up payment by public payment_id
        let payment = sqlx::query!(
            r#"
            SELECT id, merchant_id FROM payment_transactions
            WHERE payment_id = $1
            "#,
            payment_id
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or("Payment not found")?;

        // Verify merchant ownership
        if payment.merchant_id != merchant_id {
            return Err(format!(
                "Payment {} does not belong to merchant {}. Access denied.",
                payment_id, merchant_id
            ).into());
        }

        // Delegate to internal verification method
        self.verify_payment_by_hash(payment.id, transaction_hash, merchant_id).await
    }

    /// Verify a payment using user-provided transaction hash
    /// This is the PRIMARY verification method - prevents duplicate payments
    /// 
    /// # Requirements
    /// * 3.1: Verify transaction hash exists on blockchain
    /// * 3.2: Confirm amount matches expected payment amount
    /// * 3.3: Confirm recipient address matches merchant's wallet
    /// * 3.4: Mark payment as confirmed when sufficient confirmations received
    /// * 3.5: Reject verification if transaction hash is invalid or doesn't match
    /// * 3.7: Update payment status to completed when confirmed
    pub async fn verify_payment_by_hash(
        &self,
        payment_id: i64,
        transaction_hash: &str,
        merchant_id: i64,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        info!("🔍 Verifying payment {} with transaction hash {} for merchant {}", 
            payment_id, transaction_hash, merchant_id);

        // 1. Check if transaction hash is already used by another payment
        let existing_payment = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT id FROM payment_transactions
            WHERE transaction_hash = $1
            AND id != $2
            LIMIT 1
            "#
        )
        .bind(transaction_hash)
        .bind(payment_id)
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(existing_id) = existing_payment {
            return Err(format!(
                "Transaction hash already used for payment #{}. Each transaction can only be used once.",
                existing_id
            ).into());
        }

        // 2. Get payment from database and verify merchant ownership
        let payment = sqlx::query_as::<_, PaymentTransaction>(
            r#"
            SELECT * FROM payment_transactions
            WHERE id = $1
            "#
        )
        .bind(payment_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or("Payment not found")?;

        // 3. Verify merchant ownership
        if payment.merchant_id != merchant_id {
            return Err(format!(
                "Payment {} does not belong to merchant {}. Access denied.",
                payment_id, merchant_id
            ).into());
        }

        // 4. Check if payment is already confirmed
        if payment.status == "CONFIRMED" {
            info!("✅ Payment {} already confirmed", payment_id);
            return Ok(true);
        }

        // 5. Check if payment has expired
        if payment.expires_at < Utc::now() {
            self.mark_payment_failed(payment_id, "Payment expired").await?;
            return Err("Payment has expired. Please create a new payment request.".into());
        }

        // 6. Fetch blockchain transaction using the provided hash
        // Parse crypto type from string
        let crypto_type = match payment.crypto_type.as_str() {
            "SOL" => CryptoType::Sol,
            "USDT" if payment.network == "BEP20" => CryptoType::UsdtBep20,
            "USDT" if payment.network == "ARBITRUM" => CryptoType::UsdtArbitrum,
            "USDT" if payment.network == "SOLANA_SPL" => CryptoType::UsdtSpl,
            "USDT" if payment.network == "POLYGON" => CryptoType::UsdtPolygon,
            _ => {
                return Err(format!("Unsupported crypto type: {}", payment.crypto_type).into());
            }
        };

        // Get appropriate blockchain monitor for this crypto type
        let monitor = get_blockchain_monitor(&crypto_type);

        // Fetch transaction from blockchain (Requirement 3.1)
        let blockchain_tx = monitor
            .get_transaction_details(transaction_hash)
            .await
            .map_err(|e| format!("Failed to fetch transaction from {}: {}", monitor.blockchain_name(), e))?;

        // 7. Verify transaction details match payment (Requirements 3.2, 3.3, 3.5)
        if !self.validate_transaction(&payment, &blockchain_tx)? {
            self.mark_payment_failed(payment_id, "Transaction validation failed").await?;
            return Err("Transaction validation failed: amount or address mismatch".into());
        }

        // 8. Update payment with transaction details
        sqlx::query(
            r#"
            UPDATE payment_transactions
            SET transaction_hash = $1,
                from_address = $2,
                confirmations = $3,
                block_number = $4,
                status = CASE
                    WHEN $3 >= required_confirmations THEN 'CONFIRMED'
                    ELSE 'CONFIRMING'
                END
            WHERE id = $5
            "#
        )
        .bind(transaction_hash)
        .bind(&blockchain_tx.from_address)
        .bind(blockchain_tx.confirmations as i32)
        .bind(blockchain_tx.block_number.map(|n| n as i64))
        .bind(payment_id)
        .execute(&self.db_pool)
        .await?;

        // 9. If enough confirmations, confirm the payment (Requirements 3.4, 3.7)
        if (blockchain_tx.confirmations as i32) >= payment.required_confirmations {
            self.confirm_payment(payment_id, merchant_id).await?;
            info!("✅ Payment {} confirmed with {} confirmations for merchant {}!",
                payment_id, blockchain_tx.confirmations, merchant_id);
            return Ok(true);
        } else {
            info!("⏳ Payment {} confirming ({}/{} confirmations)",
                payment_id,
                blockchain_tx.confirmations,
                payment.required_confirmations
            );
            return Ok(false);
        }
    }

    /// Validate blockchain transaction matches payment request
    /// 
    /// # Requirements
    /// * 3.2: Confirm amount matches expected payment amount (with 0.1% tolerance)
    /// * 3.3: Confirm recipient address matches merchant's wallet
    /// * 3.5: Reject verification if transaction doesn't match payment details
    fn validate_transaction(
        &self,
        payment: &PaymentTransaction,
        blockchain_tx: &BlockchainTransaction,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Check transaction was successful
        if !blockchain_tx.success {
            warn!("Transaction {} failed on blockchain", blockchain_tx.hash);
            return Ok(false);
        }

        // Check recipient address matches merchant's wallet (Requirement 3.3)
        if blockchain_tx.to_address.to_lowercase() != payment.to_address.to_lowercase() {
            warn!("Recipient address mismatch: expected merchant wallet {}, got {}",
                payment.to_address,
                blockchain_tx.to_address
            );
            return Ok(false);
        }

        // Check amount matches (allow 0.1% tolerance for fees) (Requirement 3.2)
        let amount_diff = (blockchain_tx.amount - payment.amount).abs();
        let tolerance = payment.amount * Decimal::from_str("0.001")?; // 0.1%

        if amount_diff > tolerance {
            warn!("Amount mismatch: expected {}, got {} (diff: {})",
                payment.amount,
                blockchain_tx.amount,
                amount_diff
            );
            return Ok(false);
        }

        Ok(true)
    }

    /// Mark payment as confirmed and trigger webhooks
    /// 
    /// # Requirements
    /// * 3.7: Update payment status to completed when confirmed
    /// * 4.2: Send webhook notification when payment status changes to confirmed
    /// * 6.3: Record fee amounts when payment is confirmed
    async fn confirm_payment(
        &self,
        payment_id: i64,
        merchant_id: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Fetch payment to get fee information for logging (Requirement 6.3)
        let payment = sqlx::query!(
            r#"
            SELECT fee_amount, fee_amount_usd, fee_percentage, amount, amount_usd
            FROM payment_transactions
            WHERE id = $1
            "#,
            payment_id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        // Update payment status to CONFIRMED (Requirement 3.7)
        // Fee amounts are already stored from payment creation and remain in the record
        sqlx::query(
            r#"
            UPDATE payment_transactions
            SET status = 'CONFIRMED',
                confirmed_at = $1
            WHERE id = $2
            "#
        )
        .bind(Utc::now())
        .bind(payment_id)
        .execute(&self.db_pool)
        .await?;

        // Log fee recording for audit trail (Requirement 6.3)
        info!(
            "✅ Payment {} confirmed for merchant {} - Fee recorded: {} crypto (${}) at {}% rate",
            payment_id,
            merchant_id,
            payment.fee_amount,
            payment.fee_amount_usd,
            payment.fee_percentage
        );

        // TODO: Trigger webhook notification (Requirement 4.2)
        // This will be implemented when webhook service is complete
        // For now, we log that a webhook should be sent
        info!("📢 Webhook should be triggered for payment {} confirmation (merchant {})", 
            payment_id, merchant_id);

        Ok(())
    }

    /// Mark payment as failed
    async fn mark_payment_failed(
        &self,
        payment_id: i64,
        reason: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            r#"
            UPDATE payment_transactions
            SET status = 'FAILED'
            WHERE id = $1
            "#
        )
        .bind(payment_id)
        .execute(&self.db_pool)
        .await?;

        warn!("❌ Payment {} marked as failed: {}", payment_id, reason);
        Ok(())
    }
}
