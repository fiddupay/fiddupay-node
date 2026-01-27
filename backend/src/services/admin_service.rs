// Admin Service
// Business logic for admin operations

use crate::error::ServiceError;
use crate::models::merchant::Merchant;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize)]
pub struct AdminDashboard {
    pub total_merchants: i64,
    pub active_merchants: i64,
    pub total_payments: i64,
    pub total_volume_usd: String,
    pub pending_payments: i64,
    pub failed_payments: i64,
}

#[derive(Debug, Serialize)]
pub struct SecurityEvent {
    pub event_id: String,
    pub event_type: String,
    pub severity: String,
    pub description: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct SecurityAlert {
    pub alert_id: String,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub acknowledged: bool,
    pub acknowledged_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct MerchantSummary {
    pub merchant_id: i64,
    pub email: String,
    pub business_name: String,
    pub role: String,
    pub is_active: bool,
    pub sandbox_mode: bool,
    pub created_at: String,
    pub last_payment: Option<String>,
    pub total_payments: i64,
    pub total_volume_usd: String,
}

pub struct AdminService {
    db_pool: PgPool,
}

impl AdminService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Check if merchant has admin privileges
    pub async fn verify_admin_access(&self, merchant_id: i64) -> Result<bool, ServiceError> {
        let result = sqlx::query!(
            "SELECT role::text FROM merchants WHERE id = $1",
            merchant_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        match result {
            Some(merchant) => {
                let role = merchant.role.unwrap_or("MERCHANT".to_string());
                Ok(role == "ADMIN" || role == "SUPER_ADMIN")
            }
            None => Ok(false),
        }
    }

    /// Get admin dashboard statistics
    pub async fn get_dashboard_stats(&self) -> Result<AdminDashboard, ServiceError> {
        let total_merchants = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM merchants"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        let active_merchants = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM merchants WHERE is_active = true"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        let total_payments = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM payment_transactions"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        let total_volume = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(amount_usd), 0) FROM payment_transactions WHERE status = 'CONFIRMED'"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(rust_decimal::Decimal::ZERO);

        let pending_payments = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM payment_transactions WHERE status = 'PENDING'"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        let failed_payments = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM payment_transactions WHERE status = 'FAILED'"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        Ok(AdminDashboard {
            total_merchants,
            active_merchants,
            total_payments,
            total_volume_usd: total_volume.to_string(),
            pending_payments,
            failed_payments,
        })
    }

    /// Get all merchants with summary info
    pub async fn get_merchants_summary(&self) -> Result<Vec<MerchantSummary>, ServiceError> {
        let merchants = sqlx::query!(
            r#"
            SELECT 
                m.id,
                m.email,
                m.business_name,
                m.role::text,
                m.is_active,
                m.sandbox_mode,
                m.created_at,
                COUNT(p.payment_id) as total_payments,
                COALESCE(SUM(p.amount_usd), 0) as total_volume,
                MAX(p.created_at) as last_payment
            FROM merchants m
            LEFT JOIN payment_transactions p ON m.id = p.merchant_id
            GROUP BY m.id, m.email, m.business_name, m.role, m.is_active, m.sandbox_mode, m.created_at
            ORDER BY m.created_at DESC
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut summaries = Vec::new();
        for merchant in merchants {
            summaries.push(MerchantSummary {
                merchant_id: merchant.id,
                email: merchant.email,
                business_name: merchant.business_name,
                role: merchant.role.unwrap_or("MERCHANT".to_string()),
                is_active: merchant.is_active,
                sandbox_mode: merchant.sandbox_mode,
                created_at: merchant.created_at.to_rfc3339(),
                last_payment: merchant.last_payment.map(|dt| dt.to_rfc3339()),
                total_payments: merchant.total_payments.unwrap_or(0),
                total_volume_usd: merchant.total_volume.unwrap_or(rust_decimal::Decimal::ZERO).to_string(),
            });
        }

        Ok(summaries)
    }

    /// Get security events (mock implementation)
    pub async fn get_security_events(&self) -> Result<Vec<SecurityEvent>, ServiceError> {
        // Mock security events for testing
        Ok(vec![
            SecurityEvent {
                event_id: "evt_001".to_string(),
                event_type: "suspicious_login".to_string(),
                severity: "medium".to_string(),
                description: "Multiple failed login attempts".to_string(),
                ip_address: Some("192.168.1.100".to_string()),
                user_agent: Some("Mozilla/5.0".to_string()),
                created_at: Utc::now().to_rfc3339(),
            },
            SecurityEvent {
                event_id: "evt_002".to_string(),
                event_type: "api_key_rotation".to_string(),
                severity: "low".to_string(),
                description: "API key rotated successfully".to_string(),
                ip_address: Some("10.0.0.1".to_string()),
                user_agent: Some("FidduPay-SDK/1.0".to_string()),
                created_at: Utc::now().to_rfc3339(),
            },
        ])
    }

    /// Get security alerts (mock implementation)
    pub async fn get_security_alerts(&self) -> Result<Vec<SecurityAlert>, ServiceError> {
        // Mock security alerts for testing
        Ok(vec![
            SecurityAlert {
                alert_id: "alert_001".to_string(),
                alert_type: "high_volume".to_string(),
                severity: "high".to_string(),
                message: "Unusual transaction volume detected".to_string(),
                acknowledged: false,
                acknowledged_at: None,
                created_at: Utc::now().to_rfc3339(),
            },
        ])
    }

    /// Acknowledge security alert
    pub async fn acknowledge_alert(&self, alert_id: &str) -> Result<(), ServiceError> {
        // Mock implementation - in real system would update database
        println!("Alert {} acknowledged", alert_id);
        Ok(())
    }
}
