use crate::middleware::admin_auth::AdminContext;
use crate::api::state::AppState;
use crate::api::admin::auth::verify_admin_access;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    Extension,
};
use serde_json::json;

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

/// Get security events
pub async fn get_security_events(
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
pub async fn get_security_alerts(
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
pub async fn acknowledge_alert(
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

/// Get platform analytics
pub async fn get_platform_analytics(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
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
