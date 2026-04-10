use axum::{
    extract::{State, Json},
    routing::{post},
    Router,
    response::IntoResponse,
    http::StatusCode,
};
use axum_client_ip::InsecureClientIp;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;

use crate::{
    api::state::AppState,
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

pub fn public_routes(state: AppState) -> Router<AppState> {
    let api = Router::new()
        .route("/payments/create", post(create_public_payment));
        
    Router::new().nest("/api/v1/public", api)
}

/// Create a new payment via Publishable Key (for pure no-code frontend widgets)
async fn create_public_payment(
    State(state): State<AppState>,
    InsecureClientIp(ip): InsecureClientIp,
    Json(payload): Json<PublicPaymentRequest>,
) -> impl IntoResponse {
    
    // 1. Authenticate using publishable key instead of standard API secret
    let merchant = match state
        .merchant_service
        .authenticate_publishable_key(&payload.publishable_key)
        .await {
            Ok(m) => m,
            Err(e) => return e.into_response(),
        };

    // Rejection Forwarding mode from using Standard payments (Security enforcement)
    if merchant.settlement_mode == "forwarding" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Standard payments are not available in Forwarding mode. Please use Address-Only payments.",
                "code": "SETTLEMENT_MODE_MISMATCH"
            }))
        ).into_response();
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
        is_invoice: false,
        customer_name: None,
        customer_email: None,
        items: None,
        tax: None,
        due_date: None,
        notes: None,
    };

    // 3. Leverage the robust internal payment creation engine
    match state.payment_service.create_payment(merchant.id, req).await {
        Ok(response) => {
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
            (StatusCode::CREATED, Json(json!({
                "payment_id": response.payment_id,
                "payment_url": response.payment_link, // URL generated automatically inside `create_payment`
            }))).into_response()
        },
        Err(e) => e.into_response(),
    }
}
