use crate::error::ServiceError;
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};

use crate::payment::models::CryptoType;
use crate::services::blockchain_transaction_sender::BlockchainTransactionSender;
use crate::utils::encryption::Encryption;

pub struct WithdrawalProcessor {
    db_pool: PgPool,
    config: crate::config::Config,
}

impl WithdrawalProcessor {
    pub fn new(db_pool: PgPool, config: crate::config::Config) -> Self {
        Self { db_pool, config }
    }

    pub async fn process_withdrawal(&self, withdrawal_id: &str) -> Result<(), ServiceError> {
        tracing::info!("Starting processing for withdrawal: {}", withdrawal_id);
        
        // 1. Fetch the withdrawal details
        let withdrawal = sqlx::query(
            r#"
            SELECT id, withdrawal_id, merchant_id, crypto_type, amount, destination_address, status, sandbox_mode
            FROM withdrawals 
            WHERE withdrawal_id = $1
            "#
        )
        .bind(withdrawal_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| ServiceError::NotFound("Withdrawal not found".to_string()))?;

        let wd_status: String = withdrawal.get("status");
        let wd_merchant_id: i64 = withdrawal.get("merchant_id");
        let wd_crypto_type: String = withdrawal.get("crypto_type");
        let wd_amount: Decimal = withdrawal.get("amount");
        let wd_destination_address: String = withdrawal.get("destination_address");
        let wd_sandbox_mode: bool = withdrawal.get("sandbox_mode");

        tracing::debug!(
            "Withdrawal {}: merchant={}, crypto={}, amount={}, sandbox={}", 
            withdrawal_id, wd_merchant_id, wd_crypto_type, wd_amount, wd_sandbox_mode
        );

        if wd_status != "PENDING" {
            tracing::warn!("Withdrawal {} requested for processing but has status {}", withdrawal_id, wd_status);
            return Err(ServiceError::ValidationError("Withdrawal already processed".to_string()));
        }

        // 2. Fetch the merchant's managed wallet for this crypto type
        let wallet = sqlx::query(
            r#"
            SELECT encrypted_private_key 
            FROM merchant_wallets 
            WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 AND address != ''
            "#
        )
        .bind(wd_merchant_id)
        .bind(&wd_crypto_type)
        .bind(wd_sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| ServiceError::NotFound("Merchant wallet not found or not configured".to_string()))?;

        let encrypted_key: Option<String> = wallet.get("encrypted_private_key");
        let encrypted_key = encrypted_key
            .ok_or_else(|| ServiceError::ValidationError("Managed wallet has no private key available".to_string()))?;

        let crypto_type_enum = CryptoType::from_string(&wd_crypto_type)?;

        // 3. Decrypt the private key
        let encryption = Encryption::new().map_err(|e| ServiceError::Internal(e))?;
        let private_key = encryption.decrypt(&encrypted_key).map_err(|e| ServiceError::Internal(e))?;

        tracing::info!("Blockchain submission started for withdrawal {} to address {}", withdrawal_id, wd_destination_address);

        // 4. Send the transaction on-chain
        let sender = BlockchainTransactionSender::new(self.config.clone());
        let tx_hash = match sender.send_transaction(
            crypto_type_enum,
            &private_key,
            &wd_destination_address,
            wd_amount,
            None,
            wd_sandbox_mode,
        ).await {
            Ok(hash) => {
                tracing::info!("Withdrawal {} submitted successfully. TX Hash: {}", withdrawal_id, hash);
                hash
            },
            Err(e) => {
                // If it fails on-chain, reject the withdrawal with the error
                tracing::error!("Withdrawal {} on-chain submission FAILED: {}", withdrawal_id, e);
                
                // REFUND the merchant's ledger balance (Requirement: Ledger Security)
                if let Err(refund_err) = self.refund_withdrawal_balance(wd_merchant_id, &wd_crypto_type, wd_amount, wd_sandbox_mode).await {
                    tracing::error!("CRITICAL: Failed to refund balance for failed withdrawal {}: {}", withdrawal_id, refund_err);
                } else {
                    tracing::info!("Refunded {} {} to merchant {} ledger after failed withdrawal", wd_amount, wd_crypto_type, wd_merchant_id);
                }

                self.reject_withdrawal(withdrawal_id, &e.to_string()).await?;
                return Err(e);
            }
        };

        // 5. Update the withdrawal as COMPLETED with the transaction hash
        sqlx::query(
            r#"
            UPDATE withdrawals 
            SET status = 'COMPLETED', completed_at = NOW(), transaction_hash = $1, updated_at = NOW()
            WHERE withdrawal_id = $2
            "#
        )
        .bind(&tx_hash)
        .bind(withdrawal_id)
        .execute(&self.db_pool)
        .await?;

        tracing::info!("Withdrawal {} fully completed and recorded in DB", withdrawal_id);

        Ok(())
    }

    pub async fn refund_withdrawal_balance(
        &self, 
        merchant_id: i64, 
        crypto_type: &str, 
        amount: Decimal,
        sandbox_mode: bool
    ) -> Result<(), ServiceError> {
        sqlx::query(
            r#"
            UPDATE merchant_balances 
            SET available_balance = available_balance + $1, last_updated = NOW()
            WHERE merchant_id = $2 AND crypto_type = $3 AND sandbox_mode = $4
            "#
        )
        .bind(amount)
        .bind(merchant_id)
        .bind(crypto_type)
        .bind(sandbox_mode)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn reject_withdrawal(&self, withdrawal_id: &str, reason: &str) -> Result<(), ServiceError> {
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
