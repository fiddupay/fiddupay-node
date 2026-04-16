use crate::error::ServiceError;
use serde_json::json;
use sqlx::PgPool;

pub struct WalletSecurityService {
    db_pool: PgPool,
}

impl WalletSecurityService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn log_wallet_access(
        &self,
        merchant_id: i64,
        ip_address: &str,
        details: Option<&str>,
    ) -> Result<(), ServiceError> {
        let details_json = json!({
            "details": details.unwrap_or("Wallet access"),
            "event_type": "wallet_access"
        });

        sqlx::query(
            "INSERT INTO audit_logs (merchant_id, action_type, ip_address, details) VALUES ($1, $2, $3, $4)"
        )
        .bind(merchant_id)
        .bind("WALLET_ACCESS")
        .bind(ip_address)
        .bind(&details_json)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn send_security_alert(
        &self,
        merchant_id: i64,
        alert_type: &str,
        message: &str,
    ) -> Result<(), ServiceError> {
        let details_json = json!({
            "alert_type": alert_type,
            "message": message
        });

        sqlx::query(
            "INSERT INTO audit_logs (merchant_id, action_type, details) VALUES ($1, $2, $3)",
        )
        .bind(merchant_id)
        .bind("SECURITY_ALERT")
        .bind(&details_json)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn check_suspicious_activity(
        &self,
        merchant_id: i64,
        ip_address: &str,
    ) -> Result<bool, ServiceError> {
        // High-velocity check: Has this IP address triggered many events recently?
        // We check across all merchants for this IP to detect cross-account attacks.
        let ip_event_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_logs WHERE ip_address = $1 AND created_at > NOW() - INTERVAL '1 minute'"
        )
        .bind(ip_address)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if ip_event_count > 15 {
            tracing::warn!(
                "Suspicious activity detected from IP {}: {} events in 1 minute",
                ip_address,
                ip_event_count
            );
            return Ok(true);
        }

        // Targeted check: Has THIS specific merchant had unusual failure volumes?
        let merchant_event_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_logs WHERE merchant_id = $1 AND created_at > NOW() - INTERVAL '5 minutes'"
        )
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if merchant_event_count > 50 {
            tracing::warn!(
                "Suspicious activity detected for merchant {}: {} events in 5 minutes",
                merchant_id,
                merchant_event_count
            );
            return Ok(true);
        }

        Ok(false)
    }
}
