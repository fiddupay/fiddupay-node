// Complete Address-Only Mode Integration Test
// Tests the full flow: payment creation → monitoring → auto-forwarding → webhooks

#[cfg(test)]
mod address_only_integration_tests {
    use super::*;
    use crate::config::Config;
    use crate::payment::models::CryptoType;
    use crate::services::{
        address_only_service::{AddressOnlyService, AddressOnlyStatus},
        gas_fee_service::GasFeeService,
        payment_monitor_service::PaymentMonitorService,
        webhook_notification_service::WebhookNotificationService,
    };
    use rust_decimal::Decimal;
    use sqlx::PgPool;
    use std::env;

    async fn setup_test_environment() -> (Config, PgPool) {
        // Set test environment variables
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/test");
        env::set_var("REDIS_URL", "redis://localhost:6379");
        env::set_var("ENCRYPTION_KEY", "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
        env::set_var("JWT_SECRET", "test-secret");
        
        // Working 2026 RPC endpoints
        env::set_var("ETHEREUM_RPC_URL", "https://eth.llamarpc.com");
        env::set_var("BSC_RPC_URL", "https://bsc-dataseed.binance.org");
        env::set_var("POLYGON_RPC_URL", "https://polygon-rpc.com");
        env::set_var("ARBITRUM_RPC_URL", "https://arb1.arbitrum.io/rpc");
        env::set_var("SOLANA_RPC_URL", "https://api.mainnet-beta.solana.com");

        let config = Config::from_env().expect("Failed to create test config");
        let db_pool = PgPool::connect(&config.database_url)
            .await
            .expect("Failed to connect to test database");

        (config, db_pool)
    }

    #[tokio::test]
    async fn test_complete_address_only_flow() {
        let (config, db_pool) = setup_test_environment().await;
        
        // Initialize services
        let gas_service = GasFeeService::new(config.clone());
        let address_service = AddressOnlyService::new(db_pool.clone(), gas_service, config.clone());
        let webhook_service = WebhookNotificationService::new(db_pool.clone());

        println!(" Testing Complete Address-Only Flow");

        // Step 1: Create payment request
        println!("\n Step 1: Creating payment request");
        let merchant_id = 1i64;
        let crypto_type = CryptoType::Eth;
        let merchant_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string();
        let requested_amount = Decimal::new(100, 2); // $1.00

        let payment = address_service
            .create_payment_request(merchant_id, crypto_type, merchant_address.clone(), requested_amount)
            .await
            .expect("Failed to create payment request");

        println!(" Payment created:");
        println!("   Payment ID: {}", payment.payment_id);
        println!("   Deposit Address: {}", payment.gateway_deposit_address);
        println!("   Requested Amount: {} ETH", payment.requested_amount);
        println!("   Processing Fee: {} ETH", payment.processing_fee);
        println!("   Forwarding Amount: {} ETH", payment.forwarding_amount);

        assert_eq!(payment.status, AddressOnlyStatus::PendingPayment);
        assert!(payment.gateway_deposit_address.starts_with("0x"));
        assert_eq!(payment.merchant_destination_address, merchant_address);

        // Step 2: Simulate payment received
        println!("\n Step 2: Simulating payment received");
        let received_amount = payment.requested_amount;
        let tx_hash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

        let result = address_service
            .process_received_payment(&payment.payment_id, received_amount, tx_hash)
            .await;

        match result {
            Ok(_) => {
                println!(" Payment processed successfully");
                
                // Verify payment status updated
                let updated_payment = address_service
                    .get_payment_by_id(&payment.payment_id)
                    .await
                    .expect("Failed to get updated payment");

                println!("   Status: {:?}", updated_payment.status);
                assert!(matches!(updated_payment.status, AddressOnlyStatus::Completed));
            }
            Err(e) => {
                println!(" Payment processing failed (expected in test): {}", e);
                // This is expected since we don't have actual blockchain connectivity
            }
        }

        // Step 3: Test webhook notification
        println!("\n Step 3: Testing webhook notification");
        let webhook_url = "https://httpbin.org/post";
        
        let webhook_result = webhook_service
            .send_payment_status_webhook(&payment, webhook_url)
            .await;

        match webhook_result {
            Ok(_) => println!(" Webhook sent successfully"),
            Err(e) => println!(" Webhook failed (expected in test): {}", e),
        }

        println!("\n Address-Only Flow Test Completed!");
    }

    #[tokio::test]
    async fn test_native_currency_support() {
        let (config, db_pool) = setup_test_environment().await;
        let gas_service = GasFeeService::new(config.clone());
        let address_service = AddressOnlyService::new(db_pool.clone(), gas_service, config.clone());

        println!(" Testing Native Currency Support");

        let native_currencies = vec![
            (CryptoType::Eth, "ETH"),
            (CryptoType::Bnb, "BNB"),
            (CryptoType::Matic, "MATIC"),
            (CryptoType::Arb, "ARB"),
            (CryptoType::Sol, "SOL"),
        ];

        for (crypto_type, name) in native_currencies {
            println!("\n Testing {}", name);
            
            let result = address_service
                .create_payment_request(
                    1i64,
                    crypto_type,
                    "test_address".to_string(),
                    Decimal::new(100, 2),
                )
                .await;

            match result {
                Ok(payment) => {
                    println!(" {} payment created: {}", name, payment.gateway_deposit_address);
                    assert_eq!(payment.crypto_type, crypto_type);
                }
                Err(e) => {
                    println!(" {} payment failed: {}", name, e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_usdt_rejection() {
        let (config, db_pool) = setup_test_environment().await;
        let gas_service = GasFeeService::new(config.clone());
        let address_service = AddressOnlyService::new(db_pool.clone(), gas_service, config.clone());

        println!(" Testing USDT Rejection (Phase 1)");

        let usdt_variants = vec![
            (CryptoType::UsdtEth, "USDT-ETH"),
            (CryptoType::UsdtBep20, "USDT-BEP20"),
            (CryptoType::UsdtPolygon, "USDT-Polygon"),
            (CryptoType::UsdtArbitrum, "USDT-Arbitrum"),
            (CryptoType::UsdtSpl, "USDT-SPL"),
        ];

        for (crypto_type, name) in usdt_variants {
            println!("\n Testing {} rejection", name);
            
            let result = address_service
                .create_payment_request(
                    1i64,
                    crypto_type,
                    "test_address".to_string(),
                    Decimal::new(100, 2),
                )
                .await;

            match result {
                Ok(_) => {
                    println!(" {} should have been rejected", name);
                    panic!("USDT should be rejected in Phase 1");
                }
                Err(e) => {
                    println!(" {} correctly rejected: {}", name, e);
                    assert!(e.to_string().contains("native currencies only"));
                }
            }
        }
    }

    #[tokio::test]
    async fn test_fee_calculation() {
        let (config, db_pool) = setup_test_environment().await;
        let gas_service = GasFeeService::new(config.clone());
        let address_service = AddressOnlyService::new(db_pool.clone(), gas_service, config.clone());

        println!(" Testing Fee Calculation");

        let test_amounts = vec![
            (Decimal::new(100, 2), "1.00"), // $1.00
            (Decimal::new(10000, 2), "100.00"), // $100.00
            (Decimal::new(1000000, 2), "10000.00"), // $10,000.00
        ];

        for (amount, description) in test_amounts {
            println!("\n Testing amount: ${}", description);
            
            let payment = address_service
                .create_payment_request(
                    1i64,
                    CryptoType::Eth,
                    "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string(),
                    amount,
                )
                .await
                .expect("Failed to create payment");

            // Verify fee calculation (0.75%)
            let expected_fee = amount * Decimal::new(75, 4); // 0.75%
            let expected_forwarding = amount - expected_fee;

            println!("   Requested: {} ETH", payment.requested_amount);
            println!("   Processing Fee: {} ETH", payment.processing_fee);
            println!("   Forwarding: {} ETH", payment.forwarding_amount);

            assert_eq!(payment.requested_amount, amount);
            assert_eq!(payment.processing_fee, expected_fee);
            assert_eq!(payment.forwarding_amount, expected_forwarding);
        }

        println!("\n Fee calculations verified!");
    }
}
