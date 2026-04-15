// Refund Models
// Data structures for refund operations

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {
    pub refund_id: String,
    pub payment_id: String,
    pub amount: Decimal,
    pub amount_usd: Decimal,
    pub status: String,
    pub reason: Option<String>,
    pub transaction_hash: Option<String>,
    pub crypto_type: String,
    pub target_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
