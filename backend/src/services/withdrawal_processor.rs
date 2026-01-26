use crate::error::ServiceError;
use rust_decimal::Decimal;
use sqlx::PgPool;

pub struct WithdrawalProcessor {
    db_pool: PgPool,
}

impl WithdrawalProcessor {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn process_withdrawal(&self, withdrawal_id: &str) -> Result<(), ServiceError> {
        // Simplified processing - just mark as completed
        sqlx::query!(
            "UPDATE withdrawals SET status = 'COMPLETED', completed_at = NOW() WHERE withdrawal_id = $1",
            withdrawal_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn reject_withdrawal(&self, withdrawal_id: &str, reason: &str) -> Result<(), ServiceError> {
        sqlx::query!(
            "UPDATE withdrawals SET status = 'REJECTED', rejection_reason = $1, updated_at = NOW() WHERE withdrawal_id = $2",
            reason,
            withdrawal_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}
