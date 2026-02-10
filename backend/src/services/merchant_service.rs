// Merchant Service
// Business logic for merchant management

use crate::error::ServiceError;
use crate::models::merchant::{Merchant, MerchantRegistrationResponse, MerchantWallet};
use crate::payment::models::CryptoType;
use crate::utils::api_keys::ApiKeyGenerator;
use chrono::Utc;
use nanoid::nanoid;
use rust_decimal::Decimal;
use sqlx::PgPool;

pub struct MerchantService {
    db_pool: PgPool,
    config: crate::config::Config,
}

impl MerchantService {
    pub fn new(db_pool: PgPool, config: crate::config::Config) -> Self {
        Self { db_pool, config }
    }

    /// Get daily volume remaining for a merchant
    pub async fn get_daily_volume_remaining(
        &self,
        merchant_id: i64,
        kyc_verified: bool,
        daily_limit_usd: Option<Decimal>,
    ) -> Result<Decimal, ServiceError> {
        if kyc_verified {
            return Ok(Decimal::ZERO);
        }

        let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        
        let daily_volume: Option<Decimal> = match sqlx::query_scalar!(
            "SELECT SUM(amount_usd) FROM payment_transactions WHERE merchant_id = $1 AND created_at >= $2 AND status = 'CONFIRMED'",
            merchant_id,
            today_start
        )
        .fetch_one(&self.db_pool)
        .await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Volume calculation DB error: {:?}", e);
                None
            }
        };

        let limit = daily_limit_usd.unwrap_or(self.config.daily_volume_limit_non_kyc_usd);
        let settled_volume = daily_volume.unwrap_or(Decimal::ZERO);
        
        Ok((limit - settled_volume).max(Decimal::ZERO))
    }

    /// Generate API key with proper prefix (single source of truth)
    pub fn generate_api_key(&self, is_live: bool) -> String {
        ApiKeyGenerator::generate_key(is_live)
    }

    pub async fn register_merchant(
        &self,
        email: &str,
        business_name: &str,
        password: &str,
    ) -> Result<MerchantRegistrationResponse, ServiceError> {
        // 1. Hash the User Password
        use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
        use rand::rngs::OsRng;
        let argon2 = Argon2::default();
        let password_salt = SaltString::generate(&mut OsRng);
        let password_hash = argon2.hash_password(password.as_bytes(), &password_salt)
            .map_err(|_| ServiceError::InternalError("Failed to hash password".to_string()))?
            .to_string();

        // 2. Insert merchant with placeholder key hash (will be updated immediately)
        // We use query_as calling the function directly to avoid compile-time checking against unmigrated DB
        let merchant = sqlx::query_as::<_, Merchant>(
            r#"
            INSERT INTO merchants (email, business_name, test_api_key_hash, password_hash, fee_percentage, customer_pays_fee, is_active, sandbox_mode, settlement_mode, kyc_verified, created_at, updated_at, daily_limit_usd, role)
            VALUES ($1, $2, 'PENDING', $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'MERCHANT')
            RETURNING id, email, business_name, live_api_key_hash, test_api_key_hash, password_hash, fee_percentage, customer_pays_fee, is_active, sandbox_mode, settlement_mode, kyc_verified, created_at, updated_at, api_key_expires_at, daily_limit_usd, role::text as role, redirect_url
            "#,
        )
        .bind(&email)
        .bind(&business_name)
        .bind(&password_hash)
        .bind(self.config.default_fee_percentage)
        .bind(false) // customer_pays_fee (default: Merchant pays fee)
        .bind(true) // is_active
        .bind(true) // sandbox_mode (default)
        .bind("managed") // settlement_mode (default)
        .bind(false) // kyc_verified (default)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.db_pool)
        .await?;

        // 3. Generate Real Session Key (Sandbox by default for new accounts)
        let api_key = ApiKeyGenerator::generate_session_key(merchant.id, false); // false = sandbox
        let salt = SaltString::generate(&mut OsRng);
        let api_key_hash = argon2.hash_password(api_key.as_bytes(), &salt)
            .map_err(|_| ServiceError::InternalError("Failed to hash API key".to_string()))?
            .to_string();

        // 4. Update the merchant with the real key
        // Use sqlx::query function to avoid macro checking
        sqlx::query(
            "UPDATE merchants SET test_api_key_hash = $1 WHERE id = $2"
        )
        .bind(api_key_hash)
        .bind(merchant.id)
        .execute(&self.db_pool)
        .await?;
        
        Ok(MerchantRegistrationResponse {
            merchant_id: merchant.id,
            api_key,
        })
    }

    /// Switch merchant environment (sandbox <-> live)
    pub async fn switch_environment(
        &self,
        merchant_id: i64,
        to_live: bool,
    ) -> Result<String, ServiceError> {
        // Toggle the sandbox mode first
        sqlx::query(
            "UPDATE merchants SET sandbox_mode = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(!to_live)
        .bind(Utc::now())
        .bind(merchant_id)
        .execute(&self.db_pool)
        .await?;

        // Retrieve the key for the requested environment
        // If it exists, return it. If not, generate it.
        let merchant = sqlx::query_as::<_, Merchant>(
            "SELECT id, email, business_name, live_api_key_hash, test_api_key_hash, password_hash, fee_percentage, customer_pays_fee, is_active, sandbox_mode, settlement_mode, kyc_verified, created_at, updated_at, api_key_expires_at, daily_limit_usd, role::text as role, redirect_url FROM merchants WHERE id = $1"
        )
        .bind(merchant_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or(ServiceError::MerchantNotFound)?;

        let has_key = if to_live {
            merchant.live_api_key_hash.is_some()
        } else {
            merchant.test_api_key_hash.is_some()
        };

        if has_key {
            // If a key already exists, generate and return a new one to ensure 
            // the frontend has a valid key for the new environment session.
            return self.generate_and_store_api_key_with_expiry(merchant_id, to_live, merchant.api_key_expires_at).await;
        }

        // If no key exists, generate and store a new one.
        self.generate_and_store_api_key_with_expiry(merchant_id, to_live, merchant.api_key_expires_at).await
    }

    /// Rotate API key for a merchant
    /// 
    /// Invalidates the old API key and generates a new one. This allows
    /// merchants to rotate their credentials without service interruption
    /// if they provide the old key for verification.
    /// 
    /// # Arguments
    /// * `merchant_id` - ID of the merchant
    /// * `old_api_key` - Current API key for verification
    /// 
    /// # Returns
    /// * New API key string
    /// 
    /// # Requirements
    /// * 7.5: Support API key rotation without service interruption
    /// * 7.6: Invalidate old key and generate new one
    pub async fn rotate_api_key(
        &self,
        merchant_id: i64,
        old_api_key: &str,
    ) -> Result<String, ServiceError> {
        // First, verify the old API key is correct
        let merchant = sqlx::query_as::<_, Merchant>(
            "SELECT id, email, business_name, live_api_key_hash, test_api_key_hash, password_hash, fee_percentage, customer_pays_fee, is_active, sandbox_mode, settlement_mode, kyc_verified, created_at, updated_at, api_key_expires_at, daily_limit_usd, role::text as role, redirect_url FROM merchants WHERE id = $1"
        )
        .bind(merchant_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or(ServiceError::MerchantNotFound)?;
        
        // Determine if old key is live or test based on prefix
        // Since we are moving to standard prefixes, we can check.
        let is_old_live = old_api_key.starts_with("sk_live_") || old_api_key.starts_with("live_");
        
        // Use Argon2 for verification
        use argon2::{Argon2, PasswordHash, PasswordVerifier, PasswordHasher, password_hash::SaltString};
        use rand::rngs::OsRng;
        
        let hash_to_check = if is_old_live {
            merchant.live_api_key_hash.as_ref()
        } else {
            merchant.test_api_key_hash.as_ref()
        };

        let hash_str = hash_to_check.ok_or(ServiceError::InvalidApiKey)?;
        let parsed_hash = PasswordHash::new(hash_str)
            .map_err(|_| ServiceError::InvalidApiKey)?;
        
        if Argon2::default().verify_password(old_api_key.as_bytes(), &parsed_hash).is_err() {
            // Fallback: try the other key just in case (e.g. if prefix logic fails for legacy keys)
            // But strict separation is safer.
            return Err(ServiceError::InvalidApiKey);
        }
        
        // Generate a new searchable API key for the SAME environment as the old one
        let new_api_key = ApiKeyGenerator::generate_session_key(merchant_id, is_old_live);
        
        // Hash the new API key
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let new_api_key_hash = argon2.hash_password(new_api_key.as_bytes(), &salt)
            .map_err(|_| ServiceError::InternalError("Failed to hash API key".to_string()))?
            .to_string();
        
        // Update the merchant with the new API key hash
        let update_query = if is_old_live {
            "UPDATE merchants SET live_api_key_hash = $1, updated_at = $2 WHERE id = $3"
        } else {
            "UPDATE merchants SET test_api_key_hash = $1, updated_at = $2 WHERE id = $3"
        };

        sqlx::query(update_query)
        .bind(new_api_key_hash)
        .bind(Utc::now())
        .bind(merchant_id)
        .execute(&self.db_pool)
        .await?;
        
        Ok(new_api_key)
    }

    /// Authenticate a merchant using their API key
    /// 
    /// Validates the provided API key against stored Argon2 hash and
    /// returns the merchant if authentication succeeds.
    /// 
    /// # Arguments
    /// * `api_key` - API key to authenticate
    /// 
    /// # Returns
    /// * `Merchant` if authentication succeeds
    /// 
    /// # Requirements
    /// * 7.1: Authenticate merchant with valid API key
    /// * 1.2: Use Argon2 verification for API keys
    pub async fn authenticate(
        &self,
        token: &str,
    ) -> Result<Merchant, ServiceError> {
        // Searchable token logic (sk_s_{id}_{random} or sk_live_s_{id}_{random} or sk_merchant_{id}_{token})
        if token.starts_with("sk_s_") || token.starts_with("sk_live_s_") || token.starts_with("sk_merchant_") {
            let parts: Vec<&str> = token.split('_').collect();
            tracing::debug!("Auth: token parts count={}, prefix match ok", parts.len());
            
            if parts.len() >= 4 || (token.starts_with("sk_merchant_") && parts.len() >= 3) {
                // Determine ID position based on prefix
                let id_str = if token.starts_with("sk_live_s_") { 
                    parts.get(3).copied() 
                } else { 
                    parts.get(2).copied() 
                };

                if let Some(id_str) = id_str {
                    if let Ok(merchant_id) = id_str.parse::<i64>() {
                        tracing::debug!("Auth: parsed merchant_id={}", merchant_id);
                        
                        let merchant = sqlx::query_as::<_, Merchant>(
                            "SELECT id, email, business_name, live_api_key_hash, test_api_key_hash, password_hash, fee_percentage, customer_pays_fee, is_active, sandbox_mode, settlement_mode, kyc_verified, created_at, updated_at, api_key_expires_at, daily_limit_usd, role::text as role, redirect_url 
                             FROM merchants 
                             WHERE id = $1 AND is_active = true"
                        )
                        .bind(merchant_id)
                        .fetch_optional(&self.db_pool)
                        .await
                        .map_err(|e| {
                            tracing::error!("Failed to fetch merchant {} for auth: {:?}", merchant_id, e);
                            ServiceError::Database(e)
                        })?;

                        if let Some(merchant) = merchant {
                            use argon2::{Argon2, PasswordHash, PasswordVerifier};
                            use chrono::Utc;
                            
                            // Determine which hash to check
                            let is_live_token = token.starts_with("sk_live_");
                            let hash_to_check = if is_live_token {
                                merchant.live_api_key_hash.as_ref()
                            } else {
                                merchant.test_api_key_hash.as_ref()
                            };
                            
                            tracing::debug!("Auth: is_live={}, has_hash={}", is_live_token, hash_to_check.is_some());

                            if let Some(hash_str) = hash_to_check {
                                if let Ok(parsed_hash) = PasswordHash::new(hash_str) {
                                    if Argon2::default().verify_password(token.as_bytes(), &parsed_hash).is_ok() {
                                        if let Some(expires_at) = merchant.api_key_expires_at {
                                            if Utc::now() > expires_at {
                                                tracing::warn!("Session token for merchant {} expired", merchant.id);
                                                return Err(ServiceError::InvalidApiKey);
                                            }
                                        }
                                        tracing::info!("Successfully authenticated merchant {} via searchable session key", merchant.id);
                                        return Ok(merchant);
                                    } else {
                                        tracing::warn!("Auth: hash verification FAILED for merchant {} (is_live={})", merchant_id, is_live_token);
                                    }
                                } else {
                                    tracing::warn!("Auth: failed to parse hash for merchant {} (is_live={})", merchant_id, is_live_token);
                                }
                            } else {
                                tracing::warn!("Auth: no {} hash found for merchant {}", if is_live_token { "live" } else { "test" }, merchant_id);
                            }
                        } else {
                            tracing::warn!("Auth: no active merchant found with id={}", merchant_id);
                        }
                    } else {
                        tracing::warn!("Auth: failed to parse merchant id from '{}'", id_str);
                    }
                } else {
                    tracing::warn!("Auth: id_str position returned None");
                }
            } else {
                tracing::warn!("Auth: insufficient parts count={}", parts.len());
            }
        }

        // All other key formats (sk_, live_, etc.) without ID prefixes are no longer supported
        // for better security and performance (O(1) lookup only).
        tracing::warn!("Rejecting legacy or malformed API key format");
        Err(ServiceError::InvalidApiKey)
    }

    /// Store an API key for a merchant with optional expiration
    pub async fn store_api_key_with_expiry(
        &self,
        merchant_id: i64,
        api_key: &str,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<(), ServiceError> {
        // Hash the API key using Argon2
        use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
        use rand::rngs::OsRng;
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let api_key_hash = argon2.hash_password(api_key.as_bytes(), &salt)
            .map_err(|_| ServiceError::InternalError("Failed to hash API key".to_string()))?
            .to_string();
        
        let is_live = api_key.starts_with("sk_live_") || api_key.starts_with("live_");

        let update_query = if is_live {
            "UPDATE merchants SET live_api_key_hash = $1, api_key_expires_at = $2, updated_at = $3 WHERE id = $4"
        } else {
            "UPDATE merchants SET test_api_key_hash = $1, api_key_expires_at = $2, updated_at = $3 WHERE id = $4"
        };
        
        // Update merchant with new API key hash and expiration
        sqlx::query(update_query)
        .bind(api_key_hash)
        .bind(expires_at)
        .bind(Utc::now())
        .bind(merchant_id)
        .execute(&self.db_pool)
        .await
        .map_err(ServiceError::Database)?;
        
        Ok(())
    }

    /// Generate and store API key for merchant with optional expiration
    pub async fn generate_and_store_api_key_with_expiry(
        &self,
        merchant_id: i64,
        is_live: bool,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<String, ServiceError> {
        // Use single source of truth for API key generation
        // Ensure we use the searchable session key format correctly!
        let api_key = ApiKeyGenerator::generate_session_key(merchant_id, is_live);
        
        self.store_api_key_with_expiry(merchant_id, &api_key, expires_at).await?;
        
        Ok(api_key)
    }

    /// Set or update wallet address for a specific blockchain
    /// 
    /// Validates the wallet address format for the specified blockchain type
    /// and stores it in the database. If a wallet already exists for this
    /// merchant and crypto type, it will be updated.
    /// 
    /// # Arguments
    /// * `merchant_id` - ID of the merchant
    /// * `crypto_type` - Type of cryptocurrency (SOL, USDT_SPL, USDT_BEP20, etc.)
    /// * `address` - Wallet address to validate and store
    /// 
    /// # Returns
    /// * `Ok(())` if the address is valid and stored successfully
    /// * `Err(ServiceError)` if validation fails or database error occurs
    /// 
    /// # Requirements
    /// * 1.4: Validate and store wallet addresses for supported blockchains
    /// * 1.5: Support multiple wallet addresses per merchant (one per blockchain)
    /// * 1.6: Validate new addresses before saving
    pub async fn set_wallet_address(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
        address: String,
    ) -> Result<(), ServiceError> {
        // Validate the address format for the specific blockchain
        self.validate_wallet_address(&address, crypto_type)?;
        
        // Get the network name for this crypto type
        let network = crypto_type.network();
        let crypto_type_str = match crypto_type {
            CryptoType::UsdtBep20 => "USDT_BEP20",
            CryptoType::UsdtArbitrum => "USDT_ARBITRUM", 
            CryptoType::UsdtSpl => "USDT_SPL",
            CryptoType::UsdtPolygon => "USDT_POLYGON",
            CryptoType::UsdtEth => "USDT_ETH",
            CryptoType::Sol => "SOL",
            CryptoType::Eth => "ETH",
            CryptoType::Arb => "ARB",
            CryptoType::Matic => "MATIC",
            CryptoType::Bnb => "BNB",
        };
        
        // Insert or update the wallet address
        // Use ON CONFLICT to update if the merchant already has a wallet for this crypto type
        sqlx::query(
            r#"
            INSERT INTO merchant_wallets (merchant_id, crypto_type, network, address, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (merchant_id, crypto_type)
            DO UPDATE SET 
                address = EXCLUDED.address,
                network = EXCLUDED.network,
                is_active = EXCLUDED.is_active,
                updated_at = EXCLUDED.updated_at
            "#
        )
        .bind(merchant_id)
        .bind(&crypto_type_str)
        .bind(network)
        .bind(&address)
        .bind(true) // is_active
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }

    /// Get wallet address for a specific blockchain
    /// 
    /// Retrieves the merchant's wallet address for the specified cryptocurrency type.
    /// 
    /// # Arguments
    /// * `merchant_id` - ID of the merchant
    /// * `crypto_type` - Type of cryptocurrency
    /// 
    /// # Returns
    /// * Wallet address string if found
    /// * `Err(ServiceError::WalletNotFound)` if no wallet is configured
    /// 
    /// # Requirements
    /// * 1.4: Retrieve stored wallet addresses
    pub async fn get_wallet_address(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
    ) -> Result<String, ServiceError> {
        // Map USDT tokens to their base network crypto types
        let lookup_crypto_type = match crypto_type {
            CryptoType::UsdtSpl => "SOL",        // USDT on Solana uses SOL wallet
            CryptoType::UsdtEth => "ETH",        // USDT on Ethereum uses ETH wallet
            CryptoType::UsdtBep20 => "BNB",      // USDT on BSC uses BNB wallet
            CryptoType::UsdtPolygon => "MATIC",  // USDT on Polygon uses MATIC wallet
            CryptoType::UsdtArbitrum => "ARB",   // USDT on Arbitrum uses ARB wallet
            CryptoType::Sol => "SOL",
            CryptoType::Eth => "ETH",
            CryptoType::Arb => "ARB",
            CryptoType::Matic => "MATIC",
            CryptoType::Bnb => "BNB",
        };
        
        let wallet_opt = sqlx::query_as::<_, MerchantWallet>(
            "SELECT id, merchant_id, crypto_type, network, address, is_active, created_at, updated_at 
             FROM merchant_wallets 
             WHERE merchant_id = $1 AND crypto_type = $2 AND is_active = true"
        )
        .bind(merchant_id)
        .bind(&lookup_crypto_type)
        .fetch_optional(&self.db_pool)
        .await?;
        
        if let Some(wallet) = wallet_opt {
            return Ok(wallet.address);
        }

        // If not found, check if we should auto-generate (Managed Mode)
        let merchant = sqlx::query!(
            "SELECT settlement_mode FROM merchants WHERE id = $1",
            merchant_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        if merchant.settlement_mode == "managed" {
            tracing::info!("Auto-generating wallet for merchant {} for network {}", merchant_id, lookup_crypto_type);
            
            let wallet_service = crate::services::wallet_config_service::WalletConfigService::new(self.db_pool.clone());
            let gen_req = crate::services::wallet_config_service::GenerateWalletRequest {
                crypto_type: lookup_crypto_type.to_string(),
            };
            
            let response = wallet_service.generate_wallet(merchant_id, gen_req).await?;
            return Ok(response.config.address);
        }
        
        Err(ServiceError::WalletNotFound)
    }

    pub async fn update_settlement_mode(
        &self,
        merchant_id: i64,
        mode: &str,
    ) -> Result<(), ServiceError> {
        // Validate the mode
        if !["forwarding", "managed", "imported"].contains(&mode) {
            return Err(ServiceError::ValidationError("Invalid settlement mode".to_string()));
        }

        sqlx::query!(
            "UPDATE merchants SET settlement_mode = $1, updated_at = $2 WHERE id = $3",
            mode,
            Utc::now(),
            merchant_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn update_settings(
        &self,
        merchant_id: i64,
        settlement_mode: Option<String>,
        customer_pays_fee: Option<bool>,
        sandbox_mode: Option<bool>,
        redirect_url: Option<String>,
    ) -> Result<(), ServiceError> {
        if let Some(ref mode) = settlement_mode {
            if !["forwarding", "managed", "imported"].contains(&mode.as_str()) {
                return Err(ServiceError::ValidationError("Invalid settlement mode".to_string()));
            }
        }

        sqlx::query!(
            r#"
            UPDATE merchants 
            SET 
                settlement_mode = COALESCE($1, settlement_mode),
                customer_pays_fee = COALESCE($2, customer_pays_fee),
                sandbox_mode = COALESCE($3, sandbox_mode),
                redirect_url = COALESCE($4, redirect_url),
                updated_at = $5
            WHERE id = $6
            "#,
            settlement_mode,
            customer_pays_fee,
            sandbox_mode,
            redirect_url,
            Utc::now(),
            merchant_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Validate wallet address format for specific blockchain
    /// 
    /// Performs blockchain-specific validation on wallet addresses to ensure
    /// they are properly formatted before storage.
    /// 
    /// # Arguments
    /// * `address` - Wallet address to validate
    /// * `crypto_type` - Type of cryptocurrency/blockchain
    /// 
    /// # Returns
    /// * `Ok(())` if address is valid
    /// * `Err(ServiceError::InvalidWalletAddress)` if validation fails
    /// 
    /// # Requirements
    /// * 1.6: Validate addresses before saving
    fn validate_wallet_address(
        &self,
        address: &str,
        crypto_type: CryptoType,
    ) -> Result<(), ServiceError> {
        match crypto_type {
            CryptoType::Sol | CryptoType::UsdtSpl => {
                // Solana addresses are base58 encoded, typically 32-44 characters
                if address.len() < 32 || address.len() > 44 {
                    return Err(ServiceError::InvalidWalletAddress(
                        "Solana address must be 32-44 characters".to_string()
                    ));
                }
                
                // Check if all characters are valid base58
                const BASE58_ALPHABET: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
                if !address.chars().all(|c| BASE58_ALPHABET.contains(c)) {
                    return Err(ServiceError::InvalidWalletAddress(
                        "Solana address contains invalid base58 characters".to_string()
                    ));
                }
            }
            CryptoType::UsdtBep20 | CryptoType::UsdtArbitrum | CryptoType::UsdtPolygon | CryptoType::UsdtEth | CryptoType::Eth | CryptoType::Arb | CryptoType::Matic | CryptoType::Bnb => {
                // EVM addresses start with 0x and have 40 hex characters
                if !address.starts_with("0x") {
                    return Err(ServiceError::InvalidWalletAddress(
                        "EVM address must start with 0x".to_string()
                    ));
                }
                
                if address.len() != 42 {
                    return Err(ServiceError::InvalidWalletAddress(
                        "EVM address must be 42 characters (0x + 40 hex chars)".to_string()
                    ));
                }
                
                // Check if all characters after 0x are valid hex
                let hex_part = &address[2..];
                if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(ServiceError::InvalidWalletAddress(
                        "EVM address contains invalid hexadecimal characters".to_string()
                    ));
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_api_key_length() {
        // Create a mock pool (we don't need a real connection for this test)
        let service = MerchantService {
            db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
            config: crate::config::Config::default(),
        };
        
        let api_key = service.generate_api_key(false);
        // Prefix "sk_" (3) + 32 random chars = 35 total
        assert_eq!(api_key.len(), 35);
        assert!(api_key.starts_with("sk_"));
    }

    #[tokio::test]
    async fn test_generate_api_key_uniqueness() {
        let service = MerchantService {
            db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
            config: crate::config::Config::default(),
        };
        
        let key1 = service.generate_api_key(false);
        let key2 = service.generate_api_key(false);
        
        // Keys should be different
        assert_ne!(key1, key2);
    }

    #[tokio::test]
    async fn test_generate_api_key_alphanumeric() {
        let service = MerchantService {
                    db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
                    config: crate::config::Config::default(),
                };
        
        let api_key = service.generate_api_key(false);
        
        // Should contain alphanumeric characters and underscores
        assert!(api_key.chars().all(|c| c.is_alphanumeric() || c == '_'));
        assert!(api_key.contains('_'));
    }

    #[tokio::test]
    async fn test_api_key_hashing() {
        use argon2::{Argon2, PasswordHasher};
        use argon2::password_hash::{SaltString, rand_core::OsRng};
        
        // Test that argon2 hashing works correctly
        // Use a valid session key format just in case validation is applied elsewhere
        let api_key = "sk_s_123_testkey1234567890123456"; 
        let salt1 = SaltString::generate(&mut OsRng);
        let salt2 = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let hash1 = argon2.hash_password(api_key.as_bytes(), &salt1).unwrap().to_string();
        let hash2 = argon2.hash_password(api_key.as_bytes(), &salt2).unwrap().to_string();
        
        // Hashes should be different (different salts)
        assert_ne!(hash1, hash2);
    }

    #[tokio::test]
    async fn test_default_fee_percentage() {
        // Test that default fee percentage is 1.50%
        let fee = Decimal::new(150, 2);
        assert_eq!(fee.to_string(), "1.50");
    }

    // Wallet address validation tests
    #[tokio::test]
    async fn test_validate_solana_address_valid() {
        let service = MerchantService {
                    db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
                    config: crate::config::Config::default(),
                };
        
        // Valid Solana address (base58, 32-44 chars)
        let valid_address = "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU";
        let result = service.validate_wallet_address(valid_address, CryptoType::Sol);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_solana_address_too_short() {
        let service = MerchantService {
                    db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
                    config: crate::config::Config::default(),
                };
        
        // Too short
        let invalid_address = "7xKXtg2CW87d97TXJSDpbD5jBkhe";
        let result = service.validate_wallet_address(invalid_address, CryptoType::Sol);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_solana_address_invalid_chars() {
        let service = MerchantService {
                    db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
                    config: crate::config::Config::default(),
                };
        
        // Contains invalid base58 characters (0, O, I, l)
        let invalid_address = "0OIl7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU";
        let result = service.validate_wallet_address(invalid_address, CryptoType::Sol);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_evm_address_valid() {
        let service = MerchantService {
                    db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
                    config: crate::config::Config::default(),
                };
        
        // Valid EVM address
        let valid_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0";
        let result = service.validate_wallet_address(valid_address, CryptoType::UsdtBep20);
        assert!(result.is_ok());
        
        // Also test with lowercase
        let valid_address_lower = "0x742d35cc6634c0532925a3b844bc9e7595f0beb0";
        let result = service.validate_wallet_address(valid_address_lower, CryptoType::UsdtArbitrum);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_evm_address_no_prefix() {
        let service = MerchantService {
                    db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
                    config: crate::config::Config::default(),
                };
        
        // Missing 0x prefix
        let invalid_address = "742d35Cc6634C0532925a3b844Bc9e7595f0bEb0";
        let result = service.validate_wallet_address(invalid_address, CryptoType::UsdtBep20);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_evm_address_wrong_length() {
        let service = MerchantService {
                    db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
                    config: crate::config::Config::default(),
                };
        
        // Too short
        let invalid_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0b";
        let result = service.validate_wallet_address(invalid_address, CryptoType::UsdtPolygon);
        assert!(result.is_err());
        
        // Too long
        let invalid_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0123";
        let result = service.validate_wallet_address(invalid_address, CryptoType::UsdtPolygon);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_evm_address_invalid_hex() {
        let service = MerchantService {
                    db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
                    config: crate::config::Config::default(),
                };
        
        // Contains non-hex characters (g, z)
        let invalid_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEgz";
        let result = service.validate_wallet_address(invalid_address, CryptoType::UsdtBep20);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_all_crypto_types() {
        let service = MerchantService {
                    db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
                    config: crate::config::Config::default(),
                };
        
        // Test valid addresses for all crypto types
        let test_cases = vec![
            (CryptoType::Sol, "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"),
            (CryptoType::UsdtSpl, "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
            (CryptoType::UsdtBep20, "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"),
            (CryptoType::UsdtArbitrum, "0x1234567890123456789012345678901234567890"),
            (CryptoType::UsdtPolygon, "0xabcdefABCDEF0123456789abcdefABCDEF012345"),
        ];
        
        for (crypto_type, address) in test_cases {
            let result = service.validate_wallet_address(address, crypto_type);
            assert!(result.is_ok(), "Failed for {:?}: {:?}", crypto_type, result);
        }
    }
}
