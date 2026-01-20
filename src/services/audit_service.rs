use sqlx::PgPool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::error::ServiceError;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: i32,
    pub merchant_id: i32,
    pub action_type: String,
    pub ip_address: Option<String>,
    pub details: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub action_type: Option<String>,
    pub limit: Option<i64>,
}

pub struct AuditService {
    pool: PgPool,
}

impl AuditService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn log_event(
        &self,
        merchant_id: i32,
        action_type: &str,
        ip_address: Option<&str>,
        details: Option<&str>,
    ) -> Result<(), ServiceError> {
        sqlx::query!(
            "INSERT INTO audit_logs (merchant_id, action_type, ip_address, details) VALUES ($1, $2, $3, $4)",
            merchant_id,
            action_type,
            ip_address,
            details
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_logs(&self, merchant_id: i32, query: AuditLogQuery) -> Result<Vec<AuditLog>, ServiceError> {
        let limit = query.limit.unwrap_or(100).min(1000);
        
        let logs = if let Some(action_type) = query.action_type {
            sqlx::query_as!(
                AuditLog,
                r#"
                SELECT id, merchant_id, action_type, ip_address, details, created_at
                FROM audit_logs
                WHERE merchant_id = $1
                  AND action_type = $2
                  AND ($3::timestamptz IS NULL OR created_at >= $3)
                  AND ($4::timestamptz IS NULL OR created_at <= $4)
                ORDER BY created_at DESC
                LIMIT $5
                "#,
                merchant_id,
                action_type,
                query.from,
                query.to,
                limit
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                AuditLog,
                r#"
                SELECT id, merchant_id, action_type, ip_address, details, created_at
                FROM audit_logs
                WHERE merchant_id = $1
                  AND ($2::timestamptz IS NULL OR created_at >= $2)
                  AND ($3::timestamptz IS NULL OR created_at <= $3)
                ORDER BY created_at DESC
                LIMIT $4
                "#,
                merchant_id,
                query.from,
                query.to,
                limit
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(logs)
    }
}
