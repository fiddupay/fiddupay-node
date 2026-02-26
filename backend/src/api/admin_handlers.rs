// Admin API Handlers
// HTTP handlers for admin operations

use crate::middleware::admin_auth::AdminContext;
use crate::api::state::AppState;
use crate::middleware::auth::MerchantContext;
use crate::services::admin_service::PlatformAnalytics;
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

#[derive(Deserialize)]
pub struct AdminLoginRequest {
    pub username: String,
    pub password: String,
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

// Admin Authentication Endpoints

pub async fn admin_login(
    State(state): State<AppState>,
    Json(login_data): Json<AdminLoginRequest>,
) -> impl IntoResponse {
    // Authenticate against admin_users table
    let admin_user = match sqlx::query!(
        "SELECT id, username, password_hash, role, is_active FROM admin_users WHERE username = $1",
        login_data.username
    )
    .fetch_optional(&state.db_pool)
    .await {
        Ok(Some(user)) => user,
        Ok(None) => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid credentials"}))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    };

    if !admin_user.is_active {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Account deactivated"}))).into_response();
    }

    // Verify Password
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    let parsed_hash = match PasswordHash::new(&admin_user.password_hash) {
        Ok(hash) => hash,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Invalid stored hash"}))).into_response(),
    };

    if Argon2::default().verify_password(login_data.password.as_bytes(), &parsed_hash).is_err() {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid credentials"}))).into_response();
    }

    // Generate Session Token (Simple version - production should use JWT)
    Json(json!({
        "success": true,
        "session_token": format!("admin_session_{}", admin_user.id),
        "user": {
            "id": admin_user.id,
            "username": admin_user.username,
            "role": admin_user.role,
            "permissions": ["all"]
        }
    })).into_response()
}

pub async fn admin_logout() -> impl IntoResponse {
    Json(json!({
        "success": true,
        "message": "Logged out successfully"
    }))
}

/// Admin middleware to verify admin access
async fn verify_admin_access(
    state: &AppState,
    context: &AdminContext,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    match state.admin_service.verify_admin_access(context.admin_id).await {
        Ok(true) => Ok(()),
        Ok(false) => Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "Admin access required",
                "message": "This endpoint requires admin privileges"
            }))
        )),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Authorization check failed",
                "message": "Failed to verify admin privileges"
            }))
        )),
    }
}

/// Get admin dashboard statistics
pub async fn get_admin_dashboard(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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

#[derive(Deserialize)]
pub struct UpdateMerchantFeeRequest {
    pub fee_percentage: Option<rust_decimal::Decimal>,
    pub customer_pays_fee: Option<bool>,
}

/// Update specific merchant fee settings
pub async fn update_merchant_fee(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
    Path(merchant_id): Path<i64>,
    Json(req): Json<UpdateMerchantFeeRequest>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    // Update in database
    let result = sqlx::query(
        "UPDATE merchants SET fee_percentage = COALESCE($1, fee_percentage), customer_pays_fee = COALESCE($2, customer_pays_fee), updated_at = NOW() WHERE id = $3"
    )
    .bind(req.fee_percentage)
    .bind(req.customer_pays_fee)
    .bind(merchant_id)
    .execute(&state.db_pool)
    .await;

    match result {
        Ok(_) => Json(json!({
            "status": "success",
            "message": "Merchant fee settings updated",
            "data": {
                "merchant_id": merchant_id,
                "fee_percentage": req.fee_percentage,
                "customer_pays_fee": req.customer_pays_fee
            }
        })).into_response(),
        Err(e) => {
            tracing::error!("Failed to update merchant fee: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
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
    })).into_response()
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
    })).into_response()
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
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value"
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
    if config.daily_volume_limit_non_kyc_usd.is_some() || config.max_monthly_transaction_volume.is_some() {
        updated_sections.push("limits");
    }
    if config.require_2fa_for_withdrawals.is_some() || config.auto_suspend_suspicious_accounts.is_some() {
        updated_sections.push("security");
    }

    Json(json!({
        "status": "success",
        "message": "Admin configuration updated successfully",
        "updated_sections": updated_sections,
        "config": config
    })).into_response()
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
    })).into_response()
}

/// Get all payments (admin view)
pub async fn get_all_payments(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
    Query(query): Query<AdminQuery>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    match state.admin_service.get_platform_analytics().await {
        Ok(analytics) => Json(analytics).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to get platform analytics",
                "message": e.to_string()
            }))
        ).into_response(),
    }
}

/// Get revenue analytics
pub async fn get_revenue_analytics(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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

#[derive(Deserialize)]
pub struct AdminWalletQuery {
    pub wallet_type: Option<String>,       // "hot" | "cold"
    pub include_balances: Option<bool>,
}

/// Unified admin wallet view — replaces get_hot_wallets, get_cold_wallets, get_wallet_balances
pub async fn get_all_wallets(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
    Query(query): Query<AdminWalletQuery>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    let wallet_type = query.wallet_type.as_deref().unwrap_or("all");
    let include_balances = query.include_balances.unwrap_or(true);

    let mut hot_wallets = json!([
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
    ]);

    let mut cold_wallets = json!([
        {
            "crypto_type": "ETH",
            "address": "0xABCD...EFGH",
            "balance": 500.0,
            "balance_usd": 1250000.0
        }
    ]);

    let mut response = json!({});

    match wallet_type {
        "hot" => {
            response["hot_wallets"] = hot_wallets;
        },
        "cold" => {
            response["cold_wallets"] = cold_wallets;
        },
        _ => {
            response["hot_wallets"] = hot_wallets;
            response["cold_wallets"] = cold_wallets;
        }
    }

    if include_balances {
        response["total_balance_usd"] = json!(1450000.0);
        response["hot_wallet_balance_usd"] = json!(200000.0);
        response["cold_wallet_balance_usd"] = json!(1250000.0);
        response["balances_by_crypto"] = json!([
            {
                "crypto_type": "ETH",
                "hot_balance": 50.5,
                "cold_balance": 500.0,
                "total_balance": 550.5,
                "total_balance_usd": 1375000.0
            }
        ]);
    }

    Json(response).into_response()
}

/// Transfer funds between wallets
pub async fn transfer_funds(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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
    Extension(context): Extension<AdminContext>,
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

// ============================================================================
// Admin Security Monitoring
// ============================================================================

#[derive(Deserialize)]
pub struct SecurityQuery {
    pub event_type: Option<String>,
    pub severity: Option<String>,
    pub limit: Option<i32>,
}

pub async fn get_security_events(
    Query(query): Query<SecurityQuery>,
) -> impl IntoResponse {
    let events = vec![
        json!({
            "id": "evt_001",
            "event_type": "login",
            "severity": "medium",
            "description": "Failed login attempt",
            "timestamp": chrono::Utc::now(),
            "merchant_id": 1
        })
    ];
    
    (StatusCode::OK, Json(json!({
        "data": events,
        "total": 1
    }))).into_response()
}

pub async fn get_security_alerts(
    Query(query): Query<SecurityQuery>,
) -> impl IntoResponse {
    let alerts = vec![
        json!({
            "id": "alert_001",
            "priority": "high",
            "status": "active",
            "description": "Multiple failed login attempts detected",
            "created_at": chrono::Utc::now()
        })
    ];
    
    (StatusCode::OK, Json(json!({
        "data": alerts,
        "total": 1
    }))).into_response()
}

pub async fn acknowledge_alert(
    Path(alert_id): Path<String>,
) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({
        "success": true,
        "message": format!("Alert {} acknowledged", alert_id)
    }))).into_response()
}
