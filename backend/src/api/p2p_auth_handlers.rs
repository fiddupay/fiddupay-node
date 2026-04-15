// P2P Authentication Handlers
// Registration and login endpoints for P2P profiles

use crate::api::state::AppState;
use crate::error::ServiceError;
use crate::middleware::validation::validate_password_strength;
use crate::models::p2p::CreateProfileRequest;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;
use validator::Validate;

pub async fn register_p2p_user(
    State(state): State<AppState>,
    Json(req): Json<CreateProfileRequest>,
) -> impl IntoResponse {
    // 1. Validate input
    if let Err(e) = req.validate() {
        return ServiceError::ValidationError(e.to_string()).into_response();
    }

    // 2. Custom Password Strength check (Hardcoded policies)
    if let Err(e) = validate_password_strength(&req.password) {
        return ServiceError::ValidationError(e.to_string()).into_response();
    }

    match state.p2p_service.register_profile(&req).await {
        Ok(profile) => {
            // New P2P registrations also start in sandbox_mode = true
            (
                StatusCode::CREATED,
                Json(json!({
                    "message": "P2P Profile created successfully",
                    "profile": profile
                })),
            )
                .into_response()
        }
        Err(e) => e.into_response(),
    }
}
