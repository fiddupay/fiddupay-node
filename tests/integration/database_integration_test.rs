use sqlx::PgPool;

#[tokio::test]
async fn test_database_connection() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://vibes:Soledayo%402001@localhost:5432/payflow_test".to_string());
    
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to database");
    
    // Test query
    let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM merchants")
        .fetch_one(&pool)
        .await
        .expect("Failed to query merchants");
    
    assert!(result.0 >= 2, "Should have at least 2 test merchants");
    println!("✅ Found {} merchants", result.0);
}

#[tokio::test]
async fn test_merchant_data() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://vibes:Soledayo%402001@localhost:5432/payflow_test".to_string());
    
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect");
    
    let merchants: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT id, business_name, email FROM merchants ORDER BY id"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch merchants");
    
    assert_eq!(merchants.len(), 2);
    assert_eq!(merchants[0].1, "Test Merchant 1");
    assert_eq!(merchants[1].1, "Test Merchant 2");
    println!("✅ Merchants: {:?}", merchants);
}

#[tokio::test]
async fn test_payment_data() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://vibes:Soledayo%402001@localhost:5432/payflow_test".to_string());
    
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect");
    
    let payments: Vec<(String, String)> = sqlx::query_as(
        "SELECT payment_id, status FROM payment_transactions ORDER BY payment_id"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch payments");
    
    assert_eq!(payments.len(), 3);
    assert_eq!(payments[0].0, "PAY-TEST-001");
    assert_eq!(payments[0].1, "PENDING");
    println!("✅ Payments: {:?}", payments);
}

#[tokio::test]
async fn test_balance_data() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://vibes:Soledayo%402001@localhost:5432/payflow_test".to_string());
    
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect");
    
    let balances: Vec<(i64, String, String, String)> = sqlx::query_as(
        "SELECT merchant_id, crypto_type, available_balance::text, reserved_balance::text 
         FROM merchant_balances ORDER BY merchant_id, crypto_type"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch balances");
    
    assert!(balances.len() >= 3);
    println!("✅ Balances: {:?}", balances);
}

#[tokio::test]
async fn test_wallet_data() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://vibes:Soledayo%402001@localhost:5432/payflow_test".to_string());
    
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect");
    
    let wallets: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT merchant_id, crypto_type, address FROM merchant_wallets ORDER BY merchant_id"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch wallets");
    
    assert_eq!(wallets.len(), 3);
    println!("✅ Wallets: {:?}", wallets);
}
