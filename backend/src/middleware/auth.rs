// Authentication Middleware
// API key authentication

use crate::api::state::AppState;
use crate::models::merchant::UserRole;
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
    pub user_id: Option<i32>, // NULL for API keys, present for team members
    pub role: UserRole,
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
        .and_then(|auth| auth.strip_prefix("Bearer ").map(|s| s.to_string()))
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
) -> Result<Response, Response> {
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

    // Extract API key from header, fallback to protocol, query parameter, or HttpOnly cookie
    let api_key = match extract_api_key(&headers) {
        Some(key) => key,
        None => {
            // Check for HttpOnly cookie fallback (Fortress Layer)
            let cookie_token = headers
                .get("cookie")
                .and_then(|v| v.to_str().ok())
                .and_then(|c_str| {
                    c_str
                        .split(';')
                        .find(|s| s.trim().starts_with("dashboard_token="))
                        .map(|s| s.trim()["dashboard_token=".len()..].to_string())
                });

            match cookie_token {
                Some(token) => token,
                None => match protocol_token {
                    Some(token) => token.to_string(),
                    None => match query_token {
                        Some(token) => token,
                        None => {
                            tracing::warn!("Missing or invalid Authorization header/cookie and no WebSocket token provided");
                            return Err((
                                StatusCode::UNAUTHORIZED,
                                axum::Json(json!({
                                    "error": "Missing or invalid Authorization",
                                    "message": "Please login or provide a valid API key (Bearer or Secure Cookie)"
                                })),
                            ).into_response());
                        }
                    },
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
                    user_id: None,
                    role: merchant.role,
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
                )
                    .into_response())
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
                        ).into_response());
                    },
                    Err(e) => {
                        tracing::error!("Failed to verify merchant: {:?}", e);
                         return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            axum::Json(json!({
                                "error": "Internal Error",
                                "message": "Failed to validate session state"
                            }))
                        ).into_response());
                    }
                };

                let context = MerchantContext {
                    merchant_id,
                    user_id: token_data.claims.user_id,
                    role: token_data.claims.role,
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
                )
                    .into_response())
            }
        }
    }
}

/// Dashboard JWT Claims
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DashboardClaims {
    pub sub: String, // merchant_id
    pub user_id: Option<i32>,
    pub role: UserRole,
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

/// Helper to require a specific role
pub fn require_role(
    context: &MerchantContext,
    required_role: UserRole,
) -> Result<(), (StatusCode, axum::Json<serde_json::Value>)> {
    if context.role == required_role || context.role == UserRole::SuperAdmin {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            axum::Json(json!({
                "error": "Insufficient permissions",
                "message": format!("This action requires the {:?} role", required_role)
            })),
        ))
    }
}

/// Helper to require any of the specified roles
pub fn require_any_role(
    context: &MerchantContext,
    allowed_roles: &[UserRole],
) -> Result<(), (StatusCode, axum::Json<serde_json::Value>)> {
    if allowed_roles.contains(&context.role) || context.role == UserRole::SuperAdmin {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            axum::Json(json!({
                "error": "Insufficient permissions",
                "message": "You do not have the required role to perform this action"
            })),
        ))
    }
}
