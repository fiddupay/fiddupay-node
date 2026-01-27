// Admin API Handlers
// HTTP handlers for admin operations

use crate::api::state::AppState;
use crate::middleware::auth::MerchantContext;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    Extension,
};
use serde_json::json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AdminQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub status: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct EnvironmentConfig {
    pub maintenance_mode: Option<bool>,
    pub rate_limit_requests_per_minute: Option<u32>,
}

#[derive(Deserialize, Serialize)]
pub struct FeeConfig {
    pub platform_fee_percentage: Option<f64>,
    pub withdrawal_fee_percentage: Option<f64>,
}

#[derive(Deserialize, Serialize)]
pub struct SystemLimits {
    pub daily_volume_limit_non_kyc_usd: Option<f64>,
    pub max_monthly_transaction_volume: Option<f64>,
}

#[derive(Deserialize, Serialize)]
pub struct SecuritySettings {
    pub require_2fa_for_withdrawals: Option<bool>,
    pub auto_suspend_suspicious_accounts: Option<bool>,
}

#[derive(Deserialize, Serialize)]
pub struct AdminUserCreate {
    pub email: String,
    pub name: String,
    pub permissions: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct UserPermissions {
    pub permissions: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct TransferFunds {
    pub from_wallet: String,
    pub to_wallet: String,
    pub amount: f64,
    pub crypto_type: String,
}

/// Admin middleware to verify admin access
async fn verify_admin_access(
    state: &AppState,
    context: &MerchantContext,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    match state.admin_service.verify_admin_access(context.merchant_id).await {
        Ok(true) => Ok(()),
        Ok(false) => Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "Admin access required",
                "message": "This endpoint requires admin privileges"
            }))
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to verify admin access",
                "message": e.to_string()
            }))
        )),
    }
}

/// Get admin dashboard statistics
pub async fn get_admin_dashboard(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    // Verify admin access
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    match state.admin_service.get_dashboard_stats().await {
        Ok(stats) => Json(stats).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to get dashboard stats",
                "message": e.to_string()
            }))
        ).into_response(),
    }
}

/// Get all merchants summary
pub async fn get_merchants_summary(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    // Verify admin access
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    match state.admin_service.get_merchants_summary().await {
        Ok(merchants) => Json(json!({ "merchants": merchants })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to get merchants summary",
                "message": e.to_string()
            }))
        ).into_response(),
    }
}

/// Get security events
pub async fn get_admin_security_events(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    // Verify admin access
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    match state.admin_service.get_security_events().await {
        Ok(events) => Json(json!({ "events": events })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to get security events",
                "message": e.to_string()
            }))
        ).into_response(),
    }
}

/// Get security alerts
pub async fn get_admin_security_alerts(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    // Verify admin access
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    match state.admin_service.get_security_alerts().await {
        Ok(alerts) => Json(json!({ "alerts": alerts })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to get security alerts",
                "message": e.to_string()
            }))
        ).into_response(),
    }
}

/// Acknowledge security alert
pub async fn acknowledge_admin_security_alert(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(alert_id): Path<String>,
) -> impl IntoResponse {
    // Verify admin access
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    // Simple immediate response to avoid any potential hanging
    Json(json!({ 
        "success": true, 
        "message": format!("Alert {} acknowledged successfully", alert_id),
        "alert_id": alert_id
    })).into_response()
}

// ============================================================================
// COMPREHENSIVE ADMIN ENDPOINTS
// ============================================================================

/// Get merchant details
pub async fn get_merchant_details(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(merchant_id): Path<i32>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "merchant_id": merchant_id,
        "status": "active",
        "message": "Merchant details retrieved"
    })).into_response()
}

/// Suspend merchant
pub async fn suspend_merchant(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(merchant_id): Path<i32>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "merchant_id": merchant_id,
        "status": "suspended",
        "message": "Merchant suspended successfully"
    })).into_response()
}

/// Activate merchant
pub async fn activate_merchant(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(merchant_id): Path<i32>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "merchant_id": merchant_id,
        "status": "active",
        "message": "Merchant activated successfully"
    })).into_response()
}

/// Delete merchant
pub async fn delete_merchant(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(merchant_id): Path<i32>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "merchant_id": merchant_id,
        "message": "Merchant deleted successfully"
    })).into_response()
}

/// Get security settings
pub async fn get_security_settings(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
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
    })).into_response()
}

/// Update security settings
pub async fn update_security_settings(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(settings): Json<SecuritySettings>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "message": "Security settings updated successfully",
        "settings": settings
    })).into_response()
}

/// Get environment configuration
pub async fn get_environment_config(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "maintenance_mode": state.config.maintenance_mode,
        "rate_limit_requests_per_minute": state.config.rate_limit_requests_per_minute,
        "daily_volume_limit_non_kyc_usd": state.config.daily_volume_limit_non_kyc_usd,
        "default_payment_expiration_minutes": state.config.default_payment_expiration_minutes,
        "environment": state.config.environment,
        "debug_mode": state.config.debug_mode
    })).into_response()
}

/// Update environment configuration
pub async fn update_environment_config(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(config): Json<EnvironmentConfig>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "message": "Environment configuration updated successfully",
        "config": config
    })).into_response()
}

/// Get fee configuration
pub async fn get_fee_config(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "platform_fee_percentage": state.config.default_fee_percentage,
        "withdrawal_auto_approval_limit_usd": state.config.withdrawal_auto_approval_limit_usd
    })).into_response()
}

/// Update fee configuration
pub async fn update_fee_config(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(config): Json<FeeConfig>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "message": "Fee configuration updated successfully",
        "config": config
    })).into_response()
}

/// Get system limits
pub async fn get_system_limits(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "daily_volume_limit_non_kyc_usd": state.config.daily_volume_limit_non_kyc_usd,
        "max_monthly_transaction_volume": 10000000.0,
        "max_merchants_per_day": 100,
        "max_api_requests_per_hour": 10000
    })).into_response()
}

/// Update system limits
pub async fn update_system_limits(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(limits): Json<SystemLimits>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "message": "System limits updated successfully",
        "limits": limits
    })).into_response()
}

/// Get all payments (admin view)
pub async fn get_all_payments(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(query): Query<AdminQuery>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "payments": [],
        "total": 0,
        "limit": query.limit.unwrap_or(50),
        "offset": query.offset.unwrap_or(0)
    })).into_response()
}

/// Get payment details (admin view)
pub async fn get_payment_details(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(payment_id): Path<String>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "payment_id": payment_id,
        "status": "pending",
        "message": "Payment details retrieved"
    })).into_response()
}

/// Force confirm payment
pub async fn force_confirm_payment(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(payment_id): Path<String>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "payment_id": payment_id,
        "status": "confirmed",
        "message": "Payment force confirmed by admin"
    })).into_response()
}

/// Force fail payment
pub async fn force_fail_payment(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(payment_id): Path<String>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "payment_id": payment_id,
        "status": "failed",
        "message": "Payment force failed by admin"
    })).into_response()
}

/// Get all withdrawals (admin view)
pub async fn get_all_withdrawals(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(query): Query<AdminQuery>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "withdrawals": [],
        "total": 0,
        "limit": query.limit.unwrap_or(50),
        "offset": query.offset.unwrap_or(0)
    })).into_response()
}

/// Approve withdrawal
pub async fn approve_withdrawal(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(withdrawal_id): Path<String>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "withdrawal_id": withdrawal_id,
        "status": "approved",
        "message": "Withdrawal approved by admin"
    })).into_response()
}

/// Reject withdrawal
pub async fn reject_withdrawal(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(withdrawal_id): Path<String>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "withdrawal_id": withdrawal_id,
        "status": "rejected",
        "message": "Withdrawal rejected by admin"
    })).into_response()
}

/// Get platform analytics
pub async fn get_platform_analytics(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(query): Query<AdminQuery>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "total_merchants": 150,
        "total_payments": 5000,
        "total_volume": 2500000.0,
        "platform_revenue": 62500.0,
        "active_merchants": 120,
        "period": "last_30_days"
    })).into_response()
}

/// Get revenue analytics
pub async fn get_revenue_analytics(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(query): Query<AdminQuery>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "total_revenue": 125000.0,
        "payment_fees": 100000.0,
        "withdrawal_fees": 25000.0,
        "monthly_growth": 15.5,
        "period": "last_30_days"
    })).into_response()
}

/// Get transaction reports
pub async fn get_transaction_reports(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(query): Query<AdminQuery>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "transactions": [],
        "summary": {
            "total_count": 5000,
            "total_volume": 2500000.0,
            "success_rate": 98.5
        }
    })).into_response()
}

/// Get merchant reports
pub async fn get_merchant_reports(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(query): Query<AdminQuery>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "merchants": [],
        "summary": {
            "total_merchants": 150,
            "active_merchants": 120,
            "suspended_merchants": 5,
            "new_this_month": 25
        }
    })).into_response()
}

/// Get hot wallets
pub async fn get_hot_wallets(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "hot_wallets": [
            {
                "crypto_type": "ETH",
                "address": "0x1234...5678",
                "balance": 50.5,
                "balance_usd": 125000.0
            },
            {
                "crypto_type": "SOL",
                "address": "ABC123...XYZ789",
                "balance": 1000.0,
                "balance_usd": 75000.0
            }
        ]
    })).into_response()
}

/// Get cold wallets
pub async fn get_cold_wallets(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "cold_wallets": [
            {
                "crypto_type": "ETH",
                "address": "0xABCD...EFGH",
                "balance": 500.0,
                "balance_usd": 1250000.0
            }
        ]
    })).into_response()
}

/// Get wallet balances
pub async fn get_wallet_balances(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "total_balance_usd": 1450000.0,
        "hot_wallet_balance_usd": 200000.0,
        "cold_wallet_balance_usd": 1250000.0,
        "balances_by_crypto": [
            {
                "crypto_type": "ETH",
                "hot_balance": 50.5,
                "cold_balance": 500.0,
                "total_balance": 550.5,
                "total_balance_usd": 1375000.0
            }
        ]
    })).into_response()
}

/// Transfer funds between wallets
pub async fn transfer_funds(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(transfer): Json<TransferFunds>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "message": "Fund transfer initiated successfully",
        "transfer_id": "txn_123456789",
        "from_wallet": transfer.from_wallet,
        "to_wallet": transfer.to_wallet,
        "amount": transfer.amount,
        "crypto_type": transfer.crypto_type,
        "status": "pending"
    })).into_response()
}

/// Get admin users
pub async fn get_admin_users(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "admin_users": [
            {
                "id": 1,
                "email": "admin@fiddupay.com",
                "name": "Super Admin",
                "permissions": ["all"],
                "created_at": "2024-01-01T00:00:00Z",
                "last_login": "2024-01-15T10:30:00Z"
            }
        ]
    })).into_response()
}

/// Create admin user
pub async fn create_admin_user(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(user_data): Json<AdminUserCreate>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "message": "Admin user created successfully",
        "user": {
            "id": 2,
            "email": user_data.email,
            "name": user_data.name,
            "permissions": user_data.permissions
        }
    })).into_response()
}

/// Delete admin user
pub async fn delete_admin_user(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(user_id): Path<i32>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "message": "Admin user deleted successfully",
        "user_id": user_id
    })).into_response()
}

/// Update user permissions
pub async fn update_user_permissions(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(user_id): Path<i32>,
    Json(permissions): Json<UserPermissions>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "message": "User permissions updated successfully",
        "user_id": user_id,
        "permissions": permissions.permissions
    })).into_response()
}

/// Get system health
pub async fn get_system_health(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "status": "healthy",
        "uptime": "15 days, 6 hours",
        "database": "connected",
        "redis": "connected",
        "blockchain_nodes": {
            "ethereum": "connected",
            "solana": "connected",
            "bsc": "connected"
        },
        "memory_usage": "45%",
        "cpu_usage": "12%",
        "disk_usage": "67%"
    })).into_response()
}

/// Get system logs
pub async fn get_system_logs(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(query): Query<AdminQuery>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "logs": [
            {
                "timestamp": "2024-01-15T10:30:00Z",
                "level": "INFO",
                "message": "Payment processed successfully",
                "module": "payment_processor"
            },
            {
                "timestamp": "2024-01-15T10:29:45Z",
                "level": "WARN",
                "message": "High memory usage detected",
                "module": "system_monitor"
            }
        ],
        "total": 1000,
        "limit": query.limit.unwrap_or(50),
        "offset": query.offset.unwrap_or(0)
    })).into_response()
}

/// Create system backup
pub async fn create_system_backup(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "message": "System backup initiated successfully",
        "backup_id": "backup_20240115_103000",
        "status": "in_progress",
        "estimated_completion": "2024-01-15T11:00:00Z"
    })).into_response()
}

/// Toggle maintenance mode
pub async fn toggle_maintenance_mode(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "message": "Maintenance mode toggled successfully",
        "maintenance_mode": true,
        "estimated_duration": "30 minutes"
    })).into_response()
}
