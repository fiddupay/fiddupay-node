// API Handlers
// HTTP request handlers

use crate::api::state::AppState;
use chrono::Utc;
use crate::error::ServiceError;
use crate::middleware::auth::MerchantContext;
use crate::payment::models::{CreatePaymentRequest, PaymentFilters, CryptoType};
use crate::services::invoice_service::{CreateInvoiceRequest, InvoiceItem};
use axum::{
    extract::{Path, Query, State, Request, Extension},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;
use validator::Validate;
use html_escape::encode_text;

// Import validation functions
use crate::middleware::validation::{validate_business_email, validate_password_strength, validate_webhook_url};
use rust_decimal::Decimal;
use crate::models::merchant::Merchant;

pub async fn root_handler() -> &'static str {
    "backend is running"
}

// DEBUG HANDLER
pub async fn debug_auth(
    State(state): State<AppState>,
    Path(api_key): Path<String>,
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

// ============================================================================
// Merchant Endpoints
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
    pub api_key: String, // Added field
    pub created_at: String,
    pub two_factor_enabled: bool,
    pub daily_limit_usd: Option<String>,
    pub daily_volume_remaining: String,
    pub kyc_verified: bool,
    pub sandbox_mode: bool,
    pub settlement_mode: String,
}

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
    match sqlx::query!(
        "SELECT id, business_name, email, sandbox_mode, settlement_mode, kyc_verified, created_at, role::text as role, live_api_key_hash, test_api_key_hash, password_hash, daily_limit_usd FROM merchants WHERE email = $1 AND is_active = true",
        req.email
    )
    .fetch_optional(&state.db_pool)
    .await
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
                    // We ignore the result (the key string) here because we can't show it securely 
                    // without a dedicated "New Key" modal. We just ensure it exists.
                    // The user will see a masked key and can rotate if they need the raw value.
                    let _ = merchant_service.generate_and_store_api_key_with_expiry(merchant.id, false, None).await;
                }

                // Generate Dashboard JWT (No API key rotation)
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
                };
                
                let secret = &state.config.jwt_secret;
                let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
                    .unwrap_or_default();

                // Format masked key for display
                let display_key = if merchant.sandbox_mode {
                    // We just ensured test key exists or was generated
                    "sk_test_********".to_string()
                } else {
                    // Check live key
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
                        daily_limit_usd: merchant.daily_limit_usd.map(|d| d.to_string()),
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

pub async fn get_merchant_profile(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    let merchant_id = context.merchant_id;

    // 1. Fetch BASIC merchant info
    let merchant = match sqlx::query!(
        r#"
        SELECT id, business_name, email, sandbox_mode, settlement_mode, 
               kyc_verified, daily_limit_usd, created_at, redirect_url,
               test_api_key_hash, live_api_key_hash
        FROM merchants
        WHERE id = $1
        "#,
        merchant_id
    )
    .fetch_optional(&state.db_pool)
    .await {
        Ok(Some(m)) => m,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error": "Merchant not found"}))).into_response(),
        Err(e) => {
            eprintln!("Profile DB Error (Main Query): {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database error: {}", e)}))).into_response();
        }
    };

    // 2. Fetch Webhook config separately
    let (webhook_url, webhook_format) = match sqlx::query!(
        r#"SELECT url, payload_format FROM webhook_configs WHERE merchant_id = $1 AND is_active = true"#,
        merchant_id
    )
    .fetch_optional(&state.db_pool)
    .await {
        Ok(Some(cfg)) => (Some(cfg.url), Some(cfg.payload_format)),
        Ok(None) => (None, None),
        Err(e) => {
            eprintln!("Profile DB Error (Webhook Fetch): {:?}", e);
            (None, None) // Non-critical failure
        }
    };

    // 3. Construct profile with masked API key if it's a dashboard session
    let display_key = if context.api_key == "DASHBOARD_SESSION" {
        // Show masked version of the current environment's key hash
        let hash_opt = if merchant.sandbox_mode { &merchant.test_api_key_hash } else { &merchant.live_api_key_hash };
        let is_valid = hash_opt.as_ref().map(|h| h != "PENDING" && !h.is_empty()).unwrap_or(false);
        
        if !is_valid {
            "Not generated".to_string()
        } else {
            // Masked format
            format!("sk_{}_********", if merchant.sandbox_mode { "test" } else { "live" })
        }
    } else {
        context.api_key.clone()
    };

    let mut profile = json!({
        "id": merchant.id,
        "business_name": merchant.business_name,
        "email": merchant.email,
        "api_key": display_key,
        "redirect_url": merchant.redirect_url,
        "webhook_url": webhook_url,
        "webhook_format": webhook_format,
        "sandbox_mode": merchant.sandbox_mode,
        "settlement_mode": merchant.settlement_mode,
        "kyc_verified": merchant.kyc_verified,
        "daily_limit_usd": merchant.daily_limit_usd.map(|d| d.to_string()),
        "created_at": merchant.created_at.to_rfc3339(),
        "two_factor_enabled": false
    });
    
    // 4. Calculate daily volume remaining
    let remaining = state.merchant_service.get_daily_volume_remaining(
        merchant.id,
        merchant.kyc_verified,
        merchant.daily_limit_usd
    ).await.unwrap_or(Decimal::ZERO);
    
    profile["daily_volume_remaining"] = json!(remaining.to_string());
    
    (StatusCode::OK, Json(json!({ "user": profile }))).into_response()
}

pub async fn get_merchant_readiness(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    let merchant_id = context.merchant_id;
    let wallet_service = crate::services::wallet_config_service::WalletConfigService::new(state.db_pool.clone());
    let currency_service = crate::services::currency_service::CurrencyService::new(state.db_pool.clone());
    
    // 1. Fetch data
    let merchant_res = sqlx::query!("SELECT sandbox_mode, settlement_mode, kyc_verified FROM merchants WHERE id = $1", merchant_id).fetch_one(&state.db_pool).await;
    let wallets_res = wallet_service.get_wallet_configs(merchant_id).await;
    let currencies_res = currency_service.get_merchant_enabled_currencies(merchant_id).await;

    let merchant = match merchant_res {
        Ok(m) => m,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    };

    let wallets = wallets_res.unwrap_or_default();
    let enabled_currencies = currencies_res; // It's already a Vec

    // 2. Determine enabled networks
    let mut enabled_networks = std::collections::HashSet::new();
    for curr in enabled_currencies {
        enabled_networks.insert(curr.2);
    }

    // 3. Analyze wallet coverage
    let mut network_status = json!({});
    let mut issues = Vec::new();
    let mut is_ready = true;

    for network in &enabled_networks {
        let wallet = wallets.iter().find(|w| w.network == *network);
        match wallet {
            Some(w) => {
                network_status[network] = json!({
                    "status": "configured",
                    "address": w.address,
                    "is_active": w.is_active
                });
                if !w.is_active {
                    issues.push(format!("Network {} is configured but inactive", network));
                    is_ready = false;
                }
            },
            None => {
                network_status[network] = json!({
                    "status": "missing",
                    "action_required": "configure_wallet"
                });
                issues.push(format!("Wallet not configured for enabled network: {}", network));
                is_ready = false;
            }
        }
    }

    // 4. Security status check
    let security_alerts: i64 = sqlx::query_scalar!("SELECT COUNT(*) as \"count!\" FROM security_alerts WHERE merchant_id = $1 AND acknowledged = FALSE", merchant_id)
        .fetch_one(&state.db_pool)
        .await
        .unwrap_or(0);

    if security_alerts > 0 {
        issues.push(format!("{} active security alerts require attention", security_alerts));
    }

    // 5. Build final response
    let response = json!({
        "is_ready": is_ready && security_alerts == 0,
        "environment": if merchant.sandbox_mode { "sandbox" } else { "live" },
        "settlement_mode": merchant.settlement_mode,
        "kyc_verified": merchant.kyc_verified,
        "network_coverage": network_status,
        "security": {
            "active_alerts": security_alerts
        },
        "issues": issues
    });

    (StatusCode::OK, Json(response)).into_response()
}

#[derive(Deserialize)]
pub struct UnifiedTransactionQuery {
    pub limit: Option<i64>,
}

pub async fn list_unified_transactions(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(params): Query<UnifiedTransactionQuery>,
) -> impl IntoResponse {
    let merchant_id = context.merchant_id;
    let is_sandbox = context.sandbox_mode;
    let limit = params.limit.unwrap_or(50).min(100).max(1);

    // A unified query to get payments, refunds, and withdrawals in one feed
    // Filtered by sandbox_mode to isolate environments
    let query = r#"
        (SELECT 
            'payment' as txn_type,
            payment_id as id,
            amount::text as crypto_amount,
            amount_usd::text as usd_amount,
            crypto_type,
            status,
            transaction_hash,
            created_at
        FROM payment_transactions
        WHERE merchant_id = $1 AND sandbox_mode = $2)
        
        UNION ALL
        
        (SELECT 
            'refund' as txn_type,
            r.refund_id as id,
            r.amount::text as crypto_amount,
            r.amount_usd::text as usd_amount,
            p.crypto_type,
            r.status,
            r.transaction_hash,
            r.created_at
        FROM refunds r
        JOIN payment_transactions p ON r.payment_id = p.id
        WHERE r.merchant_id = $1 AND r.sandbox_mode = $2)
        
        UNION ALL
        
        (SELECT 
            'withdrawal' as txn_type,
            withdrawal_id as id,
            amount::text as crypto_amount,
            amount::text as usd_amount,
            crypto_type,
            status,
            transaction_hash,
            created_at
        FROM withdrawals
        WHERE merchant_id = $1 AND sandbox_mode = $2)
        
        ORDER BY created_at DESC
        LIMIT $3
    "#;

    match sqlx::query(query)
        .bind(merchant_id)
        .bind(is_sandbox)
        .bind(limit)
        .fetch_all(&state.db_pool)
        .await
    {
        Ok(rows) => {
            use sqlx::Row;
            let txns: Vec<serde_json::Value> = rows.into_iter().map(|row| {
                json!({
                    "type": row.get::<String, _>("txn_type"),
                    "id": row.get::<String, _>("id"),
                    "crypto_amount": row.get::<String, _>("crypto_amount"),
                    "usd_amount": row.get::<String, _>("usd_amount"),
                    "crypto_type": row.get::<String, _>("crypto_type"),
                    "status": row.get::<String, _>("status"),
                    "transaction_hash": row.get::<Option<String>, _>("transaction_hash"),
                    "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339()
                })
            }).collect();
            
            (StatusCode::OK, Json(json!({"transactions": txns}))).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn switch_environment(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<SwitchEnvironmentRequest>,
) -> impl IntoResponse {
    match state.merchant_service.switch_environment(context.merchant_id, req.to_live).await {
        Ok(maybe_key) => {
            let mut response = json!({
                "environment": if req.to_live { "live" } else { "sandbox" },
                "sandbox_mode": !req.to_live
            });
            if let Some(api_key) = maybe_key {
                response["api_key"] = json!(api_key);
            }
            (StatusCode::OK, Json(response)).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct SwitchEnvironmentRequest {
    pub to_live: bool,
}

pub async fn generate_api_key(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<GenerateApiKeyRequest>,
) -> impl IntoResponse {
    match state.merchant_service.generate_and_store_api_key_with_expiry(context.merchant_id, req.is_live, None).await {
        Ok(api_key) => (StatusCode::OK, Json(json!({"api_key": api_key}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("{}", e)}))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct GenerateApiKeyRequest {
    pub is_live: bool,
}

#[derive(Deserialize)]
pub struct RotateApiKeyRequest {
    pub is_live: bool,
}

pub async fn rotate_api_key(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    req: Option<Json<RotateApiKeyRequest>>,
) -> impl IntoResponse {
    let result = if context.api_key == "DASHBOARD_SESSION" {
        // Dashboard session: requires explicit environment in body
        match req {
            Some(Json(payload)) => {
                state.merchant_service.rotate_api_key_by_env(context.merchant_id, payload.is_live).await
            },
            None => return (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Missing parameters",
                "message": "Dashboard rotation requires 'is_live' parameter"
            }))).into_response()
        }
    } else {
        // API Key session: uses the key itself for verification/env detection
        state.merchant_service.rotate_api_key(context.merchant_id, &context.api_key).await
    };

    match result {
        Ok(new_api_key) => (StatusCode::OK, Json(json!({"api_key": new_api_key}))).into_response(),
        Err(e) => e.into_response(),
    }
}

#[derive(Deserialize)]
pub struct UpdateSettlementModeRequest {
    pub mode: String,
}

pub async fn update_settlement_mode(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<UpdateSettlementModeRequest>,
) -> impl IntoResponse {
    match state.merchant_service.update_settlement_mode(context.merchant_id, &req.mode).await {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "success", "mode": req.mode}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct SetWalletRequest {
    pub crypto_type: String,
    pub address: String,
}

pub async fn set_wallet(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<SetWalletRequest>,
) -> impl IntoResponse {
    let crypto_type = match CryptoType::from_string(&req.crypto_type) {
        Ok(ct) => ct,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid crypto_type"}))).into_response(),
    };
    
    match state.merchant_service.set_wallet_address(context.merchant_id, crypto_type, req.address).await {
        Ok(_) => (StatusCode::OK, Json(json!({"success": true}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

#[derive(Deserialize, Validate)]
pub struct UnifiedSettingsRequest {
    #[validate(custom(function = "validate_optional_webhook_url"))]
    pub webhook_url: Option<String>,
    pub redirect_url: Option<String>,
    pub webhook_format: Option<String>,
    pub settlement_mode: Option<String>,
    pub customer_pays_fee: Option<bool>,
    pub ip_whitelist: Option<Vec<String>>,
    pub sandbox_mode: Option<bool>,
    pub rotate_webhook_secret: Option<bool>,
}

fn validate_optional_webhook_url(url: &String) -> Result<(), validator::ValidationError> {
    validate_webhook_url(url)
}

pub async fn update_merchant_settings(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<UnifiedSettingsRequest>,
) -> impl IntoResponse {
    // 1. Update Merchant core settings
    if req.settlement_mode.is_some() || req.customer_pays_fee.is_some() || req.sandbox_mode.is_some() || req.redirect_url.is_some() {
        if let Err(e) = state.merchant_service.update_settings(
            context.merchant_id,
            req.settlement_mode.clone(),
            req.customer_pays_fee,
            req.sandbox_mode,
            req.redirect_url.clone(),
        ).await {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response();
        }
    }

    // 2. Update Webhook if provided
    if req.webhook_url.is_some() || req.webhook_format.is_some() {
        if let Err(e) = state.webhook_service.set_webhook_url(
            context.merchant_id, 
            req.webhook_url,
            req.webhook_format.clone()
        ).await {
            return (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response();
        }
    }

    // 3. Update IP Whitelist if provided
    if let Some(ips) = req.ip_whitelist {
        if let Err(e) = state.ip_whitelist_service.set_whitelist(context.merchant_id, ips).await {
            return (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response();
        }
    }

    // 4. Rotate Webhook Secret if requested
    if req.rotate_webhook_secret.unwrap_or(false) {
        let new_secret = hex::encode(rand::random::<[u8; 32]>());
        if let Err(e) = sqlx::query!(
            "UPDATE webhook_configs SET signing_secret = $1, updated_at = NOW() WHERE merchant_id = $2",
            new_secret,
            context.merchant_id
        )
        .execute(&state.db_pool)
        .await {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response();
        }
    }

    (StatusCode::OK, Json(json!({
        "status": "success",
        "message": "Settings updated successfully"
    }))).into_response()
}

#[derive(Deserialize, Validate)]
pub struct SetWebhookRequest {
    #[validate(url, custom(function = "validate_webhook_url"))]
    pub url: String,
}

pub async fn set_webhook(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<SetWebhookRequest>,
) -> impl IntoResponse {
    match state.webhook_service.set_webhook_url(context.merchant_id, Some(req.url), None).await {
        Ok(_) => (StatusCode::OK, Json(json!({"success": true}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn get_merchant_settings(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    let merchant_id = context.merchant_id;
    
    // 1. Get core merchant settings
    let merchant = match sqlx::query!(
        "SELECT settlement_mode, customer_pays_fee, sandbox_mode, redirect_url FROM merchants WHERE id = $1",
        merchant_id
    )
    .fetch_one(&state.db_pool)
    .await {
        Ok(m) => m,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    };

    // 2. Get webhook config
    let webhook_config = sqlx::query!(
        "SELECT url, payload_format, signing_secret FROM webhook_configs WHERE merchant_id = $1 AND is_active = true",
        merchant_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .unwrap_or(None);

    // 3. Get IP whitelist
    let ip_whitelist = sqlx::query_scalar!(
        "SELECT ip_address FROM ip_whitelist WHERE merchant_id = $1",
        merchant_id
    )
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    (StatusCode::OK, Json(json!({
        "webhook_url": webhook_config.as_ref().map(|c| &c.url),
        "webhook_format": webhook_config.as_ref().map(|c| &c.payload_format),
        "webhook_signing_secret": webhook_config.as_ref().map(|c| &c.signing_secret),
        "settlement_mode": merchant.settlement_mode,
        "customer_pays_fee": merchant.customer_pays_fee,
        "sandbox_mode": merchant.sandbox_mode,
        "redirect_url": merchant.redirect_url,
        "ip_whitelist": ip_whitelist
    }))).into_response()
}

pub async fn send_test_webhook(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    let merchant_id = context.merchant_id;
    
    let payload = crate::models::webhook::WebhookPayload {
        event_type: "webhook.test".to_string(),
        payment_id: "test_payment_123".to_string(),
        merchant_id,
        status: crate::payment::models::PaymentStatus::Confirmed,
        amount: rust_decimal::Decimal::new(100, 2), // 1.00
        crypto_type: "SOL".to_string(),
        transaction_hash: Some("test_hash_abc123".to_string()),
        timestamp: Utc::now().timestamp(),
    };

    if let Err(e) = state.webhook_service.queue_webhook(merchant_id, None, payload).await {
         return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response();
    }

    (StatusCode::OK, Json(json!({
        "status": "success",
        "message": "Test webhook queued for delivery"
    }))).into_response()
}

// ============================================================================
// Payment Endpoints
// ============================================================================

pub async fn create_payment(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<CreatePaymentRequest>,
) -> impl IntoResponse {
    match state.payment_service.create_payment(context.merchant_id, req).await {
        Ok(response) => (StatusCode::CREATED, Json(response)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn get_payment(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(payment_id): Path<String>,
) -> impl IntoResponse {
    match state.payment_service.get_payment(&payment_id, context.merchant_id).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn cancel_payment(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(payment_id): Path<String>,
) -> impl IntoResponse {
    match state.payment_service.cancel_payment(context.merchant_id, &payment_id).await {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "success", "message": "Payment cancelled"}))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct VerifyPaymentRequest {
    pub transaction_hash: String,
}

pub async fn verify_payment(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(payment_id): Path<String>,
    Json(req): Json<VerifyPaymentRequest>,
) -> impl IntoResponse {
    match state.payment_service.verify_payment(&payment_id, &req.transaction_hash, context.merchant_id).await {
        Ok(confirmed) => (StatusCode::OK, Json(json!({"confirmed": confirmed}))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn list_payments(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(filters): Query<PaymentFilters>,
) -> impl IntoResponse {
    match state.payment_service.list_payments(context.merchant_id, filters).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

// ============================================================================
// Refund Endpoints
// ============================================================================

#[derive(Deserialize)]
pub struct CreateRefundRequest {
    pub payment_id: String,
    pub amount: Option<rust_decimal::Decimal>,
    pub reason: String,
}

pub async fn create_refund(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<CreateRefundRequest>,
) -> impl IntoResponse {
    match state.refund_service.create_refund(context.merchant_id, req.payment_id, req.amount, req.reason).await {
        Ok(response) => (StatusCode::CREATED, Json(response)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn get_refund(
    State(state): State<AppState>,
    Path(refund_id): Path<String>,
) -> impl IntoResponse {
    match state.refund_service.get_refund(refund_id).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct CompleteRefundRequest {
    pub transaction_hash: String,
}

pub async fn complete_refund(
    State(state): State<AppState>,
    Path(refund_id): Path<String>,
    Json(req): Json<CompleteRefundRequest>,
) -> impl IntoResponse {
    match state.refund_service.complete_refund(refund_id, req.transaction_hash).await {
        Ok(_) => (StatusCode::OK, Json(json!({"success": true}))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

// ============================================================================
// Analytics Endpoints
// ============================================================================

#[derive(Deserialize)]
pub struct AnalyticsQuery {
    pub from_date: Option<chrono::DateTime<chrono::Utc>>,
    pub to_date: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn get_analytics(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(query): Query<AnalyticsQuery>,
) -> impl IntoResponse {
    let from = query.from_date.unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));
    let to = query.to_date.unwrap_or_else(|| chrono::Utc::now());
    
    match state.analytics_service.get_analytics(context.merchant_id, from, to, None, None, Some(context.sandbox_mode)).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn export_analytics(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(query): Query<AnalyticsQuery>,
) -> impl IntoResponse {
    let from = query.from_date.unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));
    let to = query.to_date.unwrap_or_else(|| chrono::Utc::now());
    
    match state.analytics_service.export_csv(context.merchant_id, from, to, None, None, Some(context.sandbox_mode)).await {
        Ok(csv) => (StatusCode::OK, csv).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

// ============================================================================
// Sandbox Endpoints
// ============================================================================

pub async fn enable_sandbox(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    match state.sandbox_service.create_sandbox_credentials(context.merchant_id).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct SimulatePaymentRequest {
    pub success: bool,
    pub transaction_hash: Option<String>,
    pub from_address: Option<String>,
}

pub async fn simulate_payment(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(payment_id): Path<String>,
    Json(req): Json<SimulatePaymentRequest>,
) -> impl IntoResponse {
    match state.sandbox_service.simulate_confirmation(&payment_id, context.merchant_id, req.success, req.transaction_hash, req.from_address).await {
        Ok(_) => {
            if req.success {
                (StatusCode::OK, Json(json!({"success": true, "message": "Payment simulated successfully"}))).into_response()
            } else {
                (StatusCode::OK, Json(json!({"success": true, "message": "Payment simulation failed as requested"}))).into_response()
            }
        },
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

// ============================================================================
// Hosted Payment Page
// ============================================================================

pub async fn payment_page(
    State(state): State<AppState>,
    Path(link_id): Path<String>,
) -> impl IntoResponse {
    use axum::response::Html;
    
    // 1. Try to look up by link_id in payment_links (vanity/shareable links)
    let (internal_id, public_id) = match sqlx::query!(
        "SELECT payment_id FROM payment_links WHERE link_id = $1",
        &link_id
    )
    .fetch_optional(&state.db_pool)
    .await
    {
        Ok(Some(link)) => (Some(link.payment_id), None),
        Ok(None) => {
            // 2. Fallback: Check if link_id is actually a public payment_id string
            if link_id.starts_with("pay_") {
                (None, Some(link_id.clone()))
            } else {
                return (StatusCode::NOT_FOUND, Html("Payment link not found".to_string())).into_response();
            }
        },
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("Error: {}", e))).into_response(),
    };

    // 3. Get payment details
    let payment_res = sqlx::query!(
        r#"
        SELECT merchant_id, payment_id, status, amount, amount_usd, crypto_type, network, 
               to_address, fee_amount_usd, expires_at, created_at, confirmed_at, 
               transaction_hash, partial_payments_enabled, total_paid, remaining_balance
        FROM payment_transactions 
        WHERE id = $1 OR payment_id = $2
        "#,
        internal_id,
        public_id
    )
    .fetch_optional(&state.db_pool)
    .await;

    let payment = match payment_res {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, Html("Payment not found".to_string())).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("Error: {}", e))).into_response(),
    };

    // Generate QR code for payment (only if selection is finished)
    let qr_code = if let (Some(ct_str), Some(addr)) = (&payment.crypto_type, &payment.to_address) {
        let ct = crate::payment::models::CryptoType::from_string(ct_str).unwrap_or(crate::payment::models::CryptoType::Sol);
        let prefix = ct.uri_scheme();
        
        let qr_data = if let Some(amt) = payment.amount {
            format!("{}:{}?amount={}", prefix, addr, amt)
        } else {
            format!("{}:{}", prefix, addr)
        };
        
        match crate::utils::qr::generate_qr_code(&qr_data) {
            Ok(qr) => qr,
            Err(_) => "QR_ERROR".to_string(),
        }
    } else {
        "".to_string()
    };

    // Calculate time remaining
    let now = chrono::Utc::now();
    let time_remaining = if payment.expires_at > now {
        let duration = payment.expires_at - now;
        format!("{}m {}s", duration.num_minutes(), duration.num_seconds() % 60)
    } else {
        "Expired".to_string()
    };

    // Determine status flags
    let is_selection_required = payment.status == "SELECTION_REQUIRED";
    let is_pending = payment.status == "PENDING" || payment.status == "CONFIRMING";
    let is_confirmed = payment.status == "CONFIRMED";
    let is_cancelled = payment.status == "CANCELLED";
    let is_expired = (payment.status == "FAILED" || (payment.expires_at < now)) && !is_confirmed && !is_cancelled;

    // Check if sandbox and get redirect_url
    let merchant_info = sqlx::query!(
        "SELECT sandbox_mode, redirect_url, customer_pays_fee FROM merchants WHERE id = $1", 
        payment.merchant_id
    )
    .fetch_one(&state.db_pool)
    .await
    .ok();
    
    let sandbox = merchant_info.as_ref().map(|m| m.sandbox_mode).unwrap_or(false);
    let redirect_url = merchant_info.as_ref().and_then(|m| m.redirect_url.clone());
    let customer_pays_fee = merchant_info.as_ref().map(|m| m.customer_pays_fee).unwrap_or(true);

    // Smart Verification: Trigger address scan if pending
    if is_pending {
        // We use .ok() to ignore errors as this is an opportunistic check
        // The background monitor (if active) or manual check are main safeguards
        tracing::info!("Triggering smart verification for payment {}", link_id);
        let _ = state.payment_service.verify_payment_by_address(&payment.payment_id, payment.merchant_id).await;
    }

    // Fetch supported currencies if needed
    let supported_currencies = if is_selection_required {
        sqlx::query!(
             "SELECT crypto_type, network FROM merchant_wallets WHERE merchant_id = $1 AND is_active = true",
             payment.merchant_id
        )
        .fetch_all(&state.db_pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| (r.crypto_type, r.network))
        .collect()
    } else {
        vec![]
    };

    // Render logic
    let html = render_payment_page(PaymentPageData {
        payment_id: payment.payment_id,
        amount: payment.amount.unwrap_or_default().to_string(),
        amount_usd: payment.amount_usd.to_string(),
        crypto_type: payment.crypto_type.unwrap_or_default(),
        network: payment.network.unwrap_or_default(),
        deposit_address: payment.to_address.unwrap_or_default(),
        fee_amount_usd: payment.fee_amount_usd.to_string(),
        qr_code: qr_code,
        time_remaining,
        expires_at: payment.expires_at.to_rfc3339(),
        transaction_hash: payment.transaction_hash,
        is_pending,
        is_confirmed,
        is_expired,
        is_cancelled,
        is_selection_required,
        sandbox,
        redirect_url,
        supported_currencies,
        customer_pays_fee,
    });

    (StatusCode::OK, Html(html)).into_response()
}
 
#[derive(sqlx::FromRow)]
struct PaymentStatusInfo {
    payment_id: String,
    merchant_id: i64,
    status: String,
}

pub async fn payment_status(
    State(state): State<AppState>,
    Path(link_id): Path<String>,
) -> impl IntoResponse {
    // 1. Fetch payment details (status and merchant_id)
    // Supports both link_id (payment_links) and direct payment_id (payment_transactions)
    let payment_info = if link_id.starts_with("pay_") {
        // Direct payment lookup
        sqlx::query_as!(
            PaymentStatusInfo,
            "SELECT payment_id, merchant_id, status FROM payment_transactions WHERE payment_id = $1",
            link_id
        )
        .fetch_optional(&state.db_pool)
        .await
    } else {
        // Link lookup
        sqlx::query_as!(
            PaymentStatusInfo,
            r#"
            SELECT pt.payment_id, pt.merchant_id, pt.status 
            FROM payment_transactions pt
            JOIN payment_links pl ON pl.payment_id = pt.id
            WHERE pl.link_id = $1
            "#,
            link_id
        )
        .fetch_optional(&state.db_pool)
        .await
    };

    match payment_info {
        Ok(Some(payment)) => {
            let mut current_status = payment.status.clone();
            
            // 2. Smart Verification: Trigger address scan if pending
            if current_status == "PENDING" || current_status == "CONFIRMING" {
                 // Trigger verification (fire and forget? No, we want the result if possible)
                 match state.payment_service.verify_payment_by_address(&payment.payment_id, payment.merchant_id).await {
                     Ok(true) => {
                         // Payment confirmed! Update local status for response
                         current_status = "CONFIRMED".to_string();
                     },
                     Ok(false) => {
                         // Still pending, check if it's confirming on chain (handled by verifier update but we just read the bool)
                         // If verifier found a tx but it's confirming, it updated the DB. 
                         // To be perfectly accurate we should re-fetch status, 
                         // but for now let's just return PENDING or rely on next poll.
                         // Actually, if verifying returned false, the status in DB might have changed to CONFIRMING.
                     },
                     Err(e) => {
                         tracing::warn!("Failed to auto-verify payment {}: {}", link_id, e);
                     }
                 }
            }

            // 3. Return status (either original or updated)
            (StatusCode::OK, Json(json!({"status": current_status}))).into_response()
        },
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "Payment not found"}))).into_response(),
        Err(e) => {
            let error_msg = e.to_string();
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": error_msg}))).into_response()
        },
    }
}

pub async fn finalize_payment_selection(
    State(state): State<AppState>,
    Path(link_id): Path<String>,
    Json(req): Json<SelectionRequest>,
) -> impl IntoResponse {
    use std::sync::Arc;
    let pool = state.db_pool.clone();
    
    // 1. Look up payment by link_id
    let payment_link = match sqlx::query!(
        "SELECT payment_id FROM payment_links WHERE link_id = $1",
        &link_id
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(link)) => link,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error": "Payment link not found"}))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    };

    // 2. Get payment details
    let payment_record = match sqlx::query_as!(
        crate::models::payment::Payment,
        r#"
        SELECT id, payment_id, merchant_id, amount, amount_usd, crypto_type, network,
               status, to_address, from_address, created_at, expires_at, confirmed_at,
               confirmations, required_confirmations, description, metadata,
               transaction_hash, webhook_url, fee_percentage, fee_amount, fee_amount_usd,
               user_id, subscription_id, block_number, partial_payments_enabled,
               total_paid, remaining_balance, is_non_custodial
        FROM payment_transactions 
        WHERE id = $1
        "#,
        payment_link.payment_id
    )
    .fetch_optional(&pool)
    .await {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error": "Payment record not found"}))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    };

    if payment_record.status != "SELECTION_REQUIRED" {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Payment currency already selected"}))).into_response();
    }

    // 3. Resolve crypto type and calculate amounts
    let crypto_type = req.crypto_type;
    let merchant_id = payment_record.merchant_id;
    
    // This will trigger auto-generation if in Managed Mode!
    let to_address = match state.merchant_service.get_wallet_address(merchant_id, crypto_type).await {
        Ok(addr) => addr,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Failed to get/generate merchant wallet: {}", e)}))).into_response(),
    };

    let price_service = state.price_service.clone();
    let price = match price_service.get_price(crypto_type).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Price fetch failed: {}", e)}))).into_response(),
    };
    let price_decimal = Decimal::from_f64_retain(price).unwrap_or(Decimal::ONE);

    let amount_usd = payment_record.amount_usd;
    let amount_crypto = amount_usd / price_decimal;
    let fee_amount_crypto = payment_record.fee_amount_usd / price_decimal;
    let network = crypto_type.network();

    // 4. Update payment record
    if let Err(e) = sqlx::query!(
        r#"
        UPDATE payment_transactions 
        SET crypto_type = $1, amount = $2, to_address = $3, network = $4, fee_amount = $5, status = 'PENDING'
        WHERE id = $6
        "#,
        crypto_type.to_string(),
        amount_crypto,
        to_address,
        network,
        fee_amount_crypto,
        payment_record.id
    )
    .execute(&pool)
    .await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response();
    }

    (StatusCode::OK, Json(json!({"message": "Selection finalized", "crypto_type": crypto_type.to_string()}))).into_response()
}

#[derive(serde::Deserialize)]
pub struct SelectionRequest {
    pub crypto_type: crate::payment::models::CryptoType,
}

// Helper functions
fn generate_qr_code(data: &str) -> Result<String, Box<dyn std::error::Error>> {
    use qrcode::QrCode;
    use base64::Engine;
    use image::{ImageBuffer, Luma};
    use std::io::Cursor;

    let code = QrCode::new(data.as_bytes())?;
    let size = code.width() as u32;
    let scale = 8; // Adjust scale for base64 size vs quality
    let mut image = ImageBuffer::new(size * scale, size * scale);

    for x in 0..size {
        for y in 0..size {
            // Draw a module (square) of scale x scale pixels
            let color = match code[(x as usize, y as usize)] {
                qrcode::Color::Dark => Luma([0u8]),
                qrcode::Color::Light => Luma([255u8]),
            };
            for ix in 0..scale {
                for iy in 0..scale {
                    image.put_pixel(x * scale + ix, y * scale + iy, color);
                }
            }
        }
    }

    let mut buffer = Cursor::new(Vec::new());
    image.write_to(&mut buffer, image::ImageFormat::Png)?;
    
    Ok(base64::engine::general_purpose::STANDARD.encode(buffer.into_inner()))
}

struct PaymentPageData {
    payment_id: String,
    amount: String,
    amount_usd: String,
    crypto_type: String,
    network: String,
    deposit_address: String,
    fee_amount_usd: String,
    qr_code: String,
    time_remaining: String,
    expires_at: String,
    transaction_hash: Option<String>,
    is_pending: bool,
    is_confirmed: bool,
    is_expired: bool,
    is_cancelled: bool,
    is_selection_required: bool,
    sandbox: bool,
    redirect_url: Option<String>,
    supported_currencies: Vec<(String, String)>,
    customer_pays_fee: bool,
}

fn render_payment_page(data: PaymentPageData) -> String {
    let template = include_str!("../../templates/payment_page.html");
    
    // Generate status HTML in Rust to avoid broken template logic
    let status_html = if data.is_confirmed {
        "✅ Confirmed"
    } else if data.is_pending {
        "⏳ Waiting for payment"
    } else if data.is_expired {
        "❌ Expired"
    } else {
        "⏳ Pending"
    };

    let currencies_json = data.supported_currencies.iter()
        .map(|(s, n)| json!({"symbol": s, "network": n}))
        .collect::<Vec<_>>();
    let supported_currencies_json = serde_json::to_string(&currencies_json).unwrap_or_else(|_| "[]".to_string());

    // Replace basic tags
    let mut html = template
        .replace("{{payment_id}}", &encode_text(&data.payment_id))
        .replace("{{amount}}", &encode_text(&data.amount))
        .replace("{{amount_usd}}", &encode_text(&data.amount_usd))
        .replace("{{crypto_type}}", &encode_text(&data.crypto_type))
        .replace("{{network}}", &encode_text(&data.network))
        // Use data.deposit_address which comes from payment.to_address
        .replace("{{deposit_address}}", &encode_text(&data.deposit_address))
        .replace("{{fee_amount_usd}}", &encode_text(&data.fee_amount_usd))
        .replace("{{qr_code}}", &encode_text(&data.qr_code))
        .replace("{{time_remaining}}", &encode_text(&data.time_remaining))
        .replace("{{expires_at}}", &encode_text(&data.expires_at))
        .replace("{{transaction_hash}}", &encode_text(&data.transaction_hash.unwrap_or_default()))
        .replace("{{status_display}}", status_html)
        .replace("{{redirect_url}}", &encode_text(&data.redirect_url.clone().unwrap_or_default()))
        .replace("{{status}}", &encode_text(if data.is_confirmed { "CONFIRMED" } else if data.is_cancelled { "CANCELLED" } else if data.is_expired { "EXPIRED" } else if data.is_selection_required { "SELECTION_REQUIRED" } else { "PENDING" }))
        .replace("{{is_confirmed_bool}}", if data.is_confirmed { "true" } else { "false" })
        .replace("{{is_expired_bool}}", if data.is_expired { "true" } else { "false" })
        .replace("{{is_selection_required_bool}}", if data.is_selection_required { "true" } else { "false" })
        .replace("{{supported_currencies_json}}", &supported_currencies_json);

    // Handle the two main view blocks manually for old template compatibility
    // (New template uses {{#if}} blocks handled by a proper template engine or improved manual replacement)
    // For now, we will stick to the manual replacement strategy but adapted for the new simple strings
    
    // We need to implement a simple conditional replacement for Handlebars-like syntax
    // Since we are using basic string replacement, we need to be careful.
    // The new template uses {{#if (eq status "PENDING")}} syntax which is complex for simple replace.
    // To make it work without a heavy template engine, we will simplify the HTML template to use
    // specific section blocks that we can toggle on/off.
    
    // HOWEVER, the previous replace_file_content injected a template with Handlebars syntax (eq status ...)
    // which our current Rust code CANNOT parse. We need to update the Rust code to support 
    // basic block replacement or simpler template logic.
    
    // Let's implement a robust conditional renderer using unique tag pairs.
    let status = if data.is_confirmed { "CONFIRMED" } else if data.is_cancelled { "CANCELLED" } else if data.is_expired { "EXPIRED" } else if data.is_selection_required { "SELECTION_REQUIRED" } else { "PENDING" };
    
    html = toggle_status_block(&html, "PENDING", status == "PENDING");
    html = toggle_status_block(&html, "CONFIRMED", status == "CONFIRMED");
    html = toggle_status_block(&html, "EXPIRED", status == "EXPIRED");
    html = toggle_status_block(&html, "CANCELLED", status == "CANCELLED");
    html = toggle_status_block(&html, "SELECTION_REQUIRED", status == "SELECTION_REQUIRED");
    
    // Handle generic if blocks with unique IDs
    html = toggle_feature_block(&html, "sandbox", data.sandbox);
    html = toggle_feature_block(&html, "fee_amount_usd", data.customer_pays_fee && !data.fee_amount_usd.is_empty() && data.fee_amount_usd != "0.00");
    html = toggle_feature_block(&html, "redirect_url", data.redirect_url.is_some());

    html
}

fn toggle_status_block(html: &str, status: &str, show: bool) -> String {
    let tag_id = format!("status_{}", status);
    toggle_named_conditional(html, &tag_id, show)
}

fn toggle_feature_block(html: &str, feature: &str, show: bool) -> String {
    toggle_named_conditional(html, feature, show)
}

fn toggle_named_conditional(html: &str, name: &str, show: bool) -> String {
    let start_tag = format!("{{{{#if_{}}}}}", name);
    let end_tag = format!("{{{{/if_{}}}}}", name);
    
    let parts: Vec<&str> = html.split(&start_tag).collect();
    if parts.len() < 2 {
        return html.to_string();
    }
    
    let mut result = String::new();
    result.push_str(parts[0]);
    
    for part in parts.iter().skip(1) {
        if let Some(end_index) = part.find(&end_tag) {
            if show {
                // Keep the content between start and end tags
                result.push_str(&part[..end_index]);
                // Keep the rest of the string after the end tag
                result.push_str(&part[end_index + end_tag.len()..]);
            } else {
                // Discard content between start and end tags, keep the rest
                result.push_str(&part[end_index + end_tag.len()..]);
            }
        } else {
            // Fallback: if no end tag found, restore the start tag to avoid breaking the layout
            result.push_str(&start_tag);
            result.push_str(part);
        }
    }
    result
}

// ============================================================================
// Health Check
// ============================================================================

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "healthy"})))
}

// ============================================================================
// IP Whitelist Endpoints
// ============================================================================

#[derive(Deserialize)]
pub struct SetIpWhitelistRequest {
    pub ip_addresses: Vec<String>,
}

pub async fn set_ip_whitelist(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<SetIpWhitelistRequest>,
) -> impl IntoResponse {
    match state.ip_whitelist_service.set_whitelist(context.merchant_id, req.ip_addresses).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "IP whitelist updated"}))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn get_ip_whitelist(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    match state.ip_whitelist_service.get_whitelist(context.merchant_id).await {
        Ok(ips) => (StatusCode::OK, Json(json!({"ip_addresses": ips}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

// ============================================================================
// Audit Log Endpoints
// ============================================================================

#[derive(Deserialize)]
pub struct AuditLogQueryParams {
    pub from: Option<String>,
    pub to: Option<String>,
    pub action_type: Option<String>,
    pub limit: Option<i64>,
}

pub async fn get_audit_logs(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(params): Query<AuditLogQueryParams>,
) -> impl IntoResponse {
    let query = crate::services::audit_service::AuditLogQuery {
        from: params.from.and_then(|s| s.parse().ok()),
        to: params.to.and_then(|s| s.parse().ok()),
        action_type: params.action_type,
        limit: params.limit,
    };
    
    match state.audit_service.get_logs(context.merchant_id, query).await {
        Ok(logs) => (StatusCode::OK, Json(logs)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

// ============================================================================
// Balance Endpoints
// ============================================================================

pub async fn get_balance(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    // Get all balances instead of single balance
    match state.balance_service.get_all_balances(context.merchant_id, context.sandbox_mode).await {
        Ok(balance) => (StatusCode::OK, Json(balance)).into_response(),
        Err(e) => {
            tracing::error!("Failed to get balances: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response()
        },
    }
}

#[derive(Deserialize)]
pub struct BalanceHistoryQuery {
    pub limit: Option<i64>,
}

pub async fn get_balance_history(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(params): Query<BalanceHistoryQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(100).min(1000);
    
    // Balance history not available in current implementation
    (StatusCode::NOT_IMPLEMENTED, Json(json!({"error": "Balance history not implemented"}))).into_response()
}

// ============================================================================
// Withdrawal Endpoints
// ============================================================================

pub async fn create_withdrawal(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<crate::services::withdrawal_service::WithdrawalRequest>,
) -> impl IntoResponse {
    match state.withdrawal_service.create_withdrawal(context.merchant_id, req, context.sandbox_mode).await {
        Ok(withdrawal) => (StatusCode::CREATED, Json(withdrawal)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct CurrencyFilters {
    pub merchant_id: Option<i64>,
}

pub async fn get_supported_currencies(
    State(state): State<AppState>,
    Query(filters): Query<CurrencyFilters>,
) -> impl IntoResponse {
    let currencies = if let Some(merchant_id) = filters.merchant_id {
        state.currency_service.get_merchant_enabled_currencies(merchant_id).await
    } else {
        state.currency_service.get_supported_currencies().await
    };
    
    let mut currency_groups = std::collections::HashMap::new();
    
    for (crypto_type, group, network, icon_url) in currencies {
        currency_groups.entry(group).or_insert_with(Vec::new).push(json!({
            "crypto_type": crypto_type,
            "network": network,
            "icon_url": icon_url,
            "confirmations": state.currency_service.get_required_confirmations(crypto_type)
        }));
    }
    
    (StatusCode::OK, Json(json!({
        "currency_groups": currency_groups,
        "description": "USDT can be accepted on multiple networks. Native currencies are network-specific."
    }))).into_response()
}

pub async fn get_withdrawal(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(withdrawal_id): Path<String>,
) -> impl IntoResponse {
    match state.withdrawal_service.get_withdrawal(context.merchant_id, &withdrawal_id).await {
        Ok(withdrawal) => (StatusCode::OK, Json(withdrawal)).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct WithdrawalListQuery {
    pub limit: Option<i64>,
}

pub async fn list_withdrawals(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(params): Query<WithdrawalListQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(100).min(1000);
    
    match state.withdrawal_service.list_withdrawals(context.merchant_id, context.sandbox_mode).await {
        Ok(withdrawals) => (StatusCode::OK, Json(withdrawals)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn cancel_withdrawal(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(withdrawal_id): Path<String>,
) -> impl IntoResponse {
    match state.withdrawal_service.cancel_withdrawal(context.merchant_id, &withdrawal_id).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "Withdrawal cancelled"}))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

// ============================================================================
// Public API Endpoints
// ============================================================================

#[derive(Debug, Deserialize, Validate)]
pub struct ContactFormRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 1, max = 200))]
    pub subject: String,
    
    #[validate(length(min = 1, max = 2000))]
    pub message: String,
}

pub async fn submit_contact_form(
    State(state): State<AppState>,
    Json(req): Json<ContactFormRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_errors) = req.validate() {
        return (StatusCode::BAD_REQUEST, Json(json!({
            "error": "Validation failed",
            "details": validation_errors.to_string()
        }))).into_response();
    }

    // Sanitize inputs to prevent XSS and injection attacks
    let sanitized_name = sanitize_input(&req.name);
    let sanitized_email = sanitize_input(&req.email);
    let sanitized_subject = sanitize_input(&req.subject);
    let sanitized_message = sanitize_input(&req.message);

    // Additional security checks
    if contains_malicious_content(&sanitized_name) || 
       contains_malicious_content(&sanitized_subject) || 
       contains_malicious_content(&sanitized_message) {
        return (StatusCode::BAD_REQUEST, Json(json!({
            "error": "Invalid content detected"
        }))).into_response();
    }

    // Save to database
    match save_contact_message(&state.db_pool, &sanitized_name, &sanitized_email, &sanitized_subject, &sanitized_message).await {
        Ok(contact_id) => {
            (StatusCode::OK, Json(json!({
                "message": "Contact form submitted successfully",
                "status": "received",
                "id": contact_id
            }))).into_response()
        },
        Err(e) => {
            eprintln!("Failed to save contact message: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Failed to process contact form"
            }))).into_response()
        }
    }
}

fn sanitize_input(input: &str) -> String {
    input
        .trim()
        .replace(['<', '>', '"', '\'', '&'], "")
        .replace("javascript:", "")
        .replace("data:", "")
        .replace("vbscript:", "")
        .replace("onload=", "")
        .replace("onerror=", "")
        .replace("onclick=", "")
        .replace("script", "")
        .replace("iframe", "")
        .replace("object", "")
        .replace("embed", "")
        .chars()
        .filter(|c| c.is_ascii() && !c.is_control())
        .collect()
}

fn contains_malicious_content(input: &str) -> bool {
    let malicious_patterns = [
        "javascript:", "data:", "vbscript:", "onload", "onerror", "onclick",
        "<script", "</script", "eval(", "alert(", "confirm(", "prompt(",
        "document.cookie", "window.location", "innerHTML", "outerHTML",
        "exec(", "system(", "cmd", "powershell", "bash", "sh",
        "drop table", "delete from", "insert into", "update set",
        "../", "..\\", "/etc/passwd", "c:\\windows"
    ];
    
    let input_lower = input.to_lowercase();
    malicious_patterns.iter().any(|pattern| input_lower.contains(pattern))
}

async fn save_contact_message(
    pool: &PgPool,
    name: &str,
    email: &str,
    subject: &str,
    message: &str,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        INSERT INTO contact_messages (name, email, subject, message, created_at, status)
        VALUES ($1, $2, $3, $4, NOW(), 'new')
        RETURNING id
        "#,
        name,
        email,
        subject,
        message
    )
    .fetch_one(pool)
    .await?;

    Ok(result.id)
}

pub async fn get_pricing_info(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pricing_data = json!({
        "transaction_fee_percentage": state.config.default_fee_percentage,
        "daily_volume_limit_non_kyc_usd": "1000.00",
        "supported_networks": 5,
        "supported_cryptocurrencies": [
            "SOL", "USDT (SPL)", "ETH", "USDT (ERC-20)", 
            "BNB", "USDT (BEP-20)", "MATIC", "USDT (Polygon)",
            "ARB", "USDT (Arbitrum)"
        ],
        "features": {
            "instant_settlements": true,
            "real_time_notifications": true,
            "webhook_support": true,
            "sandbox_testing": true,
            "api_access": true,
            "dashboard_analytics": true
        },
        "limits": {
            "kyc_verified": {
                "daily_volume_limit": "unlimited",
                "transaction_limit": "unlimited"
            },
            "non_kyc": {
                "daily_volume_limit": "1000.00",
                "transaction_limit": "1000.00"
            }
        }
    });

    (StatusCode::OK, Json(pricing_data)).into_response()
}

// ============================================================================
// Invoice Endpoints
// ============================================================================

pub async fn create_invoice(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<CreateInvoiceRequest>,
) -> impl IntoResponse {
    match state.invoice_service.create_invoice(context.merchant_id, req).await {
        Ok(invoice) => (StatusCode::CREATED, Json(invoice)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn list_invoices(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(50);
    match state.invoice_service.list_invoices(context.merchant_id, limit).await {
        Ok(invoices) => (StatusCode::OK, Json(invoices)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn get_invoice(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(invoice_id): Path<String>,
) -> impl IntoResponse {
    match state.invoice_service.get_invoice(context.merchant_id, &invoice_id).await {
        Ok(invoice) => (StatusCode::OK, Json(invoice)).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

// ============================================================================
// Fee Setting Endpoints
// ============================================================================

#[derive(Serialize)]
pub struct GetFeeSettingResponse {
    pub fee_percentage: Decimal,
    pub customer_pays_fee: bool,
}

pub async fn get_fee_setting(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    let merchant = sqlx::query_as::<_, crate::models::merchant::Merchant>(
        "SELECT id, email, business_name, live_api_key_hash, test_api_key_hash, password_hash, fee_percentage, customer_pays_fee, is_active, sandbox_mode, settlement_mode, kyc_verified, created_at, updated_at, api_key_expires_at, daily_limit_usd, role::text as role, redirect_url FROM merchants WHERE id = $1"
    )
    .bind(context.merchant_id)
    .fetch_optional(&state.db_pool)
    .await;

    match merchant {
        Ok(Some(m)) => Json(GetFeeSettingResponse {
            fee_percentage: m.fee_percentage,
            customer_pays_fee: m.customer_pays_fee,
        }).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Merchant not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch merchant fees: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateFeeSettingRequest {
    pub fee_percentage: Option<Decimal>,
    pub customer_pays_fee: Option<bool>,
}

pub async fn update_fee_setting(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<UpdateFeeSettingRequest>,
) -> impl IntoResponse {
    // Fetch current merchant first to handle partial updates
    let merchant_result = sqlx::query_as::<_, crate::models::merchant::Merchant>(
        "SELECT id, email, business_name, live_api_key_hash, test_api_key_hash, password_hash, fee_percentage, customer_pays_fee, is_active, sandbox_mode, settlement_mode, kyc_verified, created_at, updated_at, api_key_expires_at, daily_limit_usd, role::text as role, redirect_url FROM merchants WHERE id = $1"
    )
    .bind(context.merchant_id)
    .fetch_optional(&state.db_pool)
    .await;

    let current_merchant = match merchant_result {
        Ok(Some(m)) => m,
        Ok(None) => return (StatusCode::NOT_FOUND, "Merchant not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch merchant for update: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    let new_fee = req.fee_percentage.unwrap_or(current_merchant.fee_percentage);
    let new_payer_setting = req.customer_pays_fee.unwrap_or(current_merchant.customer_pays_fee);

    let result = sqlx::query(
        "UPDATE merchants SET fee_percentage = $1, customer_pays_fee = $2, updated_at = NOW() WHERE id = $3"
    )
    .bind(new_fee)
    .bind(new_payer_setting)
    .bind(context.merchant_id)
    .execute(&state.db_pool)
    .await;

    match result {
        Ok(_) => Json(json!({
            "status": "success",
            "message": "Fee settings updated",
            "data": {
                "fee_percentage": new_fee,
                "customer_pays_fee": new_payer_setting
            }
        })).into_response(),
        Err(e) => {
            tracing::error!("Failed to update fee settings: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct CancelPaymentRequest {
    // No body needed for now, but kept for extensibility
}

pub async fn public_cancel_payment(
    State(state): State<AppState>,
    Path(payment_id): Path<String>,
) -> impl IntoResponse {
    // 1. Fetch payment
    let payment = match sqlx::query!(
        "SELECT id, merchant_id, status::text as status FROM payment_transactions WHERE payment_id = $1",
        payment_id
    )
    .fetch_optional(&state.db_pool)
    .await {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error": "Payment not found"}))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    };

    // 2. Check status
    // status is Option<String> because of the cast status::text (though it usually yields a string or null)
    // We treat None as invalid for cancellation or just handle unwrap_or_default
    let status = payment.status.unwrap_or_default();
    
    if status != "PENDING" && status != "SELECTION_REQUIRED" {
         return (StatusCode::BAD_REQUEST, Json(json!({
             "error": "Cannot cancel payment in current status",
             "current_status": status
         }))).into_response();
    }

    // 3. Update status to CANCELLED
    if let Err(e) = sqlx::query!(
        "UPDATE payment_transactions SET status = 'CANCELLED' WHERE id = $1",
        payment.id
    )
    .execute(&state.db_pool)
    .await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response();
    }

    // 4. Get redirect URL
    let merchant_settings = sqlx::query!(
        "SELECT redirect_url FROM merchants WHERE id = $1",
        payment.merchant_id
    )
    .fetch_one(&state.db_pool)
    .await;

    let redirect_url = merchant_settings.ok().and_then(|m| m.redirect_url);

    // 5. Return success with redirect info
    (StatusCode::OK, Json(json!({
        "status": "CANCELLED",
        "redirect_url": redirect_url.map(|url| {
            if url.contains('?') {
                format!("{}&status=cancelled&payment_id={}", url, payment_id)
            } else {
                format!("{}?status=cancelled&payment_id={}", url, payment_id)
            }
        })
    }))).into_response()
}
