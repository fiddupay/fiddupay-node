// Global Data Sanitizer
// Scrub sensitive keys from JSON objects before logging or storage

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
}
