// Merchant Models
// Data structures for merchant accounts and wallets

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Merchant {
    pub id: i64,
    pub email: String,
    pub business_name: String,
    pub api_key_hash: String,
    pub fee_percentage: Decimal,
    pub customer_pays_fee: bool, // true = customer pays, false = merchant pays
    pub is_active: bool,
    pub sandbox_mode: bool,
    pub kyc_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MerchantWallet {
    pub id: i64,
    pub merchant_id: i64,
    pub crypto_type: String,
    pub network: String,
    pub address: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantRegistrationRequest {
    pub email: String,
    pub business_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantRegistrationResponse {
    pub merchant_id: i64,
    pub api_key: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merchant_struct_creation() {
        let merchant = Merchant {
            id: 1i64,
            email: "test@example.com".to_string(),
            business_name: "Test Business".to_string(),
            api_key_hash: "hashed_key".to_string(),
            fee_percentage: Decimal::new(150, 2), // 1.50%
            customer_pays_fee: true,
            is_active: true,
            sandbox_mode: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(merchant.id, 1);
        assert_eq!(merchant.email, "test@example.com");
        assert_eq!(merchant.business_name, "Test Business");
        assert_eq!(merchant.fee_percentage, Decimal::new(150, 2));
        assert!(merchant.is_active);
        assert!(!merchant.sandbox_mode);
    }

    #[test]
    fn test_merchant_wallet_struct_creation() {
        let wallet = MerchantWallet {
            id: 1i64,
            merchant_id: 42i64,
            crypto_type: "USDT_BEP20".to_string(),
            network: "BEP20".to_string(),
            address: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(wallet.id, 1);
        assert_eq!(wallet.merchant_id, 42);
        assert_eq!(wallet.crypto_type, "USDT_BEP20");
        assert_eq!(wallet.network, "BEP20");
        assert!(wallet.is_active);
    }

    #[test]
    fn test_merchant_serialization() {
        let merchant = Merchant {
            id: 1i64,
            email: "test@example.com".to_string(),
            business_name: "Test Business".to_string(),
            api_key_hash: "hashed_key".to_string(),
            fee_percentage: Decimal::new(150, 2),
            customer_pays_fee: true,            is_active: true,
            sandbox_mode: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Test serialization
        let json = serde_json::to_string(&merchant).unwrap();
        assert!(json.contains("test@example.com"));
        assert!(json.contains("Test Business"));

        // Test deserialization
        let deserialized: Merchant = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, merchant.id);
        assert_eq!(deserialized.email, merchant.email);
    }

    #[test]
    fn test_merchant_wallet_serialization() {
        let wallet = MerchantWallet {
            id: 1i64,
            merchant_id: 42i64,
            crypto_type: "SOL".to_string(),
            network: "SOLANA".to_string(),
            address: "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU".to_string(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Test serialization
        let json = serde_json::to_string(&wallet).unwrap();
        assert!(json.contains("SOL"));
        assert!(json.contains("SOLANA"));

        // Test deserialization
        let deserialized: MerchantWallet = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, wallet.id);
        assert_eq!(deserialized.merchant_id, wallet.merchant_id);
        assert_eq!(deserialized.crypto_type, wallet.crypto_type);
    }

    #[test]
    fn test_merchant_registration_request() {
        let request = MerchantRegistrationRequest {
            email: "newmerchant@example.com".to_string(),
            business_name: "New Business".to_string(),
        };

        assert_eq!(request.email, "newmerchant@example.com");
        assert_eq!(request.business_name, "New Business");

        // Test serialization
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("newmerchant@example.com"));
    }

    #[test]
    fn test_merchant_registration_response() {
        let response = MerchantRegistrationResponse {
            merchant_id: 42i64,
            api_key: "test_api_key_123".to_string(),
        };

        assert_eq!(response.merchant_id, 42);
        assert_eq!(response.api_key, "test_api_key_123");

        // Test serialization
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("42"));
        assert!(json.contains("test_api_key_123"));
    }

    #[test]
    fn test_merchant_with_different_fee_percentages() {
        // Test minimum fee (0.1%)
        let merchant_min = Merchant {
            id: 1i64,
            email: "min@example.com".to_string(),
            business_name: "Min Fee".to_string(),
            api_key_hash: "hash".to_string(),
            fee_percentage: Decimal::new(10, 2), // 0.10%
            customer_pays_fee: true,
            is_active: true,
            sandbox_mode: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert_eq!(merchant_min.fee_percentage, Decimal::new(10, 2));

        // Test maximum fee (5%)
        let merchant_max = Merchant {
            id: 2,
            email: "max@example.com".to_string(),
            business_name: "Max Fee".to_string(),
            api_key_hash: "hash".to_string(),
            fee_percentage: Decimal::new(500, 2), // 5.00%
            customer_pays_fee: true,
            is_active: true,
            sandbox_mode: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert_eq!(merchant_max.fee_percentage, Decimal::new(500, 2));
    }

    #[test]
    fn test_merchant_wallet_for_all_crypto_types() {
        let crypto_types = vec![
            ("SOL", "SOLANA"),
            ("USDT_SPL", "SOLANA_SPL"),
            ("USDT_BEP20", "BEP20"),
            ("USDT_ARBITRUM", "ARBITRUM"),
            ("USDT_POLYGON", "POLYGON"),
        ];

        for (crypto_type, network) in crypto_types {
            let wallet = MerchantWallet {
                id: 1i64,
                merchant_id: 42i64,
                crypto_type: crypto_type.to_string(),
                network: network.to_string(),
                address: "test_address".to_string(),
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            assert_eq!(wallet.crypto_type, crypto_type);
            assert_eq!(wallet.network, network);
        }
    }
}
