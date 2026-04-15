use crate::config::Config;
use crate::error::ServiceError;
use crate::utils::encryption::Encryption;
use crate::utils::keygen::KeyGenerator;
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use sqlx::{PgPool, Row};
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct DepositAddress {
    pub payment_id: String,
    pub crypto_type: String,
    pub deposit_address: String,
    pub merchant_destination: String,
    pub expires_at: DateTime<Utc>,
    pub status: String,
}

pub struct DepositAddressService {
    pool: PgPool,
    encryption: Encryption,
    config: Arc<Config>,
}

impl DepositAddressService {
    pub fn new(pool: PgPool, config: Arc<Config>) -> Result<Self, ServiceError> {
        let encryption = Encryption::new()
            .map_err(|e| ServiceError::InternalError(format!("Encryption init failed: {}", e)))?;

        Ok(Self {
            pool,
            encryption,
            config,
        })
    }

    /// Generate temporary deposit address for payment
    /// This implements the BitPay model:
    /// 1. Generate unique address per payment
    /// 2. Monitor this address for incoming payments
    /// 3. When confirmed, forward to merchant's actual wallet (minus fee)
    /// 4. Address expires after 15 minutes
    pub async fn generate_deposit_address(
        &self,
        payment_id: &str,
        crypto_type: &str,
        merchant_destination: &str,
        expiration_minutes: i64,
    ) -> Result<DepositAddress, ServiceError> {
        let expires_at = Utc::now() + Duration::minutes(expiration_minutes);

        // Generate actual blockchain keypair
        let keypair = self.generate_keypair(crypto_type)?;

        // Encrypt private key
        let private_key_encrypted = self
            .encryption
            .encrypt(&keypair.private_key)
            .map_err(|e| ServiceError::InternalError(format!("Encryption failed: {}", e)))?;

        sqlx::query(
            r#"INSERT INTO deposit_addresses 
               (payment_id, crypto_type, deposit_address, private_key_encrypted, merchant_destination, expires_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#
        )
        .bind(payment_id)
        .bind(crypto_type)
        .bind(&keypair.address)
        .bind(&private_key_encrypted)
        .bind(merchant_destination)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok(DepositAddress {
            payment_id: payment_id.to_string(),
            crypto_type: crypto_type.to_string(),
            deposit_address: keypair.address,
            merchant_destination: merchant_destination.to_string(),
            expires_at,
            status: "ACTIVE".to_string(),
        })
    }

    pub async fn get_deposit_address(
        &self,
        payment_id: &str,
    ) -> Result<DepositAddress, ServiceError> {
        let record = sqlx::query(
            "SELECT payment_id, crypto_type, deposit_address, merchant_destination, expires_at, status
             FROM deposit_addresses WHERE payment_id = $1"
        )
        .bind(payment_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ServiceError::NotFound("Deposit address not found".to_string()))?;

        Ok(DepositAddress {
            payment_id: record.get("payment_id"),
            crypto_type: record.get("crypto_type"),
            deposit_address: record.get("deposit_address"),
            merchant_destination: record.get("merchant_destination"),
            expires_at: record.get("expires_at"),
            status: record.get("status"),
        })
    }

    pub async fn get_private_key(&self, payment_id: &str) -> Result<String, ServiceError> {
        let record = sqlx::query(
            "SELECT private_key_encrypted FROM deposit_addresses WHERE payment_id = $1",
        )
        .bind(payment_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ServiceError::NotFound("Deposit address not found".to_string()))?;

        let private_key: String = record.get("private_key_encrypted");
        self.encryption
            .decrypt(&private_key)
            .map_err(|e| ServiceError::InternalError(format!("Decryption failed: {}", e)))
    }

    pub async fn mark_as_used(
        &self,
        payment_id: &str,
        forward_tx_hash: &str,
    ) -> Result<(), ServiceError> {
        sqlx::query(
            "UPDATE deposit_addresses SET status = 'USED', forwarded_at = NOW(), forward_tx_hash = $2
             WHERE payment_id = $1"
        )
        .bind(payment_id)
        .bind(forward_tx_hash)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn expire_old_addresses(&self) -> Result<u64, ServiceError> {
        let result = sqlx::query(
            "UPDATE deposit_addresses SET status = 'EXPIRED'
             WHERE expires_at < NOW() AND status = 'ACTIVE'",
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    fn generate_keypair(
        &self,
        crypto_type: &str,
    ) -> Result<crate::utils::keygen::WalletKeyPair, ServiceError> {
        match crypto_type {
            "SOL" | "USDT_SPL" => KeyGenerator::generate_solana_wallet(),
            "USDT_BEP20" | "USDT_ARBITRUM" | "USDT_POLYGON" => KeyGenerator::generate_evm_wallet(),
            "BTC" => {
                let is_sandbox = self.config.bitcoin_rpc_url.contains("testnet");
                KeyGenerator::generate_btc_wallet(is_sandbox)
            }
            _ => Err(ServiceError::ValidationError(format!(
                "Unsupported crypto type: {}",
                crypto_type
            ))),
        }
    }
}
