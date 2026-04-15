// Merchant Authentication Handlers
// Registration and login endpoints

use crate::api::state::AppState;
use crate::error::ServiceError;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;

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

    // Step 1 KYC
    pub first_name: String,
    pub last_name: String,
    pub gender: String,
    pub phone_number: String,
    pub country: String,
    pub applicant_role: String,
    pub terms_accepted: bool,

    // Step 2 Business
    pub business_country: String,
    pub business_license_number: Option<String>,
    pub business_certificate_url: Option<String>,
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
    pub has_transaction_pin: bool,
    pub pin_setup_at: Option<String>,
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
        return ServiceError::Forbidden("Registration is currently disabled".to_string())
            .into_response();
    }

    let registration_req = crate::models::merchant::MerchantRegistrationRequest {
        email: req.email.clone(),
        business_name: req.business_name.clone(),
        password: req.password.clone(),
        first_name: req.first_name.clone(),
        last_name: req.last_name.clone(),
        gender: req.gender.clone(),
        phone_number: req.phone_number.clone(),
        country: req.country.clone(),
        applicant_role: req.applicant_role.clone(),
        terms_accepted: req.terms_accepted,
        business_country: req.business_country.clone(),
        business_license_number: req.business_license_number.clone(),
        business_certificate_url: req.business_certificate_url.clone(),
    };

    match state
        .merchant_service
        .register_merchant(&registration_req)
        .await
    {
        Ok(response) => {
            // Generate JWT for new registration
            let now = chrono::Utc::now();
            let exp = (now + chrono::Duration::hours(24)).timestamp() as usize;

            use crate::middleware::auth::DashboardClaims;
            use jsonwebtoken::{encode, EncodingKey, Header};

            let claims = DashboardClaims {
                sub: response.merchant_id.to_string(),
                exp,
                iat: now.timestamp() as usize,
                sandbox_mode: true, // New registrations start in sandbox
            };

            let secret = &state.config.jwt_secret;
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(secret.as_bytes()),
            )
            .unwrap_or_default();

            let auth_response = AuthResponse {
                user: MerchantProfile {
                    id: response.merchant_id,
                    business_name: req.business_name.clone(),
                    email: req.email.clone(),
                    api_key: response.api_key, // Return the REAL key once on registration
                    created_at: chrono::Utc::now().to_rfc3339(),
                    two_factor_enabled: false,
                    daily_limit_usd: Some(state.config.daily_volume_limit_non_kyc_usd.to_string()),
                    daily_volume_remaining: state.config.daily_volume_limit_non_kyc_usd.to_string(),
                    kyc_verified: false,
                    sandbox_mode: true,
                    settlement_mode: "managed".to_string(),
                    has_transaction_pin: false,
                    pin_setup_at: None,
                },
                dashboard_token: token,
            };

            // Log registration and trace
            let _ = state
                .audit_service
                .log_event(
                    response.merchant_id,
                    "registration",
                    Some("Successfully registered new merchant"),
                    Some(json!({"email": req.email, "business_name": req.business_name})),
                )
                .await;
            tracing::info!(
                "EVENT: registration | Merchant: {} | Email: {}",
                response.merchant_id,
                req.email
            );

            (StatusCode::CREATED, Json(auth_response)).into_response()
        }
        Err(e) => e.into_response(),
    }
}

pub async fn login_merchant(
    State(state): State<AppState>,
    Json(req): Json<LoginMerchantRequest>,
) -> impl IntoResponse {
    // Query the database for the user
    let merchant_query = sqlx::query(
        "SELECT id, business_name, email, sandbox_mode, settlement_mode, kyc_verified, created_at, role::text as role, live_api_key_hash, test_api_key_hash, password_hash, daily_limit_usd, wallets_locked, customer_wallets_locked, transaction_pin_hash, pin_setup_at FROM merchants WHERE email = $1 AND is_active = true"
    )
    .bind(&req.email)
    .fetch_optional(&state.db_pool)
    .await;

    match merchant_query {
        Ok(Some(merchant)) => {
            use sqlx::Row;
            // VERIFY PASSWORD
            use argon2::{Argon2, PasswordHash, PasswordVerifier};

            let m_id: i64 = merchant.get("id");
            let m_business_name: String = merchant.get("business_name");
            let m_email: String = merchant.get("email");
            let m_sandbox_mode: bool = merchant.get("sandbox_mode");
            let m_settlement_mode: String = merchant.get("settlement_mode");
            let m_kyc_verified: bool = merchant.get("kyc_verified");
            let m_created_at: chrono::DateTime<chrono::Utc> = merchant.get("created_at");
            let m_role: Option<String> = merchant.try_get("role").ok();
            let m_live_api_key_hash: Option<String> = merchant.get("live_api_key_hash");
            let m_test_api_key_hash: Option<String> = merchant.get("test_api_key_hash");
            let m_password_hash: Option<String> = merchant.get("password_hash");
            let m_daily_limit_usd: Option<Decimal> = merchant.get("daily_limit_usd");
            let m_transaction_pin_hash: Option<String> = merchant.get("transaction_pin_hash");
            let m_pin_setup_at: Option<chrono::DateTime<chrono::Utc>> =
                merchant.get("pin_setup_at");

            // Check if password_hash exists (it might be NULL for old users or API-only users)
            let hash_to_check = m_password_hash.as_ref().ok_or_else(|| {
                // If no password hash, user cannot login via password (API key only)
                ServiceError::Unauthorized(
                    "Password login not available for this account".to_string(),
                )
            });

            if let Err(_) = hash_to_check {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": "Invalid credentials",
                        "message": "Invalid email or password"
                    })),
                )
                    .into_response();
            }

            let parsed_hash = PasswordHash::new(hash_to_check.unwrap())
                .map_err(|e| ServiceError::InternalError(format!("Invalid hash structure: {}", e)))
                .unwrap();

            let valid = Argon2::default()
                .verify_password(req.password.as_bytes(), &parsed_hash)
                .is_ok();

            if !valid {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": "Invalid credentials",
                        "message": "Invalid email or password"
                    })),
                )
                    .into_response();
            }

            let auth_response = {
                let merchant_service = crate::services::merchant_service::MerchantService::new(
                    state.db_pool.clone(),
                    state.config.clone(),
                    state.audit_service.clone(),
                    state.volume_tracking_service.clone(),
                );

                let remaining_volume: Decimal = merchant_service
                    .get_daily_volume_remaining(m_id, m_kyc_verified, m_daily_limit_usd)
                    .await
                    .unwrap_or(state.config.daily_volume_limit_non_kyc_usd);

                // Auto-generate API key if missing (e.g. legacy user or DB reset)
                let has_test_key = m_test_api_key_hash.is_some()
                    && m_test_api_key_hash.as_ref().unwrap() != "PENDING";

                if !has_test_key {
                    tracing::info!("Auto-generating missing API key for merchant {}", m_id);
                    let _ = merchant_service
                        .generate_and_store_api_key_with_expiry(m_id, false, None)
                        .await;
                }

                // Generate Dashboard JWT
                use crate::middleware::auth::DashboardClaims;
                use jsonwebtoken::{encode, EncodingKey, Header};

                let now = chrono::Utc::now();
                let duration = if req.remember_me.unwrap_or(false) {
                    chrono::Duration::days(30)
                } else {
                    chrono::Duration::hours(24)
                };

                let exp = (now + duration).timestamp() as usize;

                let claims = DashboardClaims {
                    sub: m_id.to_string(),
                    exp,
                    iat: now.timestamp() as usize,
                    sandbox_mode: m_sandbox_mode,
                };

                let secret = &state.config.jwt_secret;
                let token = encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(secret.as_bytes()),
                )
                .unwrap_or_default();

                // Format masked key for display
                let display_key = if m_sandbox_mode {
                    "sk_test_********".to_string()
                } else {
                    if let Some(h) = &m_live_api_key_hash {
                        if h != "PENDING" && !h.is_empty() {
                            "sk_live_********".to_string()
                        } else {
                            "Not generated".to_string()
                        }
                    } else {
                        "Not generated".to_string()
                    }
                };

                // Log login and trace
                let _ = state
                    .audit_service
                    .log_event(
                        m_id,
                        "login",
                        Some("Successfully logged in via dashboard"),
                        Some(json!({"email": m_email})),
                    )
                    .await;
                tracing::info!("EVENT: login | Merchant: {} | Email: {}", m_id, m_email);

                AuthResponse {
                    user: MerchantProfile {
                        id: m_id,
                        business_name: m_business_name,
                        email: m_email,
                        api_key: display_key,
                        created_at: m_created_at.to_rfc3339(),
                        two_factor_enabled: false,
                        daily_limit_usd: m_daily_limit_usd
                            .or(if !m_kyc_verified {
                                Some(state.config.daily_volume_limit_non_kyc_usd)
                            } else {
                                None
                            })
                            .map(|d: Decimal| d.to_string()),
                        daily_volume_remaining: remaining_volume.to_string(),
                        kyc_verified: m_kyc_verified,
                        sandbox_mode: m_sandbox_mode,
                        settlement_mode: m_settlement_mode,
                        has_transaction_pin: m_transaction_pin_hash.is_some(),
                        pin_setup_at: m_pin_setup_at.map(|d| d.to_rfc3339()),
                    },
                    dashboard_token: token,
                }
            };
            (StatusCode::OK, Json(auth_response)).into_response()
        }
        Ok(None) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Invalid credentials",
                "message": "Invalid email or password"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Database error",
                "message": format!("Failed to authenticate user: {}", e)
            })),
        )
            .into_response(),
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
        })),
    }
}
