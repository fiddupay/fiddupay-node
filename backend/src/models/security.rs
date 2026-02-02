// Security Models
// Database models for security alerts and events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Security alert stored in database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SecurityAlert {
    pub id: i32,
    pub alert_id: String,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub merchant_id: Option<i64>,
    pub acknowledged: bool,
    pub acknowledged_by: Option<i64>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

/// Security event stored in database (audit log)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SecurityEvent {
    pub id: i32,
    pub event_id: String,
    pub event_type: String,
    pub severity: String,
    pub description: String,
    pub merchant_id: Option<i64>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Request to create a new security alert
#[derive(Debug, Deserialize)]
pub struct CreateSecurityAlertRequest {
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub merchant_id: Option<i64>,
    pub expires_in_hours: Option<i64>,
}

/// Request to log a security event
#[derive(Debug, Deserialize)]
pub struct LogSecurityEventRequest {
    pub event_type: String,
    pub severity: String,
    pub description: String,
    pub merchant_id: Option<i64>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Response format for security alerts (API response)
#[derive(Debug, Serialize)]
pub struct SecurityAlertResponse {
    pub alert_id: String,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub acknowledged: bool,
    pub acknowledged_at: Option<String>,
    pub created_at: String,
}

impl From<SecurityAlert> for SecurityAlertResponse {
    fn from(alert: SecurityAlert) -> Self {
        Self {
            alert_id: alert.alert_id,
            alert_type: alert.alert_type,
            severity: alert.severity,
            message: alert.message,
            acknowledged: alert.acknowledged,
            acknowledged_at: alert.acknowledged_at.map(|t| t.to_rfc3339()),
            created_at: alert.created_at.to_rfc3339(),
        }
    }
}

/// Response format for security events (API response)
#[derive(Debug, Serialize)]
pub struct SecurityEventResponse {
    pub event_id: String,
    pub event_type: String,
    pub severity: String,
    pub description: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: String,
}

impl From<SecurityEvent> for SecurityEventResponse {
    fn from(event: SecurityEvent) -> Self {
        Self {
            event_id: event.event_id,
            event_type: event.event_type,
            severity: event.severity,
            description: event.description,
            ip_address: event.ip_address,
            user_agent: event.user_agent,
            created_at: event.created_at.to_rfc3339(),
        }
    }
}
