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

    pub async fn log_wallet_access(&self, merchant_id: i64, ip_address: &str, details: Option<&str>) -> Result<(), ServiceError> {
        let details_json = json!({
            "details": details.unwrap_or("Wallet access"),
            "event_type": "wallet_access"
        });

        sqlx::query!(
            "INSERT INTO audit_logs (merchant_id, action_type, ip_address, details) VALUES ($1, $2, $3, $4)",
            merchant_id,
            "WALLET_ACCESS",
            ip_address,
            details_json
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn send_security_alert(&self, merchant_id: i64, alert_type: &str, message: &str) -> Result<(), ServiceError> {
        let details_json = json!({
            "alert_type": alert_type,
            "message": message
        });

        sqlx::query!(
            "INSERT INTO audit_logs (merchant_id, action_type, details) VALUES ($1, $2, $3)",
            merchant_id,
            "SECURITY_ALERT",
            details_json
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn check_suspicious_activity(&self, merchant_id: i64, ip_address: &str) -> Result<bool, ServiceError> {
        // Simplified - no suspicious activity detection for now
        Ok(false)
    }
}
