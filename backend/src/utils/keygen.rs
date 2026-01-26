// Key Generation Utilities - Hybrid Non-Custodial System
// Generates private keys for EVM and Solana networks

use crate::error::ServiceError;
use crate::utils::encryption::encrypt_data;
use rand::rngs::OsRng;
use secp256k1::{Secp256k1, SecretKey};
use tiny_keccak::{Hasher, Keccak};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedWallet {
    pub address: String,
    pub encrypted_private_key: String,
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletKeyPair {
    pub private_key: String,
    pub public_key: String,
    pub address: String,
}

pub struct KeyGenerator;

impl KeyGenerator {
    /// Generate EVM wallet (Ethereum, BSC, Polygon, Arbitrum)
    pub fn generate_evm_wallet() -> Result<WalletKeyPair, ServiceError> {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        
        // Convert to hex strings
        let private_key_hex = hex::encode(secret_key.secret_bytes());
        let public_key_hex = hex::encode(public_key.serialize_uncompressed());
        
        // Generate Ethereum address from public key
        let address = Self::public_key_to_eth_address(&public_key_hex)?;
        
        Ok(WalletKeyPair {
            private_key: private_key_hex,
            public_key: public_key_hex,
            address,
        })
    }

    /// Generate Solana wallet (placeholder)
    pub fn generate_solana_wallet() -> Result<WalletKeyPair, ServiceError> {
        // Placeholder implementation without ed25519-dalek
        let private_key = format!("solana_private_key_{}", uuid::Uuid::new_v4());
        let public_key = format!("solana_public_key_{}", uuid::Uuid::new_v4());
        
        Ok(WalletKeyPair {
            private_key,
            public_key: public_key.clone(),
            address: public_key, // In Solana, address = public key
        })
    }

    /// Generate encrypted wallet for storage
    pub fn generate_encrypted_wallet(
        network: &str,
        encryption_key: &str,
    ) -> Result<GeneratedWallet, ServiceError> {
        let wallet = match network {
            "ethereum" | "bsc" | "polygon" | "arbitrum" => Self::generate_evm_wallet()?,
            "solana" => Self::generate_solana_wallet()?,
            _ => return Err(ServiceError::ValidationError(
                format!("Unsupported network: {}", network)
            )),
        };

        // Encrypt the private key
        let encrypted_private_key = encrypt_data(&wallet.private_key)
            .map_err(|e| ServiceError::ValidationError(format!("Encryption failed: {}", e)))?;

        Ok(GeneratedWallet {
            address: wallet.address,
            encrypted_private_key,
            network: network.to_string(),
        })
    }

    /// Validate imported private key
    pub fn validate_private_key(private_key: &str, network: &str) -> Result<String, ServiceError> {
        match network {
            "ethereum" | "bsc" | "polygon" | "arbitrum" => {
                Self::validate_evm_private_key(private_key)
            }
            "solana" => Self::validate_solana_private_key(private_key),
            _ => Err(ServiceError::ValidationError(
                format!("Unsupported network: {}", network)
            )),
        }
    }

    fn validate_evm_private_key(private_key: &str) -> Result<String, ServiceError> {
        // Remove 0x prefix if present
        let key = private_key.strip_prefix("0x").unwrap_or(private_key);
        
        // Validate hex format and length (64 characters = 32 bytes)
        if key.len() != 64 {
            return Err(ServiceError::ValidationError(
                "EVM private key must be 64 hex characters".to_string()
            ));
        }

        // Try to parse as secret key
        let key_bytes = hex::decode(key)
            .map_err(|_| ServiceError::ValidationError("Invalid hex format".to_string()))?;
        
        let secret_key = SecretKey::from_slice(&key_bytes)
            .map_err(|_| ServiceError::ValidationError("Invalid private key".to_string()))?;

        let secp = Secp256k1::new();
        let public_key = secret_key.public_key(&secp);
        let public_key_hex = hex::encode(public_key.serialize_uncompressed());
        
        // Generate address to verify key is valid
        let address = Self::public_key_to_eth_address(&public_key_hex)?;
        
        Ok(address)
    }

    fn validate_solana_private_key(private_key: &str) -> Result<String, ServiceError> {
        // Placeholder validation for Solana keys
        if private_key.len() < 32 {
            return Err(ServiceError::ValidationError(
                "Solana private key too short".to_string()
            ));
        }
        
        // Return a placeholder address
        Ok(format!("solana_address_for_{}", &private_key[..8]))
    }

    fn public_key_to_eth_address(public_key_hex: &str) -> Result<String, ServiceError> {
        // Remove 0x04 prefix (uncompressed public key indicator)
        let public_key = public_key_hex.strip_prefix("04").unwrap_or(public_key_hex);
        
        let public_key_bytes = hex::decode(public_key)
            .map_err(|_| ServiceError::ValidationError("Invalid public key hex".to_string()))?;

        // Keccak256 hash of public key
        let mut hasher = Keccak::v256();
        hasher.update(&public_key_bytes);
        let mut hash = [0u8; 32];
        hasher.finalize(&mut hash);

        // Take last 20 bytes and add 0x prefix
        let address = format!("0x{}", hex::encode(&hash[12..]));
        
        Ok(address)
    }

    /// Generate master encryption key for merchant
    pub fn generate_merchant_encryption_key() -> String {
        let mut key = [0u8; 32];
        rand::RngCore::fill_bytes(&mut OsRng, &mut key);
        hex::encode(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_evm_wallet() {
        let wallet = KeyGenerator::generate_evm_wallet().unwrap();
        assert_eq!(wallet.private_key.len(), 64); // 32 bytes in hex
        assert!(wallet.address.starts_with("0x"));
        assert_eq!(wallet.address.len(), 42); // 0x + 40 hex chars
    }

    #[test]
    fn test_generate_solana_wallet() {
        let wallet = KeyGenerator::generate_solana_wallet().unwrap();
        assert!(!wallet.private_key.is_empty());
        assert!(!wallet.address.is_empty());
        assert_eq!(wallet.address, wallet.public_key); // In Solana, address = public key
    }

    #[test]
    fn test_validate_evm_private_key() {
        // Valid private key
        let valid_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let result = KeyGenerator::validate_evm_private_key(valid_key);
        assert!(result.is_ok());

        // Invalid length
        let invalid_key = "0123456789abcdef";
        let result = KeyGenerator::validate_evm_private_key(invalid_key);
        assert!(result.is_err());
    }
}
