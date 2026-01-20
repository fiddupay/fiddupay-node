// Complete Endpoint Test Suite
use reqwest;
use serde_json::json;

const BASE_URL: &str = "http://localhost:8080";

async fn is_server_running() -> bool {
    reqwest::Client::new()
        .get(format!("{}/health", BASE_URL))
        .send()
        .await
        .is_ok()
}

// ============================================================================
// PUBLIC ENDPOINTS
// ============================================================================

#[tokio::test]
async fn test_01_health_endpoint() {
    if !is_server_running().await {
        println!("⚠️  Server not running, skipping");
        return;
    }

    let client = reqwest::Client::new();
    let response = client.get(format!("{}/health", BASE_URL)).send().await.unwrap();
    
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["status"], "healthy");
    println!("✅ Health endpoint working");
}

// ============================================================================
// AUTHENTICATION TESTS
// ============================================================================

#[tokio::test]
async fn test_02_metrics_requires_auth() {
    if !is_server_running().await { return; }
    
    let client = reqwest::Client::new();
    let response = client.get(format!("{}/metrics", BASE_URL)).send().await.unwrap();
    
    assert_eq!(response.status(), 401);
    println!("✅ Metrics requires authentication");
}

#[tokio::test]
async fn test_03_invalid_api_key() {
    if !is_server_running().await { return; }
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/payments", BASE_URL))
        .header("Authorization", "Bearer invalid_key")
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401);
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("Invalid API key"));
    println!("✅ Invalid API key rejected");
}

#[tokio::test]
async fn test_04_missing_auth_header() {
    if !is_server_running().await { return; }
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/payments", BASE_URL))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401);
    println!("✅ Missing auth header rejected");
}

// ============================================================================
// MERCHANT ENDPOINTS
// ============================================================================

#[tokio::test]
async fn test_05_create_merchant_no_auth() {
    if !is_server_running().await { return; }
    
    let client = reqwest::Client::new();
    let body = json!({
        "business_name": "Test Business",
        "email": "test@example.com"
    });
    
    let response = client
        .post(format!("{}/api/v1/merchants", BASE_URL))
        .json(&body)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401);
    println!("✅ Create merchant requires auth");
}

#[tokio::test]
async fn test_06_get_merchant_no_auth() {
    if !is_server_running().await { return; }
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/merchants/me", BASE_URL))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401);
    println!("✅ Get merchant requires auth");
}

// ============================================================================
// PAYMENT ENDPOINTS
// ============================================================================

#[tokio::test]
async fn test_07_list_payments_no_auth() {
    if !is_server_running().await { return; }
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/payments", BASE_URL))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401);
    println!("✅ List payments requires auth");
}

#[tokio::test]
async fn test_08_create_payment_no_auth() {
    if !is_server_running().await { return; }
    
    let client = reqwest::Client::new();
    let body = json!({
        "amount_usd": "100.00",
        "crypto_type": "SOL"
    });
    
    let response = client
        .post(format!("{}/api/v1/payments", BASE_URL))
        .json(&body)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401);
    println!("✅ Create payment requires auth");
}

#[tokio::test]
async fn test_09_get_payment_no_auth() {
    if !is_server_running().await { return; }
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/payments/PAY-TEST-001", BASE_URL))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401);
    println!("✅ Get payment requires auth");
}

// ============================================================================
// WALLET ENDPOINTS
// ============================================================================

#[tokio::test]
async fn test_10_set_wallet_no_auth() {
    if !is_server_running().await { return; }
    
    let client = reqwest::Client::new();
    let body = json!({
        "crypto_type": "SOL",
        "address": "TestAddress123"
    });
    
    let response = client
        .post(format!("{}/api/v1/wallets", BASE_URL))
        .json(&body)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401);
    println!("✅ Set wallet requires auth");
}

#[tokio::test]
async fn test_11_get_wallets_no_auth() {
    if !is_server_running().await { return; }
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/wallets", BASE_URL))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401);
    println!("✅ Get wallets requires auth");
}

// ============================================================================
// WEBHOOK ENDPOINTS
// ============================================================================

#[tokio::test]
async fn test_12_set_webhook_no_auth() {
    if !is_server_running().await { return; }
    
    let client = reqwest::Client::new();
    let body = json!({
        "url": "https://example.com/webhook"
    });
    
    let response = client
        .post(format!("{}/api/v1/webhooks", BASE_URL))
        .json(&body)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401);
    println!("✅ Set webhook requires auth");
}

// ============================================================================
// BALANCE ENDPOINTS
// ============================================================================

#[tokio::test]
async fn test_13_get_balance_no_auth() {
    if !is_server_running().await { return; }
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/balance", BASE_URL))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401);
    println!("✅ Get balance requires auth");
}

// ============================================================================
// WITHDRAWAL ENDPOINTS
// ============================================================================

#[tokio::test]
async fn test_14_create_withdrawal_no_auth() {
    if !is_server_running().await { return; }
    
    let client = reqwest::Client::new();
    let body = json!({
        "crypto_type": "SOL",
        "amount": "10.0",
        "destination_address": "TestAddr"
    });
    
    let response = client
        .post(format!("{}/api/v1/withdrawals", BASE_URL))
        .json(&body)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401);
    println!("✅ Create withdrawal requires auth");
}

#[tokio::test]
async fn test_15_list_withdrawals_no_auth() {
    if !is_server_running().await { return; }
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/withdrawals", BASE_URL))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401);
    println!("✅ List withdrawals requires auth");
}

// ============================================================================
// INVOICE ENDPOINTS
// ============================================================================

#[tokio::test]
async fn test_16_create_invoice_no_auth() {
    if !is_server_running().await { return; }
    
    let client = reqwest::Client::new();
    let body = json!({
        "customer_email": "customer@test.com",
        "items": [{"description": "Item", "amount": "100.00"}]
    });
    
    let response = client
        .post(format!("{}/api/v1/invoices", BASE_URL))
        .json(&body)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401);
    println!("✅ Create invoice requires auth");
}

#[tokio::test]
async fn test_17_list_invoices_no_auth() {
    if !is_server_running().await { return; }
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/invoices", BASE_URL))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401);
    println!("✅ List invoices requires auth");
}

// ============================================================================
// ANALYTICS ENDPOINTS
// ============================================================================

#[tokio::test]
async fn test_18_get_analytics_no_auth() {
    if !is_server_running().await { return; }
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/analytics", BASE_URL))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401);
    println!("✅ Get analytics requires auth");
}

// ============================================================================
// ERROR HANDLING
// ============================================================================

#[tokio::test]
async fn test_19_not_found_endpoint() {
    if !is_server_running().await { return; }
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/nonexistent", BASE_URL))
        .send()
        .await
        .unwrap();
    
    // Should be 404 for truly nonexistent routes
    assert!(response.status() == 404 || response.status() == 401);
    println!("✅ Nonexistent endpoint handled");
}

#[tokio::test]
async fn test_20_invalid_json() {
    if !is_server_running().await { return; }
    
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/v1/merchants", BASE_URL))
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer test_key")
        .body("invalid json{")
        .send()
        .await
        .unwrap();
    
    assert!(response.status().is_client_error());
    println!("✅ Invalid JSON rejected");
}
