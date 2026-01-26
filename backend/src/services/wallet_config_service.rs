use crate::error::ServiceError;
use crate::payment::models::CryptoType;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletConfig {
    pub id: i64,
    pub merchant_id: i64,
    pub crypto_type: String,
    pub network: String,
    pub address: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct WalletConfigService {
    db_pool: PgPool,
}

impl WalletConfigService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn set_wallet_address(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
        address: String,
    ) -> Result<WalletConfig, ServiceError> {
        let config = sqlx::query_as!(
            WalletConfig,
            r#"
            INSERT INTO merchant_wallets (merchant_id, crypto_type, network, address)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (merchant_id, crypto_type) 
            DO UPDATE SET address = $4, updated_at = NOW()
            RETURNING id, merchant_id, crypto_type, network, address, is_active, created_at, updated_at
            "#,
            merchant_id,
            crypto_type.to_string(),
            crypto_type.network(),
            address
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(config)
    }

    pub async fn get_wallet_address(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
    ) -> Result<Option<String>, ServiceError> {
        let wallet = sqlx::query!(
            "SELECT address FROM merchant_wallets WHERE merchant_id = $1 AND crypto_type = $2 AND is_active = true",
            merchant_id,
            crypto_type.to_string()
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(wallet.map(|w| w.address))
    }

    pub async fn get_balance(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
    ) -> Result<Decimal, ServiceError> {
        let balance = sqlx::query!(
            "SELECT available_balance FROM merchant_balances WHERE merchant_id = $1 AND crypto_type = $2",
            merchant_id,
            crypto_type.to_string()
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(balance.map(|b| b.available_balance).unwrap_or(Decimal::ZERO))
    }

    pub async fn get_wallet_configs(&self, merchant_id: i64) -> Result<Vec<WalletConfig>, ServiceError> {
        let configs = sqlx::query_as!(
            WalletConfig,
            "SELECT id, merchant_id, crypto_type, network, address, is_active, created_at, updated_at FROM merchant_wallets WHERE merchant_id = $1",
            merchant_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(configs)
    }

    pub async fn configure_address_only(&self, merchant_id: i64, request: ConfigureWalletRequest) -> Result<WalletConfig, ServiceError> {
        let crypto_type = CryptoType::from_string(&request.crypto_type);
        self.set_wallet_address(merchant_id, crypto_type, request.address).await
    }

    pub async fn generate_wallet(&self, merchant_id: i64, request: GenerateWalletRequest) -> Result<WalletConfig, ServiceError> {
        // For now, return a placeholder - this would integrate with actual wallet generation
        let crypto_type = CryptoType::from_string(&request.crypto_type);
        let placeholder_address = format!("generated_address_for_{}", request.crypto_type);
        self.set_wallet_address(merchant_id, crypto_type, placeholder_address).await
    }

    pub async fn import_wallet(&self, merchant_id: i64, request: ImportWalletRequest) -> Result<WalletConfig, ServiceError> {
        // For now, return a placeholder - this would integrate with actual wallet import
        let crypto_type = CryptoType::from_string(&request.crypto_type);
        let placeholder_address = format!("imported_address_for_{}", request.crypto_type);
        self.set_wallet_address(merchant_id, crypto_type, placeholder_address).await
    }

    pub async fn export_private_key(&self, merchant_id: i64, request: ExportKeyRequest) -> Result<String, ServiceError> {
        // For now, return a placeholder - this would integrate with actual key export
        Ok(format!("private_key_for_{}_{}", merchant_id, request.crypto_type))
    }

    pub async fn validate_gas_for_withdrawal(&self, merchant_id: i64, crypto_type: CryptoType, amount: Decimal) -> Result<GasValidationResult, ServiceError> {
        // Basic gas validation logic
        let balance = self.get_balance(merchant_id, crypto_type).await?;
        if balance >= amount {
            Ok(GasValidationResult {
                valid: true,
                message: "Sufficient balance for withdrawal".to_string(),
            })
        } else {
            Ok(GasValidationResult {
                valid: false,
                message: "Insufficient balance for withdrawal".to_string(),
            })
        }
    }

    pub async fn can_withdraw(&self, merchant_id: i64, crypto_type: CryptoType, amount: Decimal) -> Result<bool, ServiceError> {
        let balance = self.get_balance(merchant_id, crypto_type).await?;
        Ok(balance >= amount)
    }
}
#[derive(Debug, Deserialize)]
pub struct ConfigureWalletRequest {
    pub crypto_type: String,
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct GenerateWalletRequest {
    pub crypto_type: String,
}

#[derive(Debug, Deserialize)]
pub struct ImportWalletRequest {
    pub crypto_type: String,
    pub private_key: String,
}

#[derive(Debug, Deserialize)]
pub struct ExportKeyRequest {
    pub crypto_type: String,
}

#[derive(Debug, Serialize)]
pub struct GasValidationResult {
    pub valid: bool,
    pub message: String,
}

impl GasValidationResult {
    pub fn Sufficient() -> Self {
        Self {
            valid: true,
            message: "Sufficient".to_string(),
        }
    }
}
