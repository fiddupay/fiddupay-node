use crate::api::admin::auth::verify_admin_access;
use crate::api::state::AppState;
use crate::middleware::admin_auth::AdminContext;
use crate::payment::models::CryptoType;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    Extension,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct RectifyOnchainRequest {
    pub address: String,
    pub crypto_type: String,
    pub dry_run: Option<bool>,
    pub signature_limit: Option<usize>,
    pub sandbox_mode: Option<bool>,
    pub rectify_type: Option<String>, // "DEPOSIT", "WITHDRAWAL", or "BOTH"
}

/// Rectify blockchain balance by scanning on-chain history
pub async fn rectify_onchain_balance(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
    Json(req): Json<RectifyOnchainRequest>,
) -> impl IntoResponse {
    // 1. Verify admin access
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    // 2. Parse crypto type
    let crypto_type = match CryptoType::from_string(&req.crypto_type) {
        Ok(ct) => ct,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": "Invalid crypto type" })),
            )
                .into_response()
        }
    };

    // 3. Call Service Logic
    let dry_run = req.dry_run.unwrap_or(true);
    let signature_limit = req.signature_limit.unwrap_or(50);
    let rectify_type = req
        .rectify_type
        .clone()
        .unwrap_or_else(|| "DEPOSIT".to_string())
        .to_uppercase();

    match state
        .balance_service
        .rectify_onchain(
            &req.address,
            crypto_type,
            dry_run,
            signature_limit,
            req.sandbox_mode,
            &rectify_type,
        )
        .await
    {
        Ok(report) => (
            StatusCode::OK,
            Json(json!({ "success": true, "data": report })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("[ADMIN-RECTIFY] Blockchain rectification failed: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": e.to_string() })),
            )
                .into_response()
        }
    }
}
