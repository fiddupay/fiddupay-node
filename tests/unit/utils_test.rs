// fiddupay - Unit Tests
// Tests for core utilities and helpers

use crypto_payment_gateway::utils::{encryption::Encryption, keygen::*};

#[test]
fn test_encryption_roundtrip() {
    std::env::set_var("ENCRYPTION_KEY", "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
    
    let enc = Encryption::new().expect("Failed to create encryption");
    let plaintext = "secret private key data";
    
    let encrypted = enc.encrypt(plaintext).expect("Encryption failed");
    let decrypted = enc.decrypt(&encrypted).expect("Decryption failed");
    
    assert_eq!(plaintext, decrypted);
}

#[test]
fn test_encryption_different_ciphertext() {
    std::env::set_var("ENCRYPTION_KEY", "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
    
    let enc = Encryption::new().unwrap();
    let plaintext = "same data";
    
    let encrypted1 = enc.encrypt(plaintext).unwrap();
    let encrypted2 = enc.encrypt(plaintext).unwrap();
    
    // Different nonces = different ciphertext
    assert_ne!(encrypted1, encrypted2);
    
    // But both decrypt to same plaintext
    assert_eq!(enc.decrypt(&encrypted1).unwrap(), plaintext);
    assert_eq!(enc.decrypt(&encrypted2).unwrap(), plaintext);
}

#[test]
fn test_solana_keypair_generation() {
    let keypair = generate_solana_keypair().expect("Failed to generate Solana keypair");
    
    assert!(!keypair.address.is_empty());
    assert!(!keypair.private_key.is_empty());
    assert!(keypair.address.len() >= 32);
    assert!(keypair.address.len() <= 44); // Base58 encoded
}

#[test]
fn test_evm_keypair_generation() {
    let keypair = generate_evm_keypair().expect("Failed to generate EVM keypair");
    
    assert!(keypair.address.starts_with("0x"));
    assert_eq!(keypair.address.len(), 42); // 0x + 40 hex chars
    assert_eq!(keypair.private_key.len(), 64); // 32 bytes as hex
}

#[test]
fn test_solana_keypairs_unique() {
    let keypair1 = generate_solana_keypair().unwrap();
    let keypair2 = generate_solana_keypair().unwrap();
    
    assert_ne!(keypair1.address, keypair2.address);
    assert_ne!(keypair1.private_key, keypair2.private_key);
}

#[test]
fn test_evm_keypairs_unique() {
    let keypair1 = generate_evm_keypair().unwrap();
    let keypair2 = generate_evm_keypair().unwrap();
    
    assert_ne!(keypair1.address, keypair2.address);
    assert_ne!(keypair1.private_key, keypair2.private_key);
}

#[test]
fn test_encryption_invalid_key() {
    std::env::set_var("ENCRYPTION_KEY", "invalid");
    
    let result = Encryption::new();
    assert!(result.is_err());
}

#[test]
fn test_encryption_tampered_data() {
    std::env::set_var("ENCRYPTION_KEY", "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
    
    let enc = Encryption::new().unwrap();
    let encrypted = enc.encrypt("data").unwrap();
    
    // Tamper with encrypted data
    let mut tampered = encrypted.clone();
    tampered.push('X');
    
    let result = enc.decrypt(&tampered);
    assert!(result.is_err());
}
