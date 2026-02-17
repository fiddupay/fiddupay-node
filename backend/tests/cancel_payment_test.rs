#[cfg(test)]
mod tests {
    use fiddupay::api::handlers::public_cancel_payment;
    use fiddupay::api::state::AppState;
    use fiddupay::config::Config;
    use fiddupay::services::merchant_service::MerchantService;
    use sqlx::postgres::PgPoolOptions;
    use std::env;
    use axum::{
        extract::{Path, State},
        http::StatusCode,
        response::IntoResponse,
    };
    use serde_json::Value;

    async fn get_test_pool() -> sqlx::PgPool {
        dotenvy::dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to DB")
    }

    #[tokio::test]
    async fn test_cancel_payment() {
        let pool = get_test_pool().await;
        
        // Setup
        let config = Config {
            database_url: "postgres://localhost/test".to_string(),
            database_max_connections: 10,
            database_timeout_seconds: 30,
            redis_url: "redis://localhost:6379".to_string(),
            redis_max_connections: 10,
            redis_timeout_seconds: 30,
            server_host: "127.0.0.1".to_string(),
            server_port: 8080,
            server_workers: 4,
            request_timeout_seconds: 30,
            solana_rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            ethereum_rpc_url: "https://eth-mainnet.g.alchemy.com/v2/demo".to_string(),
            bsc_rpc_url: "https://bsc-dataseed.binance.org".to_string(),
            arbitrum_rpc_url: "https://arb1.arbitrum.io/rpc".to_string(),
            polygon_rpc_url: "https://polygon-rpc.com".to_string(),
            solana_devnet_rpc_url: "https://api.devnet.solana.com".to_string(),
            ethereum_sepolia_rpc_url: "https://eth-sepolia.g.alchemy.com/v2/demo".to_string(),
            bsc_testnet_rpc_url: "https://data-seed-prebsc-1-s1.binance.org:8545".to_string(),
            arbitrum_sepolia_rpc_url: "https://sepolia-rollup.arbitrum.io/rpc".to_string(),
            polygon_mumbai_rpc_url: "https://rpc-mumbai.maticvigil.com".to_string(),
            confirmation_blocks_sol: 1,
            confirmation_blocks_eth: 12,
            confirmation_blocks_bsc: 3,
            confirmation_blocks_polygon: 20,
            confirmation_blocks_arbitrum: 1,
            ethereum_chain_id: 1,
            bsc_chain_id: 56,
            polygon_chain_id: 137,
            arbitrum_chain_id: 42161,
            ethereum_sepolia_chain_id: 11155111,
            bsc_testnet_chain_id: 97,
            polygon_mumbai_chain_id: 80001,
            arbitrum_sepolia_chain_id: 421614,
            block_monitor_interval_seconds: 10,
            transaction_timeout_minutes: 60,
            etherscan_api_key: None,
            bybit_price_api_url: "https://api.bybit.com/v5/market/tickers".to_string(),
            coinbase_price_api_url: "https://api.coinbase.com/v2/exchange-rates".to_string(),
            price_cache_ttl_seconds: 30,
            price_update_interval_seconds: 15,
            encryption_key: "test_key_32_bytes_long_for_tests".to_string(),
            webhook_signing_key: "test_webhook_key".to_string(),
            jwt_secret: "test_jwt_secret".to_string(),
            password_min_length: 8,
            password_require_uppercase: true,
            password_require_lowercase: true,
            password_require_numbers: true,
            password_require_symbols: true,
            max_login_attempts: 5,
            account_lockout_duration_minutes: 30,
            session_timeout_hours: 24,
            api_key_expiry_days: 365,
            rate_limit_requests_per_minute: 100,
            rate_limit_burst_size: 20,
            rate_limit_per_api_key: true,
            default_payment_expiration_minutes: 15,
            payment_cleanup_interval_hours: 24,
            payment_page_base_url: "https://pay.fiddupay.com".to_string(),
            default_fee_percentage: rust_decimal::Decimal::new(75, 4),
            daily_volume_limit_non_kyc_usd: rust_decimal::Decimal::new(100000, 2),
            merchant_registration_enabled: true,
            merchant_email_verification_required: true,
            merchant_kyc_required: false,
            merchant_auto_approval: false,
            webhook_timeout_seconds: 30,
            webhook_max_retries: 3,
            webhook_retry_delay_seconds: 5,
            webhook_signature_required: true,
            withdrawal_enabled: true,
            withdrawal_auto_approval_limit_usd: rust_decimal::Decimal::new(100000, 2),
            two_factor_enabled: false,
            deposit_address_enabled: true,
            invoice_enabled: true,
            multi_user_enabled: false,
            analytics_enabled: true,
            maintenance_mode: false,
            environment: "test".to_string(),
            debug_mode: true,
            frontend_url: "http://localhost:3000".to_string(),
            backend_url: "http://localhost:8080".to_string(),
            allowed_origins: vec!["http://localhost:3000".to_string()],
            etherscan_api_url: "https://api.etherscan.io/v2/api".to_string(),
            bscscan_api_url: "https://api.bscscan.com/api".to_string(),
            arbiscan_api_url: "https://api.arbiscan.io/api".to_string(),
            polygonscan_api_url: "https://api.polygonscan.com/api".to_string(),
            email_enabled: false,
            email_from: "noreply@fiddupay.com".to_string(),
            smtp_host: None,
            smtp_port: None,
            smtp_username: None,
            smtp_password: None,
            fee_wallet_sol: "".to_string(),
            fee_wallet_eth: "".to_string(),
            fee_wallet_bsc: "".to_string(),
            fee_wallet_polygon: "".to_string(),
            fee_wallet_arbitrum: "".to_string(),
        };

        let state = AppState::new(pool.clone(), config.clone());
        
        // Let's Insert a dummy payment with unique identifiers to avoid conflicts
        let merchant_email = format!("cancel_test_{}@example.com", nanoid::nanoid!());
        let payment_id = format!("pay_test_{}", nanoid::nanoid!());
        
        let _ = sqlx::query!("INSERT INTO merchants (email, business_name, settlement_mode, is_active, fee_percentage, customer_pays_fee, daily_limit_usd, role, password_hash, test_api_key_hash, live_api_key_hash, redirect_url) VALUES ($1, 'Cancel Test', 'managed', true, 0.0, false, 1000.0, 'MERCHANT', 'hash', 'hash', 'hash', 'https://merchant.com/callback')", merchant_email).execute(&pool).await.unwrap();

        let merchant = sqlx::query!("SELECT id FROM merchants WHERE email = $1", merchant_email).fetch_one(&pool).await.unwrap();
        let merchant_id = merchant.id;

        let _ = sqlx::query!("INSERT INTO payment_transactions (merchant_id, payment_id, amount, amount_usd, crypto_type, status, created_at, expires_at) VALUES ($1, $2, 10.0, 10.0, 'USDT_BEP20', 'PENDING', NOW(), NOW() + INTERVAL '1 hour')", merchant_id, payment_id).execute(&pool).await.unwrap();

        // Execute Handler
        let response = public_cancel_payment(
            State(state.clone()),
            Path(payment_id.clone())
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

        // CLEANUP
        sqlx::query!("DELETE FROM merchants WHERE id = $1", merchant_id).execute(&pool).await.unwrap();
    }
}
