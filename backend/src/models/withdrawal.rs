// Withdrawal Model
// Database model for withdrawals

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Withdrawal {
    pub id: i32,
    pub withdrawal_id: String,
    pub merchant_id: i64,
    pub crypto_type: String,
    pub amount: Decimal,
    pub amount_usd: Decimal,
    pub destination_address: String,
    pub status: String,
    pub fee: Decimal,
    pub net_amount: Decimal,
    pub transaction_hash: Option<String>,
    pub rejection_reason: Option<String>,
    pub requires_approval: bool,
    pub approved_by: Option<i32>,
    pub approved_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWithdrawalRequest {
    pub crypto_type: String,
    pub amount: Decimal,
    pub destination_address: String,
}

#[derive(Debug, Deserialize)]
pub struct ApproveWithdrawalRequest {
    pub approved: bool,
    pub rejection_reason: Option<String>,
}
