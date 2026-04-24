// Settings Handlers
// Merchant settings, webhook, fee, and IP whitelist management

use crate::api::state::AppState;
use crate::error::ServiceError;
use crate::middleware::auth::{require_kyc_tier, MerchantContext};
use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use validator::Validate;

use crate::middleware::validation::validate_webhook_url;

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
               test_api_key_hash, live_api_key_hash, wallets_locked, customer_wallets_locked,
               transaction_pin_hash, pin_setup_at, low_balance_threshold_usd, low_balance_alerts_enabled,
               kyc_tier, social_handles, username, pay_id,
               fee_percentage, customer_pays_fee, business_license_number, business_certificate_url, nin_bvn_hash
        FROM merchants
        WHERE id = $1
        "#,
    )
    .bind(merchant_id)
    .fetch_optional(&state.db_pool)
    .await
    {
        Ok(Some(m)) => m,
        Ok(None) => return ServiceError::MerchantNotFound.into_response(),
        Err(e) => {
            tracing::error!(error = ?e, "Failed to fetch merchant profile");
            return ServiceError::Database(e).into_response();
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
            tracing::warn!(error = ?e, "Failed to fetch webhook configuration");
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
    let m_wallets_locked: bool = merchant.get("wallets_locked");
    let m_customer_wallets_locked: bool = merchant.get("customer_wallets_locked");
    let m_transaction_pin_hash: Option<String> = merchant.get("transaction_pin_hash");
    let m_pin_setup_at: Option<chrono::DateTime<chrono::Utc>> = merchant.get("pin_setup_at");
    let m_low_balance_threshold_usd: Decimal = merchant.get("low_balance_threshold_usd");
    let m_low_balance_alerts_enabled: bool = merchant.get("low_balance_alerts_enabled");

    let display_key = if context.api_key == "DASHBOARD_SESSION" {
        let hash_opt = if m_sandbox_mode {
            &m_test_api_key_hash
        } else {
            &m_live_api_key_hash
        };
        let is_valid = hash_opt
            .as_deref()
            .map(|h: &str| h != "PENDING" && !h.is_empty())
            .unwrap_or(false);

        if !is_valid {
            "Not generated".to_string()
        } else {
            format!(
                "sk_{}_********",
                if m_sandbox_mode { "test" } else { "live" }
            )
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
        "wallets_locked": m_wallets_locked,
        "customer_wallets_locked": m_customer_wallets_locked,
        "sandbox_mode": m_sandbox_mode,
        "settlement_mode": m_settlement_mode,
        "kyc_verified": m_kyc_verified,
        "daily_limit_usd": m_daily_limit_usd
            .or(Some(if m_kyc_verified { state.config.daily_volume_limit_verified_usd } else { state.config.daily_volume_limit_non_kyc_usd }))
            .map(|d: Decimal| d.to_string()),
        "created_at": m_created_at.to_rfc3339(),
        "two_factor_enabled": false,
        "has_transaction_pin": m_transaction_pin_hash.is_some(),
        "pin_setup_at": m_pin_setup_at.map(|d| d.to_rfc3339()),
        "low_balance_threshold_usd": m_low_balance_threshold_usd.to_string(),
        "low_balance_alerts_enabled": m_low_balance_alerts_enabled,
        "kyc_tier": merchant.get::<i32, _>("kyc_tier"),
        "social_handles": merchant.get::<serde_json::Value, _>("social_handles"),
        "username": merchant.get::<Option<String>, _>("username"),
        "pay_id": merchant.get::<Option<String>, _>("pay_id"),
        "managed_mode_only": state.config.managed_mode_only,
        "fee_percentage": merchant.get::<Decimal, _>("fee_percentage").to_string(),
        "customer_pays_fee": merchant.get::<bool, _>("customer_pays_fee"),
        "business_license_number": merchant.get::<Option<String>, _>("business_license_number"),
        "business_certificate_url": merchant.get::<Option<String>, _>("business_certificate_url"),
        "has_national_id": merchant.get::<Option<String>, _>("nin_bvn_hash").is_some(),
        "withdrawal_fee_percentage": state.config.withdrawal_fee_percentage,
        "withdrawal_enabled": state.config.withdrawal_enabled,
        "trust_score": crate::services::trust_score_service::TrustScoreService::calculate_score(
            merchant.get::<i32, _>("kyc_tier"),
            &merchant.get::<serde_json::Value, _>("social_handles")
        )
    });

    // 4. Calculate daily volume remaining
    let remaining = state
        .merchant_service
        .get_daily_volume_remaining(m_id, m_kyc_verified, m_daily_limit_usd)
        .await
        .unwrap_or(Decimal::ZERO);

    profile["daily_volume_remaining"] = json!(remaining.to_string());

    (StatusCode::OK, Json(json!({ "user": profile }))).into_response()
}

pub async fn get_merchant_readiness(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    let merchant_id = context.merchant_id;
    let wallet_service = crate::services::wallet_config_service::WalletConfigService::new(
        state.db_pool.clone(),
        state.config.clone(),
    );
    let currency_service = crate::services::currency_service::CurrencyService::new(
        state.db_pool.clone(),
        std::sync::Arc::new(state.config.clone()),
    );

    // 1. Fetch data
    let merchant_res = sqlx::query(
        "SELECT sandbox_mode, settlement_mode, kyc_verified FROM merchants WHERE id = $1",
    )
    .bind(merchant_id)
    .fetch_one(&state.db_pool)
    .await;

    let merchant = match merchant_res {
        Ok(m) => m,
        Err(e) => return ServiceError::Database(e).into_response(),
    };

    let m_sandbox_mode: bool = merchant.get("sandbox_mode");
    let m_settlement_mode: String = merchant.get("settlement_mode");
    let m_kyc_verified: bool = merchant.get("kyc_verified");

    let wallets_res = if m_settlement_mode == "forwarding" {
        wallet_service
            .get_forwarding_configs(merchant_id, m_sandbox_mode)
            .await
    } else {
        wallet_service
            .get_wallet_configs(merchant_id, m_sandbox_mode)
            .await
    };

    let currencies_res = currency_service
        .get_merchant_enabled_currencies(merchant_id)
        .await;

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
            }
            None => {
                network_status[network] = json!({
                    "status": "missing",
                    "action_required": "configure_wallet"
                });
                issues.push(format!(
                    "Wallet not configured for enabled network: {}",
                    network
                ));
                is_ready = false;
            }
        }
    }

    // 4. Security status check
    let security_alerts_res: Result<i64, sqlx::Error> = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM security_alerts WHERE merchant_id = $1 AND acknowledged = FALSE",
    )
    .bind(merchant_id)
    .fetch_one(&state.db_pool)
    .await;

    let security_alerts = security_alerts_res.unwrap_or(0);

    if security_alerts > 0 {
        issues.push(format!(
            "{} active security alerts require attention",
            security_alerts
        ));
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
    match state
        .merchant_service
        .switch_environment(context.merchant_id, req.to_live)
        .await
    {
        Ok(maybe_key) => {
            let mut response = json!({
                "environment": if req.to_live { "live" } else { "sandbox" },
                "sandbox_mode": !req.to_live
            });
            if let Some(api_key) = maybe_key {
                response["api_key"] = json!(api_key);
            }
            // Log switch and trace
            let _ = state
                .audit_service
                .log_event(
                    context.merchant_id,
                    "environment_switch",
                    Some(&format!(
                        "Switched to {}",
                        if req.to_live { "live" } else { "sandbox" }
                    )),
                    Some(json!({"to_live": req.to_live})),
                )
                .await;
            tracing::info!(
                "EVENT: environment_switch | Merchant: {} | To: {}",
                context.merchant_id,
                if req.to_live { "live" } else { "sandbox" }
            );

            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => ServiceError::Internal(e.to_string()).into_response(),
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
    // 0. Requirement: Must be Tier 1 to generate LIVE keys
    if req.is_live {
        if let Err(e) = require_kyc_tier(&context, 1) {
            return e.into_response();
        }
    }
    match state
        .merchant_service
        .generate_and_store_api_key_with_expiry(context.merchant_id, req.is_live, None)
        .await
    {
        Ok(api_key) => {
            // Log key generation and trace
            let _ = state
                .audit_service
                .log_event(
                    context.merchant_id,
                    "api_key_generation",
                    Some(&format!(
                        "Generated new {} API key",
                        if req.is_live { "live" } else { "test" }
                    )),
                    Some(json!({"is_live": req.is_live})),
                )
                .await;
            tracing::info!(
                "EVENT: api_key_generation | Merchant: {} | Live: {}",
                context.merchant_id,
                req.is_live
            );

            (StatusCode::OK, Json(json!({"api_key": api_key}))).into_response()
        }
        Err(e) => ServiceError::Internal(format!("{}", e)).into_response(),
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
                state
                    .merchant_service
                    .rotate_api_key_by_env(context.merchant_id, payload.is_live)
                    .await
            }
            None => {
                return ServiceError::BadRequest(
                    "Dashboard rotation requires 'is_live' parameter".to_string(),
                )
                .into_response()
            }
        }
    } else {
        state
            .merchant_service
            .rotate_api_key(context.merchant_id, &context.api_key)
            .await
    };

    // 0. Requirement: Must be Tier 1 to rotate/access LIVE keys
    if let Ok(ref key) = result {
        if key.starts_with("sk_live_") && context.kyc_tier == 0 {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": "KYC upgrade required",
                    "message": "Tier 1 verification is required to manage Live API keys.",
                    "code": "KYC_INSUFFICIENT_TIER"
                })),
            )
                .into_response();
        }
    }

    match result {
        Ok(new_api_key) => {
            // Log key rotation and trace
            let is_live = new_api_key.starts_with("sk_live_");
            let _ = state
                .audit_service
                .log_event(
                    context.merchant_id,
                    "api_key_rotation",
                    Some(&format!(
                        "Rotated {} API key",
                        if is_live { "live" } else { "test" }
                    )),
                    Some(json!({"is_live": is_live})),
                )
                .await;
            tracing::info!(
                "EVENT: api_key_rotation | Merchant: {} | Live: {}",
                context.merchant_id,
                is_live
            );

            (StatusCode::OK, Json(json!({"api_key": new_api_key}))).into_response()
        }
        Err(e) => e.into_response(),
    }
}

// ============================================================================
// Unified Settings
// ============================================================================

#[derive(Deserialize, Serialize, Validate)]
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
    pub low_balance_threshold_usd: Option<Decimal>,
    pub low_balance_alerts_enabled: Option<bool>,
}

fn validate_optional_webhook_url(url: &str) -> Result<(), validator::ValidationError> {
    validate_webhook_url(url)
}

pub async fn update_merchant_settings(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<UnifiedSettingsRequest>,
) -> impl IntoResponse {
    // 1. Update Merchant core settings
    if req.settlement_mode.is_some()
        || req.customer_pays_fee.is_some()
        || req.sandbox_mode.is_some()
        || req.redirect_url.is_some()
        || req.low_balance_alerts_enabled.is_some()
    {
        // 0. Requirement: Must be Tier 1 to switch to LIVE mode
        if req.sandbox_mode == Some(false) {
            if let Err(e) = require_kyc_tier(&context, 1) {
                return e.into_response();
            }
        }
        if let Err(e) = state
            .merchant_service
            .update_settings(
                context.merchant_id,
                req.settlement_mode.clone(),
                req.customer_pays_fee,
                req.sandbox_mode,
                req.redirect_url.clone(),
                req.low_balance_alerts_enabled,
            )
            .await
        {
            return ServiceError::Internal(e.to_string()).into_response();
        }
    }

    // 2. Update Fee & Balance Settings if provided
    if req.fee_percentage.is_some()
        || req.customer_pays_fee.is_some()
        || req.low_balance_threshold_usd.is_some()
    {
        if let Err(e) = sqlx::query(
            r#"
            UPDATE merchants 
            SET fee_percentage = COALESCE($1, fee_percentage), 
                customer_pays_fee = COALESCE($2, customer_pays_fee), 
                low_balance_threshold_usd = COALESCE($3, low_balance_threshold_usd),
                updated_at = NOW() 
            WHERE id = $4
            "#,
        )
        .bind(req.fee_percentage)
        .bind(req.customer_pays_fee)
        .bind(req.low_balance_threshold_usd)
        .bind(context.merchant_id)
        .execute(&state.db_pool)
        .await
        {
            return ServiceError::Database(e).into_response();
        }
    }

    // 3. Update Webhook if provided
    if req.webhook_url.is_some() || req.webhook_format.is_some() {
        if let Err(e) = state
            .webhook_service
            .set_webhook_url(
                context.merchant_id,
                req.webhook_url.clone(),
                req.webhook_format.clone(),
            )
            .await
        {
            return e.into_response();
        }
    }

    // 4. Update IP Whitelist if provided
    if let Some(ips) = req.ip_whitelist.clone() {
        if let Err(e) = state
            .ip_whitelist_service
            .set_whitelist(context.merchant_id, ips)
            .await
        {
            return ServiceError::BadRequest(e.to_string()).into_response();
        }
    }

    // 5. Rotate Webhook Secret if requested
    let mut new_webhook_secret = None;
    if req.rotate_webhook_secret.unwrap_or(false) {
        let secret = hex::encode(rand::random::<[u8; 32]>());

        // Try update first (this updates all active endpoints for the merchant)
        let update_res = sqlx::query(
            "UPDATE webhook_configs SET signing_secret = $1, updated_at = NOW() WHERE merchant_id = $2"
        )
        .bind(&secret)
        .bind(context.merchant_id)
        .execute(&state.db_pool)
        .await;

        match update_res {
            Ok(res) if res.rows_affected() == 0 => {
                // If no configs exist, create a default "standard" placeholder so the secret is persisted
                let _ = sqlx::query(
                    r#"
                    INSERT INTO webhook_configs (merchant_id, signing_secret, payload_format, is_active, url)
                    VALUES ($1, $2, 'standard', false, '')
                    "#
                )
                .bind(context.merchant_id)
                .bind(&secret)
                .execute(&state.db_pool)
                .await;
            }
            Err(e) => {
                tracing::error!(error = ?e, "Failed to rotate webhook secret");
                return ServiceError::Database(e).into_response();
            }
            _ => {}
        }
        new_webhook_secret = Some(secret);
    }

    // Log settings update and trace
    let _ = state
        .audit_service
        .log_event(
            context.merchant_id,
            "settings_update",
            Some("Updated merchant profile settings"),
            Some(json!(req)),
        )
        .await;
    tracing::info!("EVENT: settings_update | Merchant: {}", context.merchant_id);

    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "message": "Settings updated successfully",
            "new_webhook_secret": new_webhook_secret
        })),
    )
        .into_response()
}

pub async fn get_merchant_settings(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    let merchant_id = context.merchant_id;

    // 1. Get core merchant settings
    let merchant = match sqlx::query(
        "SELECT settlement_mode, customer_pays_fee, sandbox_mode, redirect_url, low_balance_alerts_enabled FROM merchants WHERE id = $1"
    )
    .bind(merchant_id)
    .fetch_one(&state.db_pool)
    .await {
        Ok(m) => m,
        Err(e) => return ServiceError::Database(e).into_response(),
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
        "SELECT ip_address FROM ip_whitelist WHERE merchant_id = $1",
    )
    .bind(merchant_id)
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    let m_settlement_mode: String = merchant.get("settlement_mode");
    let m_customer_pays_fee: bool = merchant.get("customer_pays_fee");
    let m_sandbox_mode: bool = merchant.get("sandbox_mode");
    let m_redirect_url: Option<String> = merchant.get("redirect_url");

    (
        StatusCode::OK,
        Json(json!({
            "webhook_url": webhook_config.as_ref().map(|c| c.get::<String, _>("url")),
            "webhook_format": webhook_config.as_ref().map(|c| c.get::<String, _>("payload_format")),
            "webhook_signing_secret": webhook_config.as_ref().and_then(|c| {
                c.try_get::<String, _>("signing_secret").ok().map(|s| {
                    if s.len() > 12 {
                        format!("{}**********", &s[..12])
                    } else {
                        "**********".to_string()
                    }
                })
            }),
            "settlement_mode": m_settlement_mode,
            "customer_pays_fee": m_customer_pays_fee,
            "sandbox_mode": m_sandbox_mode,
            "redirect_url": m_redirect_url,
            "low_balance_alerts_enabled": merchant.get::<bool, _>("low_balance_alerts_enabled"),
            "ip_whitelist": ip_whitelist
        })),
    )
        .into_response()
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
        customer_external_id: None,
        timestamp: chrono::Utc::now().timestamp(),
    };

    if let Err(e) = state
        .webhook_service
        .queue_webhook(merchant_id, None, payload)
        .await
    {
        return ServiceError::Internal(e.to_string()).into_response();
    }

    // Log audit event
    let _ = state
        .audit_service
        .log_event(
            context.merchant_id,
            "test_webhook_trigger",
            Some("Triggered test webhook delivery"),
            None,
        )
        .await;

    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "message": "Test webhook queued for delivery"
        })),
    )
        .into_response()
}

// ============================================================================
// IP Whitelist (GET only — updates now via PATCH /settings)
// ============================================================================

pub async fn get_ip_whitelist(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    match state
        .ip_whitelist_service
        .get_whitelist(context.merchant_id)
        .await
    {
        Ok(ips) => (StatusCode::OK, Json(json!({"ip_addresses": ips}))).into_response(),
        Err(e) => ServiceError::Internal(e.to_string()).into_response(),
    }
}

// ============================================================================
// Fee Settings (GET only — updates now via PATCH /settings)
// ============================================================================

#[derive(Serialize, sqlx::FromRow)]
pub struct GetFeeSettingResponse {
    pub fee_percentage: Decimal,
    pub customer_pays_fee: bool,
}

pub async fn get_fee_setting(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    let merchant_res = sqlx::query_as::<_, GetFeeSettingResponse>(
        r#"
        SELECT fee_percentage, customer_pays_fee 
        FROM merchants WHERE id = $1
        "#,
    )
    .bind(context.merchant_id)
    .fetch_optional(&state.db_pool)
    .await;

    match merchant_res {
        Ok(Some(m)) => (StatusCode::OK, Json(m)).into_response(),
        Ok(None) => ServiceError::MerchantNotFound.into_response(),
        Err(e) => {
            tracing::error!(
                "Failed to fetch merchant fees for merchant {}: {:?}",
                context.merchant_id,
                e
            );
            ServiceError::Database(e).into_response()
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
    match state
        .invoice_service
        .create_invoice(context.merchant_id, req)
        .await
    {
        Ok(invoice) => {
            // Log audit event
            let _ = state
                .audit_service
                .log_event(
                    context.merchant_id,
                    "invoice_creation",
                    Some(&format!("Created invoice {}", invoice.invoice_id)),
                    Some(json!({
                        "invoice_id": invoice.invoice_id,
                        "amount": invoice.total
                    })),
                )
                .await;

            (StatusCode::CREATED, Json(invoice)).into_response()
        }
        Err(e) => ServiceError::Internal(e.to_string()).into_response(),
    }
}

pub async fn list_invoices(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let limit = params
        .get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(50);
    match state
        .invoice_service
        .list_invoices(context.merchant_id, limit)
        .await
    {
        Ok(invoices) => (StatusCode::OK, Json(invoices)).into_response(),
        Err(e) => ServiceError::Internal(e.to_string()).into_response(),
    }
}

pub async fn get_invoice(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    axum::extract::Path(invoice_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state
        .invoice_service
        .get_invoice(context.merchant_id, &invoice_id)
        .await
    {
        Ok(invoice) => (StatusCode::OK, Json(invoice)).into_response(),
        Err(e) => ServiceError::Internal(e.to_string()).into_response(),
    }
}

// ============================================================================
// Wallet Security
// ============================================================================

#[derive(Deserialize)]
pub struct SetLockRequest {
    pub locked: bool,
    pub password: Option<String>,
}

pub async fn toggle_wallet_lock(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<SetLockRequest>,
) -> impl IntoResponse {
    // 1. Verify password if provided (required for security)
    let password = match req.password {
        Some(p) => p,
        None => {
            return ServiceError::BadRequest("Password required for this action".to_string())
                .into_response()
        }
    };

    // 2. Fetch password hash
    let password_hash: Option<String> = match sqlx::query_scalar::<_, Option<String>>(
        "SELECT password_hash FROM merchants WHERE id = $1",
    )
    .bind(context.merchant_id)
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(h) => h,
        Err(e) => return ServiceError::Database(e).into_response(),
    };

    let hash_str = match password_hash {
        Some(h) => h,
        None => {
            return ServiceError::Unauthorized(
                "Account does not have a password configured".to_string(),
            )
            .into_response()
        }
    };

    // 3. Verify password
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    let parsed_hash = match PasswordHash::new(&hash_str) {
        Ok(h) => h,
        Err(_) => {
            return ServiceError::Internal("Invalid stored password format".to_string())
                .into_response()
        }
    };

    if Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return ServiceError::Unauthorized("Invalid password".to_string()).into_response();
    }

    // 4. Proceed with lock toggle
    match state
        .merchant_service
        .set_wallet_lock(context.merchant_id, req.locked)
        .await
    {
        Ok(_) => {
            // Log lock toggle and trace
            let _ = state
                .audit_service
                .log_event(
                    context.merchant_id,
                    "wallet_lock_toggle",
                    Some(&format!(
                        "Merchant wallets {}",
                        if req.locked { "locked" } else { "unlocked" }
                    )),
                    Some(json!({"locked": req.locked})),
                )
                .await;
            tracing::info!(
                "EVENT: wallet_lock_toggle | Merchant: {} | Locked: {}",
                context.merchant_id,
                req.locked
            );

            (
                StatusCode::OK,
                Json(json!({"status": "success", "locked": req.locked})),
            )
                .into_response()
        }
        Err(e) => ServiceError::Internal(e.to_string()).into_response(),
    }
}

pub async fn toggle_customer_wallet_lock(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<SetLockRequest>,
) -> impl IntoResponse {
    // 1. Verify password if provided
    let password = match req.password {
        Some(p) => p,
        None => {
            return ServiceError::BadRequest("Password required for this action".to_string())
                .into_response()
        }
    };

    // 2. Fetch password hash
    let password_hash: Option<String> = match sqlx::query_scalar::<_, Option<String>>(
        "SELECT password_hash FROM merchants WHERE id = $1",
    )
    .bind(context.merchant_id)
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(h) => h,
        Err(e) => return ServiceError::Database(e).into_response(),
    };

    let hash_str = match password_hash {
        Some(h) => h,
        None => {
            return ServiceError::Unauthorized(
                "Account does not have a password configured".to_string(),
            )
            .into_response()
        }
    };

    // 3. Verify password
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    let parsed_hash = match PasswordHash::new(&hash_str) {
        Ok(h) => h,
        Err(_) => {
            return ServiceError::Internal("Invalid stored password format".to_string())
                .into_response()
        }
    };

    if Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return ServiceError::Unauthorized("Invalid password".to_string()).into_response();
    }

    match state
        .merchant_service
        .set_customer_wallet_lock(context.merchant_id, req.locked)
        .await
    {
        Ok(_) => {
            // Log lock toggle and trace
            let _ = state
                .audit_service
                .log_event(
                    context.merchant_id,
                    "customer_wallet_lock_toggle",
                    Some(&format!(
                        "Customer wallets {}",
                        if req.locked { "locked" } else { "unlocked" }
                    )),
                    Some(json!({"locked": req.locked})),
                )
                .await;
            tracing::info!(
                "EVENT: customer_wallet_lock_toggle | Merchant: {} | Locked: {}",
                context.merchant_id,
                req.locked
            );

            (
                StatusCode::OK,
                Json(json!({"status": "success", "locked": req.locked})),
            )
                .into_response()
        }
        Err(e) => ServiceError::Internal(e.to_string()).into_response(),
    }
}

// ============================================================================
// Transaction PIN Management
// ============================================================================

#[derive(serde::Deserialize)]
pub struct SetTransactionPinRequest {
    pub pin: String,
}

pub async fn set_transaction_pin(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<SetTransactionPinRequest>,
) -> impl IntoResponse {
    // 1. Validate PIN format (4 digits)
    if req.pin.len() != 4 || !req.pin.chars().all(|c| c.is_ascii_digit()) {
        return ServiceError::BadRequest("PIN must be 4 digits".to_string()).into_response();
    }

    // 2. Hash PIN
    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
    use rand::rngs::OsRng;
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let pin_hash = match argon2.hash_password(req.pin.as_bytes(), &salt) {
        Ok(h) => h.to_string(),
        Err(e) => return ServiceError::Internal(format!("Hashing error: {}", e)).into_response(),
    };

    // 3. Update database
    if let Err(e) = sqlx::query(
        "UPDATE merchants SET transaction_pin_hash = $1, pin_setup_at = NOW(), updated_at = NOW() WHERE id = $2"
    )
    .bind(pin_hash)
    .bind(context.merchant_id)
    .execute(&state.db_pool)
    .await {
        return ServiceError::Database(e).into_response();
    }

    // 4. Log event
    let _ = state
        .audit_service
        .log_event(
            context.merchant_id,
            "transaction_pin_set",
            Some("Merchant set transaction PIN"),
            None,
        )
        .await;

    (
        StatusCode::OK,
        Json(json!({"status": "success", "message": "Transaction PIN set successfully"})),
    )
        .into_response()
}

#[derive(serde::Deserialize)]
pub struct VerifyTransactionPinRequest {
    pub pin: String,
}

pub async fn verify_transaction_pin(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<VerifyTransactionPinRequest>,
) -> impl IntoResponse {
    // 1. Fetch PIN hash
    let pin_hash: Option<String> = match sqlx::query_scalar::<_, Option<String>>(
        "SELECT transaction_pin_hash FROM merchants WHERE id = $1",
    )
    .bind(context.merchant_id)
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(h) => h,
        Err(e) => return ServiceError::Database(e).into_response(),
    };

    let hash_str = match pin_hash {
        Some(h) => h,
        None => {
            return ServiceError::BadRequest("Transaction PIN not configured".to_string())
                .into_response()
        }
    };

    // 2. Verify PIN
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    let parsed_hash = match PasswordHash::new(&hash_str) {
        Ok(h) => h,
        Err(_) => {
            return ServiceError::Internal("Invalid stored PIN format".to_string()).into_response()
        }
    };

    if Argon2::default()
        .verify_password(req.pin.as_bytes(), &parsed_hash)
        .is_err()
    {
        return ServiceError::Unauthorized("Invalid PIN".to_string()).into_response();
    }

    (
        StatusCode::OK,
        Json(json!({"status": "success", "message": "PIN verified"})),
    )
        .into_response()
}
// ============================================================================
// Trust & Identity Handlers
// ============================================================================

#[derive(Deserialize)]
pub struct ClaimUsernameRequest {
    pub username: String,
}

pub async fn claim_username(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<ClaimUsernameRequest>,
) -> impl IntoResponse {
    match state
        .merchant_service
        .claim_username(context.merchant_id, &req.username)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "status": "success", "message": "Username claimed" })),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
}

#[derive(Deserialize)]
pub struct UpdateKycDraftRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub gender: Option<String>,
    pub phone_number: Option<String>,
    pub country: Option<String>,
    pub social_handles: Option<serde_json::Value>,
    pub business_country: Option<String>,
    pub business_license_number: Option<String>,
    pub business_certificate_url: Option<String>,
    pub nin_bvn: Option<String>,
}

pub async fn update_kyc_draft(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<UpdateKycDraftRequest>,
) -> impl IntoResponse {
    match state
        .merchant_service
        .save_kyc_data(
            context.merchant_id,
            req.first_name,
            req.last_name,
            req.gender,
            req.phone_number,
            req.country,
            req.social_handles,
            req.business_country,
            req.business_license_number,
            req.business_certificate_url,
            req.nin_bvn,
        )
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "status": "success", "message": "KYC draft updated" })),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
}
