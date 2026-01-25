// Account Lockout Service
// Prevents brute force attacks by tracking failed login attempts

use crate::error::ServiceError;
use sqlx::PgPool;
use std::net::IpAddr;
use chrono::{DateTime, Utc, Duration};

pub struct AccountLockoutService {
    db_pool: PgPool,
}

impl AccountLockoutService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Check if account/IP is locked due to failed attempts
    pub async fn is_locked(&self, email: &str, ip: IpAddr) -> Result<bool, ServiceError> {
        let cutoff = Utc::now() - Duration::minutes(15);
        
        // Check email-based lockout (5 attempts in 15 minutes)
        let email_attempts = sqlx::query!(
            "SELECT COUNT(*) as count FROM login_attempts 
             WHERE email = $1 AND attempted_at > $2 AND success = false",
            email, cutoff.naive_utc()
        )
        .fetch_one(&self.db_pool)
        .await?;

        if email_attempts.count.unwrap_or(0) >= 5 {
            return Ok(true);
        }

        // Check IP-based lockout (10 attempts in 15 minutes)
        let ip_network: ipnetwork::IpNetwork = ip.into();
        let ip_attempts = sqlx::query!(
            "SELECT COUNT(*) as count FROM login_attempts 
             WHERE ip_address = $1 AND attempted_at > $2 AND success = false",
            ip_network, cutoff.naive_utc()
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(ip_attempts.count.unwrap_or(0) >= 10)
    }

    /// Record login attempt
    pub async fn record_attempt(&self, email: &str, ip: IpAddr, success: bool) -> Result<(), ServiceError> {
        let ip_network: ipnetwork::IpNetwork = ip.into();
        sqlx::query!(
            "INSERT INTO login_attempts (email, ip_address, success) VALUES ($1, $2, $3)",
            email, ip_network, success
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Get lockout status with time remaining
    pub async fn get_lockout_info(&self, email: &str, ip: IpAddr) -> Result<Option<DateTime<Utc>>, ServiceError> {
        let cutoff = Utc::now() - Duration::minutes(15);
        
        // Get most recent failed attempt
        let ip_network: ipnetwork::IpNetwork = ip.into();
        let last_attempt = sqlx::query!(
            "SELECT attempted_at FROM login_attempts 
             WHERE (email = $1 OR ip_address = $2) AND attempted_at > $3 AND success = false
             ORDER BY attempted_at DESC LIMIT 1",
            email, ip_network, cutoff.naive_utc()
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(attempt) = last_attempt {
            if let Some(attempted_at) = attempt.attempted_at {
                let unlock_time = DateTime::<Utc>::from_naive_utc_and_offset(attempted_at, Utc) + Duration::minutes(15);
                if unlock_time > Utc::now() {
                    return Ok(Some(unlock_time));
                }
            }
        }

        Ok(None)
    }

    /// Clear old attempts (cleanup job)
    pub async fn cleanup_old_attempts(&self) -> Result<u64, ServiceError> {
        let cutoff = Utc::now() - Duration::hours(24);
        
        let result: sqlx::postgres::PgQueryResult = sqlx::query!(
            "DELETE FROM login_attempts WHERE attempted_at < $1",
            cutoff.naive_utc()
        )
        .execute(&self.db_pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    async fn setup_test_db() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://vibes:password@localhost:5432/fiddupay_test".to_string());
        PgPool::connect(&database_url).await.unwrap()
    }

    #[tokio::test]
    async fn test_account_lockout() {
        let pool = setup_test_db().await;
        let service = AccountLockoutService::new(pool);
        let ip = IpAddr::from_str("192.168.1.1").unwrap();
        
        // Initially not locked
        assert!(!service.is_locked("test@example.com", ip).await.unwrap());
        
        // Record 5 failed attempts
        for _ in 0..5 {
            service.record_attempt("test@example.com", ip, false).await.unwrap();
        }
        
        // Should be locked now
        assert!(service.is_locked("test@example.com", ip).await.unwrap());
        
        // Successful attempt should not affect lockout
        service.record_attempt("test@example.com", ip, true).await.unwrap();
        assert!(service.is_locked("test@example.com", ip).await.unwrap());
    }

    #[tokio::test]
    async fn test_ip_lockout() {
        let pool = setup_test_db().await;
        let service = AccountLockoutService::new(pool);
        let ip = IpAddr::from_str("192.168.1.2").unwrap();
        
        // Record 10 failed attempts from same IP, different emails
        for i in 0..10 {
            let email = format!("test{}@example.com", i);
            service.record_attempt(&email, ip, false).await.unwrap();
        }
        
        // Should be locked by IP
        assert!(service.is_locked("new@example.com", ip).await.unwrap());
    }
}
