// Balance Model
// Database models for merchant balances and balance history

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MerchantBalance {
    pub id: i32,
    pub merchant_id: i64,
    pub crypto_type: String,
    pub available_balance: Decimal,
    pub reserved_balance: Decimal,
    pub total_balance: Decimal,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BalanceHistory {
    pub id: i32,
    pub merchant_id: i64,
    pub crypto_type: String,
    pub amount: Decimal,
    pub balance_type: String, // 'AVAILABLE' or 'RESERVED'
    pub change_type: String,  // 'CREDIT' or 'DEBIT'
    pub reason: String,       // 'PAYMENT_CONFIRMED', 'REFUND', 'WITHDRAWAL', etc.
    pub reference_id: Option<String>,
    pub balance_before: Decimal,
    pub balance_after: Decimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct BalanceSummary {
    pub crypto_type: String,
    pub available: String,
    pub reserved: String,
    pub total: String,
}
