use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::api::state::AppState;
use crate::middleware::auth::MerchantContext;
use crate::models::p2p::{CreateAdRequest, CreateTradeRequest, CreateRatingRequest, CreateSupportTicketRequest};

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
    let sandbox_mode = match sqlx::query_scalar!("SELECT sandbox_mode FROM merchants WHERE id = $1", context.merchant_id)
        .fetch_one(&state.db_pool)
        .await {
            Ok(s) => s,
            Err(_) => false,
        };

    match service.get_balance(context.merchant_id, &crypto_type, sandbox_mode).await {
        Ok(balance) => (StatusCode::OK, Json(json!({"balance": balance}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

// Ads
pub async fn create_p2p_ad(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(payload): Json<CreateAdRequest>,
) -> impl IntoResponse {
    let service = state.p2p_service.clone();

    let sandbox_mode = match sqlx::query_scalar!("SELECT sandbox_mode FROM merchants WHERE id = $1", context.merchant_id)
        .fetch_one(&state.db_pool)
        .await {
            Ok(s) => s,
            Err(_) => false,
        };

    match service.create_ad(context.merchant_id, payload, sandbox_mode).await {
        Ok(ad) => (StatusCode::CREATED, Json(json!({"ad": ad}))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn list_p2p_ads(
    State(state): State<AppState>,
    Path((fiat_currency, crypto_type, ad_type)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let service = state.p2p_service.clone();
    
    match service.list_ads(&fiat_currency, &crypto_type, &ad_type, false).await {
         Ok(ads) => (StatusCode::OK, Json(json!({"ads": ads}))).into_response(),
         Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

// Trades & Escrow
pub async fn create_p2p_trade(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(payload): Json<CreateTradeRequest>,
) -> impl IntoResponse {
    let service = state.p2p_service.clone();

    let sandbox_mode = match sqlx::query_scalar!("SELECT sandbox_mode FROM merchants WHERE id = $1", context.merchant_id)
        .fetch_one(&state.db_pool)
        .await {
            Ok(s) => s,
            Err(_) => false,
        };

    match service.create_trade(context.merchant_id, payload, sandbox_mode).await {
         Ok(trade) => (StatusCode::CREATED, Json(json!({"trade": trade}))).into_response(),
         Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn release_p2p_trade(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(trade_id): Path<String>,
) -> impl IntoResponse {
    let service = state.p2p_service.clone();

    match service.release_trade(context.merchant_id, &trade_id).await {
         Ok(_) => (StatusCode::OK, Json(json!({"message": "Escrow released successfully"}))).into_response(),
         Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn submit_p2p_rating(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(trade_id): Path<String>,
    Json(payload): Json<CreateRatingRequest>,
) -> impl IntoResponse {
    let service = state.p2p_service.clone();

    match service.submit_rating(context.merchant_id, &trade_id, payload).await {
         Ok(rating) => (StatusCode::CREATED, Json(json!({"rating": rating}))).into_response(),
         Err(e) => {
            let status = if e.to_string().contains("already rated") { StatusCode::CONFLICT } else { StatusCode::BAD_REQUEST };
            (status, Json(json!({"error": e.to_string()}))).into_response()
         },
    }
}

pub async fn create_p2p_support_ticket(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(payload): Json<CreateSupportTicketRequest>,
) -> impl IntoResponse {
    let service = state.p2p_service.clone();

    match service.create_support_ticket(context.merchant_id, payload).await {
         Ok(ticket) => (StatusCode::CREATED, Json(json!({"ticket": ticket}))).into_response(),
         Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    }
}
