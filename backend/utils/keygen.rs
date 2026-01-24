use ed25519_dalek::SigningKey;
use secp256k1::{Secp256k1, PublicKey};
use rand::rngs::OsRng;
use rand::RngCore;

pub struct KeyPair {
    pub address: String,
    pub private_key: String,
}

/// Generate Solana keypair (Ed25519)
pub fn generate_solana_keypair() -> Result<KeyPair, String> {
    let mut secret_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut secret_bytes);
    
    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let verifying_key = signing_key.verifying_key();

    // Solana address is base58 encoded public key
    let address = bs58::encode(verifying_key.as_bytes()).into_string();
    
    // Private key is base58 encoded secret key
    let private_key = bs58::encode(signing_key.to_bytes()).into_string();

    Ok(KeyPair { address, private_key })
}

/// Generate EVM keypair (secp256k1) for BSC, Arbitrum, Polygon
pub fn generate_evm_keypair() -> Result<KeyPair, String> {
    let secp = Secp256k1::new();
    let mut rng = OsRng;
    
    let (secret_key, public_key) = secp.generate_keypair(&mut rng);

    // EVM address is last 20 bytes of keccak256(public_key)
    let address = public_key_to_address(&public_key);
    
    // Private key as hex
    let private_key = hex::encode(secret_key.secret_bytes());

    Ok(KeyPair { address, private_key })
}

fn public_key_to_address(public_key: &PublicKey) -> String {
    // Get uncompressed public key (65 bytes: 0x04 + 32 bytes x + 32 bytes y)
    let public_key_bytes = public_key.serialize_uncompressed();
    
    // Skip the 0x04 prefix, hash the remaining 64 bytes
    let hash = keccak256(&public_key_bytes[1..]);
    
    // Take last 20 bytes and format as 0x...
    format!("0x{}", hex::encode(&hash[12..]))
}

fn keccak256(data: &[u8]) -> [u8; 32] {
    use tiny_keccak::{Hasher, Keccak};
    let mut hasher = Keccak::v256();
    let mut output = [0u8; 32];
    hasher.update(data);
    hasher.finalize(&mut output);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solana_keypair_generation() {
        let keypair = generate_solana_keypair().unwrap();
        assert!(!keypair.address.is_empty());
        assert!(!keypair.private_key.is_empty());
        assert!(keypair.address.len() >= 32);
    }

    #[test]
    fn test_evm_keypair_generation() {
        let keypair = generate_evm_keypair().unwrap();
        assert!(keypair.address.starts_with("0x"));
        assert_eq!(keypair.address.len(), 42); // 0x + 40 hex chars
        assert_eq!(keypair.private_key.len(), 64); // 32 bytes as hex
    }
}
