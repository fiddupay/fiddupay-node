use crate::error::ServiceError;
use crate::payment::models::CryptoType;
use crate::utils::keygen::KeyGenerator;
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
    pub encrypted_private_key: Option<String>,
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
        is_active: bool,
        encrypted_private_key: Option<String>,
    ) -> Result<WalletConfig, ServiceError> {
        let network = crypto_type.network().to_string();
        
        let config = sqlx::query_as!(
            WalletConfig,
            r#"
            INSERT INTO merchant_wallets (merchant_id, crypto_type, network, address, is_active, encrypted_private_key)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (merchant_id, crypto_type) 
            DO UPDATE SET address = $4, is_active = $5, encrypted_private_key = COALESCE($6, merchant_wallets.encrypted_private_key), updated_at = NOW()
            RETURNING id, merchant_id, crypto_type, network, address, is_active, encrypted_private_key, created_at, updated_at
            "#,
            merchant_id,
            crypto_type.to_string(),
            network,
            address,
            is_active,
            encrypted_private_key
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Side effect: If this is a base currency or a known network-wide token like USDT,
        // we might want to ensure other tokens on the same network also get updated if they don't have an address.
        // However, the cleanest way for this UI is to just ensure the frontend groups them.
        // To satisfy the user's request that "USDT for any blockchain uses same wallet address if generated",
        // we can explicitly update the "sister" currency.
        
        let sister_crypto = match crypto_type {
            CryptoType::Sol => Some("USDT_SOL"),
            CryptoType::UsdtSpl => Some("SOL"),
            CryptoType::Eth => Some("USDT_ETH"),
            CryptoType::UsdtEth => Some("ETH"),
            CryptoType::Bnb => Some("USDT_BSC"),
            CryptoType::UsdtBep20 => Some("BNB"),
            CryptoType::Matic => Some("USDT_POLYGON"),
            CryptoType::UsdtPolygon => Some("MATIC"),
            CryptoType::Arb => Some("USDT_ARBITRUM"),
            CryptoType::UsdtArbitrum => Some("ARB"),
        };

        if let Some(sister) = sister_crypto {
             sqlx::query!(
                "INSERT INTO merchant_wallets (merchant_id, crypto_type, network, address, is_active, encrypted_private_key)
                 VALUES ($1, $2, $3, $4, $5, $6)
                 ON CONFLICT (merchant_id, crypto_type) 
                 DO UPDATE SET address = $4, is_active = $5, encrypted_private_key = COALESCE($6, merchant_wallets.encrypted_private_key), updated_at = NOW()",
                merchant_id,
                sister,
                network,
                address,
                is_active,
                encrypted_private_key
            )
            .execute(&self.db_pool)
            .await?;
        }

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
            "SELECT id, merchant_id, crypto_type, network, address, is_active, encrypted_private_key, created_at, updated_at FROM merchant_wallets WHERE merchant_id = $1",
            merchant_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(configs)
    }

    pub async fn configure_address_only(&self, merchant_id: i64, request: ConfigureWalletRequest) -> Result<WalletConfig, ServiceError> {
        let crypto_type = CryptoType::from_string(&request.crypto_type);
        self.set_wallet_address(merchant_id, crypto_type, request.address, request.is_active.unwrap_or(true), None).await
    }

    pub async fn generate_wallet(&self, merchant_id: i64, request: GenerateWalletRequest) -> Result<GeneratedWalletResponse, ServiceError> {
        let crypto_type = CryptoType::from_string(&request.crypto_type);
        
        // Generate real wallet based on network
        let wallet = match crypto_type {
            CryptoType::Sol | CryptoType::UsdtSpl => KeyGenerator::generate_solana_wallet()?,
            _ => KeyGenerator::generate_evm_wallet()?,
        };
        
        // Save the address to merchant_wallets and encrypt private key
        let encryption = crate::utils::encryption::Encryption::new()
            .map_err(|e| ServiceError::InternalError(format!("Encryption error: {}", e)))?;
        let encrypted_key = encryption.encrypt(&wallet.private_key)
            .map_err(|e| ServiceError::InternalError(format!("Encryption error: {}", e)))?;

        let config = self.set_wallet_address(
            merchant_id, 
            crypto_type, 
            wallet.address.clone(), 
            true, 
            Some(encrypted_key)
        ).await?;
        
        Ok(GeneratedWalletResponse {
            config,
            private_key: Some(wallet.private_key),
        })
    }

    /// Generate wallet in managed mode — private key is stored but never returned to the merchant
    pub async fn generate_wallet_managed(&self, merchant_id: i64, request: GenerateWalletRequest) -> Result<GeneratedWalletResponse, ServiceError> {
        let mut response = self.generate_wallet(merchant_id, request).await?;
        // Strip private key — platform manages it, merchant never sees it
        response.private_key = None;
        Ok(response)
    }

    pub async fn import_wallet(&self, merchant_id: i64, request: ImportWalletRequest) -> Result<WalletConfig, ServiceError> {
        let crypto_type = CryptoType::from_string(&request.crypto_type);
        
        // Validate and get address from private key
        let address = match crypto_type {
            CryptoType::Sol | CryptoType::UsdtSpl => KeyGenerator::validate_private_key(&request.private_key, "solana")?,
            _ => KeyGenerator::validate_private_key(&request.private_key, "ethereum")?, // Works for all EVM
        };

        // Encrypt private key
        let encryption = crate::utils::encryption::Encryption::new()
            .map_err(|e| ServiceError::InternalError(format!("Encryption error: {}", e)))?;
        let encrypted_key = encryption.encrypt(&request.private_key)
            .map_err(|e| ServiceError::InternalError(format!("Encryption error: {}", e)))?;
            
        self.set_wallet_address(
            merchant_id, 
            crypto_type, 
            address, 
            request.is_active.unwrap_or(true), 
            Some(encrypted_key)
        ).await
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

    pub async fn delete_wallet_config(&self, merchant_id: i64, crypto_type_str: String) -> Result<(), ServiceError> {
        let crypto_type = CryptoType::from_string(&crypto_type_str);
        
        sqlx::query!(
            "UPDATE merchant_wallets SET address = '', is_active = false, updated_at = NOW() 
             WHERE merchant_id = $1 AND crypto_type = $2",
            merchant_id,
            crypto_type.to_string()
        )
        .execute(&self.db_pool)
        .await?;

        // Also update sister currency
        let sister_crypto = match crypto_type {
            CryptoType::Sol => Some("USDT_SOL"),
            CryptoType::UsdtSpl => Some("SOL"),
            CryptoType::Eth => Some("USDT_ETH"),
            CryptoType::UsdtEth => Some("ETH"),
            CryptoType::Bnb => Some("USDT_BSC"),
            CryptoType::UsdtBep20 => Some("BNB"),
            CryptoType::Matic => Some("USDT_POLYGON"),
            CryptoType::UsdtPolygon => Some("MATIC"),
            CryptoType::Arb => Some("USDT_ARBITRUM"),
            CryptoType::UsdtArbitrum => Some("ARB"),
        };

        if let Some(sister) = sister_crypto {
            sqlx::query!(
                "UPDATE merchant_wallets SET address = '', is_active = false, updated_at = NOW() 
                 WHERE merchant_id = $1 AND crypto_type = $2",
                merchant_id,
                sister
            )
            .execute(&self.db_pool)
            .await?;
        }

        Ok(())
    }

    // =========================================================================
    // Forwarding-mode wallet methods (separate table: merchant_forwarding_wallets)
    // =========================================================================

    /// Set a forwarding destination address for a specific crypto type.
    /// This writes to `merchant_forwarding_wallets`, NOT `merchant_wallets`.
    pub async fn set_forwarding_address(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
        address: String,
        is_active: bool,
    ) -> Result<WalletConfig, ServiceError> {
        // Validate the address format for the specific blockchain
        crate::utils::validation::validate_wallet_address(&address, crypto_type)?;

        let network = crypto_type.network().to_string();

        let config = sqlx::query_as!(
            WalletConfig,
            r#"
            INSERT INTO merchant_forwarding_wallets (merchant_id, crypto_type, network, address, is_active)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (merchant_id, crypto_type)
            DO UPDATE SET address = $4, is_active = $5, updated_at = NOW()
            RETURNING id, merchant_id, crypto_type, network, address, is_active,
                      NULL::text as encrypted_private_key, created_at, updated_at
            "#,
            merchant_id,
            crypto_type.to_string(),
            network,
            address,
            is_active
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Add sister crypto logic for forwarding wallets (e.g., USDT -> Base coin)
        let sister_crypto = match crypto_type {
            CryptoType::Sol => Some("USDT_SOL"),
            CryptoType::UsdtSpl => Some("SOL"),
            CryptoType::Eth => Some("USDT_ETH"),
            CryptoType::UsdtEth => Some("ETH"),
            CryptoType::Bnb => Some("USDT_BSC"),
            CryptoType::UsdtBep20 => Some("BNB"),
            CryptoType::Matic => Some("USDT_POLYGON"),
            CryptoType::UsdtPolygon => Some("MATIC"),
            CryptoType::Arb => Some("USDT_ARBITRUM"),
            CryptoType::UsdtArbitrum => Some("ARB"),
        };

        if let Some(sister) = sister_crypto {
             sqlx::query!(
                "INSERT INTO merchant_forwarding_wallets (merchant_id, crypto_type, network, address, is_active)
                 VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT (merchant_id, crypto_type) 
                 DO UPDATE SET address = $4, is_active = $5, updated_at = NOW()",
                merchant_id,
                sister,
                network,
                address,
                is_active
            )
            .execute(&self.db_pool)
            .await?;
        }

        Ok(config)
    }

    /// Get all forwarding wallet configs for a merchant.
    pub async fn get_forwarding_configs(&self, merchant_id: i64) -> Result<Vec<WalletConfig>, ServiceError> {
        let configs = sqlx::query_as!(
            WalletConfig,
            r#"
            SELECT id, merchant_id, crypto_type, network, address, is_active,
                   NULL::text as encrypted_private_key, created_at, updated_at
            FROM merchant_forwarding_wallets
            WHERE merchant_id = $1
            "#,
            merchant_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(configs)
    }

    /// Delete (soft-delete) a forwarding wallet config.
    pub async fn delete_forwarding_config(&self, merchant_id: i64, crypto_type_str: String) -> Result<(), ServiceError> {
        let crypto_type = CryptoType::from_string(&crypto_type_str);

        sqlx::query!(
            "UPDATE merchant_forwarding_wallets SET address = '', is_active = false, updated_at = NOW()
             WHERE merchant_id = $1 AND crypto_type = $2",
            merchant_id,
            crypto_type.to_string()
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}
#[derive(Debug, Deserialize)]
pub struct ConfigureWalletRequest {
    pub crypto_type: String,
    pub address: String,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateWalletRequest {
    pub crypto_type: String,
}

#[derive(Debug, Deserialize)]
pub struct ImportWalletRequest {
    pub crypto_type: String,
    pub private_key: String,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ExportKeyRequest {
    pub crypto_type: String,
}

#[derive(Debug, Serialize)]
pub struct GeneratedWalletResponse {
    pub config: WalletConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
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
