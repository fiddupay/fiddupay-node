// PayFlow - Service Integration Tests
// Tests for service layer with database

use sqlx::PgPool;
use crypto_payment_gateway::services::{
    merchant_service::MerchantService,
    balance_service::BalanceService,
    deposit_address_service::DepositAddressService,
};
use rust_decimal::Decimal;
use std::str::FromStr;

async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://vibes:Soledayo@2001@localhost:5432/payflow_test".to_string());
    
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

#[tokio::test]
async fn test_merchant_registration() {
    let pool = setup_test_db().await;
    let service = MerchantService::new(pool);
    
    let email = format!("test{}@example.com", nanoid::nanoid!(8));
    let business_name = "Test Business";
    
    let result = service.register_merchant(email.clone(), business_name.to_string()).await;
    assert!(result.is_ok());
    
    let merchant = result.unwrap();
    assert_eq!(merchant.email, email);
    assert_eq!(merchant.business_name, business_name);
    assert!(!merchant.api_key.is_empty());
}

#[tokio::test]
async fn test_balance_credit_debit() {
    let pool = setup_test_db().await;
    let service = BalanceService::new(pool);
    
    let merchant_id = 1;
    let crypto_type = "SOL";
    let amount = Decimal::from_str("10.5").unwrap();
    
    // Credit
    let result = service.credit_available(
        merchant_id,
        crypto_type,
        amount,
        "TEST_CREDIT",
        Some("test_ref")
    ).await;
    assert!(result.is_ok());
    
    // Get balance
    let balance = service.get_balance(merchant_id).await.unwrap();
    let sol_balance = balance.balances.iter()
        .find(|b| b.crypto_type == crypto_type);
    assert!(sol_balance.is_some());
    
    // Debit
    let debit_amount = Decimal::from_str("5.0").unwrap();
    let result = service.debit_available(
        merchant_id,
        crypto_type,
        debit_amount,
        "TEST_DEBIT",
        Some("test_ref2")
    ).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_balance_insufficient_funds() {
    let pool = setup_test_db().await;
    let service = BalanceService::new(pool);
    
    let merchant_id = 999; // Non-existent merchant
    let crypto_type = "SOL";
    let amount = Decimal::from_str("1000.0").unwrap();
    
    let result = service.debit_available(
        merchant_id,
        crypto_type,
        amount,
        "TEST",
        None
    ).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_deposit_address_generation_solana() {
    std::env::set_var("ENCRYPTION_KEY", "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
    
    let pool = setup_test_db().await;
    let service = DepositAddressService::new(pool).unwrap();
    
    let payment_id = format!("pay_{}", nanoid::nanoid!(12));
    let merchant_wallet = "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU";
    
    let result = service.generate_deposit_address(
        &payment_id,
        "SOL",
        merchant_wallet,
        15
    ).await;
    
    assert!(result.is_ok());
    let deposit = result.unwrap();
    assert_eq!(deposit.payment_id, payment_id);
    assert!(!deposit.deposit_address.is_empty());
    assert_eq!(deposit.merchant_destination, merchant_wallet);
}

#[tokio::test]
async fn test_deposit_address_generation_evm() {
    std::env::set_var("ENCRYPTION_KEY", "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
    
    let pool = setup_test_db().await;
    let service = DepositAddressService::new(pool).unwrap();
    
    let payment_id = format!("pay_{}", nanoid::nanoid!(12));
    let merchant_wallet = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
    
    let result = service.generate_deposit_address(
        &payment_id,
        "USDT_BEP20",
        merchant_wallet,
        15
    ).await;
    
    assert!(result.is_ok());
    let deposit = result.unwrap();
    assert!(deposit.deposit_address.starts_with("0x"));
    assert_eq!(deposit.deposit_address.len(), 42);
}

#[tokio::test]
async fn test_deposit_address_private_key_encryption() {
    std::env::set_var("ENCRYPTION_KEY", "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
    
    let pool = setup_test_db().await;
    let service = DepositAddressService::new(pool).unwrap();
    
    let payment_id = format!("pay_{}", nanoid::nanoid!(12));
    
    // Generate address
    service.generate_deposit_address(
        &payment_id,
        "SOL",
        "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
        15
    ).await.unwrap();
    
    // Retrieve private key
    let private_key = service.get_private_key(&payment_id).await;
    assert!(private_key.is_ok());
    assert!(!private_key.unwrap().is_empty());
}
