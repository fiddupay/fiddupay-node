// Merchant Authentication Handlers
// Registration and login endpoints

use crate::api::state::AppState;
use crate::error::ServiceError;
use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use validator::Validate;

use crate::middleware::validation::{validate_business_email, validate_password_strength};
use crate::models::merchant::UserRole;
use sqlx::Row;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Deserialize, Validate)]
pub struct RegisterMerchantRequest {
    #[validate(email, custom(function = "validate_business_email"))]
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
    pub business_license_number: String,
    pub business_certificate_url: Option<String>,
    pub website_url: String,

    // Optional KYC/Socials
    pub nin_bvn: Option<String>,
    pub twitter_handle: Option<String>,
    pub instagram_handle: Option<String>,
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
    pub role: UserRole,
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
    // Intelligence & Compliance
    pub nin_bvn_hash: Option<String>,
    pub social_handles: serde_json::Value,
    pub kyc_tier: i32,
    pub compliance_status: String,
}

// ============================================================================
// Handlers
// ============================================================================

pub async fn register_merchant(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<RegisterMerchantRequest>,
) -> impl IntoResponse {
    // 1. Check if registration is enabled
    if !state.config.merchant_registration_enabled {
        return ServiceError::Forbidden("Registration is currently disabled".to_string())
            .into_response();
    }

    // 2. Validate input
    if let Err(e) = req.validate() {
        return ServiceError::ValidationError(e.to_string()).into_response();
    }

    // 3. Custom Password Strength check (Hardcoded policies)
    if let Err(e) = validate_password_strength(&req.password) {
        return ServiceError::ValidationError(e.to_string()).into_response();
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
        website_url: req.website_url.clone(),
        nin_bvn: req.nin_bvn.clone(),
        twitter_handle: req.twitter_handle.clone(),
        instagram_handle: req.instagram_handle.clone(),
    };

    // 4. Capture metadata for compliance (IP and User-Agent)
    let ip_address = Some(addr.ip().to_string());
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    match state
        .merchant_service
        .register_merchant(&registration_req, ip_address, user_agent)
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
                user_id: None,
                role: UserRole::Merchant,
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
                    role: UserRole::Merchant,
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
                    nin_bvn_hash: None,
                    social_handles: json!({}),
                    kyc_tier: 0,
                    compliance_status: "PENDING".to_string(),
                },
                dashboard_token: token.clone(),
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

            let cookie = format!(
                "dashboard_token={}; HttpOnly; SameSite=Lax; Path=/; Max-Age=86400",
                token
            );

            (
                StatusCode::CREATED,
                [
                    (axum::http::header::SET_COOKIE, cookie),
                    (
                        axum::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
                        "Set-Cookie".to_string(),
                    ),
                ],
                Json(auth_response),
            )
                .into_response()
        }
        Err(e) => ServiceError::Internal(e.to_string()).into_response(),
    }
}

pub async fn login_merchant(
    State(state): State<AppState>,
    Json(req): Json<LoginMerchantRequest>,
) -> impl IntoResponse {
    // 2. Check for account lockout
    match state
        .account_lockout_service
        .check_lockout(&req.email)
        .await
    {
        Ok(true) => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": "Account locked",
                    "message": "Too many failed attempts. Please try again in 15 minutes."
                })),
            )
                .into_response();
        }
        Err(e) => return e.into_response(),
        _ => {}
    }

    // 3. Attempt Owner Login first
    let merchant_query = sqlx::query(
        "SELECT id, business_name, email, sandbox_mode, settlement_mode, kyc_verified, created_at, role, live_api_key_hash, test_api_key_hash, password_hash, daily_limit_usd, wallets_locked, customer_wallets_locked, transaction_pin_hash, pin_setup_at, nin_bvn_hash, social_handles, kyc_tier, compliance_status, is_active FROM merchants WHERE LOWER(email) = LOWER($1)"
    )
    .bind(&req.email)
    .fetch_optional(&state.db_pool)
    .await;

    match merchant_query {
        Ok(Some(merchant)) => {
            use argon2::{Argon2, PasswordHash, PasswordVerifier};

            // 3.1 Check if active
            let is_active: bool = merchant.get("is_active");
            if !is_active {
                tracing::warn!("Login attempt for deactivated merchant: {}", req.email);
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "Account deactivated",
                        "message": "Your merchant account has been deactivated. Please contact support."
                    })),
                )
                    .into_response();
            }

            let m_password_hash: Option<String> = merchant.get("password_hash");

            if let Some(hash_str) = m_password_hash {
                if let Ok(parsed_hash) = PasswordHash::new(&hash_str) {
                    if Argon2::default()
                        .verify_password(req.password.as_bytes(), &parsed_hash)
                        .is_ok()
                    {
                        // SUCCESS: Owner Login
                        let m_role: UserRole = merchant.get("role");
                        return finalize_login(
                            state,
                            merchant_context_from_row(merchant),
                            m_role,
                            None,
                            req.remember_me.unwrap_or(false),
                        )
                        .await;
                    } else {
                        tracing::warn!(
                            "Login failed: Password mismatch for merchant {}",
                            req.email
                        );
                    }
                } else {
                    tracing::error!(
                        "Login failed: Invalid password hash format for merchant {}",
                        req.email
                    );
                }
            } else {
                tracing::warn!(
                    "Login failed: No password hash found for merchant account {}",
                    req.email
                );
            }

            // Password failed or no hash
            if let Ok(true) = state
                .account_lockout_service
                .record_failed_attempt(&req.email, "dashboard")
                .await
            {
                let _ = state.security_monitoring_service.log_security_event(crate::services::security_monitoring_service::SecurityEvent {
                    merchant_id: Some(merchant.get("id")),
                    event_type: "security.account_locked".to_string(),
                    severity: "HIGH".to_string(),
                    source_ip: "dashboard".to_string(),
                    details: serde_json::json!({ "email": req.email, "reason": "Consecutive failed login attempts" }),
                    timestamp: chrono::Utc::now(),
                }).await;
            }

            (
                StatusCode::UNAUTHORIZED,
                Json(
                    json!({"error": "Invalid credentials", "message": "Invalid email or password"}),
                ),
            )
                .into_response()
        }
        Ok(None) => {
            // 3. Fallback to Team Member login
            let multi_user_service =
                crate::services::multi_user_service::MultiUserService::new(state.db_pool.clone());
            match multi_user_service
                .authenticate(&req.email, &req.password)
                .await
            {
                Ok(user) => {
                    // Fetch the parent merchant info for the context
                    let merchant_res = sqlx::query(
                        "SELECT id, business_name, email, sandbox_mode, settlement_mode, kyc_verified, created_at, role, live_api_key_hash, test_api_key_hash, password_hash, daily_limit_usd, wallets_locked, customer_wallets_locked, transaction_pin_hash, pin_setup_at, nin_bvn_hash, social_handles, kyc_tier, compliance_status, is_active FROM merchants WHERE id = $1"
                    )
                    .bind(user.merchant_id)
                    .fetch_optional(&state.db_pool)
                    .await;

                    match merchant_res {
                        Ok(Some(m_row)) => {
                            // 3.1 Check if parent merchant is active
                            let is_active: bool = m_row.get("is_active");
                            if !is_active {
                                tracing::warn!("Team member login attempt for deactivated merchant: {} (Member: {})", user.merchant_id, req.email);
                                return (
                                    StatusCode::FORBIDDEN,
                                    Json(json!({
                                        "error": "Account deactivated",
                                        "message": "The parent merchant account has been deactivated."
                                    })),
                                )
                                    .into_response();
                            }

                            // SUCCESS: Team Member Login
                            finalize_login(
                                state,
                                merchant_context_from_row(m_row),
                                user.role,
                                Some(user.id),
                                req.remember_me.unwrap_or(false),
                            )
                            .await
                        }
                        Ok(None) => {
                            ServiceError::InternalError("Parent merchant not found".to_string())
                                .into_response()
                        }
                        Err(e) => ServiceError::Database(e).into_response(),
                    }
                }
                Err(_) => {
                    tracing::warn!(
                        "Login failed: Merchant account not found or inactive for {}",
                        req.email
                    );
                    if let Ok(true) = state
                        .account_lockout_service
                        .record_failed_attempt(&req.email, "dashboard")
                        .await
                    {
                        let _ = state.security_monitoring_service.log_security_event(crate::services::security_monitoring_service::SecurityEvent {
                            merchant_id: None, // We don't have the merchant ID for team member failures easily here without more queries
                            event_type: "security.account_locked_attempt".to_string(),
                            severity: "HIGH".to_string(),
                            source_ip: "dashboard".to_string(),
                            details: serde_json::json!({ "email": req.email, "reason": "Consecutive failed team member login attempts" }),
                            timestamp: chrono::Utc::now(),
                        }).await;
                    }
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({"error": "Invalid credentials", "message": "Invalid email or password"})),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => ServiceError::Database(e).into_response(),
    }
}

pub async fn logout_merchant() -> impl IntoResponse {
    let cookie = "dashboard_token=; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=0";
    (
        StatusCode::OK,
        [(axum::http::header::SET_COOKIE, cookie.to_string())],
        Json(json!({"status": "success", "message": "Logged out successfully"})),
    )
        .into_response()
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
// Helper to extract merchant data from a Row
fn merchant_context_from_row(row: sqlx::postgres::PgRow) -> MerchantProfileData {
    use sqlx::Row;
    MerchantProfileData {
        id: row.get("id"),
        business_name: row.get("business_name"),
        email: row.get("email"),
        sandbox_mode: row.get("sandbox_mode"),
        settlement_mode: row.get("settlement_mode"),
        kyc_verified: row.get("kyc_verified"),
        created_at: row.get("created_at"),
        live_api_key_hash: row.get("live_api_key_hash"),
        test_api_key_hash: row.get("test_api_key_hash"),
        daily_limit_usd: row.get("daily_limit_usd"),
        transaction_pin_hash: row.get("transaction_pin_hash"),
        pin_setup_at: row.get("pin_setup_at"),
        nin_bvn_hash: row.get("nin_bvn_hash"),
        social_handles: row.get("social_handles"),
        kyc_tier: row.get("kyc_tier"),
        compliance_status: row.get("compliance_status"),
    }
}

struct MerchantProfileData {
    id: i64,
    business_name: String,
    email: String,
    sandbox_mode: bool,
    settlement_mode: String,
    kyc_verified: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    live_api_key_hash: Option<String>,
    test_api_key_hash: Option<String>,
    daily_limit_usd: Option<Decimal>,
    transaction_pin_hash: Option<String>,
    pin_setup_at: Option<chrono::DateTime<chrono::Utc>>,
    nin_bvn_hash: Option<String>,
    social_handles: serde_json::Value,
    kyc_tier: i32,
    compliance_status: String,
}

// Unified logic to generate token and response
async fn finalize_login(
    state: AppState,
    m: MerchantProfileData,
    role: UserRole,
    user_id: Option<i32>,
    remember_me: bool,
) -> axum::response::Response {
    let merchant_service = crate::services::merchant_service::MerchantService::new(
        state.db_pool.clone(),
        state.config.clone(),
        state.audit_service.clone(),
        state.volume_tracking_service.clone(),
    );

    let remaining_volume: Decimal = merchant_service
        .get_daily_volume_remaining(m.id, m.kyc_verified, m.daily_limit_usd)
        .await
        .unwrap_or(state.config.daily_volume_limit_non_kyc_usd);

    // Auto-generate API key if missing (only for owners)
    if user_id.is_none() {
        let has_test_key =
            m.test_api_key_hash.is_some() && m.test_api_key_hash.as_ref().unwrap() != "PENDING";
        if !has_test_key {
            let _ = merchant_service
                .generate_and_store_api_key_with_expiry(m.id, false, None)
                .await;
        }
    }

    // Generate Dashboard JWT
    use crate::middleware::auth::DashboardClaims;
    use jsonwebtoken::{encode, EncodingKey, Header};

    let exp = if remember_me {
        chrono::Utc::now() + chrono::Duration::days(30)
    } else {
        chrono::Utc::now() + chrono::Duration::hours(24)
    };

    let claims = DashboardClaims {
        sub: m.id.to_string(),
        user_id,
        role,
        exp: exp.timestamp() as usize,
        iat: chrono::Utc::now().timestamp() as usize,
        sandbox_mode: m.sandbox_mode,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .unwrap_or_default();

    // Refresh display key logic
    let display_key = if m.sandbox_mode {
        "sk_test_********".to_string()
    } else {
        match &m.live_api_key_hash {
            Some(h) if !h.is_empty() && h != "PENDING" => "sk_live_********".to_string(),
            _ => "Not generated".to_string(),
        }
    };

    // Log login and trace
    let _ = state
        .audit_service
        .log_event(
            m.id,
            "login",
            Some(&format!(
                "Successfully logged in via dashboard (Role: {:?})",
                role
            )),
            Some(json!({
                "email": m.email,
                "user_id": user_id,
                "role": role
            })),
        )
        .await;

    tracing::info!(
        "EVENT: login | Merchant: {} | Email: {} | Role: {:?} | User: {:?}",
        m.id,
        m.email,
        role,
        user_id
    );

    let balance_monitor = state.balance_monitor.clone();
    let lockout_service = state.account_lockout_service.clone();
    let email_clone = m.email.clone();
    let merchant_id = m.id;
    let is_live = !m.sandbox_mode;

    tokio::spawn(async move {
        let _ = balance_monitor
            .check_merchant_on_demand(merchant_id, is_live)
            .await;
        let _ = lockout_service
            .record_successful_login(&email_clone, "dashboard")
            .await;
    });

    let limit_to_show = m.daily_limit_usd.unwrap_or(if m.kyc_verified {
        state.config.daily_volume_limit_verified_usd
    } else {
        state.config.daily_volume_limit_non_kyc_usd
    });

    // 6. Build response with HttpOnly cookie (Fortress Layer)
    // Note: Secure flag removed for dev compatibility. SameSite=Lax for security.
    let cookie = format!(
        "dashboard_token={}; HttpOnly; SameSite=Lax; Path=/; Max-Age={}",
        token,
        if remember_me { 86400 * 30 } else { 86400 }
    );

    (
        StatusCode::OK,
        [
            (axum::http::header::SET_COOKIE, cookie),
            (
                axum::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
                "Set-Cookie".to_string(),
            ),
        ],
        Json(AuthResponse {
            user: MerchantProfile {
                id: m.id,
                business_name: m.business_name,
                email: m.email,
                role,
                api_key: display_key,
                created_at: m.created_at.to_rfc3339(),
                two_factor_enabled: false,
                daily_limit_usd: Some(limit_to_show.to_string()),
                daily_volume_remaining: remaining_volume.to_string(),
                kyc_verified: m.kyc_verified,
                sandbox_mode: m.sandbox_mode,
                settlement_mode: m.settlement_mode,
                has_transaction_pin: m.transaction_pin_hash.is_some(),
                pin_setup_at: m.pin_setup_at.map(|d| d.to_rfc3339()),
                nin_bvn_hash: m.nin_bvn_hash.map(|h| {
                    if h.len() > 8 {
                        format!("{}...", &h[..8])
                    } else {
                        "****".to_string()
                    }
                }),
                social_handles: m.social_handles,
                kyc_tier: m.kyc_tier,
                compliance_status: m.compliance_status,
            },
            dashboard_token: token, // Kept for legacy support but frontend should move to cookies
        }),
    )
        .into_response()
}
