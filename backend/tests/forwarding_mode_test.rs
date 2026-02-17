#[cfg(test)]
mod tests {
    use super::*;
    use fiddupay::payment::models::CryptoType;
    use fiddupay::services::merchant_service::MerchantService;
    use fiddupay::services::wallet_config_service::WalletConfigService;
    use fiddupay::error::ServiceError;
    use sqlx::PgPool;

    // Helper to reset DB state for a merchant
    async fn reset_merchant(pool: &PgPool, merchant_id: i64) {
        sqlx::query!("DELETE FROM merchant_wallets WHERE merchant_id = $1", merchant_id)
            .execute(pool).await.ok();
        sqlx::query!("DELETE FROM merchant_forwarding_wallets WHERE merchant_id = $1", merchant_id)
            .execute(pool).await.ok();
        sqlx::query!("UPDATE merchants SET settlement_mode = 'managed' WHERE id = $1", merchant_id)
            .execute(pool).await.ok();
    }

    #[sqlx::test]
    async fn test_forwarding_mode_isolation(pool: PgPool) {
        // Create a minimal config for testing
        let config = fiddupay::config::Config {
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
        let merchant_service = MerchantService::new(pool.clone(), config);
        let wallet_service = WalletConfigService::new(pool.clone());
        
        // 1. Register a test merchant
        let _ = sqlx::query!("INSERT INTO merchants (id, email, business_name, settlement_mode, is_active, fee_percentage, customer_pays_fee, daily_limit_usd, role, password_hash, test_api_key_hash, live_api_key_hash) VALUES (999, 'test@example.com', 'Test Merchant', 'managed', true, 0.0, false, 1000.0, 'MERCHANT', 'hash', 'hash', 'hash') ON CONFLICT DO NOTHING").execute(&pool).await;
        
        reset_merchant(&pool, 999).await;

        // 2. Set separate addresses in managed vs forwarding tables
        // Managed: Uses USDT_BEP20 -> 0xManagedAddress
        merchant_service.set_wallet_address(999, CryptoType::UsdtBep20, "0x1111111111111111111111111111111111111111".to_string()).await.unwrap();
        
        // Forwarding: Uses USDT_BEP20 -> 0xForwardingAddress
        wallet_service.set_forwarding_address(999, CryptoType::UsdtBep20, "0x2222222222222222222222222222222222222222".to_string(), true).await.unwrap();

        // 3. Verify MANAGED mode (default)
        let addr_managed = merchant_service.get_wallet_address(999, CryptoType::UsdtBep20).await.unwrap();
        assert_eq!(addr_managed, "0x1111111111111111111111111111111111111111", "Should fetch from merchant_wallets in managed mode");

        // 4. Switch to FORWARDING mode
        merchant_service.update_settlement_mode(999, "forwarding").await.unwrap();

        // 5. Verify FORWARDING mode
        let addr_forwarding = merchant_service.get_wallet_address(999, CryptoType::UsdtBep20).await.unwrap();
        assert_eq!(addr_forwarding, "0x2222222222222222222222222222222222222222", "Should fetch from merchant_forwarding_wallets in forwarding mode");
        
        // 6. Verify Missing Wallet in Forwarding Mode
        // We haven't set a SOL forwarding address
        let result: Result<String, ServiceError> = merchant_service.get_wallet_address(999, CryptoType::Sol).await;
        assert!(result.is_err(), "Should return error for missing forwarding wallet");
    }

    #[sqlx::test]
    async fn test_forwarding_validation(pool: PgPool) {
        let wallet_service = WalletConfigService::new(pool.clone());
        let _ = sqlx::query!("INSERT INTO merchants (id, email, business_name, settlement_mode, is_active, fee_percentage, customer_pays_fee, daily_limit_usd, role, password_hash, test_api_key_hash, live_api_key_hash) VALUES (998, 'test2@example.com', 'Test Merchant 2', 'forwarding', true, 0.0, false, 1000.0, 'MERCHANT', 'hash', 'hash', 'hash') ON CONFLICT DO NOTHING").execute(&pool).await;

        // 1. Try setting EVM address for SOL -> Should Fail
        let result: Result<_, ServiceError> = wallet_service.set_forwarding_address(998, CryptoType::Sol, "0x1111111111111111111111111111111111111111".to_string(), true).await;
        assert!(result.is_err(), "Should detect invalid SOL address");

        // 2. Try setting SOL address for EVM -> Should Fail
        let result: Result<_, ServiceError> = wallet_service.set_forwarding_address(998, CryptoType::UsdtBep20, "So11111111111111111111111111111111111111112".to_string(), true).await;
        assert!(result.is_err(), "Should detect invalid EVM address");
        
        // 3. Valid SOL address
        let result: Result<_, ServiceError> = wallet_service.set_forwarding_address(998, CryptoType::Sol, "So11111111111111111111111111111111111111112".to_string(), true).await;
        assert!(result.is_ok(), "Should accept valid SOL address");
    }
}
