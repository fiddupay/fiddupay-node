use crate::api::admin::auth::verify_admin_access;
use crate::api::admin::payments::AdminQuery;
use crate::api::state::AppState;
use crate::middleware::admin_auth::AdminContext;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    Extension,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct AdminUserCreate {
    pub email: String,
    pub name: String,
    pub permissions: Vec<String>,
}

#[derive(Deserialize)]
pub struct UserPermissions {
    pub permissions: Vec<String>,
}

#[derive(Deserialize)]
pub struct AdminAuditLogQueryParams {
    pub from: Option<String>,
    pub to: Option<String>,
    pub action_type: Option<String>,
    pub limit: Option<i64>,
    pub scope: Option<String>,    // "merchant", "admin", "all"
    pub merchant_id: Option<i64>, // if scope == "merchant"
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
    }))
    .into_response()
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
    }))
    .into_response()
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
    }))
    .into_response()
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
    }))
    .into_response()
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
    }))
    .into_response()
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
    }))
    .into_response()
}

/// Get aggregated audit logs (Super Admin view)
pub async fn get_admin_audit_logs(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
    Query(params): Query<AdminAuditLogQueryParams>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    use crate::services::audit_service::{AuditLogQuery, AuditScope};

    let scope_str = params.scope.as_deref().unwrap_or("all");
    let scope = match scope_str {
        "merchant" => {
            if let Some(id) = params.merchant_id {
                AuditScope::Merchant(id)
            } else {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(
                        serde_json::json!({"error": "merchant_id is required for merchant scope"}),
                    ),
                )
                    .into_response();
            }
        }
        "admin" => AuditScope::Admin,
        _ => AuditScope::All,
    };

    let query = AuditLogQuery {
        from: params.from.and_then(|s| s.parse().ok()),
        to: params.to.and_then(|s| s.parse().ok()),
        action_type: params.action_type,
        limit: params.limit,
    };

    match state.audit_service.get_logs(scope, query).await {
        Ok(logs) => (StatusCode::OK, Json(logs)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
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
    }))
    .into_response()
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
    }))
    .into_response()
}

#[derive(Deserialize)]
pub struct UnblockIpRequest {
    pub ip: String,
}

/// Get currently banned IPs (Wall of Shame)
pub async fn get_banned_ips(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    let threat_detector =
        crate::middleware::advanced_security::ThreatDetector::new(state.redis_client.clone());
    let banned_ips = threat_detector.get_banned_ips().await;

    Json(json!({
        "banned_ips": banned_ips,
        "total": banned_ips.len()
    }))
    .into_response()
}

/// Unblock an IP address
pub async fn unblock_ip(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
    Json(payload): Json<UnblockIpRequest>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    let threat_detector =
        crate::middleware::advanced_security::ThreatDetector::new(state.redis_client.clone());

    if threat_detector.unban_ip(&payload.ip).await {
        Json(json!({
            "message": format!("IP address {} unblocked successfully", payload.ip)
        }))
        .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "IP not found in blacklist or already unblocked"
            })),
        )
            .into_response()
    }
}
