use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use std::env;

pub fn encrypt_data(data: &str) -> Result<String, String> {
    let encryption = Encryption::new()?;
    encryption.encrypt(data)
}

pub struct Encryption {
    cipher: Aes256Gcm,
}

impl Encryption {
    pub fn new() -> Result<Self, String> {
        let key_hex = env::var("ENCRYPTION_KEY")
            .map_err(|_| "ENCRYPTION_KEY not set in environment".to_string())?;
        
        let key_bytes = hex::decode(&key_hex)
            .map_err(|e| format!("Invalid ENCRYPTION_KEY hex: {}", e))?;
        
        if key_bytes.len() != 32 {
            return Err("ENCRYPTION_KEY must be 32 bytes (64 hex chars)".to_string());
        }

        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        Ok(Self { cipher })
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String, String> {
        // Generate random nonce
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        // Combine nonce + ciphertext and encode
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        
        Ok(BASE64.encode(&result))
    }

    pub fn decrypt(&self, encrypted: &str) -> Result<String, String> {
        // Decode
        let data = BASE64.decode(encrypted)
            .map_err(|e| format!("Base64 decode failed: {}", e))?;

        if data.len() < 12 {
            return Err("Invalid encrypted data".to_string());
        }

        // Split nonce and ciphertext
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext)
            .map_err(|e| format!("Invalid UTF-8: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        std::env::set_var("ENCRYPTION_KEY", "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
        
        let enc = Encryption::new().unwrap();
        let plaintext = "secret data";
        
        let encrypted = enc.encrypt(plaintext).unwrap();
        let decrypted = enc.decrypt(&encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }
}
