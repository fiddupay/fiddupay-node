use nanoid::nanoid;

/// Centralized API key generation utility
/// Single source of truth for all API key generation in the system
pub struct ApiKeyGenerator;

impl ApiKeyGenerator {
    const ALPHABET: [char; 62] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J',
        'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
        'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd',
        'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n',
        'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',
        'y', 'z',
    ];

    /// Generate sandbox API key (sk_ prefix)
    pub fn generate_sandbox_key() -> String {
        format!("sk_{}", nanoid!(32, &Self::ALPHABET))
    }

    /// Generate live API key (live_ prefix)
    pub fn generate_live_key() -> String {
        format!("live_{}", nanoid!(32, &Self::ALPHABET))
    }

    /// Generate API key based on environment
    pub fn generate_key(is_live: bool) -> String {
        if is_live {
            Self::generate_live_key()
        } else {
            Self::generate_sandbox_key()
        }
    }

    /// Generate payment ID
    pub fn generate_payment_id() -> String {
        format!("pay_{}", nanoid!())
    }

    /// Generate invoice ID
    pub fn generate_invoice_id() -> String {
        format!("inv_{}", nanoid!(12))
    }

    /// Generate refund ID
    pub fn generate_refund_id() -> String {
        format!("ref_{}", nanoid!(16))
    }
}
