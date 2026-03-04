use crate::error::ServiceError;
use rust_decimal::Decimal;
use sqlx::PgPool;

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
        // 1. Fetch the withdrawal details
        let withdrawal = sqlx::query!(
            r#"
            SELECT id, withdrawal_id, merchant_id, crypto_type, amount, destination_address, status, sandbox_mode
            FROM withdrawals 
            WHERE withdrawal_id = $1
            "#,
            withdrawal_id
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| ServiceError::NotFound("Withdrawal not found".to_string()))?;

        if withdrawal.status != "PENDING" {
            return Err(ServiceError::ValidationError("Withdrawal already processed".to_string()));
        }

        // 2. Fetch the merchant's managed wallet for this crypto type
        let wallet = sqlx::query!(
            r#"
            SELECT encrypted_private_key 
            FROM merchant_wallets 
            WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 AND address != ''
            "#,
            withdrawal.merchant_id,
            withdrawal.crypto_type,
            withdrawal.sandbox_mode
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| ServiceError::NotFound("Merchant wallet not found or not configured".to_string()))?;

        let encrypted_key = wallet.encrypted_private_key
            .ok_or_else(|| ServiceError::ValidationError("Managed wallet has no private key available".to_string()))?;

        let crypto_type_enum = CryptoType::from_string(&withdrawal.crypto_type)?;

        // 3. Decrypt the private key
        let encryption = Encryption::new().map_err(|e| ServiceError::Internal(e))?;
        let private_key = encryption.decrypt(&encrypted_key).map_err(|e| ServiceError::Internal(e))?;

        // 4. Send the transaction on-chain
        let sender = BlockchainTransactionSender::new(self.config.clone());
        let tx_hash = match sender.send_native_transaction(
            crypto_type_enum,
            &private_key,
            &withdrawal.destination_address,
            withdrawal.amount,
            None,
            withdrawal.sandbox_mode,
        ).await {
            Ok(hash) => hash,
            Err(e) => {
                // If it fails on-chain, reject the withdrawal with the error
                self.reject_withdrawal(withdrawal_id, &e.to_string()).await?;
                return Err(e);
            }
        };

        // 5. Update the withdrawal as COMPLETED with the transaction hash
        sqlx::query!(
            r#"
            UPDATE withdrawals 
            SET status = 'COMPLETED', completed_at = NOW(), transaction_hash = $1, updated_at = NOW()
            WHERE withdrawal_id = $2
            "#,
            tx_hash,
            withdrawal_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn reject_withdrawal(&self, withdrawal_id: &str, reason: &str) -> Result<(), ServiceError> {
        sqlx::query!(
            "UPDATE withdrawals SET status = 'REJECTED', rejection_reason = $1, updated_at = NOW() WHERE withdrawal_id = $2",
            reason,
            withdrawal_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}
