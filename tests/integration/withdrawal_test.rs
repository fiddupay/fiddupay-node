// PayFlow - Withdrawal Service Tests

use sqlx::PgPool;
use crypto_payment_gateway::services::{
    withdrawal_service::{WithdrawalService, WithdrawalRequest},
    balance_service::BalanceService,
};
use rust_decimal::Decimal;
use std::{str::FromStr, sync::Arc};

async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://vibes:Soledayo@2001@localhost:5432/payflow_test".to_string());
    
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

#[tokio::test]
async fn test_create_withdrawal() {
    let pool = setup_test_db().await;
    let balance_service = Arc::new(BalanceService::new(pool.clone()));
    let service = WithdrawalService::new(pool, balance_service.clone());
    
    let merchant_id = 1;
    
    // Credit balance first
    balance_service.credit_available(
        merchant_id,
        "SOL",
        Decimal::from_str("100.0").unwrap(),
        "TEST_CREDIT",
        None
    ).await.unwrap();
    
    // Create withdrawal
    let request = WithdrawalRequest {
        crypto_type: "SOL".to_string(),
        amount: Decimal::from_str("50.0").unwrap(),
        destination_address: "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU".to_string(),
    };
    
    let result = service.create_withdrawal(merchant_id, request).await;
    assert!(result.is_ok());
    
    let withdrawal = result.unwrap();
    assert!(withdrawal.withdrawal_id.starts_with("wd_"));
    assert_eq!(withdrawal.status, "APPROVED"); // < $1000 auto-approved
}

#[tokio::test]
async fn test_withdrawal_minimum_amount() {
    let pool = setup_test_db().await;
    let balance_service = Arc::new(BalanceService::new(pool.clone()));
    let service = WithdrawalService::new(pool, balance_service);
    
    let request = WithdrawalRequest {
        crypto_type: "SOL".to_string(),
        amount: Decimal::from_str("5.0").unwrap(), // Below $10 minimum
        destination_address: "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU".to_string(),
    };
    
    let result = service.create_withdrawal(1, request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_withdrawal_insufficient_balance() {
    let pool = setup_test_db().await;
    let balance_service = Arc::new(BalanceService::new(pool.clone()));
    let service = WithdrawalService::new(pool, balance_service);
    
    let request = WithdrawalRequest {
        crypto_type: "SOL".to_string(),
        amount: Decimal::from_str("1000.0").unwrap(),
        destination_address: "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU".to_string(),
    };
    
    let result = service.create_withdrawal(999, request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_cancel_withdrawal() {
    let pool = setup_test_db().await;
    let balance_service = Arc::new(BalanceService::new(pool.clone()));
    let service = WithdrawalService::new(pool, balance_service.clone());
    
    let merchant_id = 1;
    
    // Credit balance
    balance_service.credit_available(
        merchant_id,
        "SOL",
        Decimal::from_str("2000.0").unwrap(),
        "TEST_CREDIT",
        None
    ).await.unwrap();
    
    // Create withdrawal (>= $1000 requires approval)
    let request = WithdrawalRequest {
        crypto_type: "SOL".to_string(),
        amount: Decimal::from_str("1500.0").unwrap(),
        destination_address: "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU".to_string(),
    };
    
    let withdrawal = service.create_withdrawal(merchant_id, request).await.unwrap();
    assert_eq!(withdrawal.status, "PENDING");
    
    // Cancel it
    let result = service.cancel_withdrawal(merchant_id, &withdrawal.withdrawal_id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_withdrawals() {
    let pool = setup_test_db().await;
    let balance_service = Arc::new(BalanceService::new(pool.clone()));
    let service = WithdrawalService::new(pool, balance_service);
    
    let result = service.list_withdrawals(1, 10).await;
    assert!(result.is_ok());
}
