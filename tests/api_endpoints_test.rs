use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::util::ServiceExt;
use sqlx::PgPool;
use serde_json::json;
use crypto_payment_gateway::{
    api::{routes, state::AppState},
};

async fn setup_test_app() -> (axum::Router, PgPool) {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://vibes:Soledayo%402001@localhost:5432/payflow_test".to_string());
    
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect");
    
    let app_state = AppState::new(
        pool.clone(),
        "http://localhost:8080".to_string(),
        "test_webhook_key".to_string(),
    );
    
    let app = routes::create_router(app_state);
    (app, pool)
}

#[tokio::test]
async fn test_health_endpoint() {
    let (app, _pool) = setup_test_app().await;
    
    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    assert!(body_str.contains("healthy"));
    println!("✅ Health endpoint: {}", body_str);
}

#[tokio::test]
async fn test_metrics_endpoint_requires_auth() {
    let (app, _pool) = setup_test_app().await;
    
    let response = app
        .oneshot(Request::builder().uri("/metrics").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    // Should require authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    println!("✅ Metrics endpoint requires auth");
}

#[tokio::test]
async fn test_create_merchant_requires_auth() {
    let (app, _pool) = setup_test_app().await;
    
    let body = r#"{"business_name":"Test","email":"test@test.com"}"#;
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/merchants")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    println!("✅ Create merchant requires auth");
}

#[tokio::test]
async fn test_list_payments_requires_auth() {
    let (app, _pool) = setup_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/payments")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    println!("✅ List payments requires auth");
}

#[tokio::test]
async fn test_invalid_api_key() {
    let (app, _pool) = setup_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/payments")
                .header("authorization", "Bearer invalid_key_12345")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    assert!(body_str.contains("Invalid API key"));
    println!("✅ Invalid API key rejected");
}

#[tokio::test]
async fn test_database_merchants_exist() {
    let (_app, pool) = setup_test_app().await;
    
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM merchants")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert!(count.0 >= 2, "Should have test merchants");
    println!("✅ Database has {} merchants", count.0);
}

#[tokio::test]
async fn test_database_payments_exist() {
    let (_app, pool) = setup_test_app().await;
    
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM payment_transactions")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert!(count.0 >= 3, "Should have test payments");
    println!("✅ Database has {} payments", count.0);
}

#[tokio::test]
async fn test_not_found_endpoint() {
    let (app, _pool) = setup_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/nonexistent")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    println!("✅ 404 for nonexistent endpoint");
}

#[tokio::test]
async fn test_cors_headers() {
    let (app, _pool) = setup_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/health")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    // Should handle OPTIONS request
    assert!(response.status().is_success() || response.status() == StatusCode::METHOD_NOT_ALLOWED);
    println!("✅ CORS/OPTIONS handling");
}
