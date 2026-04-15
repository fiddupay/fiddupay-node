use crate::api::state::AppState;
use crate::error::ServiceError;
use crate::middleware::auth::MerchantContext;
use crate::models::notification::NotificationListResponse;
use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct NotificationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_notifications(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(params): Query<NotificationQuery>,
) -> Result<Json<NotificationListResponse>, ServiceError> {
    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);

    let res = state
        .notification_service
        .list_notifications(context.merchant_id, limit, offset, context.sandbox_mode)
        .await?;

    Ok(Json(res))
}

pub async fn mark_notification_read(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    notif_id: Option<Path<Uuid>>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let id = notif_id.map(|Path(id)| id);
    let affected = state
        .notification_service
        .mark_as_read(context.merchant_id, id, context.sandbox_mode)
        .await?;

    Ok(Json(
        serde_json::json!({ "status": "success", "affected": affected }),
    ))
}

pub async fn delete_notifications(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    notif_id: Option<Path<Uuid>>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let id = notif_id.map(|Path(id)| id);
    let affected = state
        .notification_service
        .delete_notifications(context.merchant_id, id, context.sandbox_mode)
        .await?;

    Ok(Json(
        serde_json::json!({ "status": "success", "affected": affected }),
    ))
}
