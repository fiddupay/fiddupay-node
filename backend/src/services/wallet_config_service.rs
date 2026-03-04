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
    pub sandbox_mode: bool,
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
        sandbox_mode: bool,
        encrypted_private_key: Option<String>,
    ) -> Result<WalletConfig, ServiceError> {
        let network = crypto_type.network().to_string();
        
        let config_res: Result<WalletConfig, sqlx::Error> = sqlx::query_as!(
            WalletConfig,
            r#"
            INSERT INTO merchant_wallets (merchant_id, crypto_type, network, address, is_active, sandbox_mode, encrypted_private_key)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (merchant_id, crypto_type, sandbox_mode) 
            DO UPDATE SET address = $4, is_active = $5, encrypted_private_key = COALESCE($7, merchant_wallets.encrypted_private_key), updated_at = NOW()
            RETURNING id, merchant_id, crypto_type, network, address, is_active, sandbox_mode, encrypted_private_key, created_at, updated_at
            "#,
            merchant_id,
            crypto_type.to_string(),
            network,
            address,
            is_active,
            sandbox_mode,
            encrypted_private_key
        )
        .fetch_one(&self.db_pool)
        .await;

        let config = config_res?;

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
            CryptoType::Bnb => Some("USDT_BEP20"),
            CryptoType::UsdtBep20 => Some("BNB"),
            CryptoType::Matic => Some("USDT_POLYGON"),
            CryptoType::UsdtPolygon => Some("MATIC"),
            CryptoType::Arb => Some("USDT_ARBITRUM"),
            CryptoType::UsdtArbitrum => Some("ARB"),
        };

        if let Some(sister) = sister_crypto {
             sqlx::query!(
                "INSERT INTO merchant_wallets (merchant_id, crypto_type, network, address, is_active, sandbox_mode, encrypted_private_key)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)
                 ON CONFLICT (merchant_id, crypto_type, sandbox_mode) 
                 DO UPDATE SET address = $4, is_active = $5, encrypted_private_key = COALESCE($7, merchant_wallets.encrypted_private_key), updated_at = NOW()",
                merchant_id,
                sister,
                network,
                address,
                is_active,
                sandbox_mode,
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
        let wallet_res: Result<Option<_>, sqlx::Error> = sqlx::query!(
            "SELECT address FROM merchant_wallets WHERE merchant_id = $1 AND crypto_type = $2 AND is_active = true",
            merchant_id,
            crypto_type.to_string()
        )
        .fetch_optional(&self.db_pool)
        .await;

        let wallet = wallet_res?;

        Ok(wallet.map(|w| w.address))
    }

    pub async fn get_balance(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
        sandbox_mode: bool,
    ) -> Result<Decimal, ServiceError> {
        let balance_res: Result<Option<_>, sqlx::Error> = sqlx::query!(
            "SELECT available_balance FROM merchant_balances WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3",
            merchant_id,
            crypto_type.to_string(),
            sandbox_mode
        )
        .fetch_optional(&self.db_pool)
        .await;

        let balance = balance_res?;

        Ok(balance.map(|b| b.available_balance).unwrap_or(Decimal::ZERO))
    }

    pub async fn get_wallet_configs(&self, merchant_id: i64, sandbox_mode: bool) -> Result<Vec<WalletConfig>, ServiceError> {
        let configs = sqlx::query_as!(
            WalletConfig,
            "SELECT id, merchant_id, crypto_type, network, address, is_active, sandbox_mode, encrypted_private_key, created_at, updated_at FROM merchant_wallets WHERE merchant_id = $1 AND sandbox_mode = $2",
            merchant_id,
            sandbox_mode
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(configs)
    }

    pub async fn configure_address_only(&self, merchant_id: i64, sandbox_mode: bool, request: ConfigureWalletRequest) -> Result<WalletConfig, ServiceError> {
        let crypto_type = CryptoType::from_string(&request.crypto_type)?;
        self.set_wallet_address(merchant_id, crypto_type, request.address, request.is_active.unwrap_or(true), sandbox_mode, None).await
    }

    pub async fn generate_wallet(&self, merchant_id: i64, sandbox_mode: bool, request: GenerateWalletRequest) -> Result<GeneratedWalletResponse, ServiceError> {
        let crypto_type = CryptoType::from_string(&request.crypto_type)?;
        
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
            crypto_type.clone(), 
            wallet.address.clone(), 
            true, 
            sandbox_mode,
            Some(encrypted_key.clone())
        ).await?;

        // If enable_all_evm is set and base crypto is EVM, propagate to the others
        if request.enable_all_evm.unwrap_or(false) && is_evm(&crypto_type) {
            let evm_networks = vec![
                CryptoType::Eth,
                CryptoType::Bnb,
                CryptoType::Matic,
                CryptoType::Arb,
            ];
            for network in evm_networks {
                if network != crypto_type {
                    self.set_wallet_address(
                        merchant_id,
                        network,
                        wallet.address.clone(),
                        true,
                        sandbox_mode,
                        Some(encrypted_key.clone())
                    ).await?;
                }
            }
        }
        
        Ok(GeneratedWalletResponse {
            config,
            private_key: Some(wallet.private_key),
        })
    }

    /// Generate wallet in managed mode — private key is stored but never returned to the merchant
    pub async fn generate_wallet_managed(&self, merchant_id: i64, sandbox_mode: bool, request: GenerateWalletRequest) -> Result<GeneratedWalletResponse, ServiceError> {
        let mut response = self.generate_wallet(merchant_id, sandbox_mode, request).await?;
        // Strip private key — platform manages it, merchant never sees it
        response.private_key = None;
        Ok(response)
    }

    pub async fn import_wallet(&self, merchant_id: i64, sandbox_mode: bool, request: ImportWalletRequest) -> Result<WalletConfig, ServiceError> {
        let crypto_type = CryptoType::from_string(&request.crypto_type)?;
        
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
        let config = self.set_wallet_address(
            merchant_id, 
            crypto_type.clone(), 
            address.clone(), 
            request.is_active.unwrap_or(true), 
            sandbox_mode,
            Some(encrypted_key.clone())
        ).await?;

        // If enable_all_evm is set and base crypto is EVM, propagate to the others
        if request.enable_all_evm.unwrap_or(false) && is_evm(&crypto_type) {
            let evm_networks = vec![
                CryptoType::Eth,
                CryptoType::Bnb,
                CryptoType::Matic,
                CryptoType::Arb,
            ];
            for network in evm_networks {
                if network != crypto_type {
                    self.set_wallet_address(
                        merchant_id,
                        network,
                        address.clone(),
                        request.is_active.unwrap_or(true),
                        sandbox_mode,
                        Some(encrypted_key.clone())
                    ).await?;
                }
            }
        }

        Ok(config)
    }

    pub async fn export_private_key(&self, merchant_id: i64, sandbox_mode: bool, request: ExportKeyRequest) -> Result<String, ServiceError> {
        // Fetch encrypted key (mock logic, adapt based on actual storage)
        let row = sqlx::query!(
            "SELECT encrypted_private_key FROM merchant_wallets WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3",
            merchant_id,
            request.crypto_type,
            sandbox_mode
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(r) = row {
             if let Some(key) = r.encrypted_private_key {
                  return Ok(key);
             }
        }
        Err(ServiceError::NotFound("Private key not found".to_string()))
    }

    pub async fn validate_gas_for_withdrawal(&self, merchant_id: i64, sandbox_mode: bool, crypto_type: CryptoType, amount: Decimal) -> Result<GasValidationResult, ServiceError> {
        // Basic gas validation logic
        let balance = self.get_balance(merchant_id, crypto_type, sandbox_mode).await?;
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

    pub async fn can_withdraw(&self, merchant_id: i64, sandbox_mode: bool, crypto_type: CryptoType, amount: Decimal) -> Result<bool, ServiceError> {
        let balance = self.get_balance(merchant_id, crypto_type, sandbox_mode).await?;
        Ok(balance >= amount)
    }

    pub async fn delete_wallet_config(&self, merchant_id: i64, sandbox_mode: bool, crypto_type_str: String) -> Result<(), ServiceError> {
        let crypto_type = CryptoType::from_string(&crypto_type_str)?;
        
        let delete_res: Result<_, sqlx::Error> = sqlx::query!(
            "UPDATE merchant_wallets SET address = '', is_active = false, updated_at = NOW() 
             WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3",
            merchant_id,
            crypto_type.to_string(),
            sandbox_mode
        )
        .execute(&self.db_pool)
        .await;
        delete_res?;

        // Also update sister currency
        let sister_crypto = match crypto_type {
            CryptoType::Sol => Some("USDT_SOL"),
            CryptoType::UsdtSpl => Some("SOL"),
            CryptoType::Eth => Some("USDT_ETH"),
            CryptoType::UsdtEth => Some("ETH"),
            CryptoType::Bnb => Some("USDT_BEP20"),
            CryptoType::UsdtBep20 => Some("BNB"),
            CryptoType::Matic => Some("USDT_POLYGON"),
            CryptoType::UsdtPolygon => Some("MATIC"),
            CryptoType::Arb => Some("USDT_ARBITRUM"),
            CryptoType::UsdtArbitrum => Some("ARB"),
        };

        if let Some(sister) = sister_crypto {
            sqlx::query!(
                "UPDATE merchant_wallets SET address = '', is_active = false, updated_at = NOW() 
                 WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3",
                merchant_id,
                sister,
                sandbox_mode
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
        sandbox_mode: bool,
    ) -> Result<WalletConfig, ServiceError> {
        // Validate the address format for the specific blockchain
        crate::utils::validation::validate_wallet_address(&address, crypto_type)?;

        let network = crypto_type.network().to_string();

        let config = sqlx::query_as!(
            WalletConfig,
            r#"
            INSERT INTO merchant_forwarding_wallets (merchant_id, crypto_type, network, address, is_active, sandbox_mode)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (merchant_id, crypto_type, sandbox_mode)
            DO UPDATE SET address = $4, is_active = $5, updated_at = NOW()
            RETURNING id, merchant_id, crypto_type, network, address, is_active, sandbox_mode,
                      NULL::text as encrypted_private_key, created_at, updated_at
            "#,
            merchant_id,
            crypto_type.to_string(),
            network,
            address,
            is_active,
            sandbox_mode
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Add sister crypto logic for forwarding wallets (e.g., USDT -> Base coin)
        let sister_crypto = match crypto_type {
            CryptoType::Sol => Some("USDT_SOL"),
            CryptoType::UsdtSpl => Some("SOL"),
            CryptoType::Eth => Some("USDT_ETH"),
            CryptoType::UsdtEth => Some("ETH"),
            CryptoType::Bnb => Some("USDT_BEP20"),
            CryptoType::UsdtBep20 => Some("BNB"),
            CryptoType::Matic => Some("USDT_POLYGON"),
            CryptoType::UsdtPolygon => Some("MATIC"),
            CryptoType::Arb => Some("USDT_ARBITRUM"),
            CryptoType::UsdtArbitrum => Some("ARB"),
        };

        if let Some(sister) = sister_crypto {
             sqlx::query!(
                "INSERT INTO merchant_forwarding_wallets (merchant_id, crypto_type, network, address, is_active, sandbox_mode)
                 VALUES ($1, $2, $3, $4, $5, $6)
                 ON CONFLICT (merchant_id, crypto_type, sandbox_mode) 
                 DO UPDATE SET address = $4, is_active = $5, updated_at = NOW()",
                merchant_id,
                sister,
                network,
                address,
                is_active,
                sandbox_mode
            )
            .execute(&self.db_pool)
            .await?;
        }

        Ok(config)
    }

    /// Get all forwarding wallet configs for a merchant.
    pub async fn get_forwarding_configs(&self, merchant_id: i64, sandbox_mode: bool) -> Result<Vec<WalletConfig>, ServiceError> {
        let configs = sqlx::query_as!(
            WalletConfig,
            r#"
            SELECT id, merchant_id, crypto_type, network, address, is_active, sandbox_mode,
                   NULL::text as encrypted_private_key, created_at, updated_at
            FROM merchant_forwarding_wallets
            WHERE merchant_id = $1 AND sandbox_mode = $2
            "#,
            merchant_id,
            sandbox_mode
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(configs)
    }

    /// Delete (soft-delete) a forwarding wallet config.
    /// Also cleans up entries with legacy naming conventions.
    pub async fn delete_forwarding_config(&self, merchant_id: i64, sandbox_mode: bool, crypto_type_str: String) -> Result<(), ServiceError> {
        let crypto_type = CryptoType::from_string(&crypto_type_str)?;
        let network = crypto_type.network();

        // Delete by both the canonical name AND by network to catch legacy-named entries
        sqlx::query!(
            "UPDATE merchant_forwarding_wallets SET address = '', is_active = false, updated_at = NOW()
             WHERE merchant_id = $1 AND (crypto_type = $2 OR network = $3) AND sandbox_mode = $4",
            merchant_id,
            crypto_type.to_string(),
            network,
            sandbox_mode
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
    pub enable_all_evm: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ImportWalletRequest {
    pub crypto_type: String,
    pub private_key: String,
    pub is_active: Option<bool>,
    pub enable_all_evm: Option<bool>,
}

// Helper to determine if a CryptoType is an EVM network
fn is_evm(crypto_type: &CryptoType) -> bool {
    matches!(
        crypto_type,
        CryptoType::Eth | CryptoType::UsdtEth |
        CryptoType::Bnb | CryptoType::UsdtBep20 |
        CryptoType::Matic | CryptoType::UsdtPolygon |
        CryptoType::Arb | CryptoType::UsdtArbitrum
    )
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
