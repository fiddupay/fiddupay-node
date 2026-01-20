// PayFlow - Payment Service Tests

use sqlx::PgPool;
use crypto_payment_gateway::services::payment_service::PaymentService;
use crypto_payment_gateway::payment::models::CreatePaymentRequest;
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
async fn test_create_payment() {
    let pool = setup_test_db().await;
    let service = PaymentService::new(pool, "http://localhost:8080".to_string());
    
    let request = CreatePaymentRequest {
        amount_usd: Decimal::from_str("100.00").unwrap(),
        crypto_type: "SOL".to_string(),
        description: Some("Test payment".to_string()),
        metadata: None,
        expiration_minutes: Some(15),
    };
    
    let result = service.create_payment(1, request).await;
    assert!(result.is_ok());
    
    let payment = result.unwrap();
    assert!(payment.payment_id.starts_with("pay_"));
    assert_eq!(payment.amount_usd, Decimal::from_str("100.00").unwrap());
}

#[tokio::test]
async fn test_payment_fee_calculation() {
    let pool = setup_test_db().await;
    let service = PaymentService::new(pool, "http://localhost:8080".to_string());
    
    let request = CreatePaymentRequest {
        amount_usd: Decimal::from_str("100.00").unwrap(),
        crypto_type: "SOL".to_string(),
        description: None,
        metadata: None,
        expiration_minutes: Some(15),
    };
    
    let payment = service.create_payment(1, request).await.unwrap();
    
    // Default fee is 1.5%
    let expected_fee = Decimal::from_str("1.50").unwrap();
    assert_eq!(payment.fee_amount_usd, expected_fee);
}

#[tokio::test]
async fn test_list_payments() {
    let pool = setup_test_db().await;
    let service = PaymentService::new(pool, "http://localhost:8080".to_string());
    
    let result = service.list_payments(1, None, None, None, 10).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_payment() {
    let pool = setup_test_db().await;
    let service = PaymentService::new(pool, "http://localhost:8080".to_string());
    
    // Create payment first
    let request = CreatePaymentRequest {
        amount_usd: Decimal::from_str("50.00").unwrap(),
        crypto_type: "USDT_SPL".to_string(),
        description: None,
        metadata: None,
        expiration_minutes: Some(15),
    };
    
    let created = service.create_payment(1, request).await.unwrap();
    
    // Get payment
    let result = service.get_payment(&created.payment_id, 1).await;
    assert!(result.is_ok());
    
    let payment = result.unwrap();
    assert_eq!(payment.payment_id, created.payment_id);
}

#[tokio::test]
async fn test_payment_not_found() {
    let pool = setup_test_db().await;
    let service = PaymentService::new(pool, "http://localhost:8080".to_string());
    
    let result = service.get_payment("pay_nonexistent", 1).await;
    assert!(result.is_err());
}
