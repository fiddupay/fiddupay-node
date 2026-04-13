use axum::{
    extract::{State, Query, Path},
    Json,
};
use crate::api::state::AppState;
use crate::error::ServiceError;
use crate::models::notification::NotificationListResponse;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct NotificationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_notifications(
    State(state): State<AppState>,
    merchant_id: crate::middleware::auth::MerchantId,
    Query(params): Query<NotificationQuery>,
) -> Result<Json<NotificationListResponse>, ServiceError> {
    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);
    
    let res = state.notification_service.list_notifications(
        merchant_id.0,
        limit,
        offset,
        merchant_id.1, // sandbox_mode from auth middleware
    ).await?;
    
    Ok(Json(res))
}

pub async fn mark_notification_read(
    State(state): State<AppState>,
    merchant_id: crate::middleware::auth::MerchantId,
    notif_id: Option<Path<Uuid>>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let id = notif_id.map(|Path(id)| id);
    let affected = state.notification_service.mark_as_read(
        merchant_id.0,
        id,
        merchant_id.1,
    ).await?;
    
    Ok(Json(serde_json::json!({ "status": "success", "affected": affected })))
}

pub async fn delete_notifications(
    State(state): State<AppState>,
    merchant_id: crate::middleware::auth::MerchantId,
    notif_id: Option<Path<Uuid>>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let id = notif_id.map(|Path(id)| id);
    let affected = state.notification_service.delete_notifications(
        merchant_id.0,
        id,
        merchant_id.1,
    ).await?;
    
    Ok(Json(serde_json::json!({ "status": "success", "affected": affected })))
}
