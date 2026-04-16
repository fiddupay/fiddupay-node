use crate::error::ServiceError;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
// use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;

use crate::api::state::AppState;
use crate::middleware::auth::MerchantContext;
use crate::models::p2p::{
    CreateAdRequest, CreateRatingRequest, CreateSupportTicketRequest, CreateTradeRequest,
};

// Profile & Balances
pub async fn get_p2p_profile(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    let service = state.p2p_service.clone();

    match service.get_profile(context.merchant_id).await {
        Ok(profile) => (StatusCode::OK, Json(json!({"profile": profile}))).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn get_p2p_balance(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(crypto_type): Path<String>,
) -> impl IntoResponse {
    let service = state.p2p_service.clone();

    // Check sandbox mode
    let sandbox_mode =
        sqlx::query_scalar::<_, bool>("SELECT sandbox_mode FROM merchants WHERE id = $1")
            .bind(context.merchant_id)
            .fetch_one(&state.db_pool)
            .await
            .unwrap_or_default();

    match service
        .get_balance(context.merchant_id, &crypto_type, sandbox_mode)
        .await
    {
        Ok(balance) => (StatusCode::OK, Json(json!({"balance": balance}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// Ads
pub async fn create_p2p_ad(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(payload): Json<CreateAdRequest>,
) -> impl IntoResponse {
    // 0. Validate input
    if let Err(e) = payload.validate() {
        return ServiceError::ValidationError(e.to_string()).into_response();
    }

    let service = state.p2p_service.clone();

    match service
        .create_ad(context.merchant_id, payload, context.sandbox_mode)
        .await
    {
        Ok(ad) => (StatusCode::CREATED, Json(json!({"ad": ad}))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn list_p2p_ads(
    State(state): State<AppState>,
    Path((fiat_currency, crypto_type, ad_type)): Path<(String, String, String)>,
    Extension(context): Extension<crate::middleware::auth::MerchantContext>,
) -> impl IntoResponse {
    let service = state.p2p_service.clone();

    match service
        .list_ads(&fiat_currency, &crypto_type, &ad_type, context.sandbox_mode)
        .await
    {
        Ok(ads) => (StatusCode::OK, Json(json!({"ads": ads}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_p2p_ad(
    State(state): State<AppState>,
    Path(ad_id): Path<i64>,
) -> impl IntoResponse {
    let service = state.p2p_service.clone();

    match service.get_ad_by_id(ad_id).await {
        Ok(ad) => (StatusCode::OK, Json(json!({"ad": ad}))).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

// Trades
pub async fn create_p2p_trade(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(payload): Json<CreateTradeRequest>,
) -> impl IntoResponse {
    if let Err(e) = payload.validate() {
        return ServiceError::ValidationError(e.to_string()).into_response();
    }

    let service = state.p2p_service.clone();

    match service.create_trade(context.merchant_id, payload, context.sandbox_mode).await {
        Ok(trade) => (StatusCode::CREATED, Json(json!({"trade": trade}))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_p2p_trade(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(trade_id): Path<i64>,
) -> impl IntoResponse {
    let service = state.p2p_service.clone();

    match service.get_trade_by_id(trade_id).await {
        Ok(trade) => {
            // Verify ownership
            if trade.buyer_id != context.merchant_id && trade.seller_id != context.merchant_id {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({"error": "You do not have access to this trade"})),
                )
                    .into_response();
            }
            (StatusCode::OK, Json(json!({"trade": trade}))).into_response()
        }
        Err(e) => (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

// Ratings & Support
pub async fn rate_p2p_user(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(payload): Json<CreateRatingRequest>,
) -> impl IntoResponse {
    if let Err(e) = payload.validate() {
        return ServiceError::ValidationError(e.to_string()).into_response();
    }

    let service = state.p2p_service.clone();

    match service.rate_user(context.merchant_id, payload).await {
        Ok(rating) => (StatusCode::CREATED, Json(json!({"rating": rating}))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create_p2p_dispute(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(trade_id): Path<i64>,
    Json(payload): Json<CreateSupportTicketRequest>,
) -> impl IntoResponse {
    if let Err(e) = payload.validate() {
        return ServiceError::ValidationError(e.to_string()).into_response();
    }

    let service = state.p2p_service.clone();

    match service
        .create_dispute(context.merchant_id, trade_id, payload)
        .await
    {
        Ok(ticket) => (StatusCode::CREATED, Json(json!({"dispute": ticket}))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
