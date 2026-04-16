use crate::error::ServiceError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub merchant_id: Option<i64>,
    pub event_type: String,
    pub severity: String,
    pub source_ip: String,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecuritySummary {
    pub total_events: u32,
    pub high_severity_events: u32,
    pub blocked_ips: u32,
    pub suspicious_activities: u32,
}

pub struct SecurityMonitoringService {
    db_pool: PgPool,
}

impl SecurityMonitoringService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn log_security_event(&self, event: SecurityEvent) -> Result<(), ServiceError> {
        sqlx::query(
            r#"INSERT INTO audit_logs 
               (merchant_id, action_type, ip_address, details, created_at)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(event.merchant_id)
        .bind(&event.event_type)
        .bind(&event.source_ip)
        .bind(serde_json::json!({
            "severity": event.severity,
            "details": event.details
        }))
        .bind(event.timestamp)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn get_security_summary(
        &self,
        merchant_id: i64,
    ) -> Result<SecuritySummary, ServiceError> {
        // 1. Get total events and high severity counts from audit_logs
        let counts = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE details->>'severity' = 'HIGH' OR details->>'severity' = 'CRITICAL') as high_severity
            FROM audit_logs 
            WHERE merchant_id = $1 
              AND (action_type LIKE 'security.%' OR details->>'severity' IS NOT NULL)
            "#
        )
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await?;

        let total_events: i64 = counts.get("total");
        let high_severity_events: i64 = counts.get("high_severity");

        // 2. Get currently blocked IPs for this merchant's email
        // First get merchant's email
        let email: String = sqlx::query_scalar("SELECT email FROM merchants WHERE id = $1")
            .bind(merchant_id)
            .fetch_one(&self.db_pool)
            .await?;

        let blocked_ips: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM merchant_login_attempts WHERE email = $1 AND locked_until > NOW()"
        )
        .bind(&email)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(SecuritySummary {
            total_events: total_events as u32,
            high_severity_events: high_severity_events as u32,
            blocked_ips: blocked_ips as u32,
            suspicious_activities: total_events as u32, // Simplified fallback
        })
    }

    pub async fn get_recent_events(
        &self,
        merchant_id: i64,
        limit: i32,
    ) -> Result<Vec<SecurityEvent>, ServiceError> {
        let rows = sqlx::query(
            r#"
            SELECT merchant_id, action_type, ip_address, details, created_at
            FROM audit_logs
            WHERE merchant_id = $1 
              AND (action_type LIKE 'security.%' OR details->>'severity' IS NOT NULL)
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(merchant_id)
        .bind(limit)
        .fetch_all(&self.db_pool)
        .await?;

        let events = rows
            .into_iter()
            .map(|row| {
                use sqlx::Row;
                let details: serde_json::Value = row.get("details");
                SecurityEvent {
                    merchant_id: row.get("merchant_id"),
                    event_type: row.get("action_type"),
                    severity: details["severity"].as_str().unwrap_or("INFO").to_string(),
                    source_ip: row
                        .get::<Option<String>, _>("ip_address")
                        .unwrap_or_default(),
                    details: details["details"].clone(),
                    timestamp: row.get("created_at"),
                }
            })
            .collect();

        Ok(events)
    }
}
