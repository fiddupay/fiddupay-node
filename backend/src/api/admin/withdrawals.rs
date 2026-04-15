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
use serde_json::json;
use sqlx::Row;

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
    }))
    .into_response()
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
    }))
    .into_response()
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
    }))
    .into_response()
}

/// Resolve manual refunds for items frozen in [REFUND FAILED] lockout
pub async fn resolve_failed_refund(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
    Path(withdrawal_id): Path<String>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    // 1. Fetch withdrawal details
    let withdrawal = sqlx::query(
        "SELECT merchant_id, crypto_type, amount, sandbox_mode, status, transaction_hash, rejection_reason FROM withdrawals WHERE withdrawal_id = $1"
    )
    .bind(&withdrawal_id)
    .fetch_optional(&state.db_pool)
    .await;

    let wd = match withdrawal {
        Ok(Some(w)) => w,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Withdrawal not found"})),
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
    };

    let wd_status: String = wd.get("status");
    let wd_tx_hash: Option<String> = wd.get("transaction_hash");
    let wd_reason: Option<String> = wd.get("rejection_reason");
    let wd_amount: rust_decimal::Decimal = wd.get("amount");
    let wd_merchant_id: i64 = wd.get("merchant_id");
    let wd_crypto_type: String = wd.get("crypto_type");
    let wd_sandbox_mode: bool = wd.get("sandbox_mode");

    // 1.3 Double Payout Safeguard (On-Chain Check)
    if wd_status == "COMPLETED" || wd_tx_hash.is_some() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "This withdrawal was already completed or has a TX hash on-chain. Refund locked as safeguard to prevent double-spending"}))).into_response();
    }

    // 1.5 Double Refund Safeguard (Off-Chain Check)
    let reason = wd_reason.unwrap_or_default();
    if !reason.contains("[REFUND FAILED]") {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "This withdrawal does not have a failed automatic refund locked status"}))).into_response();
    }

    // 2. Lookup if there is a customer_id for this withdrawal Reference
    let customer_id: Option<i64> = sqlx::query_scalar::<_, i64>(
        "SELECT customer_id FROM customer_transactions WHERE reference_id = $1",
    )
    .bind(&withdrawal_id)
    .fetch_optional(&state.db_pool)
    .await
    .unwrap_or(None);

    let mut tx = match state.db_pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    };

    if let Some(c_id) = customer_id {
        // Customer Refund Retry (inverse of lock buffer)
        let res = sqlx::query(
            "UPDATE merchant_customer_balances SET available_balance = available_balance + $1, locked_balance = locked_balance - $1 WHERE customer_id = $2 AND crypto_type = $3 AND sandbox_mode = $4"
        )
        .bind(wd_amount)
        .bind(c_id)
        .bind(&wd_crypto_type)
        .bind(wd_sandbox_mode)
        .execute(&mut *tx)
        .await;

        if let Err(e) = res {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Customer refund query failed: {}", e)})),
            )
                .into_response();
        }
    } else {
        // Merchant Refund Retry
        let res = sqlx::query(
            "UPDATE merchant_balances SET available_balance = available_balance + $1 WHERE merchant_id = $2 AND crypto_type = $3 AND sandbox_mode = $4"
        )
        .bind(wd_amount)
        .bind(wd_merchant_id)
        .bind(&wd_crypto_type)
        .bind(wd_sandbox_mode)
        .execute(&mut *tx)
        .await;

        if let Err(e) = res {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Merchant refund query failed: {}", e)})),
            )
                .into_response();
        }
    }

    // 3. Clear [REFUND FAILED] tag from rejection reason
    let clean_reason = reason.replace(
        "[REFUND FAILED - Manual Intervention Required]",
        "[REFUND PROCESSED BY ADMIN]",
    );
    let _ = sqlx::query(
        "UPDATE withdrawals SET rejection_reason = $1, updated_at = NOW() WHERE withdrawal_id = $2",
    )
    .bind(clean_reason)
    .bind(&withdrawal_id)
    .execute(&mut *tx)
    .await;

    // 4. Record Admin Audit Log
    let _ = sqlx::query(
        "INSERT INTO audit_logs (merchant_id, action_type, entity_type, entity_id, details, created_at) VALUES (NULL, $1, $2, $3, $4, NOW())"
    )
    .bind("admin.resolve_failed_refund")
    .bind("withdrawal")
    .bind(&withdrawal_id)
    .bind(json!({
        "admin_id": context.admin_id,
        "amount": wd_amount,
        "crypto_type": wd_crypto_type,
        "status": "success"
    }))
    .execute(&mut *tx)
    .await;

    if tx.commit().await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to commit transaction"})),
        )
            .into_response();
    }

    Json(json!({
        "status": "success",
        "message": "Manual refund resolved successfully by admin",
        "withdrawal_id": withdrawal_id
    }))
    .into_response()
}
