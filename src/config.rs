// Configuration Module
// Application configuration from environment variables

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    // Database
    pub database_url: String,
    pub database_max_connections: u32,

    // Redis
    pub redis_url: String,

    // Server
    pub server_host: String,
    pub server_port: u16,

    // Blockchain RPC URLs
    pub solana_rpc_url: String,
    pub bsc_rpc_url: String,
    pub arbitrum_rpc_url: String,
    pub polygon_rpc_url: String,

    // API Keys (optional)
    pub bscscan_api_key: Option<String>,
    pub arbiscan_api_key: Option<String>,
    pub polygonscan_api_key: Option<String>,

    // Price API
    pub bybit_price_api_url: String,
    pub price_cache_ttl_seconds: u64,

    // Webhook
    pub webhook_signing_key: String,
    pub webhook_timeout_seconds: u64,
    pub webhook_max_retries: u32,

    // Rate Limiting
    pub rate_limit_requests_per_minute: u32,

    // Payment Defaults
    pub default_payment_expiration_minutes: u32,
    pub default_fee_percentage: rust_decimal::Decimal,

    // Hosted Pages
    pub payment_page_base_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            database_max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "20".to_string())
                .parse()?,

            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),

            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,

            solana_rpc_url: env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
            bsc_rpc_url: env::var("BSC_RPC_URL")
                .unwrap_or_else(|_| "https://bsc-dataseed.binance.org".to_string()),
            arbitrum_rpc_url: env::var("ARBITRUM_RPC_URL")
                .unwrap_or_else(|_| "https://arb1.arbitrum.io/rpc".to_string()),
            polygon_rpc_url: env::var("POLYGON_RPC_URL")
                .unwrap_or_else(|_| "https://polygon-rpc.com".to_string()),

            bscscan_api_key: env::var("BSCSCAN_API_KEY").ok(),
            arbiscan_api_key: env::var("ARBISCAN_API_KEY").ok(),
            polygonscan_api_key: env::var("POLYGONSCAN_API_KEY").ok(),

            bybit_price_api_url: env::var("BYBIT_PRICE_API_URL")
                .unwrap_or_else(|_| "https://api.bybit.com".to_string()),
            price_cache_ttl_seconds: env::var("PRICE_CACHE_TTL_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,

            webhook_signing_key: env::var("WEBHOOK_SIGNING_KEY")
                .unwrap_or_else(|_| "change_me_in_production".to_string()),
            webhook_timeout_seconds: env::var("WEBHOOK_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()?,
            webhook_max_retries: env::var("WEBHOOK_MAX_RETRIES")
                .unwrap_or_else(|_| "5".to_string())
                .parse()?,

            rate_limit_requests_per_minute: env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
                .unwrap_or_else(|_| "100".to_string())
                .parse()?,

            default_payment_expiration_minutes: env::var("DEFAULT_PAYMENT_EXPIRATION_MINUTES")
                .unwrap_or_else(|_| "15".to_string())
                .parse()?,
            default_fee_percentage: env::var("DEFAULT_FEE_PERCENTAGE")
                .unwrap_or_else(|_| "1.50".to_string())
                .parse()?,

            payment_page_base_url: env::var("PAYMENT_PAGE_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.database_url.is_empty() {
            return Err("DATABASE_URL is required".to_string());
        }

        if self.webhook_signing_key == "change_me_in_production" {
            tracing::warn!("⚠️  Using default webhook signing key - change this in production!");
        }

        Ok(())
    }
}
