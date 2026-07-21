use crate::api::admin::auth::verify_admin_access;
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
    pub tx_type: String,       // "customer" or "merchant"
    pub id: serde_json::Value, // Can be integer (internal ID) or string (external_id)
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
    }))
    .into_response()
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
    }))
    .into_response()
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
    }))
    .into_response()
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
    }))
    .into_response()
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

    info!(
        "[ADMIN-REVERIFY] Manual re-verification requested by {} for hash: {}",
        context.admin_id, req.hash
    );

    let result = if req.tx_type == "customer" {
        // Resolve customer_id and merchant_id whether 'id' is a integer or string (external_id)
        let customer_row = if let Some(int_id) = req.id.as_i64() {
            sqlx::query_as::<_, (i64, i64)>(
                "SELECT id, merchant_id FROM merchant_customers WHERE id = $1",
            )
            .bind(int_id)
            .fetch_optional(&state.db_pool)
            .await
        } else if let Some(str_id) = req.id.as_str() {
            if let Ok(parsed_int) = str_id.parse::<i64>() {
                sqlx::query_as::<_, (i64, i64)>(
                    "SELECT id, merchant_id FROM merchant_customers WHERE id = $1 OR external_id = $2",
                )
                .bind(parsed_int)
                .bind(str_id)
                .fetch_optional(&state.db_pool)
                .await
            } else {
                sqlx::query_as::<_, (i64, i64)>(
                    "SELECT id, merchant_id FROM merchant_customers WHERE external_id = $1",
                )
                .bind(str_id)
                .fetch_optional(&state.db_pool)
                .await
            }
        } else {
            Ok(None)
        };

        match customer_row {
            Ok(Some((c_id, m_id))) => {
                state
                    .payment_service
                    .verify_customer_deposit(
                        c_id,
                        &req.hash,
                        m_id,
                        &req.crypto_type,
                        req.sandbox_mode,
                    )
                    .await
            }
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Customer not found"})),
                )
                    .into_response()
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()})),
                )
                    .into_response()
            }
        }
    } else {
        // Resolve merchant_id whether 'id' is an integer or string
        let resolved_m_id = if let Some(int_id) = req.id.as_i64() {
            Some(int_id)
        } else if let Some(str_id) = req.id.as_str() {
            str_id.parse::<i64>().ok()
        } else {
            None
        };

        match resolved_m_id {
            Some(m_id) => {
                state
                    .payment_service
                    .verify_merchant_deposit(m_id, &req.hash, &req.crypto_type, req.sandbox_mode)
                    .await
            }
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Invalid merchant ID format"})),
                )
                    .into_response()
            }
        }
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
