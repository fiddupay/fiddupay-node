// Configuration Module
// Application configuration from environment variables

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    // Database
    pub database_url: String,
    pub database_max_connections: u32,
    pub database_timeout_seconds: u64,

    // Redis
    pub redis_url: String,
    pub redis_max_connections: u32,
    pub redis_timeout_seconds: u64,

    // Server
    pub server_host: String,
    pub server_port: u16,
    pub server_workers: usize,
    pub request_timeout_seconds: u64,

    // Blockchain RPC URLs
    pub solana_rpc_url: String,
    pub ethereum_rpc_url: String,
    pub bsc_rpc_url: String,
    pub arbitrum_rpc_url: String,
    pub polygon_rpc_url: String,

    // Blockchain Settings
    pub confirmation_blocks_sol: u32,
    pub confirmation_blocks_eth: u32,
    pub confirmation_blocks_bsc: u32,
    pub confirmation_blocks_polygon: u32,
    pub confirmation_blocks_arbitrum: u32,

    // Transaction Monitoring
    pub block_monitor_interval_seconds: u64,
    pub transaction_timeout_minutes: u64,

    // API Keys
    pub etherscan_api_key: Option<String>,

    // Price API
    pub bybit_price_api_url: String,
    pub coinbase_price_api_url: String,
    pub price_cache_ttl_seconds: u64,
    pub price_update_interval_seconds: u64,

    // Security
    pub encryption_key: String,
    pub webhook_signing_key: String,
    pub jwt_secret: String,

    // Password Security
    pub password_min_length: u32,
    pub password_require_uppercase: bool,
    pub password_require_lowercase: bool,
    pub password_require_numbers: bool,
    pub password_require_symbols: bool,

    // Account Security
    pub max_login_attempts: u32,
    pub account_lockout_duration_minutes: u64,
    pub session_timeout_hours: u64,
    pub api_key_expiry_days: u64,

    // Rate Limiting
    pub rate_limit_requests_per_minute: u32,
    pub rate_limit_burst_size: u32,
    pub rate_limit_per_api_key: bool,

    // Payment Settings
    pub default_payment_expiration_minutes: u32,
    pub payment_cleanup_interval_hours: u64,
    pub payment_page_base_url: String,

    // Fee Configuration
    pub default_fee_percentage: rust_decimal::Decimal,
    pub minimum_fee_usd: rust_decimal::Decimal,
    pub maximum_fee_usd: rust_decimal::Decimal,

    // Payment Limits
    pub min_payment_usd: rust_decimal::Decimal,
    pub max_payment_usd: rust_decimal::Decimal,
    pub daily_payment_limit_usd: rust_decimal::Decimal,

    // Merchant Settings
    pub merchant_registration_enabled: bool,
    pub merchant_email_verification_required: bool,
    pub merchant_kyc_required: bool,
    pub merchant_auto_approval: bool,

    // Webhook Settings
    pub webhook_timeout_seconds: u64,
    pub webhook_max_retries: u32,
    pub webhook_retry_delay_seconds: u64,
    pub webhook_signature_required: bool,

    // Withdrawal Settings
    pub withdrawal_enabled: bool,
    pub withdrawal_min_amount_usd: rust_decimal::Decimal,
    pub withdrawal_max_amount_usd: rust_decimal::Decimal,
    pub withdrawal_auto_approval_limit_usd: rust_decimal::Decimal,

    // Feature Flags
    pub two_factor_enabled: bool,
    pub deposit_address_enabled: bool,
    pub invoice_enabled: bool,
    pub multi_user_enabled: bool,
    pub analytics_enabled: bool,
    pub maintenance_mode: bool,

    // Environment
    pub environment: String,
    pub debug_mode: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        Ok(Self {
            // Database
            database_url: env::var("DATABASE_URL")?,
            database_max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "20".to_string())
                .parse()?,
            database_timeout_seconds: env::var("DATABASE_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,

            // Redis
            redis_url: env::var("REDIS_URL")?,
            redis_max_connections: env::var("REDIS_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()?,
            redis_timeout_seconds: env::var("REDIS_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()?,

            // Server
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
            server_workers: env::var("SERVER_WORKERS")
                .unwrap_or_else(|_| "4".to_string())
                .parse()?,
            request_timeout_seconds: env::var("REQUEST_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,

            // Blockchain RPC URLs - All required, no defaults
            solana_rpc_url: env::var("SOLANA_RPC_URL")?,
            ethereum_rpc_url: env::var("ETHEREUM_RPC_URL")?,
            bsc_rpc_url: env::var("BSC_RPC_URL")?,
            arbitrum_rpc_url: env::var("ARBITRUM_RPC_URL")?,
            polygon_rpc_url: env::var("POLYGON_RPC_URL")?,

            // Blockchain Settings
            confirmation_blocks_sol: env::var("CONFIRMATION_BLOCKS_SOL")
                .unwrap_or_else(|_| "32".to_string())
                .parse()?,
            confirmation_blocks_eth: env::var("CONFIRMATION_BLOCKS_ETH")
                .unwrap_or_else(|_| "12".to_string())
                .parse()?,
            confirmation_blocks_bsc: env::var("CONFIRMATION_BLOCKS_BSC")
                .unwrap_or_else(|_| "15".to_string())
                .parse()?,
            confirmation_blocks_polygon: env::var("CONFIRMATION_BLOCKS_POLYGON")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
            confirmation_blocks_arbitrum: env::var("CONFIRMATION_BLOCKS_ARBITRUM")
                .unwrap_or_else(|_| "1".to_string())
                .parse()?,

            // Transaction Monitoring
            block_monitor_interval_seconds: env::var("BLOCK_MONITOR_INTERVAL_SECONDS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()?,
            transaction_timeout_minutes: env::var("TRANSACTION_TIMEOUT_MINUTES")
                .unwrap_or_else(|_| "60".to_string())
                .parse()?,

            // API Keys
            etherscan_api_key: env::var("ETHERSCAN_API_KEY").ok(),

            // Price API - Required, no defaults
            bybit_price_api_url: env::var("BYBIT_PRICE_API_URL")?,
            coinbase_price_api_url: env::var("COINBASE_PRICE_API_URL")?,
            price_cache_ttl_seconds: env::var("PRICE_CACHE_TTL_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
            price_update_interval_seconds: env::var("PRICE_UPDATE_INTERVAL_SECONDS")
                .unwrap_or_else(|_| "15".to_string())
                .parse()?,

            // Security - All required, no defaults
            encryption_key: env::var("ENCRYPTION_KEY")?,
            webhook_signing_key: env::var("WEBHOOK_SIGNING_KEY")?,
            jwt_secret: env::var("JWT_SECRET")?,

            // Password Security
            password_min_length: env::var("PASSWORD_MIN_LENGTH")
                .unwrap_or_else(|_| "8".to_string())
                .parse()?,
            password_require_uppercase: env::var("PASSWORD_REQUIRE_UPPERCASE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()?,
            password_require_lowercase: env::var("PASSWORD_REQUIRE_LOWERCASE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()?,
            password_require_numbers: env::var("PASSWORD_REQUIRE_NUMBERS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()?,
            password_require_symbols: env::var("PASSWORD_REQUIRE_SYMBOLS")
                .unwrap_or_else(|_| "false".to_string())
                .parse()?,

            // Account Security
            max_login_attempts: env::var("MAX_LOGIN_ATTEMPTS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()?,
            account_lockout_duration_minutes: env::var("ACCOUNT_LOCKOUT_DURATION_MINUTES")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
            session_timeout_hours: env::var("SESSION_TIMEOUT_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()?,
            api_key_expiry_days: env::var("API_KEY_EXPIRY_DAYS")
                .unwrap_or_else(|_| "365".to_string())
                .parse()?,

            // Rate Limiting
            rate_limit_requests_per_minute: env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
                .unwrap_or_else(|_| "100".to_string())
                .parse()?,
            rate_limit_burst_size: env::var("RATE_LIMIT_BURST_SIZE")
                .unwrap_or_else(|_| "20".to_string())
                .parse()?,
            rate_limit_per_api_key: env::var("RATE_LIMIT_PER_API_KEY")
                .unwrap_or_else(|_| "true".to_string())
                .parse()?,

            // Payment Settings
            default_payment_expiration_minutes: env::var("DEFAULT_PAYMENT_EXPIRATION_MINUTES")
                .unwrap_or_else(|_| "15".to_string())
                .parse()?,
            payment_cleanup_interval_hours: env::var("PAYMENT_CLEANUP_INTERVAL_HOURS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()?,
            payment_page_base_url: env::var("PAYMENT_PAGE_BASE_URL")?,

            // Fee Configuration
            default_fee_percentage: env::var("DEFAULT_FEE_PERCENTAGE")
                .unwrap_or_else(|_| "1.50".to_string())
                .parse()?,
            minimum_fee_usd: env::var("MINIMUM_FEE_USD")
                .unwrap_or_else(|_| "0.01".to_string())
                .parse()?,
            maximum_fee_usd: env::var("MAXIMUM_FEE_USD")
                .unwrap_or_else(|_| "100.00".to_string())
                .parse()?,

            // Payment Limits
            min_payment_usd: env::var("MIN_PAYMENT_USD")
                .unwrap_or_else(|_| "1.00".to_string())
                .parse()?,
            max_payment_usd: env::var("MAX_PAYMENT_USD")
                .unwrap_or_else(|_| "10000.00".to_string())
                .parse()?,
            daily_payment_limit_usd: env::var("DAILY_PAYMENT_LIMIT_USD")
                .unwrap_or_else(|_| "50000.00".to_string())
                .parse()?,

            // Merchant Settings
            merchant_registration_enabled: env::var("MERCHANT_REGISTRATION_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()?,
            merchant_email_verification_required: env::var("MERCHANT_EMAIL_VERIFICATION_REQUIRED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()?,
            merchant_kyc_required: env::var("MERCHANT_KYC_REQUIRED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()?,
            merchant_auto_approval: env::var("MERCHANT_AUTO_APPROVAL")
                .unwrap_or_else(|_| "true".to_string())
                .parse()?,

            // Webhook Settings
            webhook_timeout_seconds: env::var("WEBHOOK_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()?,
            webhook_max_retries: env::var("WEBHOOK_MAX_RETRIES")
                .unwrap_or_else(|_| "5".to_string())
                .parse()?,
            webhook_retry_delay_seconds: env::var("WEBHOOK_RETRY_DELAY_SECONDS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()?,
            webhook_signature_required: env::var("WEBHOOK_SIGNATURE_REQUIRED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()?,

            // Withdrawal Settings
            withdrawal_enabled: env::var("WITHDRAWAL_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()?,
            withdrawal_min_amount_usd: env::var("WITHDRAWAL_MIN_AMOUNT_USD")
                .unwrap_or_else(|_| "10.00".to_string())
                .parse()?,
            withdrawal_max_amount_usd: env::var("WITHDRAWAL_MAX_AMOUNT_USD")
                .unwrap_or_else(|_| "50000.00".to_string())
                .parse()?,
            withdrawal_auto_approval_limit_usd: env::var("WITHDRAWAL_AUTO_APPROVAL_LIMIT_USD")
                .unwrap_or_else(|_| "1000.00".to_string())
                .parse()?,

            // Feature Flags
            two_factor_enabled: env::var("TWO_FACTOR_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()?,
            deposit_address_enabled: env::var("DEPOSIT_ADDRESS_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()?,
            invoice_enabled: env::var("INVOICE_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()?,
            multi_user_enabled: env::var("MULTI_USER_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()?,
            analytics_enabled: env::var("ANALYTICS_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()?,
            maintenance_mode: env::var("MAINTENANCE_MODE")
                .unwrap_or_else(|_| "false".to_string())
                .parse()?,

            // Environment
            environment: env::var("ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string()),
            debug_mode: env::var("DEBUG_MODE")
                .unwrap_or_else(|_| "false".to_string())
                .parse()?,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.database_url.is_empty() {
            return Err("DATABASE_URL is required".to_string());
        }

        if self.encryption_key.is_empty() {
            return Err("ENCRYPTION_KEY is required".to_string());
        }

        if self.webhook_signing_key.is_empty() {
            return Err("WEBHOOK_SIGNING_KEY is required".to_string());
        }

        if self.redis_url.is_empty() {
            return Err("REDIS_URL is required".to_string());
        }

        if self.solana_rpc_url.is_empty() {
            return Err("SOLANA_RPC_URL is required".to_string());
        }

        if self.ethereum_rpc_url.is_empty() {
            return Err("ETHEREUM_RPC_URL is required".to_string());
        }

        if self.bsc_rpc_url.is_empty() {
            return Err("BSC_RPC_URL is required".to_string());
        }

        if self.arbitrum_rpc_url.is_empty() {
            return Err("ARBITRUM_RPC_URL is required".to_string());
        }

        if self.polygon_rpc_url.is_empty() {
            return Err("POLYGON_RPC_URL is required".to_string());
        }

        if self.bybit_price_api_url.is_empty() {
            return Err("BYBIT_PRICE_API_URL is required".to_string());
        }

        if self.coinbase_price_api_url.is_empty() {
            return Err("COINBASE_PRICE_API_URL is required".to_string());
        }

        if self.payment_page_base_url.is_empty() {
            return Err("PAYMENT_PAGE_BASE_URL is required".to_string());
        }

        if self.jwt_secret.is_empty() {
            return Err("JWT_SECRET is required".to_string());
        }

        Ok(())
    }
}
