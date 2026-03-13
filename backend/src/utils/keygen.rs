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

    /// Generate Solana wallet using solana-sdk
    pub fn generate_solana_wallet() -> Result<WalletKeyPair, ServiceError> {
        use solana_sdk::signature::{Keypair, Signer};
        
        // Generate a new keypair
        let keypair = Keypair::new();
        let public_key_bytes = keypair.pubkey().to_bytes();
        let secret_key_bytes = keypair.to_bytes(); // This is the 64-byte [secret; public] array
        
        let private_key_base58 = bs58::encode(secret_key_bytes).into_string();
        let address = bs58::encode(public_key_bytes).into_string();
        
        Ok(WalletKeyPair {
            private_key: private_key_base58,
            public_key: address.clone(),
            address,
        })
    }

    /// Generate Bitcoin wallet (SegWit/Bech32)
    pub fn generate_btc_wallet(is_sandbox: bool) -> Result<WalletKeyPair, ServiceError> {
        use bitcoin::key::Secp256k1;
        use bitcoin::{Network, PrivateKey, Address};
        use bitcoin::secp256k1::SecretKey;
        use bitcoin::PublicKey as BitcoinPublicKey;

        let network = if is_sandbox { Network::Testnet } else { Network::Bitcoin };
        let secp = Secp256k1::new();
        let mut entropy = [0u8; 32];
        rand::RngCore::fill_bytes(&mut OsRng, &mut entropy);
        let secret_key = SecretKey::from_slice(&entropy).map_err(|e| ServiceError::Internal(format!("BTC key generation failed: {}", e)))?;
        
        let private_key = PrivateKey::new(secret_key, network);
        let public_key = BitcoinPublicKey::new(private_key.public_key(&secp).inner);
        
        // Generate SegWit address (Bech32)
        let compressed_public_key = bitcoin::CompressedPublicKey::try_from(public_key)
            .map_err(|_| ServiceError::Internal("Failed to create compressed public key".to_string()))?;
        let address = Address::p2wpkh(&compressed_public_key, network);
        
        Ok(WalletKeyPair {
            private_key: private_key.to_wif(),
            public_key: public_key.to_string(),
            address: address.to_string(),
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
            "bitcoin" => Self::generate_btc_wallet(false)?, // Default to mainnet for this generic helper
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
            "bitcoin" => Self::validate_btc_private_key(private_key),
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

    fn validate_solana_private_key(private_key_base58: &str) -> Result<String, ServiceError> {
        use solana_sdk::signature::{Keypair, Signer, SeedDerivable};

        // Decode base58 private key
        let key_bytes = bs58::decode(private_key_base58)
            .into_vec()
            .map_err(|_| ServiceError::ValidationError("Invalid Base58 format".to_string()))?;

        // Solana private keys can be 32 bytes (seed) or 64 bytes (seed + pubkey)
        let public_key_bytes = match key_bytes.len() {
            32 => {
                let seed: [u8; 32] = key_bytes.try_into().unwrap();
                let keypair = Keypair::from_seed(&seed)
                    .map_err(|_| ServiceError::ValidationError("Invalid Solana seed".to_string()))?;
                keypair.pubkey().to_bytes()
            }
            64 => {
                let bytes: [u8; 64] = key_bytes.try_into().unwrap();
                let keypair = Keypair::try_from(&bytes[..]) // Use slice reference instead of array
                    .map_err(|_| ServiceError::ValidationError("Invalid Solana private key bytes".to_string()))?;
                keypair.pubkey().to_bytes()
            }
            _ => {
                return Err(ServiceError::ValidationError(
                    "Solana private key must be 32 or 64 bytes (Base58 encoded)".to_string()
                ));
            }
        };

        Ok(bs58::encode(public_key_bytes).into_string())
    }

    fn validate_btc_private_key(wif: &str) -> Result<String, ServiceError> {
        use bitcoin::{PrivateKey, Network};
        
        let private_key = PrivateKey::from_wif(wif)
            .map_err(|_| ServiceError::ValidationError("Invalid BTC WIF private key".to_string()))?;
        
        // We assume Mainnet if not specified, but this should be configurable
        let secp = bitcoin::key::Secp256k1::new();
        let public_key = bitcoin::PublicKey::new(private_key.public_key(&secp).inner);
        let compressed_public_key = bitcoin::CompressedPublicKey::try_from(public_key)
            .map_err(|_| ServiceError::ValidationError("Invalid BTC public key for compressed format".to_string()))?;
        let address = bitcoin::Address::p2wpkh(&compressed_public_key, Network::Bitcoin);
            
        Ok(address.to_string())
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

