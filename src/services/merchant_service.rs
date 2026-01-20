// Merchant Service
// Business logic for merchant management

use crate::error::ServiceError;
use crate::models::merchant::{Merchant, MerchantRegistrationResponse, MerchantWallet};
use crate::payment::models::CryptoType;
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use nanoid::nanoid;
use rust_decimal::Decimal;
use sqlx::PgPool;

pub struct MerchantService {
    db_pool: PgPool,
}

impl MerchantService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Register a new merchant
    /// 
    /// Creates a new merchant account with a unique identifier and generates
    /// a secure API key for authentication.
    /// 
    /// # Arguments
    /// * `email` - Merchant's email address
    /// * `business_name` - Name of the merchant's business
    /// 
    /// # Returns
    /// * `MerchantRegistrationResponse` containing merchant_id and api_key
    /// 
    /// # Requirements
    /// * 1.1: Creates merchant account with unique identifier
    /// * 1.2: Generates unique API key for authentication
    pub async fn register_merchant(
        &self,
        email: String,
        business_name: String,
    ) -> Result<MerchantRegistrationResponse, ServiceError> {
        // Generate a secure random API key (32 characters)
        let api_key = self.generate_api_key();
        
        // Hash the API key using bcrypt before storing
        let api_key_hash = hash(&api_key, DEFAULT_COST)
            .map_err(|e| ServiceError::Internal(format!("Failed to hash API key: {}", e)))?;
        
        // Default fee percentage is 1.50%
        let fee_percentage = Decimal::new(150, 2);
        
        // Insert merchant into database
        let merchant = sqlx::query_as::<_, Merchant>(
            r#"
            INSERT INTO merchants (email, business_name, api_key_hash, fee_percentage, is_active, sandbox_mode, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, email, business_name, api_key_hash, fee_percentage, is_active, sandbox_mode, created_at, updated_at
            "#
        )
        .bind(&email)
        .bind(&business_name)
        .bind(&api_key_hash)
        .bind(fee_percentage)
        .bind(true) // is_active
        .bind(false) // sandbox_mode
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(MerchantRegistrationResponse {
            merchant_id: merchant.id,
            api_key,
        })
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
        old_api_key: String,
    ) -> Result<String, ServiceError> {
        // First, verify the old API key is correct
        let merchant = sqlx::query_as::<_, Merchant>(
            "SELECT id, email, business_name, api_key_hash, fee_percentage, is_active, sandbox_mode, created_at, updated_at FROM merchants WHERE id = $1"
        )
        .bind(merchant_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or(ServiceError::MerchantNotFound)?;
        
        // Verify the old API key matches
        if !bcrypt::verify(&old_api_key, &merchant.api_key_hash)
            .map_err(|e| ServiceError::Internal(format!("Failed to verify API key: {}", e)))? {
            return Err(ServiceError::InvalidApiKey);
        }
        
        // Generate new API key
        let new_api_key = self.generate_api_key();
        
        // Hash the new API key
        let new_api_key_hash = hash(&new_api_key, DEFAULT_COST)
            .map_err(|e| ServiceError::Internal(format!("Failed to hash API key: {}", e)))?;
        
        // Update the merchant's API key in the database
        sqlx::query(
            "UPDATE merchants SET api_key_hash = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(&new_api_key_hash)
        .bind(Utc::now())
        .bind(merchant_id)
        .execute(&self.db_pool)
        .await?;
        
        Ok(new_api_key)
    }

    /// Authenticate a merchant using their API key
    /// 
    /// Validates the provided API key against stored bcrypt hash and
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
    /// * 1.2: Use bcrypt verification for API keys
    pub async fn authenticate(
        &self,
        api_key: &str,
    ) -> Result<Merchant, ServiceError> {
        // Query all merchants and check each hash
        // Note: This is not the most efficient approach for large numbers of merchants,
        // but it's secure and works well for reasonable scale.
        // For very large scale, consider adding an index on a hash of the first few
        // characters of the API key.
        let merchants = sqlx::query_as::<_, Merchant>(
            "SELECT id, email, business_name, api_key_hash, fee_percentage, is_active, sandbox_mode, created_at, updated_at FROM merchants WHERE is_active = true"
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        // Check each merchant's API key hash
        for merchant in merchants {
            if bcrypt::verify(api_key, &merchant.api_key_hash).unwrap_or(false) {
                return Ok(merchant);
            }
        }
        
        // No matching API key found
        Err(ServiceError::InvalidApiKey)
    }

    /// Generate a secure random API key
    /// 
    /// Creates a unique API key using nanoid with a custom alphabet
    /// that is URL-safe and easy to read.
    /// 
    /// # Returns
    /// * A 32-character random string suitable for use as an API key
    fn generate_api_key(&self) -> String {
        // Use nanoid with custom alphabet (alphanumeric, no ambiguous characters)
        // Length: 32 characters for high entropy
        const ALPHABET: [char; 62] = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J',
            'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
            'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd',
            'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n',
            'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',
            'y', 'z',
        ];
        
        nanoid!(32, &ALPHABET)
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
        let crypto_type_str = format!("{:?}", crypto_type);
        
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
        let crypto_type_str = format!("{:?}", crypto_type);
        
        let wallet = sqlx::query_as::<_, MerchantWallet>(
            "SELECT id, merchant_id, crypto_type, network, address, is_active, created_at, updated_at 
             FROM merchant_wallets 
             WHERE merchant_id = $1 AND crypto_type = $2 AND is_active = true"
        )
        .bind(merchant_id)
        .bind(&crypto_type_str)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or(ServiceError::WalletNotFound)?;
        
        Ok(wallet.address)
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
            CryptoType::UsdtBep20 | CryptoType::UsdtArbitrum | CryptoType::UsdtPolygon => {
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

    #[test]
    fn test_generate_api_key_length() {
        // Create a mock pool (we don't need a real connection for this test)
        let service = MerchantService {
            db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };
        
        let api_key = service.generate_api_key();
        assert_eq!(api_key.len(), 32);
    }

    #[test]
    fn test_generate_api_key_uniqueness() {
        let service = MerchantService {
            db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };
        
        let key1 = service.generate_api_key();
        let key2 = service.generate_api_key();
        
        // Keys should be different
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_generate_api_key_alphanumeric() {
        let service = MerchantService {
            db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };
        
        let api_key = service.generate_api_key();
        
        // All characters should be alphanumeric
        assert!(api_key.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_api_key_hashing() {
        // Test that bcrypt hashing works correctly
        let api_key = "test_api_key_12345678901234567890";
        let hash1 = hash(api_key, DEFAULT_COST).unwrap();
        let hash2 = hash(api_key, DEFAULT_COST).unwrap();
        
        // Hashes should be different (bcrypt uses random salt)
        assert_ne!(hash1, hash2);
        
        // But both should verify against the original key
        assert!(bcrypt::verify(api_key, &hash1).unwrap());
        assert!(bcrypt::verify(api_key, &hash2).unwrap());
    }

    #[test]
    fn test_default_fee_percentage() {
        // Test that default fee percentage is 1.50%
        let fee = Decimal::new(150, 2);
        assert_eq!(fee.to_string(), "1.50");
    }

    // Wallet address validation tests
    #[test]
    fn test_validate_solana_address_valid() {
        let service = MerchantService {
            db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };
        
        // Valid Solana address (base58, 32-44 chars)
        let valid_address = "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU";
        let result = service.validate_wallet_address(valid_address, CryptoType::Sol);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_solana_address_too_short() {
        let service = MerchantService {
            db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };
        
        // Too short
        let invalid_address = "7xKXtg2CW87d97TXJSDpbD5jBkhe";
        let result = service.validate_wallet_address(invalid_address, CryptoType::Sol);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_solana_address_invalid_chars() {
        let service = MerchantService {
            db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };
        
        // Contains invalid base58 characters (0, O, I, l)
        let invalid_address = "0OIl7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU";
        let result = service.validate_wallet_address(invalid_address, CryptoType::Sol);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_evm_address_valid() {
        let service = MerchantService {
            db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
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

    #[test]
    fn test_validate_evm_address_no_prefix() {
        let service = MerchantService {
            db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };
        
        // Missing 0x prefix
        let invalid_address = "742d35Cc6634C0532925a3b844Bc9e7595f0bEb0";
        let result = service.validate_wallet_address(invalid_address, CryptoType::UsdtBep20);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_evm_address_wrong_length() {
        let service = MerchantService {
            db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
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

    #[test]
    fn test_validate_evm_address_invalid_hex() {
        let service = MerchantService {
            db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };
        
        // Contains non-hex characters (g, z)
        let invalid_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEgz";
        let result = service.validate_wallet_address(invalid_address, CryptoType::UsdtBep20);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_all_crypto_types() {
        let service = MerchantService {
            db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
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
