// Payment Link and Partial Payment Models
// Database models for payment links and partial payments

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentLink {
    pub id: i64,
    pub link_id: String,
    pub payment_id: i64,
    pub merchant_id: i64,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PartialPayment {
    pub id: i64,
    pub payment_id: i64,
    pub transaction_hash: String,
    pub amount: Decimal,
    pub amount_usd: Decimal,
    pub confirmations: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePaymentLinkRequest {
    pub amount_usd: Decimal,
    pub crypto_type: String,
    pub description: Option<String>,
    pub expires_in_minutes: Option<i64>,
}
