// Withdrawal Handlers
// Withdrawal management endpoints

use crate::api::state::AppState;
use crate::middleware::auth::MerchantContext;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use serde_json::json;

// ============================================================================
// Withdrawal CRUD
// ============================================================================

pub async fn create_withdrawal(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<crate::services::withdrawal_service::WithdrawalRequest>,
) -> impl IntoResponse {
    match state.withdrawal_service.create_withdrawal(context.merchant_id, req, context.sandbox_mode).await {
        Ok(withdrawal) => {
            // Check settlement mode to see if we should auto-process
            if let Ok(Some(merchant)) = sqlx::query(
                "SELECT settlement_mode FROM merchants WHERE id = $1"
            )
            .bind(context.merchant_id)
            .fetch_optional(&state.db_pool)
            .await {
                use sqlx::Row;
                let sm: String = merchant.get("settlement_mode");
                if sm == "managed" {
                    // Spawn background task to process the withdrawal automatically
                    let processor = crate::services::withdrawal_processor::WithdrawalProcessor::new(
                        state.db_pool.clone(),
                        state.config.clone()
                    );
                    let withdrawal_id = withdrawal.withdrawal_id.clone();
                    
                    tokio::spawn(async move {
                        tracing::info!("Auto-processing managed withdrawal: {}", withdrawal_id);
                        if let Err(e) = processor.process_withdrawal(&withdrawal_id).await {
                            tracing::error!("Failed to auto-process withdrawal {}: {}", withdrawal_id, e);
                        }
                    });
                }
            }

            (StatusCode::CREATED, Json(withdrawal)).into_response()
        },
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn get_withdrawal(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(withdrawal_id): Path<String>,
) -> impl IntoResponse {
    match state.withdrawal_service.get_withdrawal(context.merchant_id, &withdrawal_id).await {
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
    
    match state.withdrawal_service.list_withdrawals(context.merchant_id, context.sandbox_mode).await {
        Ok(withdrawals) => (StatusCode::OK, Json(withdrawals)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn cancel_withdrawal(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(withdrawal_id): Path<String>,
) -> impl IntoResponse {
    match state.withdrawal_service.cancel_withdrawal(context.merchant_id, &withdrawal_id).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "Withdrawal cancelled"}))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    }
}
