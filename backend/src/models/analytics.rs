// Analytics Models
// Data structures for analytics and reporting

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub date: String,
    pub volume_usd: Decimal,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceTrendPoint {
    pub date: String,
    pub total_usd: Decimal,
    pub balances: HashMap<String, Decimal>, // crypto_type -> amount (crypto)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceHistory {
    pub points: Vec<BalanceTrendPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    pub total_volume_usd: Decimal,
    pub successful_payments: i64,
    pub failed_payments: i64,
    pub pending_payments: i64,
    pub total_payments: i64,
    pub total_fees_paid: Decimal,
    pub average_transaction_value: Decimal,
    pub by_blockchain: HashMap<String, BlockchainStats>,
    pub payment_trends: Vec<TimeSeriesPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainStats {
    pub volume_usd: Decimal,
    pub payment_count: i64,
    pub average_value: Decimal,
}



