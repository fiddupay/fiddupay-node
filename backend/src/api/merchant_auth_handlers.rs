// Merchant Authentication Handlers
// Registration and login endpoints

use crate::api::state::AppState;
use crate::error::ServiceError;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;
use rust_decimal::Decimal;

use crate::middleware::validation::{validate_business_email, validate_password_strength};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Deserialize, Validate)]
pub struct RegisterMerchantRequest {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 1, max = 100))]
    pub business_name: String,
    
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Deserialize, Validate)]
pub struct LoginMerchantRequest {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 1))]
    pub password: String,
    
    #[validate(length(equal = 6))]
    pub two_factor_code: Option<String>,

    pub remember_me: Option<bool>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub user: MerchantProfile,
    pub dashboard_token: String,
}

#[derive(Serialize)]
pub struct MerchantProfile {
    pub id: i64,
    pub business_name: String,
    pub email: String,
    pub api_key: String,
    pub created_at: String,
    pub two_factor_enabled: bool,
    pub daily_limit_usd: Option<String>,
    pub daily_volume_remaining: String,
    pub kyc_verified: bool,
    pub sandbox_mode: bool,
    pub settlement_mode: String,
}

// ============================================================================
// Handlers
// ============================================================================

pub async fn register_merchant(
    State(state): State<AppState>,
    Json(req): Json<RegisterMerchantRequest>,
) -> impl IntoResponse {
    // 1. Check if registration is enabled
    if !state.config.merchant_registration_enabled {
        return ServiceError::Forbidden("Registration is currently disabled".to_string()).into_response();
    }

    match state.merchant_service.register_merchant(&req.email, &req.business_name, &req.password).await {
        Ok(response) => {
            // Generate JWT for new registration
            let now = chrono::Utc::now();
            let exp = (now + chrono::Duration::hours(24)).timestamp() as usize;
            
            use jsonwebtoken::{encode, Header, EncodingKey};
            use crate::middleware::auth::DashboardClaims;
            
            let claims = DashboardClaims {
                sub: response.merchant_id.to_string(),
                exp,
                iat: now.timestamp() as usize,
                sandbox_mode: true, // New registrations start in sandbox
            };
            
            let secret = &state.config.jwt_secret;
            let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
                .unwrap_or_default();
                
            let auth_response = AuthResponse {
                user: MerchantProfile {
                    id: response.merchant_id,
                    business_name: req.business_name,
                    email: req.email,
                    api_key: response.api_key, // Return the REAL key once on registration
                    created_at: chrono::Utc::now().to_rfc3339(),
                    two_factor_enabled: false,
                    daily_limit_usd: None,
                    daily_volume_remaining: state.config.daily_volume_limit_non_kyc_usd.to_string(),
                    kyc_verified: false,
                    sandbox_mode: true,
                    settlement_mode: "managed".to_string(),
                },
                dashboard_token: token,
            };
            
            (StatusCode::CREATED, Json(auth_response)).into_response()
        },
        Err(e) => e.into_response(),
    }
}

pub async fn login_merchant(
    State(state): State<AppState>,
    Json(req): Json<LoginMerchantRequest>,
) -> impl IntoResponse {
    // Query the database for the user
    let merchant_query: Result<_, sqlx::Error> = sqlx::query!(
        "SELECT id, business_name, email, sandbox_mode, settlement_mode, kyc_verified, created_at, role::text as role, live_api_key_hash, test_api_key_hash, password_hash, daily_limit_usd FROM merchants WHERE email = $1 AND is_active = true",
        req.email
    )
    .fetch_optional(&state.db_pool)
    .await;

    match merchant_query
    {
        Ok(Some(merchant)) => {
            // VERIFY PASSWORD
            use argon2::{Argon2, PasswordHash, PasswordVerifier};
            
            // Check if password_hash exists (it might be NULL for old users or API-only users)
            let hash_to_check = merchant.password_hash.as_ref().ok_or_else(|| {
                // If no password hash, user cannot login via password (API key only)
                 ServiceError::Unauthorized("Password login not available for this account".to_string())
            });

            if let Err(_) = hash_to_check {
                return (StatusCode::UNAUTHORIZED, Json(json!({
                    "error": "Invalid credentials",
                    "message": "Invalid email or password"
                }))).into_response();
            }

            let parsed_hash = PasswordHash::new(hash_to_check.unwrap())
                .map_err(|e| ServiceError::InternalError(format!("Invalid hash structure: {}", e)))
                .unwrap(); 

            let valid = Argon2::default().verify_password(req.password.as_bytes(), &parsed_hash).is_ok();

            if !valid {
                return (StatusCode::UNAUTHORIZED, Json(json!({
                    "error": "Invalid credentials",
                    "message": "Invalid email or password"
                }))).into_response();
            }

            let auth_response = {
                let merchant_service = crate::services::merchant_service::MerchantService::new(
                    state.db_pool.clone(),
                    state.config.clone(),
                );

                let remaining_volume: Decimal = merchant_service.get_daily_volume_remaining(
                    merchant.id,
                    merchant.kyc_verified,
                    merchant.daily_limit_usd
                ).await.unwrap_or(Decimal::new(1000, 0));

                // Auto-generate API key if missing (e.g. legacy user or DB reset)
                let has_test_key = merchant.test_api_key_hash.is_some() && merchant.test_api_key_hash.as_ref().unwrap() != "PENDING";
                
                if !has_test_key {
                    tracing::info!("Auto-generating missing API key for merchant {}", merchant.id);
                    let _ = merchant_service.generate_and_store_api_key_with_expiry(merchant.id, false, None).await;
                }

                // Generate Dashboard JWT
                use jsonwebtoken::{encode, Header, EncodingKey};
                use crate::middleware::auth::DashboardClaims;
                
                let now = chrono::Utc::now();
                let duration = if req.remember_me.unwrap_or(false) {
                    chrono::Duration::days(30)
                } else {
                    chrono::Duration::hours(24)
                };
                
                let exp = (now + duration).timestamp() as usize;
                
                let claims = DashboardClaims {
                    sub: merchant.id.to_string(),
                    exp,
                    iat: now.timestamp() as usize,
                    sandbox_mode: merchant.sandbox_mode,
                };
                
                let secret = &state.config.jwt_secret;
                let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
                    .unwrap_or_default();

                // Format masked key for display
                let display_key = if merchant.sandbox_mode {
                    "sk_test_********".to_string()
                } else {
                    if let Some(h) = &merchant.live_api_key_hash {
                         if h != "PENDING" && !h.is_empty() {
                             "sk_live_********".to_string()
                         } else {
                             "Not generated".to_string()
                         }
                    } else {
                         "Not generated".to_string()
                    }
                };

                AuthResponse {
                    user: MerchantProfile {
                        id: merchant.id,
                        business_name: merchant.business_name,
                        email: merchant.email,
                        api_key: display_key,
                        created_at: merchant.created_at.to_rfc3339(),
                        two_factor_enabled: false,
                        daily_limit_usd: merchant.daily_limit_usd.map(|d: Decimal| d.to_string()),
                        daily_volume_remaining: remaining_volume.to_string(),
                        kyc_verified: merchant.kyc_verified,
                        sandbox_mode: merchant.sandbox_mode,
                        settlement_mode: merchant.settlement_mode,
                    },
                    dashboard_token: token,
                }
            };
            (StatusCode::OK, Json(auth_response)).into_response()
        }
        Ok(None) => {
            (StatusCode::UNAUTHORIZED, Json(json!({
                "error": "Invalid credentials",
                "message": "Invalid email or password"
            }))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Database error",
                "message": format!("Failed to authenticate user: {}", e)
            }))).into_response()
        }
    }
}

// DEBUG HANDLER (disabled in production routes but kept for reference)
pub async fn debug_auth(
    State(state): State<AppState>,
    axum::extract::Path(api_key): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.merchant_service.authenticate(&api_key).await {
        Ok(merchant) => Json(json!({
            "success": true,
            "merchant_id": merchant.id,
            "email": merchant.email,
            "sandbox_mode": merchant.sandbox_mode
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("{:?}", e)
        }))
    }
}
