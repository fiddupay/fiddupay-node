use rust_decimal::Decimal;
use sqlx::PgPool;
use chrono::{DateTime, Utc, NaiveDate};
use crate::error::ServiceError;

/// Service for tracking and validating daily transaction volumes
pub struct VolumeTrackingService {
    db_pool: PgPool,
}

impl VolumeTrackingService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Get total daily volume (deposits + withdrawals) for a merchant
    pub async fn get_daily_volume(&self, merchant_id: i64, date: NaiveDate) -> Result<Decimal, ServiceError> {
        let start_of_day = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end_of_day = date.and_hms_opt(23, 59, 59).unwrap().and_utc();

        // For now, return zero as a placeholder until we have actual payment/withdrawal tables
        // In a real implementation, this would query the actual tables
        Ok(Decimal::ZERO)
    }

    /// Check if a merchant can process a transaction without exceeding daily volume limit
    pub async fn can_process_transaction(
        &self,
        merchant_id: i64,
        transaction_amount_usd: Decimal,
        daily_limit_usd: Decimal,
        is_kyc_verified: bool,
    ) -> Result<bool, ServiceError> {
        // KYC verified merchants have no daily volume limit
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
        // KYC verified merchants have no limit
        if is_kyc_verified {
            return Ok(None);
        }

        let today = Utc::now().date_naive();
        let current_volume = self.get_daily_volume(merchant_id, today).await?;
        let remaining = daily_limit_usd - current_volume;

        Ok(Some(remaining.max(Decimal::ZERO)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_kyc_verified_no_limit() {
        // This would require a test database setup
        // For now, just test the logic
        let large_amount = dec!(10000.00);
        let small_limit = dec!(100.00);
        
        // KYC verified should always return true regardless of amount
        // This test would need actual database setup to run
    }
}
