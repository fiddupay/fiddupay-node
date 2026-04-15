// Payment Verification Service
// Verifies cryptocurrency payments and updates payment status

use chrono::Utc;
use rust_decimal::Decimal;
use serde_json::json;
use sqlx::{PgPool, Row};
use std::str::FromStr;
use tracing::{error, info, warn};

use super::blockchain_monitor::get_blockchain_monitor;
use super::models::{BlockchainTransaction, CryptoType, PaymentStatus, PaymentTransaction};
use crate::services::webhook_service::WebhookService;

#[derive(Clone)]
pub struct PaymentVerifier {
    db_pool: PgPool,
    webhook_service: WebhookService,
    price_service: std::sync::Arc<crate::services::price_service::PriceService>,
    config: crate::config::Config,
    redis_client: redis::Client,
    notification_service:
        std::sync::Arc<crate::services::notification_service::NotificationService>,
}

impl PaymentVerifier {
    pub fn new(
        db_pool: PgPool,
        webhook_service: WebhookService,
        price_service: std::sync::Arc<crate::services::price_service::PriceService>,
        config: crate::config::Config,
        redis_client: redis::Client,
        notification_service: std::sync::Arc<
            crate::services::notification_service::NotificationService,
        >,
    ) -> Self {
        Self {
            db_pool,
            webhook_service,
            price_service,
            config,
            redis_client,
            notification_service,
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
        let payment_row = sqlx::query(
            r#"
            SELECT id, merchant_id FROM payment_transactions
            WHERE payment_id = $1
            "#,
        )
        .bind(payment_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or("Payment not found")?;

        // Verify merchant ownership
        let payment_db_id: i64 = payment_row.get("id");
        let payment_merchant_id: i64 = payment_row.get("merchant_id");
        if payment_merchant_id != merchant_id {
            return Err(format!(
                "Payment {} does not belong to merchant {}. Access denied.",
                payment_id, merchant_id
            )
            .into());
        }

        // Delegate to internal verification method
        self.verify_payment_by_hash(payment_db_id, transaction_hash, merchant_id)
            .await
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
        info!(
            " Verifying payment {} with transaction hash {} for merchant {}",
            payment_id, transaction_hash, merchant_id
        );

        // 1. Check if transaction hash is already used by another payment
        let existing_payment = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT id FROM payment_transactions
            WHERE transaction_hash = $1
            AND id != $2
            LIMIT 1
            "#,
        )
        .bind(transaction_hash)
        .bind(payment_id)
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(existing_id) = existing_payment {
            let err_msg = format!(
                "Transaction hash already used for payment #{}. Each transaction can only be used once.",
                existing_id
            );
            warn!(
                "[VERIFY-HEARTBEAT] Payment {} | BLOCKED: {}",
                payment_id, err_msg
            );
            return Err(err_msg.into());
        }
        info!(
            "[VERIFY-HEARTBEAT] Payment {} | STEP 1: Uniqueness check passed",
            payment_id
        );

        // 2. Get payment from database and verify merchant ownership
        let payment_res: Result<Option<crate::models::payment::Payment>, sqlx::Error> =
            sqlx::query_as(
                r#"
            SELECT id, payment_id, merchant_id, amount, amount_usd, crypto_type, 
                   network, status, to_address, from_address, created_at, expires_at, confirmed_at, 
                   confirmations, required_confirmations, description, metadata, 
                   transaction_hash, webhook_url, fee_percentage, fee_amount, 
                   fee_amount_usd, user_id, subscription_id, block_number, 
                   partial_payments_enabled, total_paid, remaining_balance, is_non_custodial,
                   last_verification_at, sandbox_mode
            FROM payment_transactions
            WHERE id = $1
            "#,
            )
            .bind(payment_id)
            .fetch_optional(&self.db_pool)
            .await;

        let payment = payment_res?.ok_or_else(|| {
            error!(
                "[VERIFY-HEARTBEAT] Payment {} | ERROR: Payment not found in DB",
                payment_id
            );
            "Payment not found"
        })?;
        info!(
            "[VERIFY-HEARTBEAT] Payment {} | STEP 2: DB record retrieved",
            payment_id
        );

        // 3. Verify merchant ownership
        if payment.merchant_id != merchant_id {
            return Err(format!(
                "Payment {} does not belong to merchant {}. Access denied.",
                payment_id, merchant_id
            )
            .into());
        }

        // 4. Check if payment is already confirmed
        if payment.status == "CONFIRMED" {
            info!(
                "[VERIFY-HEARTBEAT] Payment {} | SKIP: Already confirmed",
                payment_id
            );
            return Ok(true);
        }

        // 5. Check if payment has expired
        if payment.expires_at < Utc::now() {
            self.mark_payment_failed(payment_id, "Payment expired")
                .await?;
            return Err("Payment has expired. Please create a new payment request.".into());
        }

        // 6. Fetch blockchain transaction using the provided hash
        let crypto_type_str = payment.crypto_type.as_ref().ok_or_else(|| {
            error!(
                "[VERIFY-HEARTBEAT] Payment {} | ERROR: Crypto type missing",
                payment_id
            );
            "Currency selection required before verification"
        })?;
        let crypto_type = CryptoType::from_string(crypto_type_str)?;
        info!(
            "[VERIFY-HEARTBEAT] Payment {} | STEP 3: Starting blockchain lookup for {}",
            payment_id, transaction_hash
        );

        // Get appropriate blockchain monitor for this crypto type using payment's sandbox mode
        let monitor =
            get_blockchain_monitor(&crypto_type, self.config.clone(), payment.sandbox_mode);

        // Fetch transaction from blockchain (Requirement 3.1)
        let blockchain_tx = monitor
            .get_transaction_details(transaction_hash, payment.to_address.as_deref())
            .await
            .map_err(|e| {
                error!(
                    "[VERIFY-HEARTBEAT] Payment {} | ERROR: Blockchain fetch failed: {}",
                    payment_id, e
                );
                format!(
                    "Failed to fetch transaction from {}: {}",
                    monitor.blockchain_name(),
                    e
                )
            })?;
        info!(
            "[VERIFY-HEARTBEAT] Payment {} | STEP 4: Transaction found on {}",
            payment_id,
            monitor.blockchain_name()
        );

        // 7. Verify transaction details match payment (Requirements 3.2, 3.3, 3.5)
        if !self.validate_transaction(&payment, &blockchain_tx)? {
            warn!(
                "[VERIFY-HEARTBEAT] Payment {} | FAILED: validation mismatch",
                payment_id
            );
            self.mark_payment_failed(payment_id, "Transaction validation failed")
                .await?;
            return Err("Transaction validation failed: amount or address mismatch".into());
        }
        info!(
            "[VERIFY-HEARTBEAT] Payment {} | STEP 5: Validation successful (Amount/Address match)",
            payment_id
        );

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
            "#,
        )
        .bind(transaction_hash)
        .bind(&blockchain_tx.from_address)
        .bind(blockchain_tx.confirmations as i32)
        .bind(blockchain_tx.block_number.map(|n| n as i64))
        .bind(payment_id)
        .execute(&self.db_pool)
        .await
        .map_err(|e| {
            error!(
                "[VERIFY-HEARTBEAT] Payment {} | ERROR: DB update failed: {}",
                payment_id, e
            );
            e
        })?;

        info!(
            "[VERIFY-HEARTBEAT] Payment {} | STEP 6: DB updated (status={}, confs={})",
            payment_id,
            if (blockchain_tx.confirmations as i32) >= payment.required_confirmations.unwrap_or(1) {
                "CONFIRMED"
            } else {
                "CONFIRMING"
            },
            blockchain_tx.confirmations
        );

        // 9. If enough confirmations, confirm the payment (Requirements 3.4, 3.7)
        if (blockchain_tx.confirmations as i32) >= payment.required_confirmations.unwrap_or(1) {
            self.confirm_payment(payment_id, merchant_id).await?;
            info!(
                " Payment {} confirmed with {} confirmations for merchant {}!",
                payment_id, blockchain_tx.confirmations, merchant_id
            );
            return Ok(true);
        } else {
            info!(
                "⏳ Payment {} confirming ({}/{} confirmations)",
                payment_id,
                blockchain_tx.confirmations,
                payment.required_confirmations.unwrap_or(1)
            );
            return Ok(false);
        }
    }

    /// Verify payment by scanning address for new transactions
    /// Used for automated detection on the payment page without background monitoring
    pub async fn verify_payment_by_address(
        &self,
        payment_id: &str,
        merchant_id: i64,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Look up payment
        let payment = sqlx::query_as::<_, PaymentTransaction>(
            r#"
            SELECT * FROM payment_transactions
            WHERE payment_id = $1
            "#,
        )
        .bind(payment_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or("Payment not found")?;

        if payment.merchant_id != merchant_id {
            return Err("Access denied".into());
        }

        if payment.status == "CONFIRMED"
            || payment.status == "FAILED"
            || payment.status == "CANCELLED"
        {
            return Ok(true);
        }

        // Verification Cooldown: Prevent redundant scans if triggered recently (e.g., 20s)
        if let Some(last_v) = payment.last_verification_at {
            let elapsed = Utc::now() - last_v;
            if elapsed < chrono::Duration::seconds(5) {
                tracing::debug!(
                    "Skipping verification for payment {}: cooldown active ({}s elapsed)",
                    payment_id,
                    elapsed.num_seconds()
                );
                return Ok(false);
            }
        }

        if payment.to_address.is_none() || payment.amount.is_none() || payment.crypto_type.is_none()
        {
            return Ok(false); // Not ready for verification
        }

        // Update last_verification_at before starting the scan to prevent concurrent triggers
        sqlx::query("UPDATE payment_transactions SET last_verification_at = $1 WHERE id = $2")
            .bind(Utc::now())
            .bind(payment.id)
            .execute(&self.db_pool)
            .await?;

        let crypto_type_str = payment.crypto_type.as_ref().unwrap();
        let crypto_type = CryptoType::from_string(crypto_type_str)?;
        let monitor =
            get_blockchain_monitor(&crypto_type, self.config.clone(), payment.sandbox_mode);
        let address = payment.to_address.as_ref().unwrap();

        // Get recent transactions for the address
        // Check last 20 transactions to find a match, filtering by payment creation time
        let transactions = monitor
            .get_transactions_to_address(address, 20, Some(payment.created_at))
            .await?;

        for tx in transactions {
            // Check if this transaction is already linked to another payment
            // (Unless it's this payment, which shouldn't happen if status is pending)
            let existing_payment = sqlx::query_scalar::<_, i64>(
                "SELECT id FROM payment_transactions WHERE transaction_hash = $1 AND id != $2",
            )
            .bind(&tx.hash)
            .bind(payment.id)
            .fetch_optional(&self.db_pool)
            .await?;

            if existing_payment.is_some() {
                continue;
            }

            // Validate against payment details
            if self.validate_transaction(&payment, &tx)? {
                // Found a match! Process it.
                // We can reuse the verify_payment_by_hash logic or duplicate the update logic here.
                // passing payment.id (which is i64) as required by verify_payment_by_hash
                return self
                    .verify_payment_by_hash(payment.id, &tx.hash, merchant_id)
                    .await;
            }
        }

        Ok(false)
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

        // Check token mint/address (Requirement 3.2 extension: Token validation)
        let crypto_type_str = payment.crypto_type.as_ref().ok_or("Crypto type missing")?;
        let crypto_type = CryptoType::from_string(crypto_type_str)?;
        let expected_token = crypto_type.token_address();

        if let Some(expected_addr) = expected_token {
            match &blockchain_tx.token_mint {
                Some(actual_mint) => {
                    // Normalize and compare
                    let actual_norm = actual_mint.trim().to_lowercase();
                    let expected_norm = expected_addr.trim().to_lowercase();

                    if actual_norm != expected_norm {
                        warn!("[VERIFY-VALIDATION] Payment {} | FAILED: Token mismatch: expected {}, got {}", 
                            payment.payment_id, expected_addr, actual_mint);
                        return Ok(false);
                    }
                }
                None => {
                    warn!("[VERIFY-VALIDATION] Payment {} | FAILED: Expected token {}, but transaction is native", 
                        payment.payment_id, expected_addr);
                    return Ok(false);
                }
            }
        } else {
            // Expected native payment - ensure blockchain_tx has no token_mint
            if blockchain_tx.token_mint.is_some() {
                warn!("[VERIFY-VALIDATION] Payment {} | FAILED: Expected native payment, but transaction is token transfer", 
                    payment.payment_id);
                return Ok(false);
            }
        }

        // Check recipient address matches merchant's wallet (Requirement 3.3)
        let payment_to_address = payment
            .to_address
            .as_ref()
            .ok_or("Merchant address missing")?;

        let addresses_match = if payment
            .network
            .as_ref()
            .map(|n| n.to_lowercase().contains("solana"))
            .unwrap_or(false)
        {
            // Solana addresses are case-sensitive (Base58)
            blockchain_tx.to_address.trim() == payment_to_address.trim()
        } else {
            // Ethereum/EVM addresses are case-insensitive
            blockchain_tx.to_address.trim().to_lowercase()
                == payment_to_address.trim().to_lowercase()
        };

        if !addresses_match {
            tracing::debug!(
                "[VERIFY-VALIDATION] Payment {} | FAILED: Recipient address mismatch",
                payment.payment_id
            );
            return Ok(false);
        }

        // Check timestamp (Requirement 3.8: Replay Protection)
        // Transaction must have occurred after the payment was created
        if let Some(tx_time) = blockchain_tx.timestamp {
            // Allow a small buffer (e.g., 60 seconds) for clock skew, though normally tx_time must be > created_at
            if tx_time < payment.created_at - chrono::Duration::seconds(60) {
                tracing::debug!(
                    "[VERIFY-VALIDATION] Payment {} | FAILED: Timestamp mismatch (Replay attack?)",
                    payment.payment_id
                );
                return Ok(false);
            }
        }

        // Check amount matches (allow 0.1% tolerance for fees) (Requirement 3.2)
        let payment_amount = payment.amount.ok_or("Payment amount missing")?;
        let amount_diff = (blockchain_tx.amount - payment_amount).abs();
        let tolerance = payment_amount * Decimal::from_str("0.001")?; // 0.1%

        if amount_diff > tolerance {
            tracing::debug!(
                "[VERIFY-VALIDATION] Payment {} | FAILED: Amount mismatch",
                payment.payment_id
            );
            return Ok(false);
        }

        info!(
            "✅ Transaction validation successful for {}",
            blockchain_tx.hash
        );
        Ok(true)
    }

    /// Mark payment as confirmed and trigger webhooks
    /// # Requirements
    /// * 3.7: Update payment status to completed when confirmed
    /// * 4.2: Send webhook notification when payment status changes to confirmed
    /// * 6.3: Record fee amounts when payment is confirmed
    async fn confirm_payment(
        &self,
        payment_id: i64,
        merchant_id: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut tx = self.db_pool.begin().await?;

        // Fetch payment to get fee information and sandbox_mode (Requirement 6.3)
        let payment_row = sqlx::query(
            r#"
            SELECT payment_id as public_id, fee_amount, fee_amount_usd, fee_percentage, amount, amount_usd, crypto_type, sandbox_mode, transaction_hash
            FROM payment_transactions
            WHERE id = $1
            FOR UPDATE
            "#
        )
        .bind(payment_id)
        .fetch_one(&mut *tx)
        .await?;

        use sqlx::Row;
        struct ConfirmPaymentData {
            public_id: String,
            fee_amount: Option<Decimal>,
            fee_amount_usd: Decimal,
            fee_percentage: Decimal,
            amount: Option<Decimal>,
            amount_usd: Decimal,
            crypto_type: Option<String>,
            sandbox_mode: bool,
            transaction_hash: Option<String>,
        }
        let payment = ConfirmPaymentData {
            public_id: payment_row.get("public_id"),
            fee_amount: payment_row.get("fee_amount"),
            fee_amount_usd: payment_row.get("fee_amount_usd"),
            fee_percentage: payment_row.get("fee_percentage"),
            amount: payment_row.get("amount"),
            amount_usd: payment_row.get("amount_usd"),
            crypto_type: payment_row.get("crypto_type"),
            sandbox_mode: payment_row.get("sandbox_mode"),
            transaction_hash: payment_row.get("transaction_hash"),
        };
        sqlx::query(
            r#"
            UPDATE payment_transactions
            SET status = 'CONFIRMED',
                confirmed_at = $1
            WHERE id = $2
            "#,
        )
        .bind(Utc::now())
        .bind(payment_id)
        .execute(&mut *tx)
        .await?;

        // Credit merchant balance (net amount = payment amount - platform fee)
        let gross_amount = payment.amount.unwrap_or(Decimal::ZERO);
        let fee_amount = payment.fee_amount.unwrap_or(Decimal::ZERO);
        let net_amount = gross_amount - fee_amount;
        let crypto_type_str = payment
            .crypto_type
            .clone()
            .unwrap_or_else(|| "UNKNOWN".to_string());
        let sandbox_mode = payment.sandbox_mode;

        if net_amount > Decimal::ZERO {
            sqlx::query(
                r#"
                INSERT INTO merchant_balances (merchant_id, crypto_type, available_balance, reserved_balance, last_updated, sandbox_mode)
                VALUES ($1, $2, $3, 0, NOW(), $4)
                ON CONFLICT (merchant_id, crypto_type, sandbox_mode)
                DO UPDATE SET
                    available_balance = merchant_balances.available_balance + $3,
                    last_updated = NOW()
                "#
            )
            .bind(merchant_id)
            .bind(&crypto_type_str)
            .bind(net_amount)
            .bind(sandbox_mode)
            .execute(&mut *tx)
            .await?;

            info!(
                "💰 Merchant {} balance credited: {} {} (gross: {}, fee: {}, sandbox: {})",
                merchant_id, net_amount, crypto_type_str, gross_amount, fee_amount, sandbox_mode
            );
        }

        // Commit transaction BEFORE triggering side-effects like webhooks
        tx.commit().await?;

        // Log fee recording for audit trail (Requirement 6.3)
        info!(
            " Payment {} confirmed for merchant {} - Fee recorded: {} {} (est. ${}) at {}% rate",
            payment_id,
            merchant_id,
            payment.fee_amount.unwrap_or(Decimal::ZERO),
            crypto_type_str,
            payment.fee_amount_usd,
            payment.fee_percentage
        );

        // Trigger webhook notification
        let webhook_payload = crate::models::webhook::WebhookPayload {
            event_type: "payment.confirmed".to_string(),
            payment_id: payment.public_id.clone(),
            merchant_id,
            status: crate::payment::models::PaymentStatus::Confirmed,
            amount: payment.amount.clone().unwrap_or_default(),
            crypto_type: payment
                .crypto_type
                .clone()
                .unwrap_or_else(|| "UNKNOWN".to_string()),
            transaction_hash: payment.transaction_hash.clone(),
            customer_external_id: None,
            timestamp: chrono::Utc::now().timestamp(),
        };

        if let Err(e) = self
            .webhook_service
            .queue_webhook(merchant_id, Some(payment_id), webhook_payload)
            .await
        {
            warn!("Failed to queue webhook for payment {}: {}", payment_id, e);
        }

        // 4. Publish to Redis for real-time dashboard notification (LiveDropToast)
        let channel_name = format!("merchant_notifications:{}", merchant_id);
        let notification = json!({
            "event": "customer.deposit",
            "amount": payment.amount.unwrap_or_default().to_string(),
            "amount_usd": payment.amount_usd.to_string(),
            "crypto_type": payment.crypto_type.clone().unwrap_or_else(|| "UNKNOWN".to_string()),
            "payment_id": payment.public_id.clone(),
        });

        if let Ok(mut redis_conn) = self.redis_client.get_multiplexed_async_connection().await {
            use redis::AsyncCommands;
            let _: Result<(), _> = redis_conn
                .publish::<_, _, ()>(channel_name, notification.to_string())
                .await;
        }

        // Platform fee will be collected by the smart fee sweeping background job
        // based on accumulated thresholds and admin settings.
        // The old immediate FeeCollectionService call has been removed.

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
            "#,
        )
        .bind(payment_id)
        .execute(&self.db_pool)
        .await?;

        warn!(" Payment {} marked as failed: {}", payment_id, reason);
        Ok(())
    }

    /// Record a partial payment
    ///
    /// # Requirements
    /// * 20.2: Track total amount paid across multiple transactions
    /// * 20.3: Update remaining balance
    /// * 20.4: Mark payment as completed when total >= required amount
    pub async fn record_partial_payment(
        &self,
        payment_id: i64,
        transaction_hash: &str,
        amount: rust_decimal::Decimal,
        amount_usd: rust_decimal::Decimal,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut tx = self.db_pool.begin().await?;

        // Insert partial payment record
        sqlx::query(
            r#"
            INSERT INTO partial_payments (payment_id, transaction_hash, amount, amount_usd, confirmations, status, created_at)
            VALUES ($1, $2, $3, $4, 0, 'CONFIRMED', $5)
            "#
        )
        .bind(payment_id)
        .bind(transaction_hash)
        .bind(amount)
        .bind(amount_usd)
        .bind(chrono::Utc::now())
        .execute(&mut *tx)
        .await?;

        // Update payment total_paid and remaining_balance
        let payment_row = sqlx::query(
            r#"
            UPDATE payment_transactions
            SET total_paid = total_paid + $1,
                remaining_balance = remaining_balance - $1,
                expires_at = expires_at + INTERVAL '15 minutes'
            WHERE id = $2
            RETURNING amount, total_paid, remaining_balance, merchant_id, crypto_type, amount_usd, public_id, sandbox_mode
            "#
        )
        .bind(amount)
        .bind(payment_id)
        .fetch_one(&mut *tx)
        .await?;

        use sqlx::Row;
        let payment_amount: Option<Decimal> = payment_row.get("amount");
        let total_paid: Option<Decimal> = payment_row.get("total_paid");
        let merchant_id: i64 = payment_row.get("merchant_id");
        let crypto_type_str: String = payment_row.get("crypto_type");
        let public_id: String = payment_row.get("public_id");
        let sandbox_mode: bool = payment_row.get("sandbox_mode");

        // Check if payment is now complete
        let is_complete = if let (Some(amt), Some(paid)) = (payment_amount, total_paid) {
            paid >= amt
        } else {
            false
        };

        if is_complete {
            sqlx::query(
                "UPDATE payment_transactions SET status = 'CONFIRMED', confirmed_at = $1 WHERE id = $2"
            )
            .bind(chrono::Utc::now())
            .bind(payment_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        info!(
            " Partial payment recorded for payment {}: {} (total: {:?}/{:?})",
            payment_id, amount, total_paid, payment_amount
        );

        // PERSISTENT NOTIFICATION: Payment Received
        let _ = self
            .notification_service
            .create_notification(
                merchant_id,
                if is_complete {
                    "✅ Payment Fully Received"
                } else {
                    "💰 Partial Payment Received"
                },
                &format!(
                    "Received {} {} for payment {}. (USD: ${})",
                    amount, crypto_type_str, public_id, amount_usd
                ),
                "success",
                "payment.received",
                sandbox_mode,
            )
            .await;

        // Publish to Redis for real-time dashboard notification
        if let Ok(mut publish_conn) = self.redis_client.get_multiplexed_async_connection().await {
            let notification = serde_json::json!({
                "event": "customer.deposit",
                "amount": amount.to_string(),
                "amount_usd": amount_usd.to_string(),
                "crypto_type": crypto_type_str,
                "payment_id": public_id,
                "is_partial": true
            });
            let channel = format!("merchant_notifications:{}", merchant_id);
            let _: redis::RedisResult<()> = redis::cmd("PUBLISH")
                .arg(&channel)
                .arg(notification.to_string())
                .query_async(&mut publish_conn)
                .await;
        }

        Ok(is_complete)
    }

    /// Verify and credit a static deposit for a customer
    pub async fn verify_customer_deposit(
        &self,
        customer_id: i64,
        transaction_hash: &str,
        merchant_id: i64,
        crypto_str: &str,
        sandbox_mode: bool,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Verifying static deposit for customer {} with hash {}",
            customer_id, transaction_hash
        );

        // 1. Check if transaction hash is already used for customer_transactions (Idempotency)
        let existing_tx = sqlx::query_scalar::<_, i64>(
            "SELECT id FROM customer_transactions WHERE transaction_hash = $1 LIMIT 1",
        )
        .bind(transaction_hash)
        .fetch_optional(&self.db_pool)
        .await?;

        if existing_tx.is_some() {
            info!(
                "Transaction hash {} already processed for customer",
                transaction_hash
            );
            return Ok(true);
        }

        // 2. Fetch customer wallet first to confirm address match
        let customer_wallet_address = sqlx::query_scalar::<_, String>(
            "SELECT address FROM merchant_customer_wallets WHERE customer_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"
        )
        .bind(customer_id)
        .bind(crypto_str)
        .bind(sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or("Customer wallet not found")?;

        // 3. Fetch blockchain details
        let crypto_type = CryptoType::from_string(crypto_str)?;
        let monitor = get_blockchain_monitor(&crypto_type, self.config.clone(), sandbox_mode);
        let blockchain_tx = monitor
            .get_transaction_details(transaction_hash, Some(&customer_wallet_address))
            .await?;

        if !blockchain_tx.success {
            warn!("Transaction {} failed on blockchain", transaction_hash);
            return Ok(false);
        }

        let addresses_match = if crypto_str.to_lowercase().contains("sol") {
            blockchain_tx.to_address.trim() == customer_wallet_address.trim()
        } else {
            blockchain_tx.to_address.trim().to_lowercase()
                == customer_wallet_address.trim().to_lowercase()
        };

        if !addresses_match {
            warn!(
                "Address mismatch for static deposit {}: expected {}, got {}",
                transaction_hash, customer_wallet_address, blockchain_tx.to_address
            );
            return Err("Recipient address mismatch".into());
        }

        // 3.1 Silent Ignore for Merchant Auto-Funding (Gas Station)
        // If this deposit came from the Merchant's own Master Wallet, it's an internal gas fund for a sweep.
        // It should NOT be credited to the customer's balance, nor trigger webhooks.
        let is_from_master = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT id FROM merchant_wallets 
            WHERE merchant_id = $1 AND lower(address) = lower($2) AND sandbox_mode = $3 AND is_active = true
            "#
        )
        .bind(merchant_id)
        .bind(blockchain_tx.from_address.trim())
        .bind(sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await?;

        if is_from_master.is_some() {
            tracing::info!(
                "[VERIFIER] Silently ignoring internal gas auto-fund deposit {} from Merchant {}",
                transaction_hash,
                merchant_id
            );
            return Ok(true); // Return success so the scanner marks it processed and ignores it forever
        }

        // 3.5. Dynamic Asset Detection & Mint Check
        let (actual_crypto, actual_amount) = if let Some(mint) = &blockchain_tx.token_mint {
            // It's a token transfer - resolve the CryptoType
            let detected = CryptoType::from_mint(crypto_type.network(), mint).ok_or_else(|| {
                format!(
                    "Unsupported token mint: {} on network {}",
                    mint,
                    crypto_type.network()
                )
            })?;
            (detected, blockchain_tx.amount)
        } else {
            // It's a native transfer
            let native = crypto_type.get_native_currency();
            (native, blockchain_tx.amount)
        };

        // If the detected crypto differs from the original 'crypto_str' (e.g., USDT sent to a SOL address)
        // we must verify the customer actually has a configured wallet for this target crypto at this address.
        let (final_crypto_type, final_crypto_str) = if actual_crypto.to_string() != crypto_str {
            let actual_crypto_str = actual_crypto.to_string();
            info!("[VERIFY-CUSTOMER-DEPOSIT] Detected different asset: {} (monitored as {}). checking wallet...", 
                actual_crypto_str, crypto_str);

            let has_wallet = sqlx::query_scalar::<_, i64>(
                "SELECT id FROM merchant_customer_wallets 
                 WHERE customer_id = $1 AND crypto_type = $2 AND address = $3 AND sandbox_mode = $4"
            )
            .bind(customer_id)
            .bind(&actual_crypto_str)
            .bind(&customer_wallet_address)
            .bind(sandbox_mode)
            .fetch_optional(&self.db_pool)
            .await?;

            if has_wallet.is_none() {
                warn!("[VERIFY-CUSTOMER-DEPOSIT] FAILED: Customer {} does not have a {} wallet on address {}", 
                    customer_id, actual_crypto_str, customer_wallet_address);
                return Ok(false);
            }
            (actual_crypto, actual_crypto_str)
        } else {
            (actual_crypto, crypto_str.to_string())
        };

        // Fetch merchant's dynamic fee percentage
        let fee_percentage =
            sqlx::query_scalar::<_, Decimal>("SELECT fee_percentage FROM merchants WHERE id = $1")
                .bind(merchant_id)
                .fetch_one(&self.db_pool)
                .await?;

        let fee_amount = (actual_amount * (fee_percentage / Decimal::from(100))).round_dp(8);
        let net_amount = actual_amount - fee_amount;

        // 4. Credit ledger atomically
        let mut tx = self.db_pool.begin().await?;

        // Update balance
        sqlx::query(
            r#"
            INSERT INTO merchant_customer_balances (
                customer_id, merchant_id, crypto_type, available_balance, 
                total_balance, last_updated_at, sandbox_mode
            )
            VALUES ($1, $2, $3, $4, $4, NOW(), $5)
            ON CONFLICT (customer_id, crypto_type, sandbox_mode)
            DO UPDATE SET
                available_balance = merchant_customer_balances.available_balance + EXCLUDED.available_balance,
                total_balance = merchant_customer_balances.total_balance + EXCLUDED.total_balance,
                last_updated_at = NOW()
            "#
        )
        .bind(customer_id)
        .bind(merchant_id)
        .bind(&final_crypto_str)
        .bind(net_amount)
        .bind(sandbox_mode)
        .execute(&mut *tx)
        .await?;

        // Calculate USD amount for ledger
        let crypto_price = self
            .price_service
            .get_price(final_crypto_type.clone())
            .await
            .unwrap_or(1.0);
        let amount_usd = (actual_amount
            * Decimal::from_f64_retain(crypto_price).unwrap_or(Decimal::ONE))
        .round_dp(2);

        // Record Ledger transaction
        sqlx::query(
            r#"
            INSERT INTO customer_transactions (
                customer_id, merchant_id, "type", crypto_type, amount, amount_usd, fee, status, 
                destination_address, transaction_hash, description, sandbox_mode
            )
            VALUES ($1, $2, 'DEPOSIT', $3, $4, $5, $6, 'COMPLETED', $7, $8, 'Static wallet deposit', $9)
            "#
        )
        .bind(customer_id)
        .bind(merchant_id)
        .bind(&final_crypto_str)
        .bind(actual_amount)
        .bind(amount_usd)
        .bind(fee_amount)
        .bind(&customer_wallet_address)
        .bind(transaction_hash)
        .bind(sandbox_mode)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        info!(
            "💰 Static deposit confirmed for customer {}: {} {} (Fee: {} {})",
            customer_id, net_amount, final_crypto_str, fee_amount, final_crypto_str
        );

        // PERSISTENT NOTIFICATION: Static Deposit Received
        let _ = self
            .notification_service
            .create_notification(
                merchant_id,
                "📩 Static Deposit Received",
                &format!(
                    "Customer {} deposited {} {}. (USD: ${})",
                    customer_id, actual_amount, final_crypto_str, amount_usd
                ),
                "success",
                "customer.deposit",
                sandbox_mode,
            )
            .await;

        // Publish to Redis for Merchant Dashboard Toast Notification (Customer Activity)
        if let Ok(mut publish_conn) = self.redis_client.get_multiplexed_async_connection().await {
            let notification = serde_json::json!({
                "event": "customer.deposit",
                "amount": actual_amount.to_string(),
                "net_amount": net_amount.to_string(),
                "fee_amount": fee_amount.to_string(),
                "crypto_type": final_crypto_str,
                "transaction_hash": transaction_hash,
            });
            let channel = format!("merchant_notifications:{}", merchant_id);
            let _: redis::RedisResult<()> = redis::cmd("PUBLISH")
                .arg(&channel)
                .arg(notification.to_string())
                .query_async(&mut publish_conn)
                .await;
        }

        // Fetch customer external_id for webhook identification
        let customer_external_id = sqlx::query_scalar::<_, String>(
            "SELECT external_id FROM merchant_customers WHERE id = $1",
        )
        .bind(customer_id)
        .fetch_optional(&self.db_pool)
        .await?
        .unwrap_or_else(|| "unknown".to_string());

        // 5. Trigger Webhook
        let webhook_payload = crate::models::webhook::WebhookPayload {
            event_type: "customer.deposit".to_string(),
            payment_id: format!(
                "dep_c_{}_{}",
                Utc::now().timestamp_millis(),
                &transaction_hash[0..10]
            ), // Synthetic Payment ID for webhook conformity
            merchant_id,
            status: PaymentStatus::Confirmed,
            amount: actual_amount,
            crypto_type: final_crypto_str.clone(),
            transaction_hash: Some(transaction_hash.to_string()),
            customer_external_id: Some(customer_external_id),
            timestamp: Utc::now().timestamp(),
        };

        if let Err(e) = self
            .webhook_service
            .queue_webhook(merchant_id, None, webhook_payload)
            .await
        {
            warn!(
                "Failed to queue webhook for static deposit {}: {}",
                transaction_hash, e
            );
        }

        Ok(true)
    }

    /// Verify and credit a static deposit for a merchant
    pub async fn verify_merchant_deposit(
        &self,
        merchant_id: i64,
        transaction_hash: &str,
        crypto_str: &str,
        sandbox_mode: bool,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Verifying static deposit for merchant {} with hash {}",
            merchant_id, transaction_hash
        );

        // 1. Check if transaction hash is already used in payment_transactions (Idempotency)
        let existing_tx = sqlx::query_scalar::<_, i64>(
            "SELECT id FROM payment_transactions WHERE transaction_hash = $1 LIMIT 1",
        )
        .bind(transaction_hash)
        .fetch_optional(&self.db_pool)
        .await?;

        if existing_tx.is_some() {
            info!(
                "Transaction hash {} already processed for merchant",
                transaction_hash
            );
            return Ok(true);
        }

        // 2. Fetch merchant static wallet to confirm address match (Withdrawal trigger protection)
        let expected_address = sqlx::query_scalar::<_, String>(
            "SELECT address FROM merchant_wallets WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 AND is_active = true"
        )
        .bind(merchant_id)
        .bind(crypto_str)
        .bind(sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or("Merchant static wallet not found")?;

        // 3. Fetch blockchain details
        let crypto_type = CryptoType::from_string(crypto_str)?;
        let monitor = get_blockchain_monitor(&crypto_type, self.config.clone(), sandbox_mode);
        let blockchain_tx = monitor
            .get_transaction_details(transaction_hash, Some(&expected_address))
            .await?;

        if !blockchain_tx.success {
            warn!("Transaction {} failed on blockchain", transaction_hash);
            return Ok(false);
        }

        let addresses_match = if crypto_str.to_lowercase().contains("sol") {
            blockchain_tx.to_address.trim() == expected_address.trim()
        } else {
            blockchain_tx.to_address.trim().to_lowercase() == expected_address.trim().to_lowercase()
        };

        if !addresses_match {
            warn!("Address mismatch for merchant static deposit {}: expected {}, got {} as destination!", transaction_hash, expected_address, blockchain_tx.to_address);
            return Ok(false); // Gracefully skip withdrawals
        }

        // 3.5. Dynamic Asset Detection & Mint Check
        let (actual_crypto, actual_amount) = if let Some(mint) = &blockchain_tx.token_mint {
            // It's a token transfer - resolve the CryptoType
            let detected = CryptoType::from_mint(crypto_type.network(), mint).ok_or_else(|| {
                format!(
                    "Unsupported token mint: {} on network {}",
                    mint,
                    crypto_type.network()
                )
            })?;
            (detected, blockchain_tx.amount)
        } else {
            // It's a native transfer
            let native = crypto_type.get_native_currency();
            (native, blockchain_tx.amount)
        };

        // If the detected crypto differs from the original 'crypto_str' (e.g., USDT sent to a SOL address)
        // we must verify the merchant actually has a configured wallet for this target crypto at this address.
        let (final_crypto_type, final_crypto_str) = if actual_crypto.to_string() != crypto_str {
            let actual_crypto_str = actual_crypto.to_string();
            info!("[VERIFY-MERCHANT-DEPOSIT] Detected different asset: {} (monitored as {}). checking wallet...", 
                actual_crypto_str, crypto_str);

            let has_wallet = sqlx::query_scalar::<_, i64>(
                "SELECT id FROM merchant_wallets 
                 WHERE merchant_id = $1 AND crypto_type = $2 AND address = $3 AND sandbox_mode = $4 AND is_active = true"
            )
            .bind(merchant_id)
            .bind(&actual_crypto_str)
            .bind(&expected_address)
            .bind(sandbox_mode)
            .fetch_optional(&self.db_pool)
            .await?;

            if has_wallet.is_none() {
                warn!("[VERIFY-MERCHANT-DEPOSIT] FAILED: Merchant {} does not have a {} wallet on address {}", 
                    merchant_id, actual_crypto_str, expected_address);
                return Ok(false);
            }
            (actual_crypto, actual_crypto_str)
        } else {
            (actual_crypto, crypto_str.to_string())
        };

        // Fetch merchant's dynamic fee percentage
        let fee_percentage =
            sqlx::query_scalar::<_, Decimal>("SELECT fee_percentage FROM merchants WHERE id = $1")
                .bind(merchant_id)
                .fetch_one(&self.db_pool)
                .await?;

        // Calculate USD amounts using PriceService
        let crypto_price = self
            .price_service
            .get_price(final_crypto_type.clone())
            .await
            .unwrap_or(1.0);
        let price_decimal = Decimal::from_f64_retain(crypto_price).unwrap_or(Decimal::ONE);
        let amount_usd = (actual_amount * price_decimal).round_dp(2);

        let fee_amount = actual_amount * (fee_percentage / Decimal::from(100));
        let fee_amount_usd = (fee_amount * price_decimal).round_dp(2);
        let net_amount = actual_amount - fee_amount;

        // 3. Credit merchant balance atomically AND Record Payment record
        let mut tx = self.db_pool.begin().await?;

        // Generate synthetic row in payment_transactions with a short ID to prevent UI modal overflow
        let payment_id_str = format!(
            "dep_m_{}_{}",
            Utc::now().timestamp_millis(),
            &transaction_hash[0..10]
        );

        let payment_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO payment_transactions (
                payment_id, merchant_id, crypto_type, amount, amount_usd, to_address, from_address,
                status, expires_at, fee_percentage, fee_amount, fee_amount_usd, network,
                required_confirmations, confirmations, block_number, transaction_hash, description, sandbox_mode, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'CONFIRMED', NOW() + INTERVAL '1 hour', $8, $9, $10, $3, 1, 1, $11, $12, 'Static wallet deposit', $13, NOW())
            RETURNING id
            "#
        )
        .bind(&payment_id_str)
        .bind(merchant_id)
        .bind(&final_crypto_str)
        .bind(actual_amount)
        .bind(amount_usd)
        .bind(&blockchain_tx.to_address)
        .bind(&blockchain_tx.from_address)
        .bind(fee_percentage)
        .bind(fee_amount)
        .bind(fee_amount_usd)
        .bind(blockchain_tx.block_number.map(|n| n as i64))
        .bind(transaction_hash)
        .bind(sandbox_mode)
        .fetch_one(&mut *tx)
        .await?;

        // 4. Update merchant balance with NET amount (amount - fee)
        sqlx::query(
            r#"
            INSERT INTO merchant_balances (merchant_id, crypto_type, available_balance, reserved_balance, last_updated, sandbox_mode)
            VALUES ($1, $2, $3, 0, NOW(), $4)
            ON CONFLICT (merchant_id, crypto_type, sandbox_mode)
            DO UPDATE SET
                available_balance = merchant_balances.available_balance + $3,
                last_updated = NOW()
            "#
        )
        .bind(merchant_id)
        .bind(&final_crypto_str)
        .bind(net_amount)
        .bind(sandbox_mode)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        info!(
            "💰 Static deposit confirmed for merchant {}: {} {}",
            merchant_id, actual_amount, final_crypto_str
        );

        // Publish to Redis for Merchant Dashboard Toast Notification
        if let Ok(mut publish_conn) = self.redis_client.get_multiplexed_async_connection().await {
            let notification = serde_json::json!({
                "event": "merchant.deposit",
                "amount": actual_amount.to_string(),
                "amount_usd": amount_usd.to_string(),
                "crypto_type": final_crypto_str,
                "transaction_hash": transaction_hash,
                "payment_id": payment_id_str,
            });
            let channel = format!("merchant_notifications:{}", merchant_id);
            let _: redis::RedisResult<()> = redis::cmd("PUBLISH")
                .arg(&channel)
                .arg(notification.to_string())
                .query_async(&mut publish_conn)
                .await;
        }

        // 5. Trigger Webhook
        let webhook_payload = crate::models::webhook::WebhookPayload {
            event_type: "merchant.deposit".to_string(),
            payment_id: payment_id_str,
            merchant_id,
            status: PaymentStatus::Confirmed,
            amount: actual_amount,
            crypto_type: final_crypto_str.to_string(),
            transaction_hash: Some(transaction_hash.to_string()),
            customer_external_id: None,
            timestamp: Utc::now().timestamp(),
        };

        if let Err(e) = self
            .webhook_service
            .queue_webhook(merchant_id, Some(payment_id), webhook_payload)
            .await
        {
            warn!(
                "Failed to queue webhook for static merchant deposit {}: {}",
                transaction_hash, e
            );
        }

        Ok(true)
    }
}
