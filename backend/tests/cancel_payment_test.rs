#[cfg(test)]
mod tests {
    use super::*;
    use fiddupay::api::handlers::cancel_payment;
    use fiddupay::api::state::AppState;
    use fiddupay::config::Config;
    use fiddupay::services::merchant_service::MerchantService;
    use sqlx::PgPool;
    use axum::{
        extract::{Path, State},
        http::StatusCode,
        response::IntoResponse,
        Json,
    };
    use serde_json::Value;

    #[sqlx::test]
    async fn test_cancel_payment(pool: PgPool) {
        // Setup
        let config = Config::default();
        let state = AppState {
            db_pool: pool.clone(),
            config: config.clone(),
            merchant_service: MerchantService::new(pool.clone(), config.clone()),
            // ... other services are not needed for this specific handler test if we mock or just use the DB
            ..Default::default() 
        };
        // Note: AppState might need other fields depending on implementation, but for cancel_payment
        // it only uses db_pool. However, AppState construction might be complex.
        // Easier approach: Test logic directly or use a full integration test if we can spin up the app.
        // Since we are inside the crate, we can test the handler if we can construct State.
        
        // Let's Insert a dummy payment
        let merchant_id = 1001;
        let _ = sqlx::query!("INSERT INTO merchants (id, email, business_name, settlement_mode, is_active, fee_percentage, customer_pays_fee, daily_limit_usd, role, password_hash, test_api_key_hash, live_api_key_hash, redirect_url) VALUES ($1, 'cancel_test@example.com', 'Cancel Test', 'managed', true, 0.0, false, 1000.0, 'MERCHANT', 'hash', 'hash', 'hash', 'https://merchant.com/callback')", merchant_id).execute(&pool).await;

        let payment_id = "pay_cancel_test_123";
        let _ = sqlx::query!("INSERT INTO payment_transactions (id, payment_id, merchant_id, amount, amount_usd, crypto_type, status, created_at, updated_at, expires_at) VALUES (1, $1, $2, 10.0, 10.0, 'USDT_BEP20', 'PENDING', NOW(), NOW(), NOW() + INTERVAL '1 hour')", payment_id, merchant_id).execute(&pool).await;

        // Execute Handler
        let response = cancel_payment(
            State(state.clone()),
            Path(payment_id.to_string())
        ).await.into_response();

        // Verify Status Code
        assert_eq!(response.status(), StatusCode::OK);

        // Verify Body (Redirect URL)
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&body_bytes).unwrap();
        
        assert_eq!(body["status"], "CANCELLED");
        assert!(body["redirect_url"].as_str().unwrap().contains("https://merchant.com/callback"));
        assert!(body["redirect_url"].as_str().unwrap().contains("status=cancelled"));

        // Verify DB Update
        let status = sqlx::query!("SELECT status::text as status FROM payment_transactions WHERE payment_id = $1", payment_id)
            .fetch_one(&pool)
            .await
            .unwrap()
            .status
            .unwrap();
            
        assert_eq!(status, "CANCELLED");
    }
}
