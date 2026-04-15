// Withdrawal Handlers
// Withdrawal management endpoints

use crate::api::state::AppState;
use crate::error::ServiceError;
use crate::middleware::auth::{require_any_role, MerchantContext};
use crate::models::merchant::UserRole;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use serde_json::json;
use validator::Validate;

// ============================================================================
// Withdrawal CRUD
// ============================================================================

pub async fn create_withdrawal(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<crate::services::withdrawal_service::WithdrawalRequest>,
) -> impl IntoResponse {
    // -1. Authorization Check (Strict: Only the Merchant Owner can create withdrawals)
    if let Err(e) = require_role(&context, UserRole::Merchant) {
        return e.into_response();
    }

    // 0. Validate input
    if let Err(e) = req.validate() {
        return ServiceError::ValidationError(e.to_string()).into_response();
    }

    // 1. Verify Transaction PIN (Merchant)
    if let Err(e) = state
        .merchant_service
        .verify_transaction_pin(context.merchant_id, &req.pin)
        .await
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response();
    }

    // 1. Enforce settlement mode (Requirement: Managed mode only for manual withdrawals)
    if context.settlement_mode != "managed" {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Manual withdrawals are only available in Managed settlement mode"}))
        ).into_response();
    }

    match state
        .withdrawal_service
        .create_withdrawal(context.merchant_id, req, context.sandbox_mode)
        .await
    {
        Ok(withdrawal) => {
            // Check settlement mode to see if we should auto-process
            if let Ok(Some(merchant)) =
                sqlx::query("SELECT settlement_mode FROM merchants WHERE id = $1")
                    .bind(context.merchant_id)
                    .fetch_optional(&state.db_pool)
                    .await
            {
                use sqlx::Row;
                let sm: String = merchant.get("settlement_mode");
                if sm == "managed" {
                    // Spawn background task to process the withdrawal automatically
                    let processor = crate::services::withdrawal_processor::WithdrawalProcessor::new(
                        state.db_pool.clone(),
                        state.config.clone(),
                        state.notification_service.clone(),
                    );
                    let withdrawal_id = withdrawal.withdrawal_id.clone();

                    tokio::spawn(async move {
                        tracing::info!("Auto-processing managed withdrawal: {}", withdrawal_id);
                        if let Err(e) = processor.process_withdrawal(&withdrawal_id).await {
                            tracing::error!(
                                "Failed to auto-process withdrawal {}: {}",
                                withdrawal_id,
                                e
                            );
                        }
                    });
                }
            }

            // Log withdrawal creation and trace
            let _ = state
                .audit_service
                .log_event(
                    context.merchant_id,
                    "withdrawal_creation",
                    Some(&format!(
                        "Created withdrawal request for {}{}",
                        withdrawal.amount, withdrawal.crypto_type
                    )),
                    Some(json!({
                        "withdrawal_id": withdrawal.withdrawal_id,
                        "currency": withdrawal.crypto_type,
                        "amount": withdrawal.amount,
                        "destination": withdrawal.destination_address
                    })),
                )
                .await;
            tracing::info!(
                "EVENT: withdrawal_creation | Merchant: {} | Withdrawal: {} | Amount: {} {}",
                context.merchant_id,
                withdrawal.withdrawal_id,
                withdrawal.amount,
                withdrawal.crypto_type
            );

            (StatusCode::CREATED, Json(withdrawal)).into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_withdrawal(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(withdrawal_id): Path<String>,
) -> impl IntoResponse {
    match state
        .withdrawal_service
        .get_withdrawal(context.merchant_id, &withdrawal_id)
        .await
    {
        Ok(withdrawal) => (StatusCode::OK, Json(withdrawal)).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct WithdrawalListQuery {
    pub limit: Option<i64>,
}

pub async fn list_withdrawals(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(params): Query<WithdrawalListQuery>,
) -> impl IntoResponse {
    let _limit = params.limit.unwrap_or(100).min(1000);

    match state
        .withdrawal_service
        .list_withdrawals(context.merchant_id, context.sandbox_mode)
        .await
    {
        Ok(withdrawals) => (StatusCode::OK, Json(withdrawals)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn cancel_withdrawal(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(withdrawal_id): Path<String>,
) -> impl IntoResponse {
    // -1. Authorization Check (Strict: Only the Merchant Owner can cancel withdrawals)
    if let Err(e) = require_role(&context, UserRole::Merchant) {
        return e.into_response();
    }

    match state
        .withdrawal_service
        .cancel_withdrawal(context.merchant_id, &withdrawal_id)
        .await
    {
        Ok(_) => {
            // Log withdrawal cancellation and trace
            let _ = state
                .audit_service
                .log_event(
                    context.merchant_id,
                    "withdrawal_cancellation",
                    Some(&format!("Cancelled withdrawal {}", withdrawal_id)),
                    Some(json!({"withdrawal_id": withdrawal_id})),
                )
                .await;
            tracing::info!(
                "EVENT: withdrawal_cancellation | Merchant: {} | Withdrawal: {}",
                context.merchant_id,
                withdrawal_id
            );

            (
                StatusCode::OK,
                Json(json!({"message": "Withdrawal cancelled"})),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
