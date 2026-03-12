use crate::error::ServiceError;
use crate::payment::models::CryptoType;
use crate::utils::keygen::KeyGenerator;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct WalletConfig {
    pub id: i64,
    pub merchant_id: i64,
    pub crypto_type: String,
    pub network: String,
    pub address: String,
    pub is_active: bool,
    pub sandbox_mode: bool,
    pub wallet_mode: Option<String>,
    #[serde(skip_serializing)]
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
        let crypto_type_str = crypto_type.to_string();
        let mode = if encrypted_private_key.is_some() { "managed" } else { "address_only" };

        tracing::info!(
            "set_wallet_address: merchant={}, crypto={}, mode={}, sandbox={}",
            merchant_id, crypto_type_str, mode, sandbox_mode
        );

        // 1. Check if the merchant has wallets locked
        let wallets_locked = sqlx::query_scalar::<_, bool>(
            "SELECT wallets_locked FROM merchants WHERE id = $1"
        )
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // 2. Fetch current wallet if it exists
        let current_wallet_row = sqlx::query(
            "SELECT address, wallet_mode, encrypted_private_key, is_active FROM merchant_wallets WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"
        )
        .bind(merchant_id)
        .bind(&crypto_type_str)
        .bind(sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if let Some(row) = current_wallet_row {
            let current_address: String = row.get("address");
            let current_mode: Option<String> = row.get("wallet_mode");
            let current_key: Option<String> = row.get("encrypted_private_key");
            let current_active: bool = row.get("is_active");

            // If the address or mode is different, we check the lock and record history
            let address_changed = current_address != address;
            let mode_changed = current_mode.as_deref().unwrap_or("address_only") != mode;
            let active_changed = current_active != is_active;

            if address_changed || mode_changed || active_changed {
                if wallets_locked {
                    tracing::warn!("Blocked wallet change for merchant {} (wallets locked)", merchant_id);
                    return Err(ServiceError::BadRequest(
                        "Wallets are locked. Please unlock in settings to change wallet configuration.".to_string()
                    ));
                }

                tracing::info!(
                    "Archiving wallet state for merchant {}: address_changed={}, mode_changed={}, active_changed={}",
                    merchant_id, address_changed, mode_changed, active_changed
                );

                // Archive the old address to history before updating (including key and mode)
                sqlx::query(
                    r#"
                    INSERT INTO merchant_wallet_history (
                        merchant_id, owner_type, crypto_type, network, 
                        old_address, new_address, wallet_mode, 
                        encrypted_private_key, is_active, reason
                    )
                    VALUES ($1, 'merchant', $2, $3, $4, $5, $6, $7, $8, $9)
                    "#
                )
                .bind(merchant_id)
                .bind(&crypto_type_str)
                .bind(&network)
                .bind(&current_address)
                .bind(&address)
                .bind(current_mode.as_deref().unwrap_or("address_only"))
                .bind(&current_key)
                .bind(current_active)
                .bind("Updated via wallet management")
                .execute(&self.db_pool)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
            }
        }

        let config_res: Result<WalletConfig, sqlx::Error> = sqlx::query_as::<_, WalletConfig>(
            r#"
            INSERT INTO merchant_wallets (
                merchant_id, crypto_type, network, address, is_active, 
                sandbox_mode, encrypted_private_key, wallet_mode
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (merchant_id, crypto_type, sandbox_mode) 
            DO UPDATE SET 
                address = EXCLUDED.address, 
                is_active = EXCLUDED.is_active, 
                encrypted_private_key = EXCLUDED.encrypted_private_key,
                wallet_mode = EXCLUDED.wallet_mode,
                updated_at = NOW()
            RETURNING id, merchant_id, crypto_type, network, address, is_active, sandbox_mode, 
                      wallet_mode, encrypted_private_key, created_at, updated_at
            "#
        )
        .bind(merchant_id)
        .bind(&crypto_type_str)
        .bind(&network)
        .bind(&address)
        .bind(is_active)
        .bind(sandbox_mode)
        .bind(&encrypted_private_key)
        .bind(mode)
        .fetch_one(&self.db_pool)
        .await;

        let config = config_res?;

        let sisters = match crypto_type {
            CryptoType::Sol | CryptoType::WSol | CryptoType::UsdtSpl => vec!["SOL", "WSOL", "USDT_SPL"],
            CryptoType::Eth | CryptoType::UsdtEth => vec!["ETH", "USDT_ETH"],
            CryptoType::Bnb | CryptoType::UsdtBep20 => vec!["BNB", "USDT_BEP20"],
            CryptoType::Matic | CryptoType::UsdtPolygon => vec!["MATIC", "USDT_POLYGON"],
            CryptoType::Arb | CryptoType::UsdtArbitrum => vec!["ARB", "USDT_ARBITRUM"],
        };

        for sister in sisters {
            if sister == crypto_type.to_string() { continue; }

            // Fetch current sister state to determine if we should archive
            let current_sister_row = sqlx::query(
                "SELECT address, wallet_mode, encrypted_private_key, is_active FROM merchant_wallets WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"
            )
            .bind(merchant_id)
            .bind(sister)
            .bind(sandbox_mode)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

            if let Some(row) = current_sister_row {
                let current_address: String = row.get("address");
                let current_mode: Option<String> = row.get("wallet_mode");
                let current_key: Option<String> = row.get("encrypted_private_key");
                let current_active: bool = row.get("is_active");

                let address_changed = current_address != address;
                let mode_changed = current_mode.as_deref().unwrap_or("address_only") != mode;
                let active_changed = current_active != is_active;

                if address_changed || mode_changed || active_changed {
                    tracing::info!("Archiving sister wallet state for merchant {}: crypto={}", merchant_id, sister);
                    
                    sqlx::query(
                        r#"
                        INSERT INTO merchant_wallet_history (
                            merchant_id, owner_type, crypto_type, network, 
                            old_address, new_address, wallet_mode, 
                            encrypted_private_key, is_active, reason
                        )
                        VALUES ($1, 'merchant', $2, $3, $4, $5, $6, $7, $8, $9)
                        "#
                    )
                    .bind(merchant_id)
                    .bind(sister)
                    .bind(&network)
                    .bind(&current_address)
                    .bind(&address)
                    .bind(current_mode.as_deref().unwrap_or("address_only"))
                    .bind(&current_key)
                    .bind(current_active)
                    .bind("Sister wallet updated")
                    .execute(&self.db_pool)
                    .await
                    .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
                }
            }

            sqlx::query(
                "INSERT INTO merchant_wallets (merchant_id, crypto_type, network, address, is_active, sandbox_mode, encrypted_private_key, wallet_mode)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                 ON CONFLICT (merchant_id, crypto_type, sandbox_mode) 
                 DO UPDATE SET 
                    address = EXCLUDED.address, 
                    is_active = EXCLUDED.is_active, 
                    encrypted_private_key = EXCLUDED.encrypted_private_key,
                    wallet_mode = EXCLUDED.wallet_mode,
                    updated_at = NOW()"
            )
            .bind(merchant_id)
            .bind(sister)
            .bind(&network)
            .bind(&address)
            .bind(is_active)
            .bind(sandbox_mode)
            .bind(&encrypted_private_key)
            .bind(mode)
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
        let wallet = sqlx::query(
            "SELECT address FROM merchant_wallets WHERE merchant_id = $1 AND crypto_type = $2 AND is_active = true"
        )
        .bind(merchant_id)
        .bind(crypto_type.to_string())
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(wallet.map(|w| w.get("address")))
    }

    pub async fn get_balance(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
        sandbox_mode: bool,
    ) -> Result<Decimal, ServiceError> {
        let balance = sqlx::query(
            "SELECT available_balance FROM merchant_balances WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"
        )
        .bind(merchant_id)
        .bind(crypto_type.to_string())
        .bind(sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(balance.map(|b| b.get::<Decimal, _>("available_balance")).unwrap_or(Decimal::ZERO))
    }

    pub async fn get_wallet_configs(&self, merchant_id: i64, sandbox_mode: bool) -> Result<Vec<WalletConfig>, ServiceError> {
        let configs = sqlx::query_as::<_, WalletConfig>(
            "SELECT id, merchant_id, crypto_type, network, address, is_active, sandbox_mode, wallet_mode, encrypted_private_key, created_at, updated_at FROM merchant_wallets WHERE merchant_id = $1 AND sandbox_mode = $2"
        )
        .bind(merchant_id)
        .bind(sandbox_mode)
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
        
        let wallet = match crypto_type {
            CryptoType::Sol | CryptoType::UsdtSpl | CryptoType::WSol => KeyGenerator::generate_solana_wallet()?,
            _ => KeyGenerator::generate_evm_wallet()?,
        };
        
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
        })
    }

    pub async fn generate_wallet_managed(&self, merchant_id: i64, sandbox_mode: bool, request: GenerateWalletRequest) -> Result<GeneratedWalletResponse, ServiceError> {
        self.generate_wallet(merchant_id, sandbox_mode, request).await
    }


    pub async fn validate_gas_for_withdrawal(&self, merchant_id: i64, sandbox_mode: bool, crypto_type: CryptoType, amount: Decimal) -> Result<GasValidationResult, ServiceError> {
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
        let network = crypto_type.network().to_string();

        tracing::info!(
            "delete_wallet_config: merchant={}, crypto={}, sandbox={}",
            merchant_id, crypto_type_str, sandbox_mode
        );

        // Check if the merchant has wallets locked
        let wallets_locked = sqlx::query_scalar::<_, bool>(
            "SELECT wallets_locked FROM merchants WHERE id = $1"
        )
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if wallets_locked {
            tracing::warn!("Blocked wallet deletion for merchant {} (locked)", merchant_id);
            return Err(ServiceError::BadRequest(
                "Wallets are locked. Please unlock in settings to remove configuration.".to_string()
            ));
        }

        // Fetch current to archive
        let current_row = sqlx::query(
            "SELECT address, wallet_mode, encrypted_private_key FROM merchant_wallets WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"
        )
        .bind(merchant_id)
        .bind(&crypto_type_str)
        .bind(sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if let Some(row) = current_row {
            let current_address: String = row.get("address");
            let current_mode: Option<String> = row.get("wallet_mode");
            let current_key: Option<String> = row.get("encrypted_private_key");

            if !current_address.is_empty() {
                sqlx::query(
                    r#"
                    INSERT INTO merchant_wallet_history (
                        merchant_id, owner_type, crypto_type, network, 
                        old_address, new_address, wallet_mode, 
                        encrypted_private_key, is_active, reason
                    )
                    VALUES ($1, 'merchant', $2, $3, $4, '', $5, $6, true, $7)
                    "#
                )
                .bind(merchant_id)
                .bind(&crypto_type_str)
                .bind(&network)
                .bind(&current_address)
                .bind(current_mode.as_deref().unwrap_or("address_only"))
                .bind(&current_key)
                .bind("Deleted via dashboard")
                .execute(&self.db_pool)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
            }
        }
        
        sqlx::query(
            "UPDATE merchant_wallets SET address = '', is_active = false, updated_at = NOW() 
             WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"
        )
        .bind(merchant_id)
        .bind(crypto_type.to_string())
        .bind(sandbox_mode)
        .execute(&self.db_pool)
        .await?;

        let sisters = match crypto_type {
            CryptoType::Sol | CryptoType::WSol | CryptoType::UsdtSpl => vec!["SOL", "WSOL", "USDT_SPL"],
            CryptoType::Eth | CryptoType::UsdtEth => vec!["ETH", "USDT_ETH"],
            CryptoType::Bnb | CryptoType::UsdtBep20 => vec!["BNB", "USDT_BEP20"],
            CryptoType::Matic | CryptoType::UsdtPolygon => vec!["MATIC", "USDT_POLYGON"],
            CryptoType::Arb | CryptoType::UsdtArbitrum => vec!["ARB", "USDT_ARBITRUM"],
        };

        for sister in sisters {
            if sister == crypto_type.to_string() { continue; }
            sqlx::query(
                "UPDATE merchant_wallets SET address = '', is_active = false, updated_at = NOW() 
                 WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"
            )
            .bind(merchant_id)
            .bind(sister)
            .bind(sandbox_mode)
            .execute(&self.db_pool)
            .await?;
        }

        Ok(())
    }

    // =========================================================================
    // Forwarding-mode wallet methods (separate table: merchant_forwarding_wallets)
    // =========================================================================

    pub async fn set_forwarding_address(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
        address: String,
        is_active: bool,
        sandbox_mode: bool,
    ) -> Result<WalletConfig, ServiceError> {
        crate::utils::validation::validate_wallet_address(&address, crypto_type)?;
        let crypto_type_str = crypto_type.to_string();
        let network = crypto_type.network().to_string();

        tracing::info!(
            "set_forwarding_address: merchant={}, crypto={}, sandbox={}",
            merchant_id, crypto_type_str, sandbox_mode
        );

        // Check if the merchant has wallets locked
        let wallets_locked = sqlx::query_scalar::<_, bool>(
            "SELECT wallets_locked FROM merchants WHERE id = $1"
        )
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // Archive if changing
        let current_address = sqlx::query_scalar::<_, String>(
            "SELECT address FROM merchant_forwarding_wallets WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"
        )
        .bind(merchant_id)
        .bind(&crypto_type_str)
        .bind(sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(current) = current_address {
            if current != address && !current.is_empty() {
                if wallets_locked {
                    tracing::warn!("Blocked forwarding address change for merchant {} (locked)", merchant_id);
                    return Err(ServiceError::BadRequest(
                        "Wallets are locked. Please unlock in settings to change address.".to_string()
                    ));
                }

                tracing::info!("Archiving forwarding address for merchant {}", merchant_id);

                // Archive the old address to history before updating
                sqlx::query(
                    r#"
                    INSERT INTO merchant_wallet_history (
                        merchant_id, owner_type, crypto_type, network, 
                        old_address, new_address, wallet_mode, reason
                    )
                    VALUES ($1, 'merchant', $2, $3, $4, $5, 'forwarding', $6)
                    "#
                )
                .bind(merchant_id)
                .bind(&crypto_type_str)
                .bind(&network)
                .bind(&current)
                .bind(&address)
                .bind("Updated via forwarding management")
                .execute(&self.db_pool)
                .await?;
            }
        }

        let network = crypto_type.network().to_string();

        let config = sqlx::query_as::<_, WalletConfig>(
            r#"
            INSERT INTO merchant_forwarding_wallets (merchant_id, crypto_type, network, address, is_active, sandbox_mode)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (merchant_id, crypto_type, sandbox_mode)
            DO UPDATE SET address = $4, is_active = $5, updated_at = NOW()
            RETURNING id, merchant_id, crypto_type, network, address, is_active, sandbox_mode,
                      NULL::text as wallet_mode, NULL::text as encrypted_private_key, created_at, updated_at
            "#
        )
        .bind(merchant_id)
        .bind(crypto_type.to_string())
        .bind(&network)
        .bind(&address)
        .bind(is_active)
        .bind(sandbox_mode)
        .fetch_one(&self.db_pool)
        .await?;

        let sisters = match crypto_type {
            CryptoType::Sol | CryptoType::WSol | CryptoType::UsdtSpl => vec!["SOL", "WSOL", "USDT_SPL"],
            CryptoType::Eth | CryptoType::UsdtEth => vec!["ETH", "USDT_ETH"],
            CryptoType::Bnb | CryptoType::UsdtBep20 => vec!["BNB", "USDT_BEP20"],
            CryptoType::Matic | CryptoType::UsdtPolygon => vec!["MATIC", "USDT_POLYGON"],
            CryptoType::Arb | CryptoType::UsdtArbitrum => vec!["ARB", "USDT_ARBITRUM"],
        };

        for sister in sisters {
            if sister == crypto_type.to_string() { continue; }
            sqlx::query(
                "INSERT INTO merchant_forwarding_wallets (merchant_id, crypto_type, network, address, is_active, sandbox_mode)
                 VALUES ($1, $2, $3, $4, $5, $6)
                 ON CONFLICT (merchant_id, crypto_type, sandbox_mode) 
                 DO UPDATE SET address = $4, is_active = $5, updated_at = NOW()"
            )
            .bind(merchant_id)
            .bind(sister)
            .bind(&network)
            .bind(&address)
            .bind(is_active)
            .bind(sandbox_mode)
            .execute(&self.db_pool)
            .await?;
        }

        Ok(config)
    }

    pub async fn get_forwarding_configs(&self, merchant_id: i64, sandbox_mode: bool) -> Result<Vec<WalletConfig>, ServiceError> {
        let configs = sqlx::query_as::<_, WalletConfig>(
            r#"
            SELECT id, merchant_id, crypto_type, network, address, is_active, sandbox_mode,
                   NULL::text as wallet_mode, NULL::text as encrypted_private_key, created_at, updated_at
            FROM merchant_forwarding_wallets
            WHERE merchant_id = $1 AND sandbox_mode = $2
            "#
        )
        .bind(merchant_id)
        .bind(sandbox_mode)
        .fetch_all(&self.db_pool)
        .await?;

        Ok(configs)
    }

    pub async fn delete_forwarding_config(&self, merchant_id: i64, sandbox_mode: bool, crypto_type_str: String) -> Result<(), ServiceError> {
        let crypto_type = CryptoType::from_string(&crypto_type_str)?;
        let network = crypto_type.network().to_string();

        tracing::info!(
            "delete_forwarding_config: merchant={}, crypto={}, sandbox={}",
            merchant_id, crypto_type_str, sandbox_mode
        );

        // Check if the merchant has wallets locked
        let wallets_locked = sqlx::query_scalar::<_, bool>(
            "SELECT wallets_locked FROM merchants WHERE id = $1"
        )
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if wallets_locked {
            tracing::warn!("Blocked forwarding deletion for merchant {} (locked)", merchant_id);
            return Err(ServiceError::BadRequest(
                "Wallets are locked. Please unlock in settings to remove configuration.".to_string()
            ));
        }

        // Fetch current to archive
        let current = sqlx::query_scalar::<_, String>(
            "SELECT address FROM merchant_forwarding_wallets WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"
        )
        .bind(merchant_id)
        .bind(&crypto_type_str)
        .bind(sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(curr) = current {
            if !curr.is_empty() {
                sqlx::query(
                    r#"
                    INSERT INTO merchant_wallet_history (
                        merchant_id, owner_type, crypto_type, network, 
                        old_address, new_address, wallet_mode, reason
                    )
                    VALUES ($1, 'merchant', $2, $3, $4, '', 'forwarding', $5)
                    "#
                )
                .bind(merchant_id)
                .bind(&crypto_type_str)
                .bind(&network)
                .bind(&curr)
                .bind("Deleted via dashboard")
                .execute(&self.db_pool)
                .await?;
            }
        }

        sqlx::query(
            "UPDATE merchant_forwarding_wallets SET address = '', is_active = false, updated_at = NOW()
             WHERE merchant_id = $1 AND (crypto_type = $2 OR network = $3) AND sandbox_mode = $4"
        )
        .bind(merchant_id)
        .bind(&crypto_type_str)
        .bind(&network)
        .bind(sandbox_mode)
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


#[derive(Debug, Serialize)]
pub struct GeneratedWalletResponse {
    pub config: WalletConfig,
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
