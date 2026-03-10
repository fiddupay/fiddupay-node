// Admin Service
// Business logic for admin operations

use crate::error::ServiceError;
use crate::models::merchant::Merchant;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use rust_decimal::Decimal;

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
pub struct PlatformAnalytics {
    pub total_merchants: i64,
    pub active_merchants: i64,
    pub total_payments: i64,
    pub total_volume_usd: String,
    pub platform_revenue_usd: String,
    pub period: String,
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
    /// Check if user has admin privileges
    pub async fn verify_admin_access(&self, admin_id: i64) -> Result<bool, ServiceError> {
        let result: Option<(String,)> = sqlx::query_as(
            "SELECT role FROM admin_users WHERE id = $1 AND is_active = true"
        )
        .bind(admin_id as i32)
        .fetch_optional(&self.db_pool)
        .await?;

        match result {
            Some((role,)) => {
                Ok(role == "ADMIN" || role == "SUPER_ADMIN")
            }
            None => Ok(false),
        }
    }

    /// Get admin dashboard statistics
    pub async fn get_dashboard_stats(&self) -> Result<AdminDashboard, ServiceError> {
        let total_merchants: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM merchants"
        )
        .fetch_one(&self.db_pool)
        .await?;

        let active_merchants: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM merchants WHERE is_active = true"
        )
        .fetch_one(&self.db_pool)
        .await?;

        let total_payments: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM payment_transactions"
        )
        .fetch_one(&self.db_pool)
        .await?;

        let total_volume: Decimal = sqlx::query_scalar::<_, Decimal>(
            "SELECT COALESCE(SUM(amount_usd), 0) FROM payment_transactions WHERE status = 'CONFIRMED'"
        )
        .fetch_one(&self.db_pool)
        .await?;

        let pending_payments: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM payment_transactions WHERE status = 'PENDING'"
        )
        .fetch_one(&self.db_pool)
        .await?;

        let failed_payments: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM payment_transactions WHERE status = 'FAILED'"
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(AdminDashboard {
            total_merchants,
            active_merchants,
            total_payments,
            total_volume_usd: total_volume.to_string(),
            pending_payments,
            failed_payments,
        })
    }


    /// Get platform analytics (Real Data)
    pub async fn get_platform_analytics(&self) -> Result<PlatformAnalytics, ServiceError> {
        let total_merchants: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM merchants")
            .fetch_one(&self.db_pool)
            .await?;

        let active_merchants: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM merchants WHERE is_active = true")
            .fetch_one(&self.db_pool)
            .await?;

        let total_payments: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM payment_transactions")
            .fetch_one(&self.db_pool)
            .await?;

        let total_volume: Decimal = sqlx::query_scalar::<_, Decimal>(
            "SELECT COALESCE(SUM(amount_usd), 0) FROM payment_transactions WHERE status = 'CONFIRMED'"
        )
        .fetch_one(&self.db_pool)
        .await?;

        let platform_revenue: Decimal = sqlx::query_scalar::<_, Decimal>(
            "SELECT COALESCE(SUM(fee_amount_usd), 0) FROM payment_transactions WHERE status = 'CONFIRMED'"
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(PlatformAnalytics {
            total_merchants,
            active_merchants,
            total_payments,
            total_volume_usd: total_volume.to_string(),
            platform_revenue_usd: platform_revenue.to_string(),
            period: "all_time".to_string(),
        })
    }

    /// Get all merchants summary
    pub async fn get_merchants_summary(&self) -> Result<Vec<MerchantSummary>, ServiceError> {
        let rows = sqlx::query(
            r#"
            SELECT 
                m.id,
                m.email,
                m.business_name,
                m.role::text as role,
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
        for row in rows {
            summaries.push(MerchantSummary {
                merchant_id: row.get("id"),
                email: row.get("email"),
                business_name: row.get("business_name"),
                role: row.try_get::<Option<String>, _>("role").ok().flatten().unwrap_or("MERCHANT".to_string()),
                is_active: row.get("is_active"),
                sandbox_mode: row.get("sandbox_mode"),
                created_at: row.get::<DateTime<Utc>, _>("created_at").to_rfc3339(),
                last_payment: row.try_get::<Option<DateTime<Utc>>, _>("last_payment").ok().flatten().map(|dt| dt.to_rfc3339()),
                total_payments: row.try_get::<Option<i64>, _>("total_payments").ok().flatten().unwrap_or(0),
                total_volume_usd: row.try_get::<Option<Decimal>, _>("total_volume").ok().flatten().unwrap_or(Decimal::ZERO).to_string(),
            });
        }

        Ok(summaries)
    }

    /// Get security events from real data
    pub async fn get_security_events(&self) -> Result<Vec<SecurityEvent>, ServiceError> {
        // Query recent failed payments as security events
        let failed_payments = sqlx::query(
            r#"
            SELECT 
                payment_id,
                status::text,
                created_at,
                amount_usd
            FROM payment_transactions
            WHERE status = 'FAILED'
            ORDER BY created_at DESC
            LIMIT 10
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        let mut events = Vec::new();
        
        for payment in failed_payments {
            let payment_id: String = payment.get("payment_id");
            let amount_usd: Decimal = payment.get("amount_usd");
            let created_at: DateTime<Utc> = payment.get("created_at");
            events.push(SecurityEvent {
                event_id: format!("evt_{}", payment_id),
                event_type: "payment_failed".to_string(),
                severity: "medium".to_string(),
                description: format!("Payment {} failed (${:.2})", payment_id, amount_usd),
                ip_address: None,
                user_agent: None,
                created_at: created_at.to_rfc3339(),
            });
        }
        
        // Query high-value transactions as security events
        let high_value = sqlx::query(
            r#"
            SELECT 
                payment_id,
                amount_usd,
                created_at
            FROM payment_transactions
            WHERE amount_usd > 500
            ORDER BY created_at DESC
            LIMIT 10
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        for tx in high_value {
            let payment_id: String = tx.get("payment_id");
            let amount_usd: Decimal = tx.get("amount_usd");
            let created_at: DateTime<Utc> = tx.get("created_at");
            events.push(SecurityEvent {
                event_id: format!("evt_hv_{}", payment_id),
                event_type: "high_value_transaction".to_string(),
                severity: "low".to_string(),
                description: format!("High-value transaction: ${:.2}", amount_usd),
                ip_address: None,
                user_agent: None,
                created_at: created_at.to_rfc3339(),
            });
        }
        
        // Sort by date descending
        events.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(events)
    }

    /// Get security alerts - combines persisted and dynamic alerts
    pub async fn get_security_alerts(&self) -> Result<Vec<SecurityAlert>, ServiceError> {
        // First, get persisted alerts from database
        let db_alert_rows = sqlx::query(
            r#"
            SELECT alert_id, alert_type, severity, message, acknowledged, acknowledged_at, created_at
            FROM security_alerts
            WHERE (expires_at IS NULL OR expires_at > NOW())
            ORDER BY created_at DESC
            LIMIT 50
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        let mut alerts: Vec<SecurityAlert> = db_alert_rows.into_iter().map(|a| SecurityAlert {
            alert_id: a.get("alert_id"),
            alert_type: a.get("alert_type"),
            severity: a.get("severity"),
            message: a.get("message"),
            acknowledged: a.get("acknowledged"),
            acknowledged_at: a.try_get::<Option<DateTime<Utc>>, _>("acknowledged_at").ok().flatten().map(|t| t.to_rfc3339()),
            created_at: a.get::<DateTime<Utc>, _>("created_at").to_rfc3339(),
        }).collect();
        
        // Generate and persist dynamic alerts if they don't exist
        let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        
        // Check for high failure rate
        let total_today: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM payment_transactions WHERE created_at >= $1"
        )
        .bind(today_start)
        .fetch_one(&self.db_pool)
        .await?;
        
        let failed_today: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM payment_transactions WHERE created_at >= $1 AND status = 'FAILED'"
        )
        .bind(today_start)
        .fetch_one(&self.db_pool)
        .await?;
        
        if total_today > 10 && (failed_today as f64 / total_today as f64) > 0.3 {
            let alert_id = format!("alert_failure_{}", Utc::now().format("%Y%m%d"));
            
            let _ = sqlx::query(
                "INSERT INTO security_alerts (alert_id, alert_type, severity, message, expires_at) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (alert_id) DO NOTHING"
            )
            .bind(&alert_id)
            .bind("high_failure_rate")
            .bind("high")
            .bind(format!("High failure rate: {} of {} payments failed today", failed_today, total_today))
            .bind(Utc::now() + chrono::Duration::days(1))
            .execute(&self.db_pool)
            .await;
        }
        
        // Refresh alerts list after inserting dynamic ones
        if alerts.is_empty() {
            let refreshed = sqlx::query(
                "SELECT alert_id, alert_type, severity, message, acknowledged, acknowledged_at, created_at FROM security_alerts WHERE (expires_at IS NULL OR expires_at > NOW()) ORDER BY created_at DESC LIMIT 50"
            )
            .fetch_all(&self.db_pool)
            .await?;
            
            alerts = refreshed.into_iter().map(|a| SecurityAlert {
                alert_id: a.get("alert_id"),
                alert_type: a.get("alert_type"),
                severity: a.get("severity"),
                message: a.get("message"),
                acknowledged: a.get("acknowledged"),
                acknowledged_at: a.try_get::<Option<DateTime<Utc>>, _>("acknowledged_at").ok().flatten().map(|t| t.to_rfc3339()),
                created_at: a.get::<DateTime<Utc>, _>("created_at").to_rfc3339(),
            }).collect();
        }
        
        Ok(alerts)
    }

    /// Acknowledge security alert in database
    pub async fn acknowledge_alert(&self, alert_id: &str) -> Result<(), ServiceError> {
        sqlx::query(
            "UPDATE security_alerts SET acknowledged = true, acknowledged_at = NOW() WHERE alert_id = $1"
        )
        .bind(alert_id)
        .execute(&self.db_pool)
        .await?;
        
        tracing::info!("Security alert {} acknowledged", alert_id);
        Ok(())
    }

    /// Get fee sweep settings
    pub async fn get_fee_sweep_settings(&self) -> Result<crate::models::fee_sweep::FeeSweepSettings, ServiceError> {
        let row = sqlx::query(
            "SELECT id, is_auto_sweep_enabled, min_accumulated_usd, schedule_cron, discord_webhook_url, gas_alert_threshold_gwei, gas_alert_threshold_lamports, updated_at FROM fee_sweep_settings LIMIT 1"
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(row) = row {
            use sqlx::Row;
            Ok(crate::models::fee_sweep::FeeSweepSettings {
                id: row.get("id"),
                is_auto_sweep_enabled: row.get("is_auto_sweep_enabled"),
                min_accumulated_usd: row.try_get("min_accumulated_usd").ok(),
                schedule_cron: row.try_get("schedule_cron").ok(),
                discord_webhook_url: row.try_get("discord_webhook_url").ok(),
                gas_alert_threshold_gwei: row.try_get("gas_alert_threshold_gwei").ok(),
                gas_alert_threshold_lamports: row.try_get("gas_alert_threshold_lamports").ok(),
                updated_at: row.try_get("updated_at").ok(),
            })
        } else {
            // Return defaults if not configured
            Ok(crate::models::fee_sweep::FeeSweepSettings {
                id: 1,
                is_auto_sweep_enabled: false,
                min_accumulated_usd: None,
                schedule_cron: None,
                discord_webhook_url: None,
                gas_alert_threshold_gwei: None,
                gas_alert_threshold_lamports: None,
                updated_at: None,
            })
        }
    }

    /// Update fee sweep settings
    pub async fn update_fee_sweep_settings(&self, req: crate::models::fee_sweep::UpdateFeeSweepSettingsRequest) -> Result<crate::models::fee_sweep::FeeSweepSettings, ServiceError> {
        sqlx::query(
            r#"
            INSERT INTO fee_sweep_settings (id, is_auto_sweep_enabled, min_accumulated_usd, schedule_cron, discord_webhook_url, gas_alert_threshold_gwei, gas_alert_threshold_lamports, updated_at)
            VALUES (1, COALESCE($1, false), $2, $3, $4, $5, $6, NOW())
            ON CONFLICT (id) DO UPDATE SET
                is_auto_sweep_enabled = COALESCE($1, fee_sweep_settings.is_auto_sweep_enabled),
                min_accumulated_usd = COALESCE($2, fee_sweep_settings.min_accumulated_usd),
                schedule_cron = COALESCE($3, fee_sweep_settings.schedule_cron),
                discord_webhook_url = COALESCE($4, fee_sweep_settings.discord_webhook_url),
                gas_alert_threshold_gwei = COALESCE($5, fee_sweep_settings.gas_alert_threshold_gwei),
                gas_alert_threshold_lamports = COALESCE($6, fee_sweep_settings.gas_alert_threshold_lamports),
                updated_at = NOW()
            "#
        )
        .bind(req.is_auto_sweep_enabled)
        .bind(req.min_accumulated_usd)
        .bind(req.schedule_cron)
        .bind(req.discord_webhook_url)
        .bind(req.gas_alert_threshold_gwei)
        .bind(req.gas_alert_threshold_lamports)
        .execute(&self.db_pool)
        .await?;

        self.get_fee_sweep_settings().await
    }

}
