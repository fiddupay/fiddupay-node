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
    let api_key = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|auth| {
            if auth.starts_with("Bearer ") {
                Some(auth[7..].to_string())
            } else {
                None
            }
        });
    
    if api_key.is_none() {
        if let Some(auth) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
            tracing::warn!("Malformed Authorization header: {}", auth);
        } else {
            tracing::warn!("Missing Authorization header");
        }
    }
    api_key
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
        Some(key) => {
            key
        },
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
                };
                request.extensions_mut().insert(context);
                Ok(next.run(request).await)
            },
            Err(e) => {
                let prefix = if api_key.len() > 10 { &api_key[..10] } else { &api_key };
                tracing::warn!("API Key authentication failed: {} - {:?}", prefix, e);
                Err((
                    StatusCode::UNAUTHORIZED,
                    axum::Json(json!({
                        "error": "Invalid API key",
                        "message": "The provided API key is not valid"
                    }))
                ))
            }
        }
    } else {
        // JWT / Dashboard Session Flow
        use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
        
        let secret = &state.config.jwt_secret;
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        let validation = Validation::new(Algorithm::HS256);
        
        match decode::<DashboardClaims>(&api_key, &decoding_key, &validation) {
            Ok(token_data) => {
                let merchant_id = token_data.claims.sub.parse::<i64>().unwrap_or_default();
                
                // Fetch current sandbox_mode from DB
                // We use a lightweight query to get just the mode
                let sandbox_mode = match sqlx::query_scalar!(
                    "SELECT sandbox_mode FROM merchants WHERE id = $1",
                    merchant_id
                )
                .fetch_optional(&state.db_pool)
                .await {
                    Ok(Some(mode)) => mode,
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
                        tracing::error!("Failed to fetch merchant mode: {:?}", e);
                        // Fail safe to sandbox if DB error, or error out? 
                        // Error safe is better.
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
                };
                
                request.extensions_mut().insert(context);
                Ok(next.run(request).await)
            },
            Err(e) => {
                tracing::warn!("JWT validation failed: {:?}", e);
                Err((
                    StatusCode::UNAUTHORIZED,
                    axum::Json(json!({
                        "error": "Invalid Session",
                        "message": "Your session has expired or is invalid. Please login again."
                    }))
                ))
            }
        }
    }
}

/// Dashboard JWT Claims
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DashboardClaims {
    pub sub: String, // merchant_id,
    pub exp: usize,
    pub iat: usize,
}

/// Extract merchant context from request
/// 
/// Use this in handlers to get the authenticated merchant
pub fn get_merchant_context(request: &Request) -> Option<&MerchantContext> {
    request.extensions().get::<MerchantContext>()
}
