// Payment Gateway Models
// Data structures for cryptocurrency payments

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Payment status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentStatus {
    Pending,       // Payment created, waiting for transaction
    Confirming,    // Transaction detected, waiting for confirmations
    Confirmed,     // Payment confirmed and verified
    Failed,        // Payment failed or expired
    Refunded,      // Payment refunded to user
}

/// Cryptocurrency type enumeration (5 supported payment methods)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CryptoType {
    #[serde(rename = "USDT_BEP20")]
    UsdtBep20,        // USDT on Binance Smart Chain (BEP20)
    #[serde(rename = "USDT_ARBITRUM")]
    UsdtArbitrum,     // USDT on Arbitrum One
    #[serde(rename = "USDT_SPL")]
    UsdtSpl,          // USDT on Solana (SPL token)
    #[serde(rename = "USDT_POLYGON")]
    UsdtPolygon,      // USDT on Polygon
    #[serde(rename = "SOL")]
    Sol,              // Solana native
}

impl CryptoType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CryptoType::UsdtBep20 => "USDT",
            CryptoType::UsdtArbitrum => "USDT",
            CryptoType::UsdtSpl => "USDT",
            CryptoType::UsdtPolygon => "USDT",
            CryptoType::Sol => "SOL",
        }
    }

    pub fn network(&self) -> &'static str {
        match self {
            CryptoType::UsdtBep20 => "BEP20",
            CryptoType::UsdtArbitrum => "ARBITRUM",
            CryptoType::UsdtSpl => "SOLANA_SPL",
            CryptoType::UsdtPolygon => "POLYGON",
            CryptoType::Sol => "SOLANA",
        }
    }

    pub fn required_confirmations(&self) -> u32 {
        match self {
            CryptoType::UsdtBep20 => 15,     // BSC: 15 blocks (~45 seconds)
            CryptoType::UsdtArbitrum => 1,   // Arbitrum: 1 block (~250ms)
            CryptoType::UsdtSpl => 32,       // Solana SPL: 32 confirmations (~13 seconds)
            CryptoType::UsdtPolygon => 128,  // Polygon: 128 blocks (~4 minutes)
            CryptoType::Sol => 32,           // Solana: 32 confirmations (~13 seconds)
        }
    }

    pub fn blockchain_explorer(&self) -> &'static str {
        match self {
            CryptoType::UsdtBep20 => "https://bscscan.com",
            CryptoType::UsdtArbitrum => "https://arbiscan.io",
            CryptoType::UsdtSpl => "https://solscan.io",
            CryptoType::UsdtPolygon => "https://polygonscan.com",
            CryptoType::Sol => "https://solscan.io",
        }
    }

    pub fn contract_address(&self) -> Option<&'static str> {
        match self {
            CryptoType::UsdtBep20 => Some("0x55d398326f99059fF775485246999027B3197955"),
            CryptoType::UsdtArbitrum => Some("0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9"),
            CryptoType::UsdtSpl => Some("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
            CryptoType::UsdtPolygon => Some("0xc2132D05D31c914a87C6611C10748AEb04B58e8F"),
            CryptoType::Sol => None, // Native token
        }
    }
}

/// Payment transaction record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentTransaction {
    pub id: i64,
    pub merchant_id: i64,
    pub payment_id: String,
    pub user_id: Option<i64>,
    pub subscription_id: Option<i64>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub amount: Decimal,
    pub amount_usd: Decimal,
    pub fee_percentage: Decimal,
    pub fee_amount: Decimal,
    pub fee_amount_usd: Decimal,
    pub crypto_type: String, // Stored as string in DB
    pub network: String,
    pub transaction_hash: Option<String>,
    pub from_address: Option<String>,
    pub to_address: String,
    pub status: String, // Stored as string in DB
    pub confirmations: i32,
    pub required_confirmations: i32,
    pub block_number: Option<i64>,
    pub partial_payments_enabled: bool,
    pub total_paid: Decimal,
    pub remaining_balance: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
}

/// Blockchain transaction info (from blockchain APIs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainTransaction {
    pub hash: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: Decimal,
    pub confirmations: u32,
    pub block_number: Option<u64>,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
}

/// Request to create a new payment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentRequest {
    pub amount_usd: Decimal,
    pub crypto_type: CryptoType,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub expiration_minutes: Option<u32>,  // Default: 15
    pub partial_payments_enabled: Option<bool>,
}

/// Payment response returned to merchants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub payment_id: String,  // "pay_abc123"
    pub status: PaymentStatus,
    pub amount: Decimal,
    pub amount_usd: Decimal,
    pub crypto_type: CryptoType,
    pub network: String,
    pub deposit_address: String,
    pub payment_link: String,  // URL to hosted page
    pub qr_code_data: String,  // Data for QR code generation
    pub fee_amount: Decimal,
    pub fee_amount_usd: Decimal,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub transaction_hash: Option<String>,
    pub partial_payments: Option<PartialPaymentInfo>,
}

/// Information about partial payments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialPaymentInfo {
    pub enabled: bool,
    pub total_paid: Decimal,
    pub remaining_balance: Decimal,
    pub payments: Vec<PartialPaymentRecord>,
}

/// Individual partial payment record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PartialPaymentRecord {
    pub id: i64,
    pub payment_id: i64,
    pub transaction_hash: String,
    pub amount: Decimal,
    pub amount_usd: Decimal,
    pub confirmations: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
}

/// Filters for listing payments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentFilters {
    pub status: Option<PaymentStatus>,
    pub blockchain: Option<String>,  // e.g., "SOLANA", "BEP20", "ARBITRUM", "POLYGON"
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub page: Option<u32>,  // Default: 1
    pub page_size: Option<u32>,  // Default: 20, Max: 100
}

impl Default for PaymentFilters {
    fn default() -> Self {
        Self {
            status: None,
            blockchain: None,
            from_date: None,
            to_date: None,
            page: Some(1),
            page_size: Some(20),
        }
    }
}

/// Paginated list of payments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentList {
    pub payments: Vec<PaymentResponse>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_payment_transaction_with_merchant_fields() {
        let payment = PaymentTransaction {
            id: 1,
            merchant_id: 42,
            payment_id: "pay_abc123".to_string(),
            user_id: Some(100),
            subscription_id: Some(200),
            description: Some("Test payment".to_string()),
            metadata: Some(serde_json::json!({"order_id": "12345"})),
            amount: Decimal::new(100, 0),
            amount_usd: Decimal::new(10000, 2),
            fee_percentage: Decimal::new(150, 2), // 1.50%
            fee_amount: Decimal::new(150, 2), // 1.50 in crypto
            fee_amount_usd: Decimal::new(150, 2), // $1.50
            crypto_type: "USDT_BEP20".to_string(),
            network: "BEP20".to_string(),
            transaction_hash: Some("0xabc123".to_string()),
            from_address: Some("0x123".to_string()),
            to_address: "0x456".to_string(),
            status: "PENDING".to_string(),
            confirmations: 0,
            required_confirmations: 15,
            block_number: None,
            partial_payments_enabled: false,
            total_paid: Decimal::new(0, 0),
            remaining_balance: None,
            created_at: Utc::now(),
            confirmed_at: None,
            expires_at: Utc::now(),
        };

        assert_eq!(payment.merchant_id, 42);
        assert_eq!(payment.payment_id, "pay_abc123");
        assert_eq!(payment.fee_percentage, Decimal::new(150, 2));
        assert_eq!(payment.fee_amount, Decimal::new(150, 2));
        assert_eq!(payment.fee_amount_usd, Decimal::new(150, 2));
        assert!(!payment.partial_payments_enabled);
        assert_eq!(payment.total_paid, Decimal::new(0, 0));
    }

    #[test]
    fn test_payment_transaction_with_partial_payments() {
        let payment = PaymentTransaction {
            id: 1,
            merchant_id: 42,
            payment_id: "pay_partial123".to_string(),
            user_id: None,
            subscription_id: None,
            description: None,
            metadata: None,
            amount: Decimal::new(100, 0),
            amount_usd: Decimal::new(10000, 2),
            fee_percentage: Decimal::new(150, 2),
            fee_amount: Decimal::new(150, 2),
            fee_amount_usd: Decimal::new(150, 2),
            crypto_type: "SOL".to_string(),
            network: "SOLANA".to_string(),
            transaction_hash: None,
            from_address: None,
            to_address: "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU".to_string(),
            status: "PENDING".to_string(),
            confirmations: 0,
            required_confirmations: 32,
            block_number: None,
            partial_payments_enabled: true,
            total_paid: Decimal::new(50, 0),
            remaining_balance: Some(Decimal::new(50, 0)),
            created_at: Utc::now(),
            confirmed_at: None,
            expires_at: Utc::now(),
        };

        assert!(payment.partial_payments_enabled);
        assert_eq!(payment.total_paid, Decimal::new(50, 0));
        assert_eq!(payment.remaining_balance, Some(Decimal::new(50, 0)));
    }

    #[test]
    fn test_create_payment_request() {
        let request = CreatePaymentRequest {
            amount_usd: Decimal::new(10000, 2), // $100.00
            crypto_type: CryptoType::UsdtBep20,
            description: Some("Test payment".to_string()),
            metadata: Some(serde_json::json!({"order_id": "12345"})),
            expiration_minutes: Some(30),
            partial_payments_enabled: Some(false),
        };

        assert_eq!(request.amount_usd, Decimal::new(10000, 2));
        assert_eq!(request.crypto_type, CryptoType::UsdtBep20);
        assert_eq!(request.description, Some("Test payment".to_string()));
        assert_eq!(request.expiration_minutes, Some(30));
        assert_eq!(request.partial_payments_enabled, Some(false));
    }

    #[test]
    fn test_create_payment_request_with_defaults() {
        let request = CreatePaymentRequest {
            amount_usd: Decimal::new(5000, 2), // $50.00
            crypto_type: CryptoType::Sol,
            description: None,
            metadata: None,
            expiration_minutes: None, // Should default to 15
            partial_payments_enabled: None, // Should default to false
        };

        assert_eq!(request.amount_usd, Decimal::new(5000, 2));
        assert_eq!(request.crypto_type, CryptoType::Sol);
        assert!(request.description.is_none());
        assert!(request.metadata.is_none());
        assert!(request.expiration_minutes.is_none());
        assert!(request.partial_payments_enabled.is_none());
    }

    #[test]
    fn test_payment_response() {
        let response = PaymentResponse {
            payment_id: "pay_xyz789".to_string(),
            status: PaymentStatus::Pending,
            amount: Decimal::new(100, 0),
            amount_usd: Decimal::new(10000, 2),
            crypto_type: CryptoType::UsdtArbitrum,
            network: "ARBITRUM".to_string(),
            deposit_address: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string(),
            payment_link: "https://pay.example.com/lnk_xyz789".to_string(),
            qr_code_data: "ethereum:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb?value=100".to_string(),
            fee_amount: Decimal::new(150, 2),
            fee_amount_usd: Decimal::new(150, 2),
            expires_at: Utc::now(),
            created_at: Utc::now(),
            confirmed_at: None,
            transaction_hash: None,
            partial_payments: None,
        };

        assert_eq!(response.payment_id, "pay_xyz789");
        assert_eq!(response.status, PaymentStatus::Pending);
        assert_eq!(response.crypto_type, CryptoType::UsdtArbitrum);
        assert_eq!(response.network, "ARBITRUM");
        assert!(response.partial_payments.is_none());
    }

    #[test]
    fn test_payment_response_with_partial_payments() {
        let partial_info = PartialPaymentInfo {
            enabled: true,
            total_paid: Decimal::new(50, 0),
            remaining_balance: Decimal::new(50, 0),
            payments: vec![],
        };

        let response = PaymentResponse {
            payment_id: "pay_partial456".to_string(),
            status: PaymentStatus::Pending,
            amount: Decimal::new(100, 0),
            amount_usd: Decimal::new(10000, 2),
            crypto_type: CryptoType::Sol,
            network: "SOLANA".to_string(),
            deposit_address: "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU".to_string(),
            payment_link: "https://pay.example.com/lnk_partial456".to_string(),
            qr_code_data: "solana:7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU?amount=100".to_string(),
            fee_amount: Decimal::new(150, 2),
            fee_amount_usd: Decimal::new(150, 2),
            expires_at: Utc::now(),
            created_at: Utc::now(),
            confirmed_at: None,
            transaction_hash: None,
            partial_payments: Some(partial_info),
        };

        assert!(response.partial_payments.is_some());
        let partial = response.partial_payments.unwrap();
        assert!(partial.enabled);
        assert_eq!(partial.total_paid, Decimal::new(50, 0));
        assert_eq!(partial.remaining_balance, Decimal::new(50, 0));
    }

    #[test]
    fn test_partial_payment_info() {
        let record1 = PartialPaymentRecord {
            id: 1,
            payment_id: 100,
            transaction_hash: "0xabc123".to_string(),
            amount: Decimal::new(30, 0),
            amount_usd: Decimal::new(3000, 2),
            confirmations: 15,
            status: "CONFIRMED".to_string(),
            created_at: Utc::now(),
            confirmed_at: Some(Utc::now()),
        };

        let record2 = PartialPaymentRecord {
            id: 2,
            payment_id: 100,
            transaction_hash: "0xdef456".to_string(),
            amount: Decimal::new(20, 0),
            amount_usd: Decimal::new(2000, 2),
            confirmations: 5,
            status: "PENDING".to_string(),
            created_at: Utc::now(),
            confirmed_at: None,
        };

        let partial_info = PartialPaymentInfo {
            enabled: true,
            total_paid: Decimal::new(30, 0), // Only confirmed payment
            remaining_balance: Decimal::new(70, 0),
            payments: vec![record1, record2],
        };

        assert!(partial_info.enabled);
        assert_eq!(partial_info.total_paid, Decimal::new(30, 0));
        assert_eq!(partial_info.remaining_balance, Decimal::new(70, 0));
        assert_eq!(partial_info.payments.len(), 2);
    }

    #[test]
    fn test_partial_payment_record() {
        let record = PartialPaymentRecord {
            id: 1,
            payment_id: 42,
            transaction_hash: "0x123abc".to_string(),
            amount: Decimal::new(25, 0),
            amount_usd: Decimal::new(2500, 2),
            confirmations: 10,
            status: "CONFIRMING".to_string(),
            created_at: Utc::now(),
            confirmed_at: None,
        };

        assert_eq!(record.id, 1);
        assert_eq!(record.payment_id, 42);
        assert_eq!(record.transaction_hash, "0x123abc");
        assert_eq!(record.amount, Decimal::new(25, 0));
        assert_eq!(record.confirmations, 10);
        assert_eq!(record.status, "CONFIRMING");
        assert!(record.confirmed_at.is_none());
    }

    #[test]
    fn test_create_payment_request_serialization() {
        let request = CreatePaymentRequest {
            amount_usd: Decimal::new(10000, 2),
            crypto_type: CryptoType::UsdtPolygon,
            description: Some("Test".to_string()),
            metadata: Some(serde_json::json!({"key": "value"})),
            expiration_minutes: Some(15),
            partial_payments_enabled: Some(true),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("amount_usd"));
        assert!(json.contains("USDT_POLYGON"));

        let deserialized: CreatePaymentRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.amount_usd, request.amount_usd);
        assert_eq!(deserialized.crypto_type, request.crypto_type);
    }

    #[test]
    fn test_payment_response_serialization() {
        let response = PaymentResponse {
            payment_id: "pay_test123".to_string(),
            status: PaymentStatus::Confirmed,
            amount: Decimal::new(100, 0),
            amount_usd: Decimal::new(10000, 2),
            crypto_type: CryptoType::UsdtSpl,
            network: "SOLANA_SPL".to_string(),
            deposit_address: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(),
            payment_link: "https://pay.example.com/lnk_test123".to_string(),
            qr_code_data: "solana:Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(),
            fee_amount: Decimal::new(150, 2),
            fee_amount_usd: Decimal::new(150, 2),
            expires_at: Utc::now(),
            created_at: Utc::now(),
            confirmed_at: Some(Utc::now()),
            transaction_hash: Some("0xconfirmed".to_string()),
            partial_payments: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("pay_test123"));
        assert!(json.contains("CONFIRMED"));

        let deserialized: PaymentResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.payment_id, response.payment_id);
        assert_eq!(deserialized.status, response.status);
    }

    #[test]
    fn test_partial_payment_info_serialization() {
        let partial_info = PartialPaymentInfo {
            enabled: true,
            total_paid: Decimal::new(75, 0),
            remaining_balance: Decimal::new(25, 0),
            payments: vec![],
        };

        let json = serde_json::to_string(&partial_info).unwrap();
        assert!(json.contains("enabled"));
        assert!(json.contains("total_paid"));
        assert!(json.contains("remaining_balance"));

        let deserialized: PartialPaymentInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.enabled, partial_info.enabled);
        assert_eq!(deserialized.total_paid, partial_info.total_paid);
        assert_eq!(deserialized.remaining_balance, partial_info.remaining_balance);
    }

    #[test]
    fn test_crypto_type_methods() {
        // Test all crypto types
        assert_eq!(CryptoType::Sol.as_str(), "SOL");
        assert_eq!(CryptoType::UsdtSpl.as_str(), "USDT");
        assert_eq!(CryptoType::UsdtBep20.as_str(), "USDT");
        assert_eq!(CryptoType::UsdtArbitrum.as_str(), "USDT");
        assert_eq!(CryptoType::UsdtPolygon.as_str(), "USDT");

        assert_eq!(CryptoType::Sol.network(), "SOLANA");
        assert_eq!(CryptoType::UsdtSpl.network(), "SOLANA_SPL");
        assert_eq!(CryptoType::UsdtBep20.network(), "BEP20");
        assert_eq!(CryptoType::UsdtArbitrum.network(), "ARBITRUM");
        assert_eq!(CryptoType::UsdtPolygon.network(), "POLYGON");
    }

    #[test]
    fn test_payment_status_enum() {
        let statuses = vec![
            PaymentStatus::Pending,
            PaymentStatus::Confirming,
            PaymentStatus::Confirmed,
            PaymentStatus::Failed,
            PaymentStatus::Refunded,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: PaymentStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, deserialized);
        }
    }

    #[test]
    fn test_payment_transaction_optional_fields() {
        let payment = PaymentTransaction {
            id: 1,
            merchant_id: 1,
            payment_id: "pay_test".to_string(),
            user_id: None,
            subscription_id: None,
            description: None,
            metadata: None,
            amount: Decimal::new(100, 0),
            amount_usd: Decimal::new(10000, 2),
            fee_percentage: Decimal::new(150, 2),
            fee_amount: Decimal::new(150, 2),
            fee_amount_usd: Decimal::new(150, 2),
            crypto_type: "SOL".to_string(),
            network: "SOLANA".to_string(),
            transaction_hash: None,
            from_address: None,
            to_address: "test_address".to_string(),
            status: "PENDING".to_string(),
            confirmations: 0,
            required_confirmations: 32,
            block_number: None,
            partial_payments_enabled: false,
            total_paid: Decimal::new(0, 0),
            remaining_balance: None,
            created_at: Utc::now(),
            confirmed_at: None,
            expires_at: Utc::now(),
        };

        assert!(payment.user_id.is_none());
        assert!(payment.subscription_id.is_none());
        assert!(payment.description.is_none());
        assert!(payment.metadata.is_none());
        assert!(payment.transaction_hash.is_none());
        assert!(payment.from_address.is_none());
        assert!(payment.confirmed_at.is_none());
        assert!(payment.remaining_balance.is_none());
    }

    #[test]
    fn test_fee_calculation_example() {
        // Example: $100 payment with 1.5% fee
        let base_amount = Decimal::new(10000, 2); // $100.00
        let fee_percentage = Decimal::new(150, 2); // 1.50%
        let fee_amount_usd = base_amount * fee_percentage / Decimal::new(100, 0); // $1.50

        let payment = PaymentTransaction {
            id: 1,
            merchant_id: 1,
            payment_id: "pay_fee_test".to_string(),
            user_id: None,
            subscription_id: None,
            description: None,
            metadata: None,
            amount: Decimal::new(100, 0),
            amount_usd: base_amount,
            fee_percentage,
            fee_amount: Decimal::new(150, 2),
            fee_amount_usd,
            crypto_type: "USDT_BEP20".to_string(),
            network: "BEP20".to_string(),
            transaction_hash: None,
            from_address: None,
            to_address: "0x123".to_string(),
            status: "PENDING".to_string(),
            confirmations: 0,
            required_confirmations: 15,
            block_number: None,
            partial_payments_enabled: false,
            total_paid: Decimal::new(0, 0),
            remaining_balance: None,
            created_at: Utc::now(),
            confirmed_at: None,
            expires_at: Utc::now(),
        };

        assert_eq!(payment.fee_amount_usd, Decimal::new(150, 2)); // $1.50
    }

    #[test]
    fn test_payment_filters_default() {
        let filters = PaymentFilters::default();
        assert!(filters.status.is_none());
        assert!(filters.blockchain.is_none());
        assert!(filters.from_date.is_none());
        assert!(filters.to_date.is_none());
        assert_eq!(filters.page, Some(1));
        assert_eq!(filters.page_size, Some(20));
    }

    #[test]
    fn test_payment_filters_with_status() {
        let filters = PaymentFilters {
            status: Some(PaymentStatus::Confirmed),
            blockchain: None,
            from_date: None,
            to_date: None,
            page: Some(1),
            page_size: Some(50),
        };

        assert_eq!(filters.status, Some(PaymentStatus::Confirmed));
        assert_eq!(filters.page_size, Some(50));
    }

    #[test]
    fn test_payment_filters_with_blockchain() {
        let filters = PaymentFilters {
            status: None,
            blockchain: Some("SOLANA".to_string()),
            from_date: None,
            to_date: None,
            page: Some(2),
            page_size: Some(10),
        };

        assert_eq!(filters.blockchain, Some("SOLANA".to_string()));
        assert_eq!(filters.page, Some(2));
    }

    #[test]
    fn test_payment_filters_with_date_range() {
        let from = Utc::now() - chrono::Duration::days(7);
        let to = Utc::now();

        let filters = PaymentFilters {
            status: None,
            blockchain: None,
            from_date: Some(from),
            to_date: Some(to),
            page: Some(1),
            page_size: Some(20),
        };

        assert!(filters.from_date.is_some());
        assert!(filters.to_date.is_some());
    }

    #[test]
    fn test_payment_list() {
        let payment_list = PaymentList {
            payments: vec![],
            total: 100,
            page: 1,
            page_size: 20,
            total_pages: 5,
        };

        assert_eq!(payment_list.total, 100);
        assert_eq!(payment_list.page, 1);
        assert_eq!(payment_list.page_size, 20);
        assert_eq!(payment_list.total_pages, 5);
        assert_eq!(payment_list.payments.len(), 0);
    }

    #[test]
    fn test_payment_list_pagination_calculation() {
        // Test total_pages calculation
        let total = 95;
        let page_size = 20;
        let total_pages = (total as f64 / page_size as f64).ceil() as u32;

        assert_eq!(total_pages, 5); // 95 / 20 = 4.75, ceil = 5
    }

    #[test]
    fn test_payment_filters_serialization() {
        let filters = PaymentFilters {
            status: Some(PaymentStatus::Pending),
            blockchain: Some("BEP20".to_string()),
            from_date: None,
            to_date: None,
            page: Some(1),
            page_size: Some(20),
        };

        let json = serde_json::to_string(&filters).unwrap();
        assert!(json.contains("PENDING"));
        assert!(json.contains("BEP20"));

        let deserialized: PaymentFilters = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.status, filters.status);
        assert_eq!(deserialized.blockchain, filters.blockchain);
    }
}
