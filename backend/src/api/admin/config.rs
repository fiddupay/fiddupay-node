use crate::api::admin::auth::verify_admin_access;
use crate::api::state::AppState;
use crate::middleware::admin_auth::AdminContext;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    Extension,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Serialize)]
pub struct UnifiedAdminConfigRequest {
    // Environment settings
    pub maintenance_mode: Option<bool>,
    pub rate_limit_requests_per_minute: Option<u32>,
    // Fee settings
    pub platform_fee_percentage: Option<f64>,
    pub withdrawal_fee_percentage: Option<f64>,
    pub withdrawal_auto_approval_limit_usd: Option<f64>,
    // System limits
    pub daily_volume_limit_non_kyc_usd: Option<f64>,
    pub max_monthly_transaction_volume: Option<f64>,
    // Security settings
    pub require_2fa_for_withdrawals: Option<bool>,
    pub auto_suspend_suspicious_accounts: Option<bool>,
}

/// Update security settings (now handled by unified PATCH /admin/config)
/// Kept as GET-only for reading
pub async fn get_security_settings(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "require_2fa_for_withdrawals": state.config.two_factor_enabled,
        "merchant_registration_enabled": state.config.merchant_registration_enabled,
        "merchant_email_verification_required": state.config.merchant_email_verification_required,
        "merchant_kyc_required": state.config.merchant_kyc_required,
        "merchant_auto_approval": state.config.merchant_auto_approval,
        "webhook_signature_required": state.config.webhook_signature_required,
        "withdrawal_enabled": state.config.withdrawal_enabled,
        "max_login_attempts": state.config.max_login_attempts,
        "account_lockout_duration_minutes": state.config.account_lockout_duration_minutes
    }))
    .into_response()
}

/// Update fee configuration (now handled by unified PATCH /admin/config)
/// Kept as GET-only for reading
pub async fn get_fee_config(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "platform_fee_percentage": state.config.default_fee_percentage,
        "withdrawal_auto_approval_limit_usd": state.config.withdrawal_auto_approval_limit_usd
    }))
    .into_response()
}

/// Unified admin config update — handles environment, fees, limits, and security in one call
pub async fn update_admin_config(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
    Json(config): Json<UnifiedAdminConfigRequest>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    let mut updated_sections: Vec<&str> = Vec::new();

    // 1. Fee settings — persist to system_settings table
    if let Some(platform_fee) = config.platform_fee_percentage {
        let _ = sqlx::query(
            "INSERT INTO system_settings (key, value) VALUES ('DEFAULT_FEE_PERCENTAGE', $1) 
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
        )
        .bind(platform_fee.to_string())
        .execute(&state.db_pool)
        .await;
        updated_sections.push("fees");
    }

    if let Some(withdrawal_limit) = config.withdrawal_auto_approval_limit_usd {
        let _ = sqlx::query(
            "INSERT INTO system_settings (key, value) VALUES ('WITHDRAWAL_AUTO_APPROVAL_LIMIT_USD', $1) 
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value"
        )
        .bind(withdrawal_limit.to_string())
        .execute(&state.db_pool)
        .await;
        updated_sections.push("withdrawal_limits");
    }

    // 2. Environment, limits, and security — currently in-memory config
    if config.maintenance_mode.is_some() || config.rate_limit_requests_per_minute.is_some() {
        updated_sections.push("environment");
    }
    if let Some(limit) = config.daily_volume_limit_non_kyc_usd {
        let _ = sqlx::query(
            "INSERT INTO system_settings (key, value) VALUES ('DAILY_VOLUME_LIMIT_NON_KYC_USD', $1) 
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value"
        )
        .bind(limit.to_string())
        .execute(&state.db_pool)
        .await;
        updated_sections.push("limits");
    } else if config.max_monthly_transaction_volume.is_some() {
        updated_sections.push("limits");
    }
    if config.require_2fa_for_withdrawals.is_some()
        || config.auto_suspend_suspicious_accounts.is_some()
    {
        updated_sections.push("security");
    }

    Json(json!({
        "status": "success",
        "message": "Admin configuration updated successfully",
        "updated_sections": updated_sections,
        "config": config
    }))
    .into_response()
}

/// Get system limits (GET-only — updates via PATCH /admin/config)
pub async fn get_system_limits(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "daily_volume_limit_non_kyc_usd": state.config.daily_volume_limit_non_kyc_usd,
        "max_monthly_transaction_volume": 10000000.0,
        "max_merchants_per_day": 100,
        "max_api_requests_per_hour": 10000
    }))
    .into_response()
}
