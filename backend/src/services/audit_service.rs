use crate::error::ServiceError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{PgPool, QueryBuilder};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditScope {
    Merchant(i64),
    Admin, // WHERE merchant_id IS NULL
    All,   // No merchant_id filter
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: i64,
    pub merchant_id: Option<i64>,
    pub action_type: String,
    pub ip_address: Option<String>,
    pub details: Option<JsonValue>,
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
        merchant_id: i64,
        action_type: &str,
        ip_address: Option<&str>,
        details: Option<JsonValue>,
    ) -> Result<(), ServiceError> {
        sqlx::query(
            "INSERT INTO audit_logs (merchant_id, action_type, ip_address, details) VALUES ($1, $2, $3, $4)"
        )
        .bind(merchant_id)
        .bind(action_type)
        .bind(ip_address)
        .bind(&details)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_logs(
        &self,
        scope: AuditScope,
        query: AuditLogQuery,
    ) -> Result<Vec<AuditLog>, ServiceError> {
        let limit = query.limit.unwrap_or(100).min(1000);

        let mut builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
            "SELECT id, merchant_id, action_type, ip_address, details, created_at FROM audit_logs WHERE 1=1"
        );

        match scope {
            AuditScope::Merchant(id) => {
                builder.push(" AND merchant_id = ");
                builder.push_bind(id);
            }
            AuditScope::Admin => {
                builder.push(" AND merchant_id IS NULL");
            }
            AuditScope::All => {} // No filter
        }

        if let Some(action_type) = query.action_type {
            builder.push(" AND action_type = ");
            builder.push_bind(action_type);
        }

        if let Some(from) = query.from {
            builder.push(" AND created_at >= ");
            builder.push_bind(from);
        }

        if let Some(to) = query.to {
            builder.push(" AND created_at <= ");
            builder.push_bind(to);
        }

        builder.push(" ORDER BY created_at DESC LIMIT ");
        builder.push_bind(limit);

        let logs = builder
            .build_query_as::<AuditLog>()
            .fetch_all(&self.pool)
            .await?;

        Ok(logs)
    }
}
