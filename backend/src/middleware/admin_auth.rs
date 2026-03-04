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

    // Parse session token (format: admin_session_{id})
    let admin_id = if session_token.starts_with("admin_session_") {
        session_token[14..].parse::<i64>().ok()
    } else {
        None
    };

    if let Some(id) = admin_id {
        // Verify admin exists in separate admin_users table
        match sqlx::query(
            "SELECT id, username, role, is_active FROM admin_users WHERE id = $1"
        )
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
                        }))
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
        }))
    ))
}
