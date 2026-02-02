// Two-Factor Authentication Model
// Database model for 2FA

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct TwoFactorAuth {
    pub id: i32,
    pub merchant_id: i64,
    pub secret_encrypted: String,
    pub recovery_codes_encrypted: String,
    pub is_enabled: bool,
    pub enabled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct Enable2FARequest {
    pub totp_code: String,
}

#[derive(Debug, Deserialize)]
pub struct Verify2FARequest {
    pub totp_code: String,
}

#[derive(Debug, Serialize)]
pub struct TwoFactorSetupResponse {
    pub secret: String,
    pub qr_code_url: String,
    pub recovery_codes: Vec<String>,
}
