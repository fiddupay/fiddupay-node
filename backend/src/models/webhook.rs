// Webhook Models
// Data structures for webhook notifications

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::payment::models::PaymentStatus;

/// Webhook payload sent to merchant endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event_type: String,  // "payment.confirmed", "payment.expired", "refund.completed"
    pub payment_id: String,
    pub merchant_id: i64,
    pub status: PaymentStatus,
    pub amount: Decimal,
    pub crypto_type: String,
    pub transaction_hash: Option<String>,
    pub timestamp: i64,
}

/// Webhook delivery record for tracking delivery attempts
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebhookDelivery {
    pub id: i64,
    pub merchant_id: i64,
    pub payment_id: i64,
    pub event_type: String,  // "payment.confirmed", "payment.expired", "refund.completed"
    pub url: String,
    pub payload: serde_json::Value,
    pub status: String,  // "pending", "delivered", "failed"
    pub attempts: i32,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub created_at: DateTime<Utc>,
}

