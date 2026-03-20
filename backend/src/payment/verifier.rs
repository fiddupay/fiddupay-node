// Payment Verification Service
// Verifies cryptocurrency payments and updates payment status

use chrono::Utc;
use rust_decimal::Decimal;
use std::str::FromStr;
use sqlx::{PgPool, Postgres, Transaction, Row};
use tracing::{info, warn, error};
use serde_json::json;

use super::models::{PaymentTransaction, PaymentStatus, CryptoType, BlockchainTransaction};
use super::blockchain_monitor::get_blockchain_monitor;
use crate::services::webhook_service::WebhookService;

#[derive(Clone)]
pub struct PaymentVerifier {
    db_pool: PgPool,
    webhook_service: WebhookService,
    config: crate::config::Config,
}

impl PaymentVerifier {
    pub fn new(db_pool: PgPool, webhook_service: WebhookService, config: crate::config::Config) -> Self {
        Self {
            db_pool,
            webhook_service,
            config,
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
            "#
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
            ).into());
        }

        // Delegate to internal verification method
        self.verify_payment_by_hash(payment_db_id, transaction_hash, merchant_id).await
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
        info!(" Verifying payment {} with transaction hash {} for merchant {}", 
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
            let err_msg = format!(
                "Transaction hash already used for payment #{}. Each transaction can only be used once.",
                existing_id
            );
            warn!("[VERIFY-HEARTBEAT] Payment {} | BLOCKED: {}", payment_id, err_msg);
            return Err(err_msg.into());
        }
        info!("[VERIFY-HEARTBEAT] Payment {} | STEP 1: Uniqueness check passed", payment_id);

        // 2. Get payment from database and verify merchant ownership
        let payment_res: Result<Option<crate::models::payment::Payment>, sqlx::Error> = sqlx::query_as(
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
            "#
        )
        .bind(payment_id)
        .fetch_optional(&self.db_pool)
        .await;

        let payment = payment_res?
            .ok_or_else(|| {
                error!("[VERIFY-HEARTBEAT] Payment {} | ERROR: Payment not found in DB", payment_id);
                "Payment not found"
            })?;
        info!("[VERIFY-HEARTBEAT] Payment {} | STEP 2: DB record retrieved", payment_id);

        // 3. Verify merchant ownership
        if payment.merchant_id != merchant_id {
            return Err(format!(
                "Payment {} does not belong to merchant {}. Access denied.",
                payment_id, merchant_id
            ).into());
        }

        // 4. Check if payment is already confirmed
        if payment.status == "CONFIRMED" {
            info!("[VERIFY-HEARTBEAT] Payment {} | SKIP: Already confirmed", payment_id);
            return Ok(true);
        }

        // 5. Check if payment has expired
        if payment.expires_at < Utc::now() {
            self.mark_payment_failed(payment_id, "Payment expired").await?;
            return Err("Payment has expired. Please create a new payment request.".into());
        }

        // 6. Fetch blockchain transaction using the provided hash
        let crypto_type_str = payment.crypto_type.as_ref().ok_or_else(|| {
            error!("[VERIFY-HEARTBEAT] Payment {} | ERROR: Crypto type missing", payment_id);
            "Currency selection required before verification"
        })?;
        let crypto_type = CryptoType::from_string(crypto_type_str)?;
        info!("[VERIFY-HEARTBEAT] Payment {} | STEP 3: Starting blockchain lookup for {}", payment_id, transaction_hash);

        // Get appropriate blockchain monitor for this crypto type using payment's sandbox mode
        let monitor = get_blockchain_monitor(&crypto_type, self.config.clone(), payment.sandbox_mode);

        // Fetch transaction from blockchain (Requirement 3.1)
        let blockchain_tx = monitor
            .get_transaction_details(transaction_hash)
            .await
            .map_err(|e| {
                error!("[VERIFY-HEARTBEAT] Payment {} | ERROR: Blockchain fetch failed: {}", payment_id, e);
                format!("Failed to fetch transaction from {}: {}", monitor.blockchain_name(), e)
            })?;
        info!("[VERIFY-HEARTBEAT] Payment {} | STEP 4: Transaction found on {}", payment_id, monitor.blockchain_name());

        // 7. Verify transaction details match payment (Requirements 3.2, 3.3, 3.5)
        if !self.validate_transaction(&payment, &blockchain_tx)? {
            warn!("[VERIFY-HEARTBEAT] Payment {} | FAILED: validation mismatch", payment_id);
            self.mark_payment_failed(payment_id, "Transaction validation failed").await?;
            return Err("Transaction validation failed: amount or address mismatch".into());
        }
        info!("[VERIFY-HEARTBEAT] Payment {} | STEP 5: Validation successful (Amount/Address match)", payment_id);

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
        .await
        .map_err(|e| {
            error!("[VERIFY-HEARTBEAT] Payment {} | ERROR: DB update failed: {}", payment_id, e);
            e
        })?;
        
        info!("[VERIFY-HEARTBEAT] Payment {} | STEP 6: DB updated (status={}, confs={})", 
            payment_id, 
            if (blockchain_tx.confirmations as i32) >= payment.required_confirmations.unwrap_or(1) { "CONFIRMED" } else { "CONFIRMING" },
            blockchain_tx.confirmations
        );

        // 9. If enough confirmations, confirm the payment (Requirements 3.4, 3.7)
        if (blockchain_tx.confirmations as i32) >= payment.required_confirmations.unwrap_or(1) {
            self.confirm_payment(payment_id, merchant_id).await?;
            info!(" Payment {} confirmed with {} confirmations for merchant {}!",
                payment_id, blockchain_tx.confirmations, merchant_id);
            return Ok(true);
        } else {
            info!("⏳ Payment {} confirming ({}/{} confirmations)",
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
            "#
        )
        .bind(payment_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or("Payment not found")?;

        if payment.merchant_id != merchant_id {
            return Err("Access denied".into());
        }

        if payment.status == "CONFIRMED" || payment.status == "FAILED" || payment.status == "CANCELLED" {
            return Ok(true);
        }

        // Verification Cooldown: Prevent redundant scans if triggered recently (e.g., 20s)
        if let Some(last_v) = payment.last_verification_at {
            let elapsed = Utc::now() - last_v;
            if elapsed < chrono::Duration::seconds(5) {
                tracing::debug!("Skipping verification for payment {}: cooldown active ({}s elapsed)", 
                    payment_id, elapsed.num_seconds());
                return Ok(false);
            }
        }

        if payment.to_address.is_none() || payment.amount.is_none() || payment.crypto_type.is_none() {
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
        let monitor = get_blockchain_monitor(&crypto_type, self.config.clone(), payment.sandbox_mode);
        let address = payment.to_address.as_ref().unwrap();

        // Get recent transactions for the address
        // Check last 20 transactions to find a match, filtering by payment creation time
        let transactions = monitor.get_transactions_to_address(address, 20, Some(payment.created_at)).await?;

        for tx in transactions {
            // Check if this transaction is already linked to another payment
            // (Unless it's this payment, which shouldn't happen if status is pending)
             let existing_payment = sqlx::query_scalar::<_, i64>(
                "SELECT id FROM payment_transactions WHERE transaction_hash = $1 AND id != $2"
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
                 return self.verify_payment_by_hash(payment.id, &tx.hash, merchant_id).await;
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

        // Check recipient address matches merchant's wallet (Requirement 3.3)
        let payment_to_address = payment.to_address.as_ref().ok_or("Merchant address missing")?;
        
        let addresses_match = if payment.network.as_ref().map(|n| n.to_lowercase().contains("solana")).unwrap_or(false) {
            // Solana addresses are case-sensitive (Base58)
            blockchain_tx.to_address.trim() == payment_to_address.trim()
        } else {
            // Ethereum/EVM addresses are case-insensitive
            blockchain_tx.to_address.trim().to_lowercase() == payment_to_address.trim().to_lowercase()
        };

        if !addresses_match {
            tracing::debug!("[VERIFY-VALIDATION] Payment {} | FAILED: Recipient address mismatch: expected merchant wallet '{}', got '{}'",
                payment.payment_id,
                payment_to_address.trim(),
                blockchain_tx.to_address.trim()
            );
            return Ok(false);
        }

        // Check timestamp (Requirement 3.8: Replay Protection)
        // Transaction must have occurred after the payment was created
        if let Some(tx_time) = blockchain_tx.timestamp {
            // Allow a small buffer (e.g., 60 seconds) for clock skew, though normally tx_time must be > created_at
            if tx_time < payment.created_at - chrono::Duration::seconds(60) {
                tracing::debug!("[VERIFY-VALIDATION] Payment {} | FAILED: Timestamp mismatch (Replay attack?). Payment created at {}, but transaction occurred at {}",
                    payment.payment_id,
                    payment.created_at,
                    tx_time
                );
                return Ok(false);
            }
        }

        // Check amount matches (allow 0.1% tolerance for fees) (Requirement 3.2)
        let payment_amount = payment.amount.ok_or("Payment amount missing")?;
        let amount_diff = (blockchain_tx.amount - payment_amount).abs();
        let tolerance = payment_amount * Decimal::from_str("0.001")?; // 0.1%

        if amount_diff > tolerance {
            tracing::debug!("[VERIFY-VALIDATION] Payment {} | FAILED: Amount mismatch: expected {}, got {} (diff: {})",
                payment.payment_id,
                payment_amount,
                blockchain_tx.amount,
                amount_diff
            );
            return Ok(false);
        }

        info!("✅ Transaction validation successful for {}", blockchain_tx.hash);
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
            "#
        )
        .bind(Utc::now())
        .bind(payment_id)
        .execute(&mut *tx)
        .await?;

        // Credit merchant balance (net amount = payment amount - platform fee)
        let gross_amount = payment.amount.unwrap_or(Decimal::ZERO);
        let fee_amount = payment.fee_amount.unwrap_or(Decimal::ZERO);
        let net_amount = gross_amount - fee_amount;
        let crypto_type_str = payment.crypto_type.clone().unwrap_or_else(|| "UNKNOWN".to_string());
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
            amount: payment.amount.unwrap_or_default(),
            crypto_type: payment.crypto_type.unwrap_or_else(|| "UNKNOWN".to_string()),
            transaction_hash: payment.transaction_hash.clone(),
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        if let Err(e) = self.webhook_service.queue_webhook(
            merchant_id,
            Some(payment_id),
            webhook_payload
        ).await {
            warn!("Failed to queue webhook for payment {}: {}", payment_id, e);
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
            "#
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
            RETURNING amount, total_paid, remaining_balance
            "#
        )
        .bind(amount)
        .bind(payment_id)
        .fetch_one(&mut *tx)
        .await?;

        use sqlx::Row;
        let payment_amount: Option<Decimal> = payment_row.get("amount");
        let total_paid: Option<Decimal> = payment_row.get("total_paid");

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

        info!(" Partial payment recorded for payment {}: {} (total: {:?}/{:?})", 
            payment_id, amount, total_paid, payment_amount);

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
        info!("Verifying static deposit for customer {} with hash {}", customer_id, transaction_hash);

        // 1. Check if transaction hash is already used for customer_transactions (Idempotency)
        let existing_tx = sqlx::query_scalar::<_, i64>(
            "SELECT id FROM customer_transactions WHERE transaction_hash = $1 LIMIT 1"
        )
        .bind(transaction_hash)
        .fetch_optional(&self.db_pool)
        .await?;

        if existing_tx.is_some() {
            info!("Transaction hash {} already processed for customer", transaction_hash);
            return Ok(true);
        }

        // 2. Fetch blockchain details
        let crypto_type = CryptoType::from_string(crypto_str)?;
        let monitor = get_blockchain_monitor(&crypto_type, self.config.clone(), sandbox_mode);
        let blockchain_tx = monitor.get_transaction_details(transaction_hash).await?;

        if !blockchain_tx.success {
            warn!("Transaction {} failed on blockchain", transaction_hash);
            return Ok(false);
        }

        // 3. Fetch customer wallet to confirm address match
        let customer_wallet_address = sqlx::query_scalar::<_, String>(
            "SELECT address FROM merchant_customer_wallets WHERE customer_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"
        )
        .bind(customer_id)
        .bind(crypto_str)
        .bind(sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or("Customer wallet not found")?;

        let addresses_match = if crypto_str.to_lowercase().contains("sol") {
            blockchain_tx.to_address.trim() == customer_wallet_address.trim()
        } else {
            blockchain_tx.to_address.trim().to_lowercase() == customer_wallet_address.trim().to_lowercase()
        };

        if !addresses_match {
            warn!("Address mismatch for static deposit {}: expected {}, got {}", transaction_hash, customer_wallet_address, blockchain_tx.to_address);
            return Err("Recipient address mismatch".into());
        }

        // 4. Credit ledger atomically
        let mut tx = self.db_pool.begin().await?;

        // Update balance
        sqlx::query(
            r#"
            UPDATE merchant_customer_balances 
             SET available_balance = available_balance + $1, 
                 total_balance = total_balance + $1, 
                 last_updated_at = NOW() 
             WHERE customer_id = $2 AND crypto_type = $3 AND sandbox_mode = $4
            "#
        )
        .bind(blockchain_tx.amount)
        .bind(customer_id)
        .bind(crypto_str)
        .bind(sandbox_mode)
        .execute(&mut *tx)
        .await?;

        // Record Ledger transaction
        sqlx::query(
            r#"
            INSERT INTO customer_transactions (
                customer_id, merchant_id, type, crypto_type, amount, fee, status, 
                destination_address, transaction_hash, description, sandbox_mode
            )
            VALUES ($1, $2, 'DEPOSIT', $3, $4, 0, 'COMPLETED', $5, $6, 'Static wallet deposit', $7)
            "#
        )
        .bind(customer_id)
        .bind(merchant_id)
        .bind(crypto_str)
        .bind(blockchain_tx.amount)
        .bind(&customer_wallet_address)
        .bind(transaction_hash)
        .bind(sandbox_mode)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        info!("💰 Static deposit confirmed for customer {}: {} {}", customer_id, blockchain_tx.amount, crypto_str);

        // 5. Trigger Webhook
        let webhook_payload = crate::models::webhook::WebhookPayload {
            event_type: "customer.deposit".to_string(),
            payment_id: format!("dep_{}", transaction_hash), // Synthetic Payment ID for webhook conformity
            merchant_id,
            status: PaymentStatus::Confirmed,
            amount: blockchain_tx.amount,
            crypto_type: crypto_str.to_string(),
            transaction_hash: Some(transaction_hash.to_string()),
            timestamp: Utc::now().timestamp(),
        };

        if let Err(e) = self.webhook_service.queue_webhook(
            merchant_id,
            None, 
            webhook_payload
        ).await {
            warn!("Failed to queue webhook for static deposit {}: {}", transaction_hash, e);
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
        info!("Verifying static deposit for merchant {} with hash {}", merchant_id, transaction_hash);

        // 1. Check if transaction hash is already used in payment_transactions (Idempotency)
        let existing_tx = sqlx::query_scalar::<_, i64>(
            "SELECT id FROM payment_transactions WHERE transaction_hash = $1 LIMIT 1"
        )
        .bind(transaction_hash)
        .fetch_optional(&self.db_pool)
        .await?;

        if existing_tx.is_some() {
            info!("Transaction hash {} already processed for merchant", transaction_hash);
            return Ok(true);
        }

        // 2. Fetch blockchain details
        let crypto_type = CryptoType::from_string(crypto_str)?;
        let monitor = get_blockchain_monitor(&crypto_type, self.config.clone(), sandbox_mode);
        let blockchain_tx = monitor.get_transaction_details(transaction_hash).await?;

        if !blockchain_tx.success {
            warn!("Transaction {} failed on blockchain", transaction_hash);
            return Ok(false);
        }

        // Fetch merchant's dynamic fee percentage
        let fee_percentage = sqlx::query_scalar::<_, Decimal>(
            "SELECT fee_percentage FROM merchants WHERE id = $1"
        )
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await?;

        let fee_amount = blockchain_tx.amount * (fee_percentage / Decimal::from(100));
        let net_amount = blockchain_tx.amount - fee_amount;

        // 3. Credit merchant balance atomically AND Record Payment record
        let mut tx = self.db_pool.begin().await?;

        // Generate synthetic row in payment_transactions to represent the deposit for accounting
        let payment_id_str = format!("dep_m_{}", transaction_hash);
        
        let payment_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO payment_transactions (
                payment_id, merchant_id, crypto_type, amount, amount_usd, to_address, from_address,
                status, expires_at, fee_percentage, fee_amount, fee_amount_usd, network,
                required_confirmations, confirmations, block_number, transaction_hash, description, sandbox_mode, created_at
            )
            VALUES ($1, $2, $3, $4, $4, $5, $6, 'CONFIRMED', NOW() + INTERVAL '1 hour', $7, $8, $8, $3, 1, 1, $9, $10, 'Static wallet deposit', $11, NOW())
            RETURNING id
            "#
        )
        .bind(&payment_id_str)
        .bind(merchant_id)
        .bind(crypto_str)
        .bind(blockchain_tx.amount)
        .bind(&blockchain_tx.to_address)
        .bind(&blockchain_tx.from_address)
        .bind(fee_percentage)
        .bind(fee_amount)
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
        .bind(crypto_str)
        .bind(net_amount)
        .bind(sandbox_mode)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        info!("💰 Static deposit confirmed for merchant {}: {} {}", merchant_id, blockchain_tx.amount, crypto_str);

        // 5. Trigger Webhook
        let webhook_payload = crate::models::webhook::WebhookPayload {
            event_type: "merchant.deposit".to_string(),
            payment_id: payment_id_str,
            merchant_id,
            status: PaymentStatus::Confirmed,
            amount: blockchain_tx.amount,
            crypto_type: crypto_str.to_string(),
            transaction_hash: Some(transaction_hash.to_string()),
            timestamp: Utc::now().timestamp(),
        };

        if let Err(e) = self.webhook_service.queue_webhook(
            merchant_id,
            Some(payment_id), 
            webhook_payload
        ).await {
            warn!("Failed to queue webhook for static merchant deposit {}: {}", transaction_hash, e);
        }

        Ok(true)
    }
}
