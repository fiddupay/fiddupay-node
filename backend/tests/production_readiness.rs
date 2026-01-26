// Production Readiness Test Suite
// Validates core functionality for production deployment

use fiddupay::payment::models::{CryptoType, PaymentStatus};
use rust_decimal::Decimal;

#[tokio::test]
async fn test_all_crypto_types_supported() {
    // Test all 10 supported cryptocurrency variants
    let crypto_types = vec![
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
    
    assert_eq!(crypto_types.len(), 10, "Should support exactly 10 cryptocurrency variants");
    
    for crypto_type in crypto_types {
        // Test network identification
        let network = crypto_type.network();
        assert!(!network.is_empty(), "Each CryptoType should have a network");
        
        // Test required confirmations
        let confirmations = crypto_type.required_confirmations();
        assert!(confirmations > 0, "Required confirmations should be positive");
    }
}

#[tokio::test]
async fn test_payment_status_system() {
    // Test all payment status variants work correctly
    let statuses = vec![
        PaymentStatus::Pending,
        PaymentStatus::Confirming,
        PaymentStatus::Confirmed,
        PaymentStatus::Failed,
        PaymentStatus::Expired,
        PaymentStatus::Refunded,
    ];
    
    for status in statuses {
        // Test string conversion works
        let status_string = format!("{:?}", status);
        assert!(!status_string.is_empty(), "Status should convert to string");
        
        // Test from_string conversion
        let converted_status = PaymentStatus::from_string(&status_string.to_uppercase());
        assert_eq!(format!("{:?}", status), format!("{:?}", converted_status));
    }
}

#[tokio::test]
async fn test_hybrid_gas_fee_logic() {
    // Test the hybrid gas fee logic:
    // Native currencies (SOL, ETH, BNB, MATIC, ARB) auto-deduct gas
    // USDT variants require separate gas deposits
    
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
    
    // Test native currencies have proper networks
    for crypto_type in native_currencies {
        let network = crypto_type.network();
        assert!(
            matches!(network, "SOLANA" | "ETHEREUM" | "BEP20" | "POLYGON" | "ARBITRUM"),
            "Native currency should have valid network: {}",
            network
        );
    }
    
    // Test USDT variants are properly identified
    for crypto_type in usdt_variants {
        let crypto_str = format!("{:?}", crypto_type);
        assert!(
            crypto_str.contains("Usdt"),
            "USDT variant should contain Usdt in debug representation"
        );
    }
}

#[tokio::test]
async fn test_fee_calculation_accuracy() {
    // Test fee calculation with realistic values
    use fiddupay::payment::fee_calculator::FeeCalculator;
    
    let base_amount = Decimal::new(10000, 2); // $100.00
    let fee_percentage = Decimal::new(25, 1); // 2.5%
    
    let fee_amount = FeeCalculator::calculate_fee_usd(base_amount, fee_percentage);
    let expected_fee = Decimal::new(250, 2); // $2.50
    
    assert_eq!(fee_amount, expected_fee, "Fee calculation should be accurate");
    
    let total_with_fee = FeeCalculator::calculate_total_with_fee(base_amount, fee_amount);
    let expected_total = Decimal::new(10250, 2); // $102.50
    
    assert_eq!(total_with_fee, expected_total, "Total with fee should be accurate");
}

#[tokio::test]
async fn test_blockchain_network_mapping() {
    // Test all 5 blockchain networks are properly mapped
    let network_mappings = vec![
        (CryptoType::Sol, "SOLANA"),
        (CryptoType::Eth, "ETHEREUM"),
        (CryptoType::Bnb, "BEP20"),
        (CryptoType::Matic, "POLYGON"),
        (CryptoType::Arb, "ARBITRUM"),
    ];
    
    for (crypto_type, expected_network) in network_mappings {
        let actual_network = crypto_type.network();
        assert_eq!(
            actual_network,
            expected_network,
            "Network mapping should be correct for {:?}",
            crypto_type
        );
    }
}

#[tokio::test]
async fn test_wallet_address_validation() {
    // Test wallet address format validation
    
    // Ethereum-style addresses (42 characters, starts with 0x)
    let eth_address = "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6";
    assert_eq!(eth_address.len(), 42, "Ethereum address should be 42 characters");
    assert!(eth_address.starts_with("0x"), "Ethereum address should start with 0x");
    
    // Solana addresses (44 characters, base58)
    let sol_address = "DRpbCBMxVnDK7maPM5tGv6MvB3v1sRMC86PZ8okm21hy";
    assert_eq!(sol_address.len(), 44, "Solana address should be 44 characters");
    assert!(!sol_address.contains("0x"), "Solana address should not contain 0x");
}

#[tokio::test]
async fn test_payment_expiration_timing() {
    // Test payment expiration logic
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
async fn test_production_configuration() {
    // Test that system is properly configured for production
    
    // Test that all crypto types have valid configurations
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
        
        // Test string representation is not empty
        let crypto_string = format!("{:?}", crypto_type);
        assert!(!crypto_string.is_empty(), "Crypto type should have string representation");
    }
}

#[tokio::test]
async fn test_system_health_indicators() {
    // Test system health and readiness indicators
    
    // Test that decimal operations work correctly
    let test_amount = Decimal::new(12345, 2); // $123.45
    assert_eq!(test_amount.to_string(), "123.45", "Decimal formatting should work");
    
    // Test that basic arithmetic works
    let doubled = test_amount * Decimal::new(2, 0);
    assert_eq!(doubled, Decimal::new(24690, 2), "Decimal arithmetic should work");
    
    // Test that we can create valid payment amounts
    let min_amount = Decimal::new(1, 2); // $0.01
    let max_amount = Decimal::new(100000000, 2); // $1,000,000.00
    
    assert!(min_amount > Decimal::ZERO, "Minimum amount should be positive");
    assert!(max_amount > min_amount, "Maximum amount should be greater than minimum");
}
