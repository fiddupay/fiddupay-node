// Audit and IP Whitelist Models
// Database models for audit logs and IP whitelisting

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: i64,
    pub merchant_id: Option<i64>,
    pub action_type: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IpWhitelist {
    pub id: i64,
    pub merchant_id: i64,
    pub ip_address: String,
    pub cidr_range: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddIpWhitelistRequest {
    pub ip_address: String,
    pub cidr_range: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LogAuditEventRequest {
    pub action_type: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub details: Option<serde_json::Value>,
}
