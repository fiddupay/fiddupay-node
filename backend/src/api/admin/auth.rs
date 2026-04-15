use crate::api::state::AppState;
use crate::middleware::admin_auth::AdminContext;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::Row;

#[derive(Deserialize)]
pub struct AdminLoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn admin_login(
    State(state): State<AppState>,
    Json(login_data): Json<AdminLoginRequest>,
) -> impl IntoResponse {
    // Authenticate against admin_users table
    let admin_user_res = sqlx::query(
        "SELECT id, username, password_hash, role, is_active FROM admin_users WHERE username = $1",
    )
    .bind(&login_data.username)
    .fetch_optional(&state.db_pool)
    .await;

    let admin_user = match admin_user_res {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid credentials"})),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    };

    let admin_is_active: bool = admin_user.get("is_active");
    if !admin_is_active {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Account deactivated"})),
        )
            .into_response();
    }

    // Verify Password
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    let admin_password_hash: String = admin_user.get("password_hash");
    let parsed_hash = match PasswordHash::new(&admin_password_hash) {
        Ok(hash) => hash,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Invalid stored hash"})),
            )
                .into_response()
        }
    };

    if Argon2::default()
        .verify_password(login_data.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid credentials"})),
        )
            .into_response();
    }

    let admin_db_id: i32 = admin_user.get("id");
    let admin_username: String = admin_user.get("username");
    let admin_role: Option<String> = admin_user.try_get("role").ok();

    // Generate JWT Token
    use crate::middleware::admin_auth::AdminClaims;
    use jsonwebtoken::{encode, EncodingKey, Header};

    let secret = &state.config.jwt_secret;
    let exp = chrono::Utc::now() + chrono::Duration::hours(24);

    let claims = AdminClaims {
        sub: admin_db_id.to_string(),
        exp: exp.timestamp() as usize,
        iat: chrono::Utc::now().timestamp() as usize,
        role: admin_role.clone(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap_or_else(|_| "".to_string());

    Json(json!({
        "success": true,
        "session_token": token,
        "user": {
            "id": admin_db_id,
            "username": admin_username,
            "role": admin_role,
            "permissions": ["all"]
        }
    }))
    .into_response()
}

pub async fn admin_logout() -> impl IntoResponse {
    Json(json!({
        "success": true,
        "message": "Logged out successfully"
    }))
}

/// Admin middleware to verify admin access
pub async fn verify_admin_access(
    state: &AppState,
    context: &AdminContext,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    match state
        .admin_service
        .verify_admin_access(context.admin_id)
        .await
    {
        Ok(true) => Ok(()),
        Ok(false) => Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "Admin access required",
                "message": "This endpoint requires admin privileges"
            })),
        )),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Authorization check failed",
                "message": "Failed to verify admin privileges"
            })),
        )),
    }
}
