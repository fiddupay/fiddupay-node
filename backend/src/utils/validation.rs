use crate::payment::models::CryptoType;
use crate::error::ServiceError;

pub fn validate_wallet_address(
    address: &str,
    crypto_type: CryptoType,
) -> Result<(), ServiceError> {
    match crypto_type {
        CryptoType::Sol | CryptoType::UsdtSpl | CryptoType::WSol => {
            // Solana addresses are base58 encoded, typically 32-44 characters
            if address.len() < 32 || address.len() > 44 {
                return Err(ServiceError::ValidationError(
                    "Solana address must be 32-44 characters".to_string()
                ));
            }
            
            // Check if all characters are valid base58
            const BASE58_ALPHABET: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
            if !address.chars().all(|c| BASE58_ALPHABET.contains(c)) {
                return Err(ServiceError::ValidationError(
                    "Solana address contains invalid base58 characters".to_string()
                ));
            }
        }
        CryptoType::UsdtBep20 | CryptoType::UsdtArbitrum | CryptoType::UsdtPolygon | CryptoType::UsdtEth | CryptoType::Eth | CryptoType::Arb | CryptoType::Matic | CryptoType::Bnb => {
            // EVM addresses start with 0x and have 40 hex characters
            if !address.starts_with("0x") {
                return Err(ServiceError::ValidationError(
                    "EVM address must start with 0x".to_string()
                ));
            }
            
            if address.len() != 42 {
                return Err(ServiceError::ValidationError(
                    "EVM address must be 42 characters (0x + 40 hex chars)".to_string()
                ));
            }
            
            // Check if all characters after 0x are valid hex
            let hex_part = &address[2..];
            if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(ServiceError::ValidationError(
                    "EVM address contains invalid hexadecimal characters".to_string()
                ));
            }
        }
    }
    
    Ok(())
}
