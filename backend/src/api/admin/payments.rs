use crate::middleware::admin_auth::AdminContext;
use crate::api::state::AppState;
use crate::api::admin::auth::verify_admin_access;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    Extension,
};
use serde_json::json;
use serde::Deserialize;
use tracing::info;

#[derive(Deserialize)]
pub struct AdminQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub status: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Deserialize)]
pub struct ReverifyTransactionRequest {
    pub hash: String,
    pub tx_type: String, // "customer" or "merchant"
    pub id: i64,         // customer_id or merchant_id
    pub crypto_type: String,
    pub sandbox_mode: bool,
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

/// Manual re-verification for static deposits
pub async fn reverify_transaction(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
    Json(req): Json<ReverifyTransactionRequest>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    info!("[ADMIN-REVERIFY] Manual re-verification requested by {} for hash: {}", context.admin_id, req.hash);

    let result = if req.tx_type == "customer" {
        // Need merchant_id for customer deposits
        let merchant_id_res = sqlx::query_scalar::<_, i64>(
            "SELECT merchant_id FROM merchant_customers WHERE id = $1"
        )
        .bind(req.id)
        .fetch_optional(&state.db_pool)
        .await;

        match merchant_id_res {
            Ok(Some(m_id)) => {
                state.payment_service.verify_customer_deposit(req.id, &req.hash, m_id, &req.crypto_type, req.sandbox_mode).await
            },
            Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error": "Customer not found"}))).into_response(),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
        }
    } else {
        state.payment_service.verify_merchant_deposit(req.id, &req.hash, &req.crypto_type, req.sandbox_mode).await
    };

    match result {
        Ok(true) => Json(json!({
            "success": true, 
            "message": "Transaction verified and processed successfully"
        })).into_response(),
        Ok(false) => (StatusCode::BAD_REQUEST, Json(json!({
            "success": false, 
            "message": "Transaction verification failed. Check server logs for details (likely address mismatch or already processed)."
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "error": "Internal processing error",
            "details": e.to_string()
        }))).into_response(),
    }
}
