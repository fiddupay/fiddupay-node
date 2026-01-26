#[cfg(test)]
mod gas_fee_tests {
    use super::*;
    use crate::config::Config;
    use crate::payment::models::CryptoType;
    use std::env;

    fn get_test_config() -> Config {
        // Set test RPC URLs - working 2026 endpoints
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
        
        Config::from_env().expect("Failed to create test config")
    }

    #[tokio::test]
    async fn test_ethereum_gas_fee_rpc() {
        let config = get_test_config();
        let service = GasFeeService::new(config);
        
        let result = service.get_gas_estimate(CryptoType::Eth).await;
        assert!(result.is_ok(), "Ethereum gas fee estimation failed: {:?}", result.err());
        
        let estimate = result.unwrap();
        assert_eq!(estimate.network, "ethereum");
        assert_eq!(estimate.native_currency, "ETH");
        assert!(estimate.standard_fee > rust_decimal::Decimal::ZERO);
        assert!(estimate.base_fee.is_some());
        
        println!(" Ethereum: {} ETH", estimate.standard_fee);
    }

    #[tokio::test]
    async fn test_bsc_gas_fee_rpc() {
        let config = get_test_config();
        let service = GasFeeService::new(config);
        
        let result = service.get_gas_estimate(CryptoType::Bnb).await;
        assert!(result.is_ok(), "BSC gas fee estimation failed: {:?}", result.err());
        
        let estimate = result.unwrap();
        assert_eq!(estimate.network, "bsc");
        assert_eq!(estimate.native_currency, "BNB");
        assert!(estimate.standard_fee > rust_decimal::Decimal::ZERO);
        
        println!(" BSC: {} BNB", estimate.standard_fee);
    }

    #[tokio::test]
    async fn test_polygon_gas_fee_rpc() {
        let config = get_test_config();
        let service = GasFeeService::new(config);
        
        let result = service.get_gas_estimate(CryptoType::Matic).await;
        assert!(result.is_ok(), "Polygon gas fee estimation failed: {:?}", result.err());
        
        let estimate = result.unwrap();
        assert_eq!(estimate.network, "polygon");
        assert_eq!(estimate.native_currency, "MATIC");
        assert!(estimate.standard_fee > rust_decimal::Decimal::ZERO);
        
        println!(" Polygon: {} MATIC", estimate.standard_fee);
    }

    #[tokio::test]
    async fn test_arbitrum_gas_fee_rpc() {
        let config = get_test_config();
        let service = GasFeeService::new(config);
        
        let result = service.get_gas_estimate(CryptoType::Arb).await;
        assert!(result.is_ok(), "Arbitrum gas fee estimation failed: {:?}", result.err());
        
        let estimate = result.unwrap();
        assert_eq!(estimate.network, "arbitrum");
        assert_eq!(estimate.native_currency, "ARB");
        assert!(estimate.standard_fee > rust_decimal::Decimal::ZERO);
        
        println!(" Arbitrum: {} ARB", estimate.standard_fee);
    }

    #[tokio::test]
    async fn test_solana_gas_fee_rpc() {
        let config = get_test_config();
        let service = GasFeeService::new(config);
        
        let result = service.get_gas_estimate(CryptoType::Sol).await;
        assert!(result.is_ok(), "Solana gas fee estimation failed: {:?}", result.err());
        
        let estimate = result.unwrap();
        assert_eq!(estimate.network, "solana");
        assert_eq!(estimate.native_currency, "SOL");
        assert!(estimate.standard_fee > rust_decimal::Decimal::ZERO);
        assert!(estimate.base_fee.is_some());
        
        println!(" Solana: {} SOL", estimate.standard_fee);
    }

    #[tokio::test]
    async fn test_all_networks_gas_fees() {
        let config = get_test_config();
        let service = GasFeeService::new(config);
        
        let result = service.get_all_gas_estimates().await;
        assert!(result.is_ok(), "Failed to get all gas estimates: {:?}", result.err());
        
        let estimates = result.unwrap();
        assert_eq!(estimates.len(), 5);
        
        for (network, estimate) in estimates {
            println!(" {}: {} {}", network.to_uppercase(), estimate.standard_fee, estimate.native_currency);
            assert!(estimate.standard_fee > rust_decimal::Decimal::ZERO);
        }
    }
}
