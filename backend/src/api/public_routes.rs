use axum::{
    extract::{State, Json},
    routing::{post},
    Router,
};
use axum_client_ip::InsecureClientIp;
use reqwest::StatusCode;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;

use crate::{
    api::state::AppState,
    error::ServiceError,
    payment::models::CryptoType,
};
use crate::payment::models::CreatePaymentRequest;

#[derive(Debug, Deserialize)]
pub struct PublicPaymentRequest {
    pub publishable_key: String,
    #[serde(with = "rust_decimal::serde::str_option", default)]
    pub amount: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::str_option", default)]
    pub amount_usd: Option<Decimal>,
    pub crypto_type: Option<CryptoType>,
    pub description: Option<String>,
}

pub fn public_routes(state: AppState) -> Router {
    let api = Router::new()
        .route("/payments/create", post(create_public_payment))
        .with_state(state);
        
    Router::new().nest("/api/v1/public", api)
}

/// Create a new payment via Publishable Key (for pure no-code frontend widgets)
async fn create_public_payment(
    State(state): State<AppState>,
    InsecureClientIp(ip): InsecureClientIp,
    Json(payload): Json<PublicPaymentRequest>,
) -> Result<axum::Json<serde_json::Value>, ServiceError> {
    
    // 1. Authenticate using publishable key instead of standard API secret
    let merchant = state
        .merchant_service
        .authenticate_publishable_key(&payload.publishable_key)
        .await?;

    // Rejection Forwarding mode from using Standard payments (Security enforcement)
    if merchant.settlement_mode == "forwarding" {
        return Err(ServiceError::BadRequest("Standard payments are not available in Forwarding mode. Please use Address-Only payments.".to_string()));
    }

    // 2. Map PublicRequest to Internal Request
    let req = CreatePaymentRequest {
        amount: payload.amount,
        amount_usd: payload.amount_usd,
        crypto_type: payload.crypto_type,
        description: payload.description,
        webhook_url: None, // No webhooks for static pure-frontends logically (unless overridden globally on acc)
        metadata: Some(json!({"source": "no_code_widget"})),
        expires_in: None,
        expiration_minutes: None,
        partial_payments_enabled: None,
    };

    // 3. Leverage the robust internal payment creation engine
    let response = state.payment_service.create_payment(merchant.id, req).await?;

    // Log the widget access
    let _ = state.audit_service.log_event(
        merchant.id,
        "payment_creation_public_widget",
        Some(&format!("Created public widget payment request {}", response.payment_id)),
        Some(json!({
            "payment_id": response.payment_id,
            "ip_address": ip.to_string(),
            "origin": "public_widget",
            "publishable_key_type": if payload.publishable_key.starts_with("pub_live") { "live" } else { "sandbox" }
        }))
    ).await;

    // 4. Return widget-friendly data
    // We only need to return the paymentLink (payment_id)
    Ok(axum::Json(json!({
        "payment_id": response.payment_id,
        "payment_url": response.payment_url, // URL generated automatically inside `create_payment`
    })))
}
