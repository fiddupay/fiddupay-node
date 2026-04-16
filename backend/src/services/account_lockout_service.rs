use crate::error::ServiceError;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};

pub struct AccountLockoutService {
    pool: PgPool,
    max_attempts: u32,
    lockout_duration_minutes: i64,
}

impl AccountLockoutService {
    pub fn new(pool: PgPool, max_attempts: u32, lockout_duration_minutes: i64) -> Self {
        Self {
            pool,
            max_attempts,
            lockout_duration_minutes,
        }
    }

    /// Check if an email is currently locked out
    pub async fn check_lockout(&self, email: &str) -> Result<bool, ServiceError> {
        let row = sqlx::query("SELECT locked_until FROM merchant_login_attempts WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let locked_until: Option<DateTime<Utc>> = row.get("locked_until");
            if let Some(until) = locked_until {
                if until > Utc::now() {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Record a failed login attempt and potentially lock the account.
    /// Returns Ok(true) if the account was newly locked during this attempt.
    pub async fn record_failed_attempt(
        &self,
        email: &str,
        _ip: &str,
    ) -> Result<bool, ServiceError> {
        // Upsert the failed attempt record
        sqlx::query(
            r#"
            INSERT INTO merchant_login_attempts (email, failed_attempts, last_attempt_at)
            VALUES ($1, 1, CURRENT_TIMESTAMP)
            ON CONFLICT (email) DO UPDATE SET
                failed_attempts = merchant_login_attempts.failed_attempts + 1,
                last_attempt_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(email)
        .execute(&self.pool)
        .await?;

        // Check if we need to lock the account
        let row =
            sqlx::query("SELECT failed_attempts FROM merchant_login_attempts WHERE email = $1")
                .bind(email)
                .fetch_one(&self.pool)
                .await?;

        let attempts: i32 = row.get("failed_attempts");
        if attempts >= self.max_attempts as i32 {
            let lockout_until =
                Utc::now() + chrono::Duration::minutes(self.lockout_duration_minutes);
            sqlx::query("UPDATE merchant_login_attempts SET locked_until = $1 WHERE email = $2")
                .bind(lockout_until)
                .bind(email)
                .execute(&self.pool)
                .await?;
            return Ok(true);
        }

        Ok(false)
    }

    /// Reset failed attempts on a successful login
    pub async fn record_successful_login(
        &self,
        email: &str,
        _ip: &str,
    ) -> Result<(), ServiceError> {
        sqlx::query("DELETE FROM merchant_login_attempts WHERE email = $1")
            .bind(email)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Cleanup records older than 24 hours
    pub async fn cleanup_old_attempts(&self) -> Result<(), ServiceError> {
        sqlx::query("DELETE FROM merchant_login_attempts WHERE updated_at < $1")
            .bind(Utc::now() - chrono::Duration::hours(24))
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
