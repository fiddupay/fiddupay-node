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
    pub settlement_mode: String,
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
                if !auth.is_empty() {
                    tracing::warn!("Malformed Authorization header format");
                }
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
    uri: axum::http::Uri,
    mut request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    // Extract token from Sec-WebSocket-Protocol header (more secure for WebSockets)
    let protocol_token = headers
        .get("sec-websocket-protocol")
        .and_then(|v| v.to_str().ok());

    // Extract query token for legacy WebSocket authenticity
    let query_token = uri.query().and_then(|q| {
        q.split('&')
            .find(|s| s.starts_with("token="))
            .map(|s| s[6..].to_string())
    });

    // Extract API key from header, fallback to protocol or query parameter
    let api_key = match extract_api_key(&headers) {
        Some(key) => key,
        None => {
            match protocol_token {
                Some(token) => token.to_string(),
                None => match query_token {
                    Some(token) => token,
                    None => {
                        tracing::warn!("Missing or invalid Authorization header and no WebSocket token provided");
                        return Err((
                            StatusCode::UNAUTHORIZED,
                            axum::Json(json!({
                                "error": "Missing or invalid Authorization header",
                                "message": "Expected format: Authorization: Bearer <api_key> or URL Sec-WebSocket-Protocol header"
                            })),
                        ));
                    }
                },
            }
        }
    };

    // Check if it's an API Key (starts with sk_) or likely a JWT
    // This middleware protects RESOURCES (e.g. Profile, Payments).
    // It is NOT used for Login/Register.
    if api_key.starts_with("sk_") {
        // Merchant API Integration Flow (server-to-server)
        match state.merchant_service.authenticate(&api_key).await {
            Ok(merchant) => {
                let is_live_prefix = api_key.starts_with("sk_live_");
                let context = MerchantContext {
                    merchant_id: merchant.id,
                    api_key: api_key.clone(),
                    sandbox_mode: !is_live_prefix,
                    settlement_mode: merchant.settlement_mode.clone(),
                };
                request.extensions_mut().insert(context);
                Ok(next.run(request).await)
            }
            Err(e) => {
                let prefix = if api_key.len() > 10 {
                    &api_key[..10]
                } else {
                    &api_key
                };
                tracing::warn!("API Key authentication failed: {} - {:?}", prefix, e);
                Err((
                    StatusCode::UNAUTHORIZED,
                    axum::Json(json!({
                        "error": "Invalid API key",
                        "message": "The provided API key is not valid"
                    })),
                ))
            }
        }
    } else {
        // JWT / Dashboard Session Flow
        use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

        let secret = &state.config.jwt_secret;
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        let validation = Validation::new(Algorithm::HS256);

        match decode::<DashboardClaims>(&api_key, &decoding_key, &validation) {
            Ok(token_data) => {
                let merchant_id = token_data.claims.sub.parse::<i64>().unwrap_or_default();

                // Read sandbox_mode from DB to ensure it's always current with the merchant's choice
                // This ensures environment switching in the dashboard is instant.
                let (sandbox_mode, settlement_mode) = match sqlx::query(
                    "SELECT sandbox_mode, settlement_mode FROM merchants WHERE id = $1 AND is_active = true"
                )
                .bind(merchant_id)
                .fetch_optional(&state.db_pool)
                .await {
                    Ok(Some(row)) => {
                        use sqlx::Row;
                        (row.get::<bool, _>("sandbox_mode"), row.get::<String, _>("settlement_mode"))
                    },
                    Ok(None) => {
                        return Err((
                            StatusCode::UNAUTHORIZED,
                            axum::Json(json!({
                                "error": "Invalid Merchant",
                                "message": "Merchant account not found"
                            }))
                        ));
                    },
                    Err(e) => {
                        tracing::error!("Failed to verify merchant: {:?}", e);
                         return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            axum::Json(json!({
                                "error": "Internal Error",
                                "message": "Failed to validate session state"
                            }))
                        ));
                    }
                };

                let context = MerchantContext {
                    merchant_id,
                    api_key: "DASHBOARD_SESSION".to_string(),
                    sandbox_mode,
                    settlement_mode,
                };

                request.extensions_mut().insert(context);
                Ok(next.run(request).await)
            }
            Err(e) => {
                tracing::warn!("JWT validation failed: {:?}", e);
                Err((
                    StatusCode::UNAUTHORIZED,
                    axum::Json(json!({
                        "error": "Invalid Session",
                        "message": "Your session has expired or is invalid. Please login again."
                    })),
                ))
            }
        }
    }
}

/// Dashboard JWT Claims
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DashboardClaims {
    pub sub: String, // merchant_id
    pub exp: usize,
    pub iat: usize,
    #[serde(default)]
    pub sandbox_mode: bool,
}

/// Extract merchant context from request
///
/// Use this in handlers to get the authenticated merchant
pub fn get_merchant_context(request: &Request) -> Option<&MerchantContext> {
    request.extensions().get::<MerchantContext>()
}
