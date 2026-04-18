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
pub struct RectifySolanaRequest {
    pub address: String,
    pub crypto_type: String, // SOL or USDT_SPL
    pub dry_run: Option<bool>,
    pub signature_limit: Option<usize>,
    pub sandbox_mode: Option<bool>, // Allows super_admin to explicitly target Devnet or Mainnet overriding DB Defaults
}

/// Rectify Solana balance by scanning on-chain history
pub async fn rectify_solana_balance(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
    Json(req): Json<RectifySolanaRequest>,
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

    // 3. Ensure it's a Solana-based crypto
    if crypto_type != CryptoType::Sol && crypto_type != CryptoType::UsdtSpl {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "Only SOL and USDT_SPL are supported for this rectification" })),
        )
            .into_response();
    }

    // 4. Call Service Logic
    let dry_run = req.dry_run.unwrap_or(true);
    let signature_limit = req.signature_limit.unwrap_or(50);

    match state
        .balance_service
        .rectify_solana_onchain(
            &req.address,
            crypto_type,
            dry_run,
            signature_limit,
            req.sandbox_mode,
        )
        .await
    {
        Ok(report) => (
            StatusCode::OK,
            Json(json!({ "success": true, "data": report })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("[ADMIN-RECTIFY] Solana clarification failed: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": e.to_string() })),
            )
                .into_response()
        }
    }
}
