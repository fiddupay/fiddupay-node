// System Settings Model
// Database model for dynamic system configuration

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemSetting {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSystemSettingRequest {
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct SystemSettingResponse {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
    pub updated_at: String,
}

impl From<SystemSetting> for SystemSettingResponse {
    fn from(setting: SystemSetting) -> Self {
        Self {
            key: setting.key,
            value: setting.value,
            description: setting.description,
            updated_at: setting.updated_at.to_rfc3339(),
        }
    }
}
