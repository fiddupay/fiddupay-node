// API Module

use crate::api::state::AppState;
use crate::error::ServiceError;
use crate::middleware::auth::{require_kyc_tier, MerchantContext};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct P2PTransferRequest {
    #[validate(length(min = 3))]
    pub recipient_identifier: String,
    pub crypto_type: String,
    pub amount: Decimal,
    #[validate(length(equal = 4))]
    pub pin: String,
}

pub async fn transfer_funds_interop(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(payload): Json<P2PTransferRequest>,
) -> impl IntoResponse {
    // 0. Requirement: Must be Tier 1 for LIVE P2P Interop
    if !context.sandbox_mode {
        if let Err(e) = require_kyc_tier(&context, 1) {
            return e.into_response();
        }
    }
    // 1. Validate payload
    if let Err(e) = payload.validate() {
        return ServiceError::ValidationError(e.to_string()).into_response();
    }

    // 2. Verify Transaction PIN
    if let Err(e) = state
        .merchant_service
        .verify_transaction_pin(context.merchant_id, &payload.pin)
        .await
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": format!("Invalid PIN: {}", e)})),
        )
            .into_response();
    }

    // 3. Resolve recipient identifier to merchant_id
    let recipient_profile = match state
        .pay_service
        .resolve_merchant(&payload.recipient_identifier)
        .await
    {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };

    // 4. Check for self-payment
    if recipient_profile.merchant_id == context.merchant_id {
        return ServiceError::ValidationError("Cannot send funds to yourself".into())
            .into_response();
    }

    // 5. Execute atomic transfer
    match state
        .pay_service
        .execute_transfer(
            context.merchant_id,
            recipient_profile.merchant_id,
            &payload.crypto_type,
            payload.amount,
            context.sandbox_mode,
        )
        .await
    {
        Ok(tx_id) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "transaction_id": tx_id,
                "recipient": {
                    "business_name": recipient_profile.business_name,
                    "username": recipient_profile.username,
                    "pay_id": recipient_profile.pay_id
                }
            })),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
}
