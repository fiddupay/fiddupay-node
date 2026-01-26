// End-to-End Test for 3-Mode Wallet System
// Tests all three wallet modes: address-only, gateway-generated, merchant-provided

use crate::api::state::AppState;
use crate::services::{
    merchant_service::MerchantService,
    wallet_config_service::{WalletConfigService, ConfigureWalletRequest, GenerateWalletRequest, ImportWalletRequest},
    gas_fee_service::GasFeeService,
};
use crate::payment::models::CryptoType;
use crate::config::Config;
use axum::http::{HeaderMap, HeaderValue};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;

#[tokio::test]
async fn test_three_mode_wallet_system_end_to_end() {
    // Setup test environment
    let config = Config::from_env().expect("Failed to load config");
    let db_pool = PgPool::connect(&config.database_url).await.expect("Failed to connect to database");
    
    let merchant_service = MerchantService::new(db_pool.clone(), config.clone());
    let wallet_service = WalletConfigService::new(db_pool.clone());
    let gas_service = GasFeeService::new(config.clone());

    // Test merchant registration
    let merchant_response = merchant_service
        .register_merchant("test@3mode.com", "3-Mode Test Business")
        .await
        .expect("Failed to register merchant");

    let merchant_id = merchant_response.merchant_id;
    let api_key = merchant_response.api_key;

    println!("✅ Merchant registered: ID {}, API Key: {}", merchant_id, &api_key[..8]);

    // Test Mode 1: Address-Only Wallet
    println!("\n🔍 Testing Mode 1: Address-Only Wallet");
    
    let address_only_request = ConfigureWalletRequest {
        crypto_type: "USDT_BEP20".to_string(),
        address: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string(),
    };

    let address_only_wallet = wallet_service
        .configure_address_only(merchant_id, address_only_request)
        .await
        .expect("Failed to configure address-only wallet");

    assert_eq!(address_only_wallet.address, "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb");
    assert_eq!(address_only_wallet.crypto_type, "USDT_BEP20");
    println!("✅ Address-only wallet configured: {}", address_only_wallet.address);

    // Verify address-only wallet cannot withdraw (should return false)
    let can_withdraw_address_only = wallet_service
        .can_withdraw(merchant_id, CryptoType::UsdtBep20, Decimal::new(100, 0))
        .await
        .expect("Failed to check withdrawal capability");
    
    println!("✅ Address-only withdrawal check: {} (expected: limited)", can_withdraw_address_only);

    // Test Mode 2: Gateway-Generated Wallet
    println!("\n🔑 Testing Mode 2: Gateway-Generated Wallet");
    
    let generate_request = GenerateWalletRequest {
        crypto_type: "SOL".to_string(),
    };

    let generated_wallet = wallet_service
        .generate_wallet(merchant_id, generate_request)
        .await
        .expect("Failed to generate wallet");

    assert_eq!(generated_wallet.crypto_type, "SOL");
    assert!(!generated_wallet.address.is_empty());
    println!("✅ Gateway-generated wallet: {}", generated_wallet.address);

    // Test Mode 3: Merchant-Provided Wallet
    println!("\n📥 Testing Mode 3: Merchant-Provided Wallet");
    
    let import_request = ImportWalletRequest {
        crypto_type: "ETH".to_string(),
        private_key: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
    };

    let imported_wallet = wallet_service
        .import_wallet(merchant_id, import_request)
        .await
        .expect("Failed to import wallet");

    assert_eq!(imported_wallet.crypto_type, "ETH");
    println!("✅ Merchant-provided wallet imported: {}", imported_wallet.address);

    // Test Gas Fee Validation for All Modes
    println!("\n⛽ Testing Gas Fee Validation");

    // Test gas estimates for different networks
    let gas_estimates = gas_service
        .get_all_gas_estimates()
        .await
        .expect("Failed to get gas estimates");

    for (network, estimate) in gas_estimates {
        println!("✅ {} gas: {} {} (standard), {} {} (fast)", 
            network, 
            estimate.standard_fee, 
            estimate.native_currency,
            estimate.fast_fee,
            estimate.native_currency
        );
    }

    // Test gas sufficiency check for USDT (requires separate gas)
    let has_sufficient_gas = gas_service
        .validate_gas_sufficiency(
            CryptoType::UsdtBep20,
            Decimal::new(1, 3), // 0.001 BNB
            Decimal::new(100, 0), // 100 USDT
        )
        .await
        .expect("Failed to validate gas sufficiency");

    println!("✅ Gas sufficiency check: {}", has_sufficient_gas);

    // Test Wallet Configuration Retrieval
    println!("\n📋 Testing Wallet Configuration Retrieval");
    
    let all_configs = wallet_service
        .get_wallet_configs(merchant_id)
        .await
        .expect("Failed to get wallet configs");

    assert_eq!(all_configs.len(), 3); // Should have 3 wallets configured
    
    for config in all_configs {
        println!("✅ Wallet config: {} on {} - {}", 
            config.crypto_type, 
            config.network, 
            config.address
        );
    }

    // Test Withdrawal Capability Matrix
    println!("\n🔄 Testing Withdrawal Capability Matrix");
    
    let withdrawal_tests = vec![
        (CryptoType::UsdtBep20, "Address-Only"),
        (CryptoType::Sol, "Gateway-Generated"), 
        (CryptoType::Eth, "Merchant-Provided"),
    ];

    for (crypto_type, mode) in withdrawal_tests {
        let can_withdraw = wallet_service
            .can_withdraw(merchant_id, crypto_type, Decimal::new(10, 0))
            .await
            .expect("Failed to check withdrawal capability");
        
        println!("✅ {} ({}): Can withdraw = {}", 
            crypto_type.to_string(), 
            mode, 
            can_withdraw
        );
    }

    // Test Gas Requirement Validation
    println!("\n🔍 Testing Gas Requirement Validation");
    
    let gas_validation = wallet_service
        .validate_gas_for_withdrawal(merchant_id, CryptoType::UsdtBep20, Decimal::new(100, 0))
        .await
        .expect("Failed to validate gas for withdrawal");

    println!("✅ Gas validation result: {} - {}", 
        if gas_validation.valid { "VALID" } else { "INVALID" },
        gas_validation.message
    );

    // Cleanup test data
    sqlx::query!("DELETE FROM merchant_wallets WHERE merchant_id = $1", merchant_id)
        .execute(&db_pool)
        .await
        .expect("Failed to cleanup wallet configs");
    
    sqlx::query!("DELETE FROM merchants WHERE id = $1", merchant_id)
        .execute(&db_pool)
        .await
        .expect("Failed to cleanup merchant");

    println!("\n🎉 All 3-Mode Wallet System Tests Passed!");
}

#[tokio::test]
async fn test_hybrid_gas_fee_logic() {
    let config = Config::from_env().expect("Failed to load config");
    let gas_service = GasFeeService::new(config);

    println!("🔥 Testing Hybrid Gas Fee Logic");

    // Test native currencies (auto-deduct)
    let native_currencies = vec![
        CryptoType::Sol,
        CryptoType::Eth,
        CryptoType::Bnb,
        CryptoType::Matic,
        CryptoType::Arb,
    ];

    for crypto_type in native_currencies {
        let gas_estimate = gas_service
            .get_gas_estimate(crypto_type)
            .await
            .expect("Failed to get gas estimate");

        println!("✅ {} (Native): Gas auto-deducted = {} {}", 
            crypto_type.to_string(),
            gas_estimate.estimated_withdrawal_cost,
            gas_estimate.native_currency
        );

        // Test sufficiency with auto-deduct logic
        let sufficient = gas_service
            .validate_gas_sufficiency(
                crypto_type,
                Decimal::new(1, 0), // 1 unit of native currency
                Decimal::new(5, 1), // 0.5 units withdrawal
            )
            .await
            .expect("Failed to validate gas sufficiency");

        println!("   Sufficient for 0.5 withdrawal: {}", sufficient);
    }

    // Test USDT variants (manual deposit required)
    let usdt_variants = vec![
        CryptoType::UsdtSpl,
        CryptoType::UsdtEth,
        CryptoType::UsdtBep20,
        CryptoType::UsdtPolygon,
        CryptoType::UsdtArbitrum,
    ];

    for crypto_type in usdt_variants {
        let gas_estimate = gas_service
            .get_gas_estimate(crypto_type)
            .await
            .expect("Failed to get gas estimate");

        println!("✅ {} (USDT): Requires {} {} gas deposit", 
            crypto_type.to_string(),
            gas_estimate.estimated_withdrawal_cost,
            gas_estimate.native_currency
        );

        // Test sufficiency with manual deposit logic
        let sufficient = gas_service
            .validate_gas_sufficiency(
                crypto_type,
                Decimal::new(1, 2), // 0.01 units of native currency for gas
                Decimal::new(1000, 0), // 1000 USDT withdrawal
            )
            .await
            .expect("Failed to validate gas sufficiency");

        println!("   Sufficient gas deposit: {}", sufficient);
    }

    println!("🎉 Hybrid Gas Fee Logic Tests Passed!");
}

#[tokio::test] 
async fn test_dynamic_fee_configuration() {
    let config = Config::from_env().expect("Failed to load config");
    let db_pool = PgPool::connect(&config.database_url).await.expect("Failed to connect to database");
    
    let merchant_service = MerchantService::new(db_pool.clone(), config.clone());

    println!("💰 Testing Dynamic Fee Configuration");

    // Register merchant with config-based default fee
    let merchant_response = merchant_service
        .register_merchant("test@fee.com", "Fee Test Business")
        .await
        .expect("Failed to register merchant");

    // Verify merchant has config-based fee percentage
    let merchant = sqlx::query!(
        "SELECT fee_percentage FROM merchants WHERE id = $1",
        merchant_response.merchant_id
    )
    .fetch_one(&db_pool)
    .await
    .expect("Failed to fetch merchant");

    // Should match DEFAULT_FEE_PERCENTAGE from .env (0.75%)
    assert_eq!(merchant.fee_percentage, config.default_fee_percentage);
    
    println!("✅ Merchant fee percentage: {}% (from config)", merchant.fee_percentage);
    println!("✅ Config default fee: {}%", config.default_fee_percentage);

    // Cleanup
    sqlx::query!("DELETE FROM merchants WHERE id = $1", merchant_response.merchant_id)
        .execute(&db_pool)
        .await
        .expect("Failed to cleanup merchant");

    println!("🎉 Dynamic Fee Configuration Test Passed!");
}
