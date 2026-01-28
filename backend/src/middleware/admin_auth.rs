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

/// Admin context extracted from authentication
#[derive(Clone)]
pub struct AdminContext {
    pub admin_id: i64,
    pub username: String,
    pub permissions: Vec<String>,
}

/// Extract session token from Authorization header or Cookie
fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    // Try Authorization header first
    if let Some(auth) = headers.get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|auth| {
            if auth.starts_with("Bearer ") {
                Some(auth[7..].to_string())
            } else {
                None
            }
        }) {
        return Some(auth);
    }

    // Try session cookie
    headers.get("cookie")
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';')
                .find_map(|cookie| {
                    let cookie = cookie.trim();
                    if cookie.starts_with("admin_session=") {
                        Some(cookie[14..].to_string())
                    } else {
                        None
                    }
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
                }))
            ));
        }
    };

    // Validate session (simplified - in production use proper session store)
    if session_token == "admin_session_placeholder" {
        let context = AdminContext {
            admin_id: 1,
            username: "admin".to_string(),
            permissions: vec!["all".to_string()],
        };

        request.extensions_mut().insert(context);
        Ok(next.run(request).await)
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({
                "error": "Invalid session",
                "message": "Admin session expired or invalid"
            }))
        ))
    }
}
