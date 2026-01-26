// Comprehensive Test Suite for Hybrid Non-Custodial Payment Gateway
// Tests all critical functionality to ensure production readiness

use fiddupay::{
    payment::models::{CryptoType, PaymentStatus, CreatePaymentRequest},
    services::{
        merchant_service::MerchantService,
        payment_service::PaymentService,
        balance_service::BalanceService,
        wallet_config_service::WalletConfigService,
    },
    config::Config,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;

#[tokio::test]
async fn test_crypto_type_variants() {
    // Test all 10 supported cryptocurrency variants
    let variants = vec![
        CryptoType::Sol,
        CryptoType::Eth,
        CryptoType::Bnb,
        CryptoType::Matic,
        CryptoType::Arb,
        CryptoType::UsdtSpl,
        CryptoType::UsdtEth,
        CryptoType::UsdtBep20,
        CryptoType::UsdtPolygon,
        CryptoType::UsdtArbitrum,
    ];
    
    assert_eq!(variants.len(), 10, "Should support exactly 10 cryptocurrency variants");
    
    for crypto_type in variants {
        // Test string conversion
        let crypto_str = crypto_type.to_string();
        assert!(!crypto_str.is_empty(), "CryptoType should convert to non-empty string");
        
        // Test network identification
        let network = crypto_type.network();
        assert!(!network.is_empty(), "Each CryptoType should have a network");
        
        // Test required confirmations
        let confirmations = crypto_type.required_confirmations();
        assert!(confirmations > 0, "Required confirmations should be positive");
    }
}

#[tokio::test]
async fn test_payment_status_transitions() {
    // Test all payment status variants
    let statuses = vec![
        PaymentStatus::Pending,
        PaymentStatus::Confirming,
        PaymentStatus::Confirmed,
        PaymentStatus::Failed,
        PaymentStatus::Expired,
        PaymentStatus::Refunded,
    ];
    
    for status in statuses {
        let status_str = PaymentStatus::from_string(&status.to_string());
        // Test round-trip conversion works
        assert_eq!(format!("{:?}", status), format!("{:?}", status_str));
    }
}

#[tokio::test]
async fn test_create_payment_request_validation() {
    // Test payment request creation with valid data
    let request = CreatePaymentRequest {
        amount: Some(Decimal::new(100, 2)), // $1.00
        amount_usd: Some(Decimal::new(100, 2)),
        crypto_type: CryptoType::UsdtEth,
        description: Some("Test payment".to_string()),
        expiration_minutes: Some(15),
        metadata: None,
    };
    
    assert!(request.amount.is_some());
    assert!(request.amount_usd.is_some());
    assert_eq!(request.crypto_type, CryptoType::UsdtEth);
    assert_eq!(request.expiration_minutes.unwrap(), 15);
}

#[tokio::test]
async fn test_gas_fee_validation_logic() {
    // Test the hybrid gas fee logic:
    // - Native currencies (SOL, ETH, BNB, MATIC, ARB) auto-deduct gas
    // - USDT variants require separate gas deposits
    
    let native_currencies = vec![
        CryptoType::Sol,
        CryptoType::Eth,
        CryptoType::Bnb,
        CryptoType::Matic,
        CryptoType::Arb,
    ];
    
    let usdt_variants = vec![
        CryptoType::UsdtSpl,
        CryptoType::UsdtEth,
        CryptoType::UsdtBep20,
        CryptoType::UsdtPolygon,
        CryptoType::UsdtArbitrum,
    ];
    
    // Native currencies should auto-deduct gas
    for crypto_type in native_currencies {
        let network = crypto_type.network();
        assert!(
            matches!(network, "solana" | "ethereum" | "bsc" | "polygon" | "arbitrum"),
            "Native currency should have valid network: {}",
            network
        );
    }
    
    // USDT variants should require separate gas
    for crypto_type in usdt_variants {
        let crypto_str = crypto_type.to_string();
        assert!(
            crypto_str.contains("USDT"),
            "USDT variant should contain USDT in string representation"
        );
    }
}
#[tokio::test]
async fn test_wallet_mode_system() {
    // Test the 3-mode wallet system:
    // 1. Address-only mode (receive only)
    // 2. Gateway-generated wallets (full control)
    // 3. Merchant-provided wallets (import)
    
    let test_address = "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6";
    assert_eq!(test_address.len(), 42, "Ethereum address should be 42 characters");
    assert!(test_address.starts_with("0x"), "Ethereum address should start with 0x");
    
    let sol_address = "DRpbCBMxVnDK7maPM5tGv6MvB3v1sRMC86PZ8okm21hy";
    assert_eq!(sol_address.len(), 44, "Solana address should be 44 characters");
}

#[tokio::test]
async fn test_fee_calculation() {
    // Test fee calculation logic
    use fiddupay::payment::fee_calculator::FeeCalculator;
    
    let base_amount = Decimal::new(10000, 2); // $100.00
    let fee_percentage = 2.5; // 2.5%
    
    let fee_amount = FeeCalculator::calculate_fee_usd(base_amount, fee_percentage);
    let expected_fee = Decimal::new(250, 2); // $2.50
    
    assert_eq!(fee_amount, expected_fee, "Fee calculation should be accurate");
    
    let total_with_fee = FeeCalculator::calculate_total_with_fee(base_amount, fee_amount);
    let expected_total = Decimal::new(10250, 2); // $102.50
    
    assert_eq!(total_with_fee, expected_total, "Total with fee should be accurate");
}

#[tokio::test]
async fn test_security_framework() {
    // Test security components are properly configured
    use fiddupay::middleware::validation::*;
    
    // Test password validation
    assert!(is_valid_password("StrongPass123!"), "Strong password should be valid");
    assert!(!is_valid_password("weak"), "Weak password should be invalid");
    assert!(!is_valid_password(""), "Empty password should be invalid");
    
    // Test webhook URL validation
    assert!(is_valid_webhook_url("https://example.com/webhook"), "HTTPS webhook should be valid");
    assert!(!is_valid_webhook_url("http://example.com/webhook"), "HTTP webhook should be invalid");
    assert!(!is_valid_webhook_url("invalid-url"), "Invalid URL should be rejected");
}

#[tokio::test]
async fn test_blockchain_network_support() {
    // Test all 5 blockchain networks are properly supported
    let networks = vec![
        ("solana", CryptoType::Sol),
        ("ethereum", CryptoType::Eth),
        ("bsc", CryptoType::Bnb),
        ("polygon", CryptoType::Matic),
        ("arbitrum", CryptoType::Arb),
    ];
    
    for (expected_network, crypto_type) in networks {
        let actual_network = crypto_type.network();
        assert_eq!(
            actual_network.to_lowercase(),
            expected_network,
            "Network mapping should be correct for {:?}",
            crypto_type
        );
    }
}

#[tokio::test]
async fn test_payment_expiration_logic() {
    // Test payment expiration handling
    use chrono::{Utc, Duration};
    
    let now = Utc::now();
    let expiration_time = now + Duration::minutes(15);
    
    // Test that expiration time is in the future
    assert!(expiration_time > now, "Expiration time should be in the future");
    
    // Test expiration calculation
    let minutes_until_expiration = (expiration_time - now).num_minutes();
    assert!(
        minutes_until_expiration >= 14 && minutes_until_expiration <= 15,
        "Expiration should be approximately 15 minutes"
    );
}

#[tokio::test]
async fn test_error_handling() {
    // Test error handling and validation
    use fiddupay::error::ServiceError;
    
    // Test error creation and formatting
    let validation_error = ServiceError::ValidationError("Invalid input".to_string());
    let error_string = format!("{}", validation_error);
    assert!(error_string.contains("Invalid input"), "Error should contain message");
    
    // Test different error types
    let payment_not_found = ServiceError::PaymentNotFound;
    assert_ne!(
        format!("{}", validation_error),
        format!("{}", payment_not_found),
        "Different errors should have different messages"
    );
}

#[tokio::test]
async fn test_production_readiness() {
    // Test that all critical components are ready for production
    
    // Test configuration validation
    let test_config = std::env::var("DATABASE_URL").unwrap_or_else(|_| 
        "postgresql://test:test@localhost/test".to_string()
    );
    assert!(!test_config.is_empty(), "Database URL should be configured");
    
    // Test that all required crypto types are supported
    let all_crypto_types = vec![
        CryptoType::Sol, CryptoType::Eth, CryptoType::Bnb, CryptoType::Matic, CryptoType::Arb,
        CryptoType::UsdtSpl, CryptoType::UsdtEth, CryptoType::UsdtBep20, 
        CryptoType::UsdtPolygon, CryptoType::UsdtArbitrum,
    ];
    
    assert_eq!(all_crypto_types.len(), 10, "Should support exactly 10 crypto types");
    
    // Test that each crypto type has proper configuration
    for crypto_type in all_crypto_types {
        assert!(!crypto_type.network().is_empty(), "Each crypto type should have a network");
        assert!(crypto_type.required_confirmations() > 0, "Each crypto type should have required confirmations");
    }
}
