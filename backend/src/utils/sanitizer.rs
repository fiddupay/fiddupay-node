// Global Data Sanitizer
// Scrub sensitive keys from JSON objects before logging or storage
// + PII masking utilities for log-safe string output

use serde_json::Value;

/// Keys that should always be masked in logs and audit trails
const SENSITIVE_KEYS: &[&str] = &[
    "password",
    "password_hash",
    "transaction_pin",
    "transaction_pin_hash",
    "pin",
    "secret",
    "signing_secret",
    "api_key",
    "dashboard_token",
    "private_key",
    "encrypted_private_key",
    "bvn",
    "nin",
    "nin_bvn",
    "nin_bvn_hash",
];

/// Recursively scrub sensitive data from a JSON value
pub fn scrub_sensitive_data(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                if is_sensitive_key(key) {
                    *val = Value::String("********".to_string());
                } else {
                    scrub_sensitive_data(val);
                }
            }
        }
        Value::Array(arr) => {
            for val in arr.iter_mut() {
                scrub_sensitive_data(val);
            }
        }
        _ => {}
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let key_lower = key.to_lowercase();
    SENSITIVE_KEYS.iter().any(|&s| key_lower.contains(s))
}

// =============================================================================
// PII Masking Functions — for use in tracing::info!/warn!/error! log statements
// =============================================================================

/// Mask an email address for safe logging.
/// `username123@domain.com` → `use***123@domain.com`
pub fn mask_email(email: &str) -> String {
    if let Some(at_pos) = email.find('@') {
        let local = &email[..at_pos];
        let domain = &email[at_pos..]; // includes the @

        let masked_local = if local.len() <= 6 {
            // Too short to show both ends — just show first char + ***
            format!("{}***", &local[..1.min(local.len())])
        } else {
            // Show first 3 + *** + last 3
            format!("{}***{}", &local[..3], &local[local.len() - 3..])
        };

        format!("{}{}", masked_local, domain)
    } else {
        // Not a valid email, mask most of it
        if email.len() <= 3 {
            "***".to_string()
        } else {
            format!("{}***", &email[..2])
        }
    }
}

/// Mask an API key for safe logging.
/// `sk_live_abc123def456ghi789` → `sk_live_***789`
pub fn mask_api_key(key: &str) -> String {
    if key.len() <= 6 {
        return "***".to_string();
    }

    // Preserve prefix (sk_live_, sk_test_, etc.) if present
    if let Some(prefix_end) = key.find('_').and_then(|first| {
        key[first + 1..]
            .find('_')
            .map(|second| first + 1 + second + 1)
    }) {
        let prefix = &key[..prefix_end];
        let suffix = if key.len() > prefix_end + 3 {
            &key[key.len() - 3..]
        } else {
            ""
        };
        format!("{}***{}", prefix, suffix)
    } else {
        // No structured prefix, just show first 4 and last 3
        format!(
            "{}***{}",
            &key[..4.min(key.len())],
            &key[key.len().saturating_sub(3)..]
        )
    }
}

/// Mask a wallet/blockchain address for safe logging.
/// `0xAbCdEf1234567890AbCdEf1234567890AbCdEf12` → `0xAbCd...Ef12`
pub fn mask_wallet(address: &str) -> String {
    if address.len() <= 10 {
        return address.to_string();
    }

    if address.starts_with("0x") && address.len() >= 12 {
        format!("{}...{}", &address[..6], &address[address.len() - 4..])
    } else {
        // Solana or other format
        format!("{}...{}", &address[..4], &address[address.len() - 4..])
    }
}

/// Mask an IP address for safe logging.
/// `102.89.45.123` → `102.89.x.x`
pub fn mask_ip(ip: &str) -> String {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() == 4 {
        format!("{}.{}.x.x", parts[0], parts[1])
    } else if ip.contains(':') {
        // IPv6 — show first segment only
        let segments: Vec<&str> = ip.split(':').collect();
        if segments.len() >= 2 {
            format!("{}:{}:x:x", segments[0], segments[1])
        } else {
            "x:x:x:x".to_string()
        }
    } else {
        ip.to_string()
    }
}

/// Mask a NIN/BVN number for safe logging.
/// `12345678901` → `***8901`
pub fn mask_nin_bvn(value: &str) -> String {
    if value.len() <= 4 {
        "***".to_string()
    } else {
        format!("***{}", &value[value.len() - 4..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_scrubbing() {
        let mut data = json!({
            "user": {
                "email": "test@test.com",
                "password": "secret_password",
                "nested": {
                    "transaction_pin": "1234"
                }
            },
            "api_key": "sk_test_123"
        });

        scrub_sensitive_data(&mut data);

        assert_eq!(data["user"]["password"], "********");
        assert_eq!(data["user"]["nested"]["transaction_pin"], "********");
        assert_eq!(data["api_key"], "********");
        assert_eq!(data["user"]["email"], "test@test.com");
    }

    #[test]
    fn test_mask_email() {
        assert_eq!(mask_email("username@example.com"), "use***ame@example.com");
        assert_eq!(mask_email("a@b.co"), "a***@b.co");
        assert_eq!(mask_email("longuser@domain.org"), "lon***ser@domain.org");
        assert_eq!(mask_email("short@x.com"), "s***@x.com");
    }

    #[test]
    fn test_mask_api_key() {
        assert_eq!(mask_api_key("sk_live_abc123def456ghi789"), "sk_live_***789");
        assert_eq!(mask_api_key("short"), "***");
    }

    #[test]
    fn test_mask_wallet() {
        assert_eq!(
            mask_wallet("0xAbCdEf1234567890AbCdEf1234567890AbCdEf12"),
            "0xAbCd...Ef12"
        );
    }

    #[test]
    fn test_mask_ip() {
        assert_eq!(mask_ip("102.89.45.123"), "102.89.x.x");
        assert_eq!(mask_ip("192.168.1.1"), "192.168.x.x");
    }

    #[test]
    fn test_mask_nin_bvn() {
        assert_eq!(mask_nin_bvn("12345678901"), "***8901");
        assert_eq!(mask_nin_bvn("123"), "***");
    }
}
