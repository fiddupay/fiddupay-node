// Merchant User and Currency Models
// Database models for multi-user support and currency preferences

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MerchantUser {
    pub id: i32,
    pub merchant_id: i64,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub is_active: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MerchantCurrency {
    pub id: i64,
    pub merchant_id: i64,
    pub currency_group: String, // 'USDT', 'ETH', 'SOL', 'BTC'
    pub network: String,        // 'ETH', 'BSC', 'POLYGON', 'ARBITRUM', 'SOL'
    pub crypto_type: String,    // 'USDT_ETH', 'USDT_BSC', etc.
    pub is_enabled: bool,
    pub wallet_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DepositAddress {
    pub id: i32,
    pub payment_id: String,
    pub crypto_type: String,
    pub deposit_address: String,
    pub private_key_encrypted: String,
    pub merchant_destination: String,
    pub status: String,
    pub expires_at: DateTime<Utc>,
    pub forwarded_at: Option<DateTime<Utc>>,
    pub forward_tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMerchantUserRequest {
    pub email: String,
    pub password: String,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCurrencyRequest {
    pub is_enabled: bool,
    pub wallet_address: Option<String>,
}
