use crate::error::ServiceError;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceAlert {
    pub merchant_id: i64,
    pub alert_type: String,
    pub crypto_type: String,
    pub current_balance: Decimal,
    pub threshold: Decimal,
    pub created_at: DateTime<Utc>,
}

pub struct BalanceMonitoringService {
    db_pool: PgPool,
}

impl BalanceMonitoringService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn check_low_balances(&self, threshold: Decimal) -> Result<Vec<BalanceAlert>, ServiceError> {
        // Simplified - return empty for now
        Ok(vec![])
    }

    pub async fn check_large_withdrawals(&self, _hours: i32) -> Result<Vec<BalanceAlert>, ServiceError> {
        // Simplified - return empty for now
        Ok(vec![])
    }

    pub async fn send_balance_alert(&self, alert: BalanceAlert) -> Result<(), ServiceError> {
        // Simplified - just log to audit_logs
        let details = serde_json::json!({
            "alert_type": alert.alert_type,
            "crypto_type": alert.crypto_type,
            "current_balance": alert.current_balance,
            "threshold": alert.threshold
        });

        sqlx::query!(
            "INSERT INTO audit_logs (merchant_id, action_type, details) VALUES ($1, $2, $3)",
            alert.merchant_id,
            "BALANCE_ALERT",
            details
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}
