// Admin Authentication Middleware
// Session-based authentication for admin users

use crate::api::state::AppState;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::Row;

/// Admin context extracted from authentication
#[derive(Clone)]
pub struct AdminContext {
    pub admin_id: i64,
    pub username: String,
    pub permissions: Vec<String>,
}

/// Admin JWT Claims
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AdminClaims {
    pub sub: String, // admin_id
    pub exp: usize,
    pub iat: usize,
    pub role: Option<String>,
}

/// Extract session token from Authorization header or Cookie
fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    // Try Authorization header first
    if let Some(auth) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer ").map(|s| s.to_string()))
    {
        return Some(auth);
    }

    // Try session cookie
    headers
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                cookie
                    .trim()
                    .strip_prefix("admin_session=")
                    .map(|s| s.to_string())
            })
        })
}

/// Admin authentication middleware
pub async fn admin_auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    // Extract session token
    let session_token = match extract_session_token(&headers) {
        Some(token) => token,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                axum::Json(json!({
                    "error": "Missing authentication",
                    "message": "Admin session required"
                })),
            ));
        }
    };

    // Verify JWT Session Token
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

    let secret = &state.config.jwt_secret;
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);
    // Admins might not strictly need the default duration if not specified

    let admin_id = match decode::<AdminClaims>(&session_token, &decoding_key, &validation) {
        Ok(token_data) => token_data.claims.sub.parse::<i64>().ok(),
        Err(e) => {
            tracing::warn!("Admin JWT validation failed: {:?}", e);
            None
        }
    };

    if let Some(id) = admin_id {
        // Verify admin exists in separate admin_users table
        match sqlx::query("SELECT id, username, role, is_active FROM admin_users WHERE id = $1")
            .bind(id as i32)
            .fetch_optional(&state.db_pool)
            .await
        {
            Ok(Some(admin)) => {
                let is_active: bool = admin.get("is_active");
                if !is_active {
                    return Err((
                        StatusCode::FORBIDDEN,
                        axum::Json(json!({
                            "error": "Account deactivated",
                            "message": "Admin account is not active"
                        })),
                    ));
                }

                let admin_db_id: i32 = admin.get("id");
                let admin_username: String = admin.get("username");
                let context = AdminContext {
                    admin_id: admin_db_id as i64,
                    username: admin_username,
                    permissions: vec!["all".to_string()],
                };

                request.extensions_mut().insert(context);
                return Ok(next.run(request).await);
            }
            Ok(None) => {
                // Invalid session (admin not found)
            }
            Err(_) => {
                // Database error
            }
        }
    }

    Err((
        StatusCode::UNAUTHORIZED,
        axum::Json(json!({
            "error": "Invalid session",
            "message": "Admin session expired or invalid"
        })),
    ))
}
