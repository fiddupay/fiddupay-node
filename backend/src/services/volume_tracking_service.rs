use crate::error::ServiceError;
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;

/// Service for tracking and validating daily transaction volumes
pub struct VolumeTrackingService {
    db_pool: PgPool,
}

impl VolumeTrackingService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Get total daily volume (deposits + withdrawals + sweeps) for a merchant
    pub async fn get_daily_volume(
        &self,
        merchant_id: i64,
        date: NaiveDate,
    ) -> Result<Decimal, ServiceError> {
        let start_of_day = date.and_hms_opt(0, 0, 0).unwrap().and_utc();

        // 1. Sum up all confirmed payments (Inflow)
        let payments_volume: Option<Decimal> = sqlx::query_scalar(
            "SELECT SUM(amount_usd) FROM payment_transactions WHERE merchant_id = $1 AND created_at >= $2 AND status = 'CONFIRMED'"
        )
        .bind(merchant_id)
        .bind(start_of_day)
        .fetch_one(&self.db_pool)
        .await?;

        // 2. Sum up all non-rejected withdrawals (Outflow - includes Sweeps)
        let withdrawals_volume: Option<Decimal> = sqlx::query_scalar(
            "SELECT SUM(amount_usd) FROM withdrawals WHERE merchant_id = $1 AND created_at >= $2 AND status != 'REJECTED' AND status != 'CANCELLED'"
        )
        .bind(merchant_id)
        .bind(start_of_day)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(payments_volume.unwrap_or(Decimal::ZERO) + withdrawals_volume.unwrap_or(Decimal::ZERO))
    }

    /// Check if a merchant can process a transaction without exceeding daily volume limit
    pub async fn can_process_transaction(
        &self,
        merchant_id: i64,
        transaction_amount_usd: Decimal,
        daily_limit_usd: Decimal,
        is_kyc_verified: bool,
    ) -> Result<bool, ServiceError> {
        // KYC verified merchants have no daily volume limit for now
        if is_kyc_verified {
            return Ok(true);
        }

        let today = Utc::now().date_naive();
        let current_volume = self.get_daily_volume(merchant_id, today).await?;
        let new_total = current_volume + transaction_amount_usd;

        Ok(new_total <= daily_limit_usd)
    }

    /// Get remaining daily volume for a merchant
    pub async fn get_remaining_daily_volume(
        &self,
        merchant_id: i64,
        daily_limit_usd: Decimal,
        is_kyc_verified: bool,
    ) -> Result<Option<Decimal>, ServiceError> {
        // KYC verified merchants have no limit for now
        if is_kyc_verified {
            return Ok(None);
        }

        let today = Utc::now().date_naive();
        let current_volume = self.get_daily_volume(merchant_id, today).await?;
        let remaining = daily_limit_usd - current_volume;

        Ok(Some(remaining.max(Decimal::ZERO)))
    }
}
