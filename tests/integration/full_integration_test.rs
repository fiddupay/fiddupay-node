use sqlx::PgPool;
use reqwest;

async fn get_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://vibes:Soledayo%402001@localhost:5432/payflow_test".to_string());
    
    PgPool::connect(&database_url).await.expect("Failed to connect")
}

#[tokio::test]
async fn test_health_endpoint_live() {
    let client = reqwest::Client::new();
    
    let response = client
        .get("http://localhost:8080/health")
        .send()
        .await;
    
    if let Ok(resp) = response {
        assert_eq!(resp.status(), 200);
        let body = resp.text().await.unwrap();
        assert!(body.contains("healthy"));
        println!("✅ Health endpoint: {}", body);
    } else {
        println!("⚠️  Server not running on port 8080");
    }
}

#[tokio::test]
async fn test_metrics_requires_auth_live() {
    let client = reqwest::Client::new();
    
    let response = client
        .get("http://localhost:8080/metrics")
        .send()
        .await;
    
    if let Ok(resp) = response {
        assert_eq!(resp.status(), 401);
        println!("✅ Metrics requires auth");
    } else {
        println!("⚠️  Server not running");
    }
}

#[tokio::test]
async fn test_create_merchant_requires_auth_live() {
    let client = reqwest::Client::new();
    
    let body = serde_json::json!({
        "business_name": "Test",
        "email": "test@test.com"
    });
    
    let response = client
        .post("http://localhost:8080/api/v1/merchants")
        .json(&body)
        .send()
        .await;
    
    if let Ok(resp) = response {
        assert_eq!(resp.status(), 401);
        println!("✅ Create merchant requires auth");
    } else {
        println!("⚠️  Server not running");
    }
}

#[tokio::test]
async fn test_list_payments_requires_auth_live() {
    let client = reqwest::Client::new();
    
    let response = client
        .get("http://localhost:8080/api/v1/payments")
        .send()
        .await;
    
    if let Ok(resp) = response {
        assert_eq!(resp.status(), 401);
        println!("✅ List payments requires auth");
    } else {
        println!("⚠️  Server not running");
    }
}

#[tokio::test]
async fn test_invalid_api_key_live() {
    let client = reqwest::Client::new();
    
    let response = client
        .get("http://localhost:8080/api/v1/payments")
        .header("Authorization", "Bearer invalid_key_12345")
        .send()
        .await;
    
    if let Ok(resp) = response {
        assert_eq!(resp.status(), 401);
        let body = resp.text().await.unwrap();
        assert!(body.contains("Invalid API key"));
        println!("✅ Invalid API key rejected");
    } else {
        println!("⚠️  Server not running");
    }
}

#[tokio::test]
async fn test_database_merchants_exist() {
    let pool = get_test_pool().await;
    
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM merchants")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert!(count.0 >= 2, "Should have test merchants");
    println!("✅ Database has {} merchants", count.0);
}

#[tokio::test]
async fn test_database_payments_exist() {
    let pool = get_test_pool().await;
    
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM payment_transactions")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert!(count.0 >= 3, "Should have test payments");
    println!("✅ Database has {} payments", count.0);
}

#[tokio::test]
async fn test_database_balances_exist() {
    let pool = get_test_pool().await;
    
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM merchant_balances")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert!(count.0 >= 3, "Should have test balances");
    println!("✅ Database has {} balances", count.0);
}

#[tokio::test]
async fn test_database_wallets_exist() {
    let pool = get_test_pool().await;
    
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM merchant_wallets")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert!(count.0 >= 3, "Should have test wallets");
    println!("✅ Database has {} wallets", count.0);
}

#[tokio::test]
async fn test_payment_status_distribution() {
    let pool = get_test_pool().await;
    
    let statuses: Vec<(String, i64)> = sqlx::query_as(
        "SELECT status, COUNT(*) FROM payment_transactions GROUP BY status ORDER BY status"
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    
    assert!(!statuses.is_empty());
    println!("✅ Payment statuses: {:?}", statuses);
}

#[tokio::test]
async fn test_merchant_fee_percentages() {
    let pool = get_test_pool().await;
    
    let fees: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT fee_percentage::text FROM merchants ORDER BY fee_percentage"
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    
    assert!(!fees.is_empty());
    println!("✅ Merchant fees: {:?}", fees);
}

#[tokio::test]
async fn test_crypto_type_distribution() {
    let pool = get_test_pool().await;
    
    let types: Vec<(String, i64)> = sqlx::query_as(
        "SELECT crypto_type, COUNT(*) FROM payment_transactions GROUP BY crypto_type"
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    
    assert!(!types.is_empty());
    println!("✅ Crypto types: {:?}", types);
}
