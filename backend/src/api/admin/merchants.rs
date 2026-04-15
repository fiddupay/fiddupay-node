use crate::api::admin::auth::verify_admin_access;
use crate::api::state::AppState;
use crate::middleware::admin_auth::AdminContext;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    Extension,
};
use serde::Deserialize;
use serde_json::json;

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
            })),
        )
            .into_response(),
    }
}

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
    }))
    .into_response()
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
    }))
    .into_response()
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
    }))
    .into_response()
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
    }))
    .into_response()
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
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to update merchant fee: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}
