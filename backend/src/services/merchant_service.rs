// Merchant Service
// Business logic for merchant management

use crate::error::ServiceError;
use crate::models::merchant::{Merchant, MerchantRegistrationResponse, MerchantWallet};
use crate::payment::models::CryptoType;
use crate::utils::api_keys::ApiKeyGenerator;
use chrono::Utc;
use nanoid::nanoid;
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};

pub struct MerchantService {
    db_pool: PgPool,
    config: crate::config::Config,
    audit_service: std::sync::Arc<crate::services::audit_service::AuditService>,
}

impl MerchantService {
    pub fn new(db_pool: PgPool, config: crate::config::Config, audit_service: std::sync::Arc<crate::services::audit_service::AuditService>) -> Self {
        Self { db_pool, config, audit_service }
    }

    /// Validate a wallet address for a specific crypto type
    pub fn validate_wallet_address(&self, address: &str, crypto_type: CryptoType) -> Result<(), ServiceError> {
        match crypto_type {
            CryptoType::Sol | CryptoType::UsdtSpl => {
                // Solana address format: 32-44 characters, base58
                if address.len() < 32 || address.len() > 44 {
                    return Err(ServiceError::ValidationError("Invalid Solana address length".to_string()));
                }
                
                // Basic character set validation for base58 (no 0, O, I, l)
                let invalid_chars = ['0', 'O', 'I', 'l'];
                if address.chars().any(|c| !c.is_alphanumeric() || invalid_chars.contains(&c)) {
                    return Err(ServiceError::ValidationError("Invalid characters in Solana address".to_string()));
                }
            },
            CryptoType::UsdtBep20 | CryptoType::UsdtPolygon | CryptoType::UsdtArbitrum | CryptoType::Eth => {
                // EVM address format: 42 characters, starts with 0x
                if !address.starts_with("0x") {
                    return Err(ServiceError::ValidationError("EVM address must start with 0x".to_string()));
                }
                if address.len() != 42 {
                    return Err(ServiceError::ValidationError("Invalid EVM address length".to_string()));
                }
                
                // Hex validation
                if !address[2..].chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(ServiceError::ValidationError("Invalid hex in EVM address".to_string()));
                }
            },
            _ => {
                 // Unknown crypto type, skipping validation or returning generic error
                 return Err(ServiceError::ValidationError(format!("Validation not implemented for {:?}", crypto_type)));
            }
        }
        Ok(())
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
        
        let daily_volume: Option<Decimal> = match sqlx::query_scalar::<_, Option<Decimal>>(
            "SELECT SUM(amount_usd) FROM payment_transactions WHERE merchant_id = $1 AND created_at >= $2 AND status = 'CONFIRMED'"
        )
        .bind(merchant_id)
        .bind(today_start)
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
        req: &crate::models::merchant::MerchantRegistrationRequest,
    ) -> Result<MerchantRegistrationResponse, ServiceError> {
        let email = &req.email;
        let business_name = &req.business_name;
        let password = &req.password;

        // 1. Hash the User Password
        use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
        use rand::rngs::OsRng;
        let argon2 = Argon2::default();
        let password_salt = SaltString::generate(&mut OsRng);
        let password_hash = argon2.hash_password(password.as_bytes(), &password_salt)
            .map_err(|_| ServiceError::InternalError("Failed to hash password".to_string()))?
            .to_string();

        // 2. Insert merchant with placeholder key hash (will be updated immediately)
        let merchant_res: Result<Merchant, sqlx::Error> = sqlx::query_as::<_, Merchant>(
            r#"
            INSERT INTO merchants (
                email, business_name, test_api_key_hash, password_hash, 
                fee_percentage, customer_pays_fee, is_active, sandbox_mode, 
                settlement_mode, kyc_verified, created_at, updated_at, 
                daily_limit_usd, role, first_name, last_name, 
                gender, phone_number, country, applicant_role, 
                business_country, business_license_number, 
                business_certificate_url, terms_accepted,
                wallets_locked, customer_wallets_locked
            )
            VALUES ($1, $2, 'PENDING', $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'MERCHANT', $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, TRUE, TRUE)
            RETURNING id, email, business_name, live_api_key_hash, test_api_key_hash, password_hash, fee_percentage, customer_pays_fee, is_active, sandbox_mode, settlement_mode, kyc_verified, created_at, updated_at, api_key_expires_at, daily_limit_usd, role::text as role, redirect_url,
                      first_name, last_name, gender, phone_number, country, applicant_role, business_country, business_license_number, business_certificate_url, terms_accepted,
                      wallets_locked, customer_wallets_locked
            "#
        )
        .bind(&email)
        .bind(&business_name)
        .bind(&password_hash)
        .bind(self.config.default_fee_percentage)
        .bind(false) // customer_pays_fee
        .bind(true) // is_active
        .bind(true) // sandbox_mode
        .bind("managed") // settlement_mode
        .bind(false) // kyc_verified
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(None::<Decimal>)
        .bind(&req.first_name)
        .bind(&req.last_name)
        .bind(&req.gender)
        .bind(&req.phone_number)
        .bind(&req.country)
        .bind(&req.applicant_role)
        .bind(&req.business_country)
        .bind(&req.business_license_number)
        .bind(&req.business_certificate_url)
        .bind(req.terms_accepted)

        .fetch_one(&self.db_pool)
        .await;

        let merchant = merchant_res?;

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
    /// 
    /// Only toggles sandbox_mode. Returns a new API key ONLY if the target
    /// environment has no key hash stored yet (first-time switch).
    /// Existing tokens remain valid — no re-authentication needed.
    pub async fn switch_environment(
        &self,
        merchant_id: i64,
        to_live: bool,
    ) -> Result<Option<String>, ServiceError> {
        // Toggle the sandbox mode
        let target_sandbox = !to_live;
        sqlx::query(
            "UPDATE merchants SET sandbox_mode = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(target_sandbox)
        .bind(Utc::now())
        .bind(merchant_id)
        .execute(&self.db_pool)
        .await?;

        // Mirror wallet configs from the opposite environment into the target environment.
        // This ensures merchants always see their wallets regardless of which mode they're in.
        // ON CONFLICT DO NOTHING ensures we never overwrite wallets that already exist in the target.
        let source_sandbox = !target_sandbox;
        
        // Mirror merchant_wallets (managed / imported)
        sqlx::query(
            r#"
            INSERT INTO merchant_wallets (merchant_id, crypto_type, network, address, is_active, sandbox_mode, encrypted_private_key, wallet_mode)
            SELECT merchant_id, crypto_type, network, address, is_active, $1, encrypted_private_key, wallet_mode
            FROM merchant_wallets
            WHERE merchant_id = $2 AND sandbox_mode = $3 AND address != ''
            ON CONFLICT (merchant_id, crypto_type, sandbox_mode) DO NOTHING
            "#
        )

        .bind(target_sandbox)
        .bind(merchant_id)
        .bind(source_sandbox)
        .execute(&self.db_pool)
        .await?;

        // Mirror merchant_forwarding_wallets
        sqlx::query(
            r#"
            INSERT INTO merchant_forwarding_wallets (merchant_id, crypto_type, network, address, is_active, sandbox_mode)
            SELECT merchant_id, crypto_type, network, address, is_active, $1
            FROM merchant_forwarding_wallets
            WHERE merchant_id = $2 AND sandbox_mode = $3 AND address != ''
            ON CONFLICT (merchant_id, crypto_type, sandbox_mode) DO NOTHING
            "#
        )
        .bind(target_sandbox)
        .bind(merchant_id)
        .bind(source_sandbox)
        .execute(&self.db_pool)
        .await?;

        tracing::info!("Environment switch for merchant {}: mirrored wallet configs from sandbox_mode={} to sandbox_mode={}", merchant_id, source_sandbox, target_sandbox);

        // Check if the target environment already has a key
        let merchant = sqlx::query_as::<_, Merchant>(
            r#"
            SELECT id, email, business_name, live_api_key_hash, test_api_key_hash, password_hash, 
                   fee_percentage, customer_pays_fee, is_active, sandbox_mode, settlement_mode, 
                   kyc_verified, created_at, updated_at, api_key_expires_at, daily_limit_usd, 
                   role::text as role, redirect_url, first_name, last_name, gender, phone_number, 
                   country, applicant_role, business_country, business_license_number, 
                   business_certificate_url, terms_accepted, wallets_locked, customer_wallets_locked 
            FROM merchants WHERE id = $1

            "#

        )
        .bind(merchant_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or(ServiceError::MerchantNotFound)?;

        let target_hash = if to_live { &merchant.live_api_key_hash } else { &merchant.test_api_key_hash };
        let has_key = target_hash.as_ref().map(|h| h != "PENDING" && !h.is_empty()).unwrap_or(false);

        if has_key {
            // Key exists — no regeneration needed.
            // Dashboard authentication will now rely on the persistent session token,
            // not on possessing this specific API key.
            tracing::info!("Environment switch for merchant {}: to_live={}, existing key found — no regeneration", merchant_id, to_live);
            Ok(None)
        } else {
            // First time in this environment — generate a key
            tracing::info!("Environment switch for merchant {}: to_live={}, generating first-time key", merchant_id, to_live);
            let key = self.generate_and_store_api_key_with_expiry(merchant_id, to_live, merchant.api_key_expires_at).await?;
            Ok(Some(key))
        }
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
            r#"
            SELECT id, email, business_name, live_api_key_hash, test_api_key_hash, password_hash, 
                   fee_percentage, customer_pays_fee, is_active, sandbox_mode, settlement_mode, 
                   kyc_verified, created_at, updated_at, api_key_expires_at, daily_limit_usd, 
                   role::text as role, redirect_url, first_name, last_name, gender, phone_number, 
                   country, applicant_role, business_country, business_license_number, 
                   business_certificate_url, terms_accepted 
            FROM merchants WHERE id = $1
            "#

        )
        .bind(merchant_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or(ServiceError::MerchantNotFound)?;
        
        // Determine if old key is live or test based on prefix
        // Since we are moving to standard prefixes, we can check.
        let is_old_live = old_api_key.starts_with("sk_live_");
        
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

    /// Rotate API key for a specific environment (trusted caller)
    /// 
    /// Generates a new API key for the specified environment without requiring
    /// the old key for verification. Should only be used by authenticated
    /// dashboard sessions.
    pub async fn rotate_api_key_by_env(
        &self,
        merchant_id: i64,
        is_live: bool,
    ) -> Result<String, ServiceError> {
        // Generate a new searchable API key
        let new_api_key = ApiKeyGenerator::generate_session_key(merchant_id, is_live);
        
        // Hash the new API key
        use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
        use rand::rngs::OsRng;
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let new_api_key_hash = argon2.hash_password(new_api_key.as_bytes(), &salt)
            .map_err(|_| ServiceError::InternalError("Failed to hash API key".to_string()))?
            .to_string();
        
        // Update the merchant with the new API key hash
        let update_query = if is_live {
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
        // Searchable token logic (Strictly sk_sandbox_ or sk_live_)
        if token.starts_with("sk_sandbox_") || token.starts_with("sk_live_") {
            let parts: Vec<&str> = token.split('_').collect();
            
            // Expected format: prefix_type_id_random (e.g., sk_sandbox_123_...)
            if parts.len() >= 3 {
                // For all our searchable prefixes, the ID is at index 2
                if let Some(id_str) = parts.get(2) {
                    if let Ok(merchant_id) = id_str.parse::<i64>() {
                        let merchant = sqlx::query_as::<_, Merchant>(
                            r#"
                            SELECT id, email, business_name, live_api_key_hash, test_api_key_hash, password_hash, 
                                   fee_percentage, customer_pays_fee, is_active, sandbox_mode, settlement_mode, 
                                   kyc_verified, created_at, updated_at, api_key_expires_at, daily_limit_usd, 
                                   role::text as role, redirect_url, first_name, last_name, gender, phone_number, 
                                   country, applicant_role, business_country, business_license_number, 
                                   business_certificate_url, terms_accepted, wallets_locked, customer_wallets_locked 
                            FROM merchants 
                            WHERE id = $1 AND is_active = true
                            "#
                        )
                        .bind(merchant_id)
                        .fetch_optional(&self.db_pool)
                        .await
                        .map_err(ServiceError::Database)?;

                        if let Some(merchant) = merchant {
                            use argon2::{Argon2, PasswordHash, PasswordVerifier};
                            use chrono::Utc;
                            
                            // Determine which hash to check based on prefix
                            let is_live_prefix = token.starts_with("sk_live_");
                            let hash_to_check = if is_live_prefix {
                                merchant.live_api_key_hash.as_ref()
                            } else {
                                merchant.test_api_key_hash.as_ref()
                            };

                            if let Some(hash_str) = hash_to_check {
                                if let Ok(parsed_hash) = PasswordHash::new(hash_str) {
                                    if Argon2::default().verify_password(token.as_bytes(), &parsed_hash).is_ok() {
                                        if let Some(expires_at) = merchant.api_key_expires_at {
                                            if Utc::now() > expires_at {
                                                tracing::warn!("API key for merchant {} expired", merchant.id);
                                                return Err(ServiceError::InvalidApiKey);
                                            }
                                        }
                                        tracing::info!("Authenticated merchant {} via {}", merchant.id, if is_live_prefix { "Live" } else { "Sandbox" });
                                        return Ok(merchant);
                                    }
                                }
                            }
                            tracing::warn!("Authentication failed for merchant {} (prefix mismatch or invalid key)", merchant_id);
                        }
                    }
                }
            }
        }

        tracing::warn!("Rejecting malformed or unsupported API key format");
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
        
        let is_live = api_key.starts_with("sk_live_");

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
        crate::utils::validation::validate_wallet_address(&address, crypto_type)?;
        
        // Fetch current sandbox mode for this merchant
        let sandbox_mode = sqlx::query_scalar::<_, bool>("SELECT sandbox_mode FROM merchants WHERE id = $1")
            .bind(merchant_id)
            .fetch_one(&self.db_pool)
            .await
            .unwrap_or(false);

        // Get the network name for this crypto type
        let network = crypto_type.network();
        let crypto_type_str = crypto_type.to_string();
         // 1. Check if the merchant has wallets locked
        let wallets_locked = sqlx::query_scalar::<_, bool>(
            "SELECT wallets_locked FROM merchants WHERE id = $1"
        )
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // 2. Fetch current wallet if it exists
        let current_wallet = sqlx::query(
            "SELECT address, wallet_mode, encrypted_private_key, is_active FROM merchant_wallets WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"
        )
        .bind(merchant_id)
        .bind(&crypto_type_str)
        .bind(sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if let Some(row) = current_wallet {
            let current_address: String = row.get("address");
            let current_mode: Option<String> = row.get("wallet_mode");
            let current_key: Option<String> = row.get("encrypted_private_key");
            let current_active: bool = row.get("is_active");

            if current_address != address {
                if wallets_locked {
                    return Err(ServiceError::BadRequest(
                        "Wallets are locked. Please unlock in settings to change address.".to_string()
                    ));
                }

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
                .bind(network)
                .bind(&current_address)
                .bind(&address)
                .bind(current_mode.as_deref().unwrap_or("address_only"))
                .bind(&current_key)
                .bind(current_active)
                .bind("Updated via settings")
                .execute(&self.db_pool)
                .await?;
            }
        }

        // Insert or update the wallet address
        // Use ON CONFLICT to update if the merchant already has a wallet for this crypto type and mode
        sqlx::query(
            r#"
            INSERT INTO merchant_wallets (merchant_id, crypto_type, network, address, is_active, sandbox_mode, wallet_mode, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, 'address_only', $7, $8)
            ON CONFLICT (merchant_id, crypto_type, sandbox_mode)
            DO UPDATE SET 
                address = EXCLUDED.address,
                network = EXCLUDED.network,
                is_active = EXCLUDED.is_active,
                wallet_mode = EXCLUDED.wallet_mode,
                updated_at = EXCLUDED.updated_at
            "#
        )
        .bind(merchant_id)
        .bind(&crypto_type_str)
        .bind(network)
        .bind(&address)
        .bind(true) // is_active
        .bind(sandbox_mode)
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(&self.db_pool)
        .await?;

        // Log wallet address change
        self.audit_service.log_event(
            merchant_id,
            "wallet_address_update",
            Some(&format!("Updated {} wallet address", crypto_type_str)),
            Some(serde_json::json!({
                "crypto_type": crypto_type_str,
                "address": address,
                "sandbox_mode": sandbox_mode
            }))
        ).await;
        tracing::info!("EVENT: wallet_address_update | Merchant: {} | Crypto: {} | Sandbox: {}", merchant_id, crypto_type_str, sandbox_mode);
        
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
        let lookup_crypto_type = crypto_type.get_native_currency().to_string();

        // First, check settlement mode and sandbox mode
        use sqlx::Row;
        let merchant = sqlx::query(
            "SELECT settlement_mode, sandbox_mode, wallets_locked, customer_wallets_locked FROM merchants WHERE id = $1"
        )
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await?;

        let settlement_mode: String = merchant.get("settlement_mode");
        let sandbox_mode_val: bool = merchant.get("sandbox_mode");

        if settlement_mode == "forwarding" {
            // FORWARDING MODE: Look in merchant_forwarding_wallets
            let wallet_opt = sqlx::query(
                "SELECT address FROM merchant_forwarding_wallets 
                 WHERE merchant_id = $1 AND crypto_type = $2 AND is_active = true AND sandbox_mode = $3"
            )
            .bind(merchant_id)
            .bind(&lookup_crypto_type)
            .bind(sandbox_mode_val)
            .fetch_optional(&self.db_pool)
            .await?;

            if let Some(wallet) = wallet_opt {
                let addr: String = wallet.get("address");
                return Ok(addr);
            } else {
                return Err(ServiceError::WalletNotFound);
            }
        } 
        
        // MANAGED MODE: Look in merchant_wallets
        
        let wallet_opt = sqlx::query(
            "SELECT address FROM merchant_wallets 
             WHERE merchant_id = $1 AND crypto_type = $2 AND is_active = true AND sandbox_mode = $3"
        )
        .bind(merchant_id)
        .bind(&lookup_crypto_type)
        .bind(sandbox_mode_val)
        .fetch_optional(&self.db_pool)
        .await?;
        
        if let Some(wallet) = wallet_opt {
            let addr: String = wallet.get("address");
            return Ok(addr);
        }

        // If not found in merchant_wallets, auto-generate ONLY if managed mode
        if settlement_mode == "managed" {
            tracing::info!("Auto-generating wallet for merchant {} for network {}", merchant_id, lookup_crypto_type);
            
            let wallet_service = crate::services::wallet_config_service::WalletConfigService::new(self.db_pool.clone());
            let gen_req = crate::services::wallet_config_service::GenerateWalletRequest {
                crypto_type: lookup_crypto_type.to_string(),
            };
            
            let response = wallet_service.generate_wallet(merchant_id, sandbox_mode_val, gen_req).await?;
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
        if !["forwarding", "managed"].contains(&mode) {
            return Err(ServiceError::ValidationError("Invalid settlement mode".to_string()));
        }

        sqlx::query(
            "UPDATE merchants SET settlement_mode = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(mode)
        .bind(Utc::now())
        .bind(merchant_id)
        .execute(&self.db_pool)
        .await?;

        // Log settlement mode update
        self.audit_service.log_event(
            merchant_id,
            "settlement_mode_update",
            Some(&format!("Updated settlement mode to {}", mode)),
            Some(serde_json::json!({"settlement_mode": mode}))
        ).await;
        tracing::info!("EVENT: settlement_mode_update | Merchant: {} | Mode: {}", merchant_id, mode);

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
            if !["forwarding", "managed"].contains(&mode.as_str()) {
                return Err(ServiceError::ValidationError("Invalid settlement mode".to_string()));
            }
        }

        sqlx::query(
            r#"
            UPDATE merchants 
            SET 
                settlement_mode = COALESCE($1, settlement_mode),
                customer_pays_fee = COALESCE($2, customer_pays_fee),
                sandbox_mode = COALESCE($3, sandbox_mode),
                redirect_url = COALESCE($4, redirect_url),
                updated_at = $5
            WHERE id = $6
            "#
        )
        .bind(&settlement_mode)
        .bind(customer_pays_fee)
        .bind(sandbox_mode)
        .bind(&redirect_url)
        .bind(Utc::now())
        .bind(merchant_id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Toggle the wallet lock status for a merchant
    pub async fn set_wallet_lock(
        &self,
        merchant_id: i64,
        locked: bool,
    ) -> Result<(), ServiceError> {
        sqlx::query(
            "UPDATE merchants SET wallets_locked = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(locked)
        .bind(merchant_id)
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }

    /// Toggle the customer wallet lock status for a merchant
    pub async fn set_customer_wallet_lock(
        &self,
        merchant_id: i64,
        locked: bool,
    ) -> Result<(), ServiceError> {
        sqlx::query(
            "UPDATE merchants SET customer_wallets_locked = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(locked)
        .bind(merchant_id)
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
}
