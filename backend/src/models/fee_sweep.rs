use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeSweepSettings {
    pub id: i32,
    pub is_auto_sweep_enabled: bool,
    pub min_accumulated_usd: Option<Decimal>,
    pub schedule_cron: Option<String>,
    pub discord_webhook_url: Option<String>,
    pub gas_alert_threshold_gwei: Option<Decimal>,
    pub gas_alert_threshold_lamports: Option<i64>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFeeSweepSettingsRequest {
    pub is_auto_sweep_enabled: Option<bool>,
    pub min_accumulated_usd: Option<Decimal>,
    pub schedule_cron: Option<String>,
    pub discord_webhook_url: Option<String>,
    pub gas_alert_threshold_gwei: Option<Decimal>,
    pub gas_alert_threshold_lamports: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct GasHistoryRecord {
    pub id: i32,
    pub network: String,
    pub base_fee_gwei: Option<Decimal>,
    pub base_fee_lamports: Option<i64>,
    pub recorded_at: DateTime<Utc>,
}
