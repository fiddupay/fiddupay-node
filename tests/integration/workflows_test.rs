// fiddupay - End-to-End Workflow Tests

use sqlx::PgPool;
use crypto_payment_gateway::services::{
    merchant_service::MerchantService,
    payment_service::PaymentService,
    balance_service::BalanceService,
    withdrawal_service::WithdrawalService,
};
use crypto_payment_gateway::payment::models::CreatePaymentRequest;
use crypto_payment_gateway::services::withdrawal_service::WithdrawalRequest;
use rust_decimal::Decimal;
use std::{str::FromStr, sync::Arc};

async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://vibes:password@localhost:5432/fiddupay_test".to_string());
    
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

#[tokio::test]
async fn test_complete_payment_flow() {
    let pool = setup_test_db().await;
    
    // 1. Register merchant
    let merchant_service = MerchantService::new(pool.clone());
    let email = format!("flow{}@example.com", nanoid::nanoid!(8));
    let merchant = merchant_service
        .register_merchant(email, "Flow Test".to_string())
        .await
        .unwrap();
    
    // 2. Set wallet address
    merchant_service
        .set_wallet_address(
            merchant.merchant_id,
            "SOL",
            "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"
        )
        .await
        .unwrap();
    
    // 3. Create payment
    let payment_service = PaymentService::new(pool.clone(), "http://localhost:8080".to_string());
    let request = CreatePaymentRequest {
        amount_usd: Decimal::from_str("100.00").unwrap(),
        crypto_type: "SOL".to_string(),
        description: Some("E2E Test".to_string()),
        metadata: None,
        expiration_minutes: Some(15),
    };
    
    let payment = payment_service
        .create_payment(merchant.merchant_id as i64, request)
        .await
        .unwrap();
    
    assert!(payment.payment_id.starts_with("pay_"));
    assert_eq!(payment.status, "PENDING");
    
    // 4. Verify payment can be retrieved
    let retrieved = payment_service
        .get_payment(&payment.payment_id, merchant.merchant_id as i64)
        .await
        .unwrap();
    
    assert_eq!(retrieved.payment_id, payment.payment_id);
}

#[tokio::test]
async fn test_complete_withdrawal_flow() {
    let pool = setup_test_db().await;
    let balance_service = Arc::new(BalanceService::new(pool.clone()));
    let withdrawal_service = WithdrawalService::new(pool.clone(), balance_service.clone());
    
    let merchant_id = 1;
    
    // 1. Credit balance (simulate payment received)
    balance_service
        .credit_available(
            merchant_id,
            "SOL",
            Decimal::from_str("100.0").unwrap(),
            "PAYMENT_CONFIRMED",
            Some("pay_test123")
        )
        .await
        .unwrap();
    
    // 2. Check balance
    let balance = balance_service.get_balance(merchant_id).await.unwrap();
    assert!(!balance.balances.is_empty());
    
    // 3. Request withdrawal
    let request = WithdrawalRequest {
        crypto_type: "SOL".to_string(),
        amount: Decimal::from_str("50.0").unwrap(),
        destination_address: "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU".to_string(),
    };
    
    let withdrawal = withdrawal_service
        .create_withdrawal(merchant_id, request)
        .await
        .unwrap();
    
    assert_eq!(withdrawal.status, "APPROVED");
    
    // 4. List withdrawals
    let withdrawals = withdrawal_service
        .list_withdrawals(merchant_id, 10)
        .await
        .unwrap();
    
    assert!(!withdrawals.is_empty());
}

#[tokio::test]
async fn test_balance_reserve_and_release() {
    let pool = setup_test_db().await;
    let service = BalanceService::new(pool);
    
    let merchant_id = 1;
    let crypto_type = "SOL";
    let amount = Decimal::from_str("100.0").unwrap();
    
    // 1. Credit balance
    service
        .credit_available(merchant_id, crypto_type, amount, "TEST", None)
        .await
        .unwrap();
    
    // 2. Reserve some
    let reserve_amount = Decimal::from_str("30.0").unwrap();
    service
        .reserve(merchant_id, crypto_type, reserve_amount, "WITHDRAWAL", Some("wd_123"))
        .await
        .unwrap();
    
    // 3. Check balance
    let balance = service.get_balance(merchant_id).await.unwrap();
    let sol_balance = balance.balances.iter()
        .find(|b| b.crypto_type == crypto_type)
        .unwrap();
    
    assert_eq!(sol_balance.reserved_balance, reserve_amount);
    
    // 4. Release reserve
    service
        .release_reserve(merchant_id, crypto_type, reserve_amount, "CANCELLED", Some("wd_123"))
        .await
        .unwrap();
    
    // 5. Verify released
    let balance = service.get_balance(merchant_id).await.unwrap();
    let sol_balance = balance.balances.iter()
        .find(|b| b.crypto_type == crypto_type)
        .unwrap();
    
    assert_eq!(sol_balance.reserved_balance, Decimal::ZERO);
}

#[tokio::test]
async fn test_merchant_authentication_flow() {
    let pool = setup_test_db().await;
    let service = MerchantService::new(pool);
    
    // 1. Register
    let email = format!("auth{}@example.com", nanoid::nanoid!(8));
    let merchant = service
        .register_merchant(email.clone(), "Auth Test".to_string())
        .await
        .unwrap();
    
    let api_key = merchant.api_key.clone();
    
    // 2. Authenticate with API key
    let result = service.authenticate(&api_key).await;
    assert!(result.is_ok());
    
    // 3. Rotate API key
    let new_key = service
        .rotate_api_key(merchant.merchant_id)
        .await
        .unwrap();
    
    assert_ne!(api_key, new_key);
    
    // 4. Old key should fail
    let result = service.authenticate(&api_key).await;
    assert!(result.is_err());
    
    // 5. New key should work
    let result = service.authenticate(&new_key).await;
    assert!(result.is_ok());
}
