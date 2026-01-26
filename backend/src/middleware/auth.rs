// Authentication Middleware
// API key authentication

use crate::api::state::AppState;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// Merchant context extracted from authentication
#[derive(Clone)]
pub struct MerchantContext {
    pub merchant_id: i64,
    pub api_key: String,
    pub sandbox_mode: bool,
}

/// Extract API key from Authorization header
/// 
/// Expected format: "Bearer <api_key>"
fn extract_api_key(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|auth| {
            if auth.starts_with("Bearer ") {
                Some(auth[7..].to_string())
            } else {
                None
            }
        })
}

/// Authentication middleware
/// 
/// Validates API key and attaches merchant context to request
/// 
/// # Requirements
/// * 7.1: Authenticate requests with valid API key
/// * 7.2: Reject requests with invalid or missing API key (401)
pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    // Extract API key from header
    let api_key = match extract_api_key(&headers) {
        Some(key) => key,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                axum::Json(json!({
                    "error": "Missing or invalid Authorization header",
                    "message": "Expected format: Authorization: Bearer <api_key>"
                }))
            ));
        }
    };

    // Authenticate with merchant service
    match state.merchant_service.authenticate(&api_key).await {
        Ok(merchant) => {
            // Create merchant context
            let context = MerchantContext {
                merchant_id: merchant.id,
                api_key: api_key.clone(),
                sandbox_mode: merchant.sandbox_mode,
            };

            // Attach context to request extensions
            request.extensions_mut().insert(context);

            // Continue to next middleware/handler
            Ok(next.run(request).await)
        }
        Err(_) => {
            Err((
                StatusCode::UNAUTHORIZED,
                axum::Json(json!({
                    "error": "Invalid API key",
                    "message": "The provided API key is not valid"
                }))
            ))
        }
    }
}

/// Extract merchant context from request
/// 
/// Use this in handlers to get the authenticated merchant
pub fn get_merchant_context(request: &Request) -> Option<&MerchantContext> {
    request.extensions().get::<MerchantContext>()
}
