// Payment Models
// Core data structures for payment processing

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Re-export from payment module
pub use crate::payment::models::{CryptoType, PaymentStatus};

/// Payment transaction record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Payment {
    pub id: i64,
    pub payment_id: String,
    pub merchant_id: i64,
    pub amount: Decimal,
    pub amount_usd: Decimal,
    pub crypto_type: String,
    pub status: String,
    pub to_address: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub confirmations: Option<i32>,
    pub required_confirmations: Option<i32>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}
