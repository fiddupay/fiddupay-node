// PayFlow - API Endpoint Tests

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use serde_json::json;

#[tokio::test]
async fn test_health_check() {
    let app = crypto_payment_gateway::api::routes::create_router(
        setup_app_state().await
    );
    
    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_register_merchant() {
    let app = crypto_payment_gateway::api::routes::create_router(
        setup_app_state().await
    );
    
    let body = json!({
        "email": format!("test{}@example.com", nanoid::nanoid!(8)),
        "business_name": "Test Business"
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/merchants/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_payment_unauthorized() {
    let app = crypto_payment_gateway::api::routes::create_router(
        setup_app_state().await
    );
    
    let body = json!({
        "amount_usd": 100.00,
        "crypto_type": "SOL"
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/payments")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    // Should fail without API key
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_rate_limiting() {
    let app = crypto_payment_gateway::api::routes::create_router(
        setup_app_state().await
    );
    
    // Make 101 requests (limit is 100)
    for i in 0..101 {
        let response = app.clone()
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        
        if i < 100 {
            assert_eq!(response.status(), StatusCode::OK);
        } else {
            assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        }
    }
}

async fn setup_app_state() -> crypto_payment_gateway::api::state::AppState {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://vibes:Soledayo@2001@localhost:5432/payflow_test".to_string());
    
    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");
    
    crypto_payment_gateway::api::state::AppState::new(
        pool,
        "http://localhost:8080".to_string(),
        "test_webhook_key".to_string()
    )
}
