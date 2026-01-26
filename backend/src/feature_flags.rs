// Configuration Module with Feature Flags
// Application configuration from environment variables

use std::env;

#[derive(Debug, Clone)]
pub struct FeatureFlags {
    pub email_enabled: bool,
    pub two_factor_enabled: bool,
    pub deposit_address_enabled: bool,
    pub withdrawal_enabled: bool,
    pub invoice_enabled: bool,
    pub multi_user_enabled: bool,
    pub maintenance_mode: bool,
}

#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub from: String,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
}

impl FeatureFlags {
    pub fn from_env() -> Self {
        Self {
            email_enabled: env::var("EMAIL_ENABLED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            two_factor_enabled: env::var("TWO_FACTOR_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            deposit_address_enabled: env::var("DEPOSIT_ADDRESS_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            withdrawal_enabled: env::var("WITHDRAWAL_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            invoice_enabled: env::var("INVOICE_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            multi_user_enabled: env::var("MULTI_USER_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            maintenance_mode: env::var("MAINTENANCE_MODE")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        }
    }
}

impl EmailConfig {
    pub fn from_env() -> Self {
        Self {
            from: env::var("EMAIL_FROM")
                .unwrap_or_else(|_| "noreply@cryptogateway.com".to_string()),
            smtp_host: env::var("SMTP_HOST").ok(),
            smtp_port: env::var("SMTP_PORT").ok().and_then(|s| s.parse().ok()),
            smtp_username: env::var("SMTP_USERNAME").ok(),
            smtp_password: env::var("SMTP_PASSWORD").ok(),
        }
    }
}
