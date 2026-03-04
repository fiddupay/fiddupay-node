// Settings Handlers
// Merchant settings, webhook, fee, and IP whitelist management

use crate::api::state::AppState;
use crate::middleware::auth::MerchantContext;
use crate::payment::models::CryptoType;
use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;
use rust_decimal::Decimal;
use sqlx::Row;

use crate::middleware::validation::validate_webhook_url;
use crate::models::merchant::Merchant;

// ============================================================================
// Profile & Readiness
// ============================================================================

pub async fn get_merchant_profile(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    let merchant_id = context.merchant_id;

    // 1. Fetch BASIC merchant info
    let merchant = match sqlx::query(
        r#"
        SELECT id, business_name, email, sandbox_mode, settlement_mode, 
               kyc_verified, daily_limit_usd, created_at, redirect_url,
               test_api_key_hash, live_api_key_hash
        FROM merchants
        WHERE id = $1
        "#
    )
    .bind(merchant_id)
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
    let (webhook_url, webhook_format) = match sqlx::query(
        r#"SELECT url, payload_format FROM webhook_configs WHERE merchant_id = $1 AND is_active = true"#
    )
    .bind(merchant_id)
    .fetch_optional(&state.db_pool)
    .await {
        Ok(Some(cfg)) => (Some(cfg.get::<String, _>("url")), Some(cfg.get::<String, _>("payload_format"))),
        Ok(None) => (None, None),
        Err(e) => {
            eprintln!("Profile DB Error (Webhook Fetch): {:?}", e);
            (None, None) // Non-critical failure
        }
    };

    // 3. Construct profile with masked API key if it's a dashboard session
    let m_sandbox_mode: bool = merchant.get("sandbox_mode");
    let m_test_api_key_hash: Option<String> = merchant.get("test_api_key_hash");
    let m_live_api_key_hash: Option<String> = merchant.get("live_api_key_hash");
    let m_id: i64 = merchant.get("id");
    let m_business_name: String = merchant.get("business_name");
    let m_email: String = merchant.get("email");
    let m_settlement_mode: String = merchant.get("settlement_mode");
    let m_kyc_verified: bool = merchant.get("kyc_verified");
    let m_daily_limit_usd: Option<Decimal> = merchant.get("daily_limit_usd");
    let m_created_at: chrono::DateTime<chrono::Utc> = merchant.get("created_at");
    let m_redirect_url: Option<String> = merchant.get("redirect_url");

    let display_key = if context.api_key == "DASHBOARD_SESSION" {
        let hash_opt = if m_sandbox_mode { &m_test_api_key_hash } else { &m_live_api_key_hash };
        let is_valid = hash_opt.as_ref().map(|h: &String| h != "PENDING" && !h.is_empty()).unwrap_or(false);
        
        if !is_valid {
            "Not generated".to_string()
        } else {
            format!("sk_{}_********", if m_sandbox_mode { "test" } else { "live" })
        }
    } else {
        context.api_key.clone()
    };

    let mut profile = json!({
        "id": m_id,
        "business_name": m_business_name,
        "email": m_email,
        "api_key": display_key,
        "redirect_url": m_redirect_url,
        "webhook_url": webhook_url,
        "webhook_format": webhook_format,
        "sandbox_mode": m_sandbox_mode,
        "settlement_mode": m_settlement_mode,
        "kyc_verified": m_kyc_verified,
        "daily_limit_usd": m_daily_limit_usd.map(|d: Decimal| d.to_string()),
        "created_at": m_created_at.to_rfc3339(),
        "two_factor_enabled": false
    });
    
    // 4. Calculate daily volume remaining
    let remaining = state.merchant_service.get_daily_volume_remaining(
        m_id,
        m_kyc_verified,
        m_daily_limit_usd
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
    let merchant_res = sqlx::query("SELECT sandbox_mode, settlement_mode, kyc_verified FROM merchants WHERE id = $1")
        .bind(merchant_id)
        .fetch_one(&state.db_pool)
        .await;
    
    let merchant = match merchant_res {
        Ok(m) => m,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    };

    let m_sandbox_mode: bool = merchant.get("sandbox_mode");
    let m_settlement_mode: String = merchant.get("settlement_mode");
    let m_kyc_verified: bool = merchant.get("kyc_verified");

    let wallets_res = if m_settlement_mode == "forwarding" {
        wallet_service.get_forwarding_configs(merchant_id, m_sandbox_mode).await
    } else {
        wallet_service.get_wallet_configs(merchant_id, m_sandbox_mode).await
    };
    
    let currencies_res = currency_service.get_merchant_enabled_currencies(merchant_id).await;

    let wallets = wallets_res.unwrap_or_default();
    let enabled_currencies = currencies_res;

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
    let security_alerts_res: Result<i64, sqlx::Error> = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM security_alerts WHERE merchant_id = $1 AND acknowledged = FALSE")
        .bind(merchant_id)
        .fetch_one(&state.db_pool)
        .await;

    let security_alerts = security_alerts_res.unwrap_or(0);

    if security_alerts > 0 {
        issues.push(format!("{} active security alerts require attention", security_alerts));
    }

    // 5. Build final response
    let response = json!({
        "is_ready": is_ready && security_alerts == 0,
        "environment": if m_sandbox_mode { "sandbox" } else { "live" },
        "settlement_mode": m_settlement_mode,
        "kyc_verified": m_kyc_verified,
        "network_coverage": network_status,
        "security": {
            "active_alerts": security_alerts
        },
        "issues": issues
    });

    (StatusCode::OK, Json(response)).into_response()
}

// ============================================================================
// Environment & API Keys
// ============================================================================

#[derive(Deserialize)]
pub struct SwitchEnvironmentRequest {
    pub to_live: bool,
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
pub struct GenerateApiKeyRequest {
    pub is_live: bool,
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
pub struct RotateApiKeyRequest {
    pub is_live: bool,
}

pub async fn rotate_api_key(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    req: Option<Json<RotateApiKeyRequest>>,
) -> impl IntoResponse {
    let result = if context.api_key == "DASHBOARD_SESSION" {
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
        state.merchant_service.rotate_api_key(context.merchant_id, &context.api_key).await
    };

    match result {
        Ok(new_api_key) => (StatusCode::OK, Json(json!({"api_key": new_api_key}))).into_response(),
        Err(e) => e.into_response(),
    }
}

// ============================================================================
// Wallet (deprecated set_wallet)
// ============================================================================

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

// ============================================================================
// Unified Settings
// ============================================================================

#[derive(Deserialize, Validate)]
pub struct UnifiedSettingsRequest {
    #[validate(custom(function = "validate_optional_webhook_url"))]
    pub webhook_url: Option<String>,
    pub redirect_url: Option<String>,
    pub webhook_format: Option<String>,
    pub settlement_mode: Option<String>,
    pub customer_pays_fee: Option<bool>,
    pub fee_percentage: Option<Decimal>,
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

    // 2. Update Fee Settings if provided
    if req.fee_percentage.is_some() || req.customer_pays_fee.is_some() {
        if let Err(e) = sqlx::query(
            "UPDATE merchants SET fee_percentage = COALESCE($1, fee_percentage), customer_pays_fee = COALESCE($2, customer_pays_fee), updated_at = NOW() WHERE id = $3"
        )
        .bind(req.fee_percentage)
        .bind(req.customer_pays_fee)
        .bind(context.merchant_id)
        .execute(&state.db_pool)
        .await {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response();
        }
    }

    // 3. Update Webhook if provided
    if req.webhook_url.is_some() || req.webhook_format.is_some() {
        if let Err(e) = state.webhook_service.set_webhook_url(
            context.merchant_id, 
            req.webhook_url,
            req.webhook_format.clone()
        ).await {
            return (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response();
        }
    }

    // 4. Update IP Whitelist if provided
    if let Some(ips) = req.ip_whitelist {
        if let Err(e) = state.ip_whitelist_service.set_whitelist(context.merchant_id, ips).await {
            return (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response();
        }
    }

    // 5. Rotate Webhook Secret if requested
    if req.rotate_webhook_secret.unwrap_or(false) {
        let new_secret = hex::encode(rand::random::<[u8; 32]>());
        if let Err(e) = sqlx::query(
            "UPDATE webhook_configs SET signing_secret = $1, updated_at = NOW() WHERE merchant_id = $2"
        )
        .bind(&new_secret)
        .bind(context.merchant_id)
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



pub async fn get_merchant_settings(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    let merchant_id = context.merchant_id;
    
    // 1. Get core merchant settings
    let merchant = match sqlx::query(
        "SELECT settlement_mode, customer_pays_fee, sandbox_mode, redirect_url FROM merchants WHERE id = $1"
    )
    .bind(merchant_id)
    .fetch_one(&state.db_pool)
    .await {
        Ok(m) => m,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    };

    // 2. Get webhook config
    let webhook_config = sqlx::query(
        "SELECT url, payload_format, signing_secret FROM webhook_configs WHERE merchant_id = $1 AND is_active = true"
    )
    .bind(merchant_id)
    .fetch_optional(&state.db_pool)
    .await
    .unwrap_or(None);

    // 3. Get IP whitelist
    let ip_whitelist: Vec<String> = sqlx::query_scalar::<_, String>(
        "SELECT ip_address FROM ip_whitelist WHERE merchant_id = $1"
    )
    .bind(merchant_id)
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    let m_settlement_mode: String = merchant.get("settlement_mode");
    let m_customer_pays_fee: bool = merchant.get("customer_pays_fee");
    let m_sandbox_mode: bool = merchant.get("sandbox_mode");
    let m_redirect_url: Option<String> = merchant.get("redirect_url");

    (StatusCode::OK, Json(json!({
        "webhook_url": webhook_config.as_ref().map(|c| c.get::<String, _>("url")),
        "webhook_format": webhook_config.as_ref().map(|c| c.get::<String, _>("payload_format")),
        "webhook_signing_secret": webhook_config.as_ref().map(|c| c.get::<String, _>("signing_secret")),
        "settlement_mode": m_settlement_mode,
        "customer_pays_fee": m_customer_pays_fee,
        "sandbox_mode": m_sandbox_mode,
        "redirect_url": m_redirect_url,
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
        timestamp: chrono::Utc::now().timestamp(),
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
// IP Whitelist (GET only — updates now via PATCH /settings)
// ============================================================================

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
// Fee Settings (GET only — updates now via PATCH /settings)
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
    let merchant_res = sqlx::query_as::<_, crate::models::merchant::Merchant>(
        "SELECT id, email, business_name, live_api_key_hash, test_api_key_hash, password_hash, fee_percentage, customer_pays_fee, is_active, sandbox_mode, settlement_mode, kyc_verified, created_at, updated_at, api_key_expires_at, daily_limit_usd, role::text as role, redirect_url FROM merchants WHERE id = $1"
    )
    .bind(context.merchant_id)
    .fetch_optional(&state.db_pool)
    .await;

    match merchant_res {
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

// ============================================================================
// Invoice Management
// ============================================================================

pub async fn create_invoice(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<crate::services::invoice_service::CreateInvoiceRequest>,
) -> impl IntoResponse {
    match state.invoice_service.create_invoice(context.merchant_id, req).await {
        Ok(invoice) => (StatusCode::CREATED, Json(invoice)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn list_invoices(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(params): Query<std::collections::HashMap<String, String>>,
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
    axum::extract::Path(invoice_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.invoice_service.get_invoice(context.merchant_id, &invoice_id).await {
        Ok(invoice) => (StatusCode::OK, Json(invoice)).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response(),
    }
}
