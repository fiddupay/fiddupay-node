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
        let config = fiddupay::config::Config::default();
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
