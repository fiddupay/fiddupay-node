use crate::error::ServiceError;
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use std::sync::Arc;

use crate::payment::models::CryptoType;
use crate::services::blockchain_transaction_sender::BlockchainTransactionSender;
use crate::services::notification_service::NotificationService;
use crate::utils::encryption::Encryption;

pub struct WithdrawalProcessor {
    db_pool: PgPool,
    config: crate::config::Config,
    notification_service: Arc<NotificationService>,
    sender: Arc<BlockchainTransactionSender>,
}

impl WithdrawalProcessor {
    pub fn new(
        db_pool: PgPool,
        config: crate::config::Config,
        notification_service: Arc<NotificationService>,
        sender: Arc<BlockchainTransactionSender>,
    ) -> Self {
        Self {
            db_pool,
            config,
            notification_service,
            sender,
        }
    }

    pub async fn process_withdrawal(&self, withdrawal_id: &str) -> Result<(), ServiceError> {
        // Global Safety Checks
        if !self.config.withdrawal_enabled {
            return Err(ServiceError::ValidationError(
                "Withdrawals are globally disabled in system configuration".to_string(),
            ));
        }
        if self.config.maintenance_mode {
            return Err(ServiceError::ValidationError(
                "System is currently in maintenance mode. Withdrawals are paused.".to_string(),
            ));
        }

        tracing::info!("Starting processing for withdrawal: {}", withdrawal_id);

        // 1. Fetch the withdrawal with FOR UPDATE lock inside a transaction
        let mut tx = self.db_pool.begin().await?;

        let withdrawal = sqlx::query(
            r#"
            SELECT id, withdrawal_id, merchant_id, crypto_type, amount, destination_address, status, transaction_hash, sandbox_mode
            FROM withdrawals 
            WHERE withdrawal_id = $1 FOR UPDATE
            "#
        )
        .bind(withdrawal_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ServiceError::NotFound("Withdrawal not found".to_string()))?;

        let wd_status: String = withdrawal.get("status");
        let wd_merchant_id: i64 = withdrawal.get("merchant_id");
        let wd_crypto_type: String = withdrawal.get("crypto_type");
        let wd_amount: Decimal = withdrawal.get("amount");
        let wd_destination_address: String = withdrawal.get("destination_address");
        let wd_sandbox_mode: bool = withdrawal.get("sandbox_mode");
        let wd_existing_hash: Option<String> = withdrawal.get("transaction_hash");

        // CRITICAL ENFORCEMENT: If it's already COMPLETED or REJECTED, stop immediately.
        if wd_status == "COMPLETED" || wd_status == "REJECTED" || wd_status == "CANCELLED" {
            return Err(ServiceError::ValidationError(format!(
                "Withdrawal already {}",
                wd_status
            )));
        }

        // RECOVERY LOGIC: If it's in PROCESSING but HAS a hash, it was successful on-chain but crashed before DB update.
        if wd_status == "PROCESSING" {
            if let Some(hash) = wd_existing_hash {
                tracing::info!(
                    "RECOVERY: Withdrawal {} was already sent (hash: {}). Completing now.",
                    withdrawal_id,
                    hash
                );
                tx.rollback().await?; // Release lock
                self.finalize_completed_withdrawal(
                    withdrawal_id,
                    &hash,
                    wd_merchant_id,
                    &wd_crypto_type,
                    wd_amount,
                    wd_sandbox_mode,
                )
                .await?;
                return Ok(());
            } else {
                tracing::warn!("Withdrawal {} is in PROCESSING without hash. Proceeding with safe retry attempt.", withdrawal_id);
            }
        }

        if wd_status != "PENDING" && wd_status != "PROCESSING" {
            tracing::warn!(
                "Withdrawal {} requested for processing but has status {}",
                withdrawal_id,
                wd_status
            );
            return Err(ServiceError::ValidationError(
                "Withdrawal already processed or processing".to_string(),
            ));
        }

        // Atomically set state to PROCESSING to lock it against concurrent handlers
        sqlx::query("UPDATE withdrawals SET status = 'PROCESSING', updated_at = NOW() WHERE withdrawal_id = $1")
            .bind(withdrawal_id)
            .execute(&mut *tx)
            .await?;

        // COMMIT state lock before slow on-chain call
        tx.commit().await?;

        tracing::debug!(
            "Withdrawal {}: merchant={}, crypto={}, amount={}, sandbox={}",
            withdrawal_id,
            wd_merchant_id,
            wd_crypto_type,
            wd_amount,
            wd_sandbox_mode
        );

        // 2. Determine Source Wallet
        let customer_tx =
            sqlx::query("SELECT customer_id FROM customer_transactions WHERE reference_id = $1")
                .bind(withdrawal_id)
                .fetch_optional(&self.db_pool)
                .await?;

        let mut wd_fee = Decimal::ZERO;
        if let Ok(fee_val) = withdrawal.try_get::<Decimal, _>("fee") {
            wd_fee = fee_val;
        }

        let encrypted_key_opt: Option<String>;
        let source_address: String;

        let crypto_type_enum = CryptoType::from_string(&wd_crypto_type)?;

        if let Some(c_row) = customer_tx {
            // Sweep from Customer Wallet
            let customer_id: i64 = c_row.get("customer_id");
            let c_wallet = sqlx::query(
                r#"
                SELECT address, encrypted_private_key 
                FROM merchant_customer_wallets 
                WHERE customer_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 AND address != ''
                "#,
            )
            .bind(customer_id)
            .bind(&wd_crypto_type)
            .bind(wd_sandbox_mode)
            .fetch_optional(&self.db_pool)
            .await?
            .ok_or_else(|| {
                ServiceError::NotFound("Customer wallet not found for sweep".to_string())
            })?;

            encrypted_key_opt = c_wallet.get("encrypted_private_key");
            source_address = c_wallet.get("address");

            // GAS AUTO-FUNDING LOGIC (Using Merchant's Master Wallet)
            if !crypto_type_enum.is_native_currency() && wd_fee > Decimal::ZERO {
                let native_crypto_str = match crypto_type_enum {
                    CryptoType::UsdtEth => "ETH",
                    CryptoType::UsdtBep20 | CryptoType::BusdBep20 => "BNB",
                    CryptoType::UsdtPolygon => "MATIC",
                    CryptoType::UsdtArbitrum => "ARB",
                    CryptoType::UsdtSpl => "SOL",
                    _ => &wd_crypto_type,
                };

                tracing::info!("Gas Auto-fund initiated for sweep {}, funding exactly {} {} from MERCHANT'S master wallet", withdrawal_id, wd_fee, native_crypto_str);

                let merchant_master_wallet = sqlx::query(
                    r#"
                    SELECT encrypted_private_key, address 
                    FROM merchant_wallets 
                    WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 AND is_active = true
                    "#
                )
                .bind(wd_merchant_id)
                .bind(native_crypto_str)
                .bind(wd_sandbox_mode)
                .fetch_optional(&self.db_pool)
                .await?;

                if let Some(mw) = merchant_master_wallet {
                    if let Some(m_enc) = mw.get::<Option<String>, _>("encrypted_private_key") {
                        if let Ok(encryption) = Encryption::new() {
                            let decrypt_res: Result<String, String> = encryption.decrypt(&m_enc);
                            if let Ok(m_priv) = decrypt_res {
                                let native_enum = CryptoType::from_string(native_crypto_str)
                                    .unwrap_or(CryptoType::Eth);

                                match self
                                    .sender
                                    .send_transaction(
                                        native_enum,
                                        &m_priv,
                                        &source_address,
                                        wd_fee,
                                        None,
                                        wd_sandbox_mode,
                                    )
                                    .await
                                {
                                    Ok(gas_tx) => {
                                        tracing::info!("Merchant Gas funded successfully, tx: {}. Waiting 15s...", gas_tx);
                                        tokio::time::sleep(tokio::time::Duration::from_secs(15))
                                            .await;
                                    }
                                    Err(e) => {
                                        tracing::error!("Merchant Auto-fund failed securely: {}", e)
                                    }
                                }
                            }
                        }
                    }
                } else {
                    tracing::warn!(
                        "Merchant {} has no {} master wallet configured for auto-funding sweep {}",
                        wd_merchant_id,
                        native_crypto_str,
                        withdrawal_id
                    );
                }
            }
        } else {
            // Standard Merchant Withdrawal
            let m_wallet = sqlx::query(
                r#"
                SELECT address, encrypted_private_key 
                FROM merchant_wallets 
                WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 AND address != ''
                "#,
            )
            .bind(wd_merchant_id)
            .bind(&wd_crypto_type)
            .bind(wd_sandbox_mode)
            .fetch_optional(&self.db_pool)
            .await?
            .ok_or_else(|| {
                ServiceError::NotFound("Merchant wallet not found or not configured".to_string())
            })?;

            encrypted_key_opt = m_wallet.get("encrypted_private_key");
            source_address = m_wallet.get("address");
        }

        let encrypted_key = encrypted_key_opt.ok_or_else(|| {
            ServiceError::ValidationError("Source wallet has no private key available".to_string())
        })?;

        // 3. Decrypt the private key
        let encryption = Encryption::new().map_err(|e: String| ServiceError::Internal(e))?;
        let private_key: String = encryption
            .decrypt(&encrypted_key)
            .map_err(|e: String| ServiceError::Internal(e))?;

        tracing::info!(
            "Blockchain submission started for withdrawal {} from {} to address {}",
            withdrawal_id,
            source_address,
            wd_destination_address
        );

        // 4. Send the transaction on-chain
        if wd_destination_address == "Internal Ledger" {
            tracing::info!("Withdrawal {} is an Internal Ledger movement. Marking as COMPLETED without on-chain tx.", withdrawal_id);
            sqlx::query("UPDATE withdrawals SET status = 'COMPLETED', updated_at = NOW() WHERE withdrawal_id = $1")
                .bind(withdrawal_id)
                .execute(&self.db_pool)
                .await?;
            return Ok(());
        }

        let tx_hash = match self
            .sender
            .send_transaction(
                crypto_type_enum,
                &private_key,
                &wd_destination_address,
                wd_amount,
                None,
                wd_sandbox_mode,
            )
            .await
        {
            Ok(hash) => {
                // IMMEDIATE PERSISTENCE: Save the hash before anything else can fail
                sqlx::query("UPDATE withdrawals SET transaction_hash = $1, updated_at = NOW() WHERE withdrawal_id = $2")
                    .bind(&hash)
                    .bind(withdrawal_id)
                    .execute(&self.db_pool)
                    .await?;

                tracing::info!(
                    "Withdrawal {} submitted successfully. TX Hash persisted: {}",
                    withdrawal_id,
                    hash
                );

                hash
            }
            Err(e) => {
                // If it fails on-chain, reject the withdrawal with the error
                tracing::error!(
                    "Withdrawal {} on-chain submission FAILED: {}",
                    withdrawal_id,
                    e
                );
                let mut error_msg: String = e.to_string();

                // Create error notification
                let _ = self
                    .notification_service
                    .create_notification(
                        wd_merchant_id,
                        "Withdrawal Failed",
                        &format!(
                            "Your withdrawal of {} {} failed: {}",
                            wd_amount, wd_crypto_type, error_msg
                        ),
                        "error",
                        "withdrawal.failed",
                        wd_sandbox_mode,
                    )
                    .await;

                // Lookup IF this withdrawal belongs to a Customer Transaction
                let customer_id: Option<i64> = sqlx::query_scalar::<_, i64>(
                    "SELECT customer_id FROM customer_transactions WHERE reference_id = $1",
                )
                .bind(withdrawal_id)
                .fetch_optional(&self.db_pool)
                .await
                .unwrap_or(None);

                if let Some(c_id) = customer_id {
                    // REFUND Customer Ledger (inverse of lock: lock -, avail +)
                    if let Err(refund_err) = self
                        .refund_customer_balance(c_id, &wd_crypto_type, wd_amount, wd_sandbox_mode)
                        .await
                    {
                        tracing::error!("CRITICAL: Automatic customer refund FAILED for withdrawal {}: {}. Manual intervention required.", withdrawal_id, refund_err);
                        error_msg = format!(
                            "{} [REFUND FAILED - Manual Intervention Required]",
                            error_msg
                        );
                    } else {
                        tracing::info!(
                            "Refunded {} {} to customer {} ledger after failed withdrawal",
                            wd_amount,
                            &wd_crypto_type,
                            c_id
                        );
                    }

                    // Set customer transaction failure status
                    let _ = sqlx::query("UPDATE customer_transactions SET status = 'FAILED' WHERE reference_id = $1")
                        .bind(withdrawal_id)
                        .execute(&self.db_pool)
                        .await;
                } else {
                    // Merchant ledger refund fallback
                    if let Err(refund_err) = self
                        .refund_withdrawal_balance(
                            wd_merchant_id,
                            &wd_crypto_type,
                            wd_amount,
                            wd_sandbox_mode,
                        )
                        .await
                    {
                        tracing::error!(
                            "CRITICAL: Failed to refund balance for failed withdrawal {}: {}",
                            withdrawal_id,
                            refund_err
                        );
                        error_msg = format!(
                            "{} [REFUND FAILED - Manual Intervention Required]",
                            error_msg
                        );
                    } else {
                        tracing::info!(
                            "Refunded {} {} to merchant {} ledger after failed withdrawal",
                            wd_amount,
                            wd_crypto_type,
                            wd_merchant_id
                        );
                    }
                }

                self.reject_withdrawal(withdrawal_id, &error_msg).await?;
                return Err(e);
            }
        };

        // 5. Finalize the withdrawal as COMPLETED
        self.finalize_completed_withdrawal(
            withdrawal_id,
            &tx_hash,
            wd_merchant_id,
            &wd_crypto_type,
            wd_amount,
            wd_sandbox_mode,
        )
        .await?;

        Ok(())
    }

    /// Helper to finalize a confirmed/completed withdrawal in the database and notifications
    async fn finalize_completed_withdrawal(
        &self,
        withdrawal_id: &str,
        tx_hash: &str,
        merchant_id: i64,
        crypto_type: &str,
        amount: Decimal,
        sandbox_mode: bool,
    ) -> Result<(), ServiceError> {
        // Create successful notification if not already done
        let _ = self
            .notification_service
            .create_notification(
                merchant_id,
                "Withdrawal Completed",
                &format!(
                    "Your withdrawal of {} {} has been confirmed.",
                    amount, crypto_type
                ),
                "success",
                "withdrawal.confirmed",
                sandbox_mode,
            )
            .await;

        // Update to COMPLETED
        sqlx::query(
            r#"
            UPDATE withdrawals 
            SET status = 'COMPLETED', completed_at = NOW(), transaction_hash = $1, updated_at = NOW()
            WHERE withdrawal_id = $2
            "#
        )
        .bind(tx_hash)
        .bind(withdrawal_id)
        .execute(&self.db_pool)
        .await?;

        tracing::info!("Withdrawal {} fully finalized in DB", withdrawal_id);

        Ok(())
    }

    pub async fn refund_withdrawal_balance(
        &self,
        merchant_id: i64,
        crypto_type: &str,
        amount: Decimal,
        sandbox_mode: bool,
    ) -> Result<(), ServiceError> {
        sqlx::query(
            r#"
            UPDATE merchant_balances 
            SET available_balance = available_balance + $1, last_updated = NOW()
            WHERE merchant_id = $2 AND crypto_type = $3 AND sandbox_mode = $4
            "#,
        )
        .bind(amount)
        .bind(merchant_id)
        .bind(crypto_type)
        .bind(sandbox_mode)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn refund_customer_balance(
        &self,
        customer_id: i64,
        crypto_type: &str,
        amount: Decimal,
        sandbox_mode: bool,
    ) -> Result<(), ServiceError> {
        sqlx::query(
            r#"
            UPDATE merchant_customer_balances 
            SET available_balance = available_balance + $1, locked_balance = locked_balance - $1, last_updated_at = NOW()
            WHERE customer_id = $2 AND crypto_type = $3 AND sandbox_mode = $4
            "#
        )
        .bind(amount)
        .bind(customer_id)
        .bind(crypto_type)
        .bind(sandbox_mode)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn reject_withdrawal(
        &self,
        withdrawal_id: &str,
        reason: &str,
    ) -> Result<(), ServiceError> {
        sqlx::query(
            "UPDATE withdrawals SET status = 'REJECTED', rejection_reason = $1, updated_at = NOW() WHERE withdrawal_id = $2"
        )
        .bind(reason)
        .bind(withdrawal_id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}
