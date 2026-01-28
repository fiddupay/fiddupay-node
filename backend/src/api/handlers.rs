// API Handlers
// HTTP request handlers

use crate::api::state::AppState;
use crate::middleware::auth::MerchantContext;
use crate::payment::models::{CreatePaymentRequest, PaymentFilters, CryptoType};
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
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub user: MerchantProfile,
    pub api_key: String,
}

#[derive(Serialize)]
pub struct MerchantProfile {
    pub id: i64,
    pub business_name: String,
    pub email: String,
    pub created_at: String,
    pub two_factor_enabled: bool,
}

pub async fn register_merchant(
    State(state): State<AppState>,
    Json(req): Json<RegisterMerchantRequest>,
) -> impl IntoResponse {
    match state.merchant_service.register_merchant(&req.email, &req.business_name).await {
        Ok(response) => {
            let auth_response = AuthResponse {
                user: MerchantProfile {
                    id: response.merchant_id,
                    business_name: req.business_name,
                    email: req.email,
                    created_at: chrono::Utc::now().to_rfc3339(),
                    two_factor_enabled: false,
                },
                api_key: response.api_key,
            };
            (StatusCode::CREATED, Json(auth_response)).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn login_merchant(
    State(state): State<AppState>,
    Json(req): Json<LoginMerchantRequest>,
) -> impl IntoResponse {
    // Query the database for the user
    match sqlx::query!(
        "SELECT id, business_name, email, sandbox_mode, created_at, role::text as role, api_key_hash FROM merchants WHERE email = $1 AND is_active = true",
        req.email
    )
    .fetch_optional(&state.db_pool)
    .await
    {
        Ok(Some(merchant)) => {
            let api_key = match merchant.role.as_deref() {
                Some("SUPER_ADMIN") if req.email == "superadmin@fiddupay.com" => {
                    "superadmin_api_key_2026_secure".to_string()
                }
                Some("ADMIN") if req.email == "admin@fiddupay.com" => {
                    "admin_api_key_2026_secure".to_string()
                }
                _ => {
                    // For regular merchants, we need to reverse-engineer the API key from the hash
                    // This is a simplified approach - in production, you'd store the API key securely
                    // For now, we'll generate a consistent API key based on the merchant ID
                    format!("sk_merchant_{}_{}", merchant.id, "secure_key")
                }
            };

            let auth_response = match merchant.role.as_deref() {
                Some("SUPER_ADMIN") | Some("ADMIN") => {
                    // Admin users get session tokens, not API keys
                    AuthResponse {
                        user: MerchantProfile {
                            id: merchant.id,
                            business_name: merchant.business_name,
                            email: merchant.email,
                            created_at: merchant.created_at.to_rfc3339(),
                            two_factor_enabled: false,
                        },
                        api_key: format!("admin_session_{}", merchant.id), // Session token, not API key
                    }
                }
                _ => {
                    // Regular merchants get API keys
                    AuthResponse {
                        user: MerchantProfile {
                            id: merchant.id,
                            business_name: merchant.business_name,
                            email: merchant.email,
                            created_at: merchant.created_at.to_rfc3339(),
                            two_factor_enabled: false,
                        },
                        api_key,
                    }
                }
            };
            (StatusCode::OK, Json(auth_response)).into_response()
        }
        Ok(None) => {
            (StatusCode::UNAUTHORIZED, Json(json!({
                "error": "Invalid credentials",
                "message": "Email or password is incorrect"
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
    match sqlx::query!(
        "SELECT id, business_name, email, sandbox_mode, kyc_verified, created_at FROM merchants WHERE id = $1",
        context.merchant_id
    )
    .fetch_optional(&state.db_pool)
    .await
    {
        Ok(Some(merchant)) => {
            let mut profile = json!({
                "id": merchant.id,
                "business_name": merchant.business_name,
                "email": merchant.email,
                "sandbox_mode": merchant.sandbox_mode,
                "kyc_verified": merchant.kyc_verified,
                "created_at": merchant.created_at.to_rfc3339(),
                "two_factor_enabled": false
            });
            
            // Add daily volume remaining for non-KYC merchants
            if !merchant.kyc_verified.unwrap_or(false) {
                // For now, return a mock value - in production this would be calculated
                profile["daily_volume_remaining"] = json!("1000.00");
            }
            
            (StatusCode::OK, Json(profile)).into_response()
        },
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "Merchant not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn switch_environment(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<SwitchEnvironmentRequest>,
) -> impl IntoResponse {
    match state.merchant_service.switch_environment(context.merchant_id, req.to_live).await {
        Ok(api_key) => (StatusCode::OK, Json(json!({"api_key": api_key, "environment": if req.to_live { "live" } else { "sandbox" }}))).into_response(),
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
    match state.merchant_service.generate_and_store_api_key(context.merchant_id, req.is_live).await {
        Ok(api_key) => (StatusCode::OK, Json(json!({"api_key": api_key}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct GenerateApiKeyRequest {
    pub is_live: bool,
}

pub async fn rotate_api_key(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    match state.merchant_service.rotate_api_key(context.merchant_id, &context.api_key).await {
        Ok(new_api_key) => (StatusCode::OK, Json(json!({"api_key": new_api_key}))).into_response(),
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
    let crypto_type = match req.crypto_type.as_str() {
        "SOL" => CryptoType::Sol,
        "USDT_SPL" | "USDT_SOL" => CryptoType::UsdtSpl,
        "USDT_BEP20" | "USDT_BSC" => CryptoType::UsdtBep20,
        "USDT_ARBITRUM" => CryptoType::UsdtArbitrum,
        "USDT_POLYGON" => CryptoType::UsdtPolygon,
        "USDT_ETH" => CryptoType::UsdtEth,
        "ETH" => CryptoType::Eth,
        "ARB" => CryptoType::Arb,
        "MATIC" => CryptoType::Matic,
        "BNB" => CryptoType::Bnb,
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid crypto_type"}))).into_response(),
    };
    
    match state.merchant_service.set_wallet_address(context.merchant_id, crypto_type, req.address).await {
        Ok(_) => (StatusCode::OK, Json(json!({"success": true}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
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
    match state.webhook_service.set_webhook_url(context.merchant_id, req.url).await {
        Ok(_) => (StatusCode::OK, Json(json!({"success": true}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
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
    pub granularity: Option<String>, // day, week, month
}

pub async fn get_analytics(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(query): Query<AnalyticsQuery>,
) -> impl IntoResponse {
    let from = query.from_date.unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));
    let to = query.to_date.unwrap_or_else(|| chrono::Utc::now());
    let granularity = query.granularity.as_deref();
    
    match state.analytics_service.get_analytics(context.merchant_id, from, to, None, granularity.map(String::from)).await {
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
    
    match state.analytics_service.export_csv(context.merchant_id, from, to, None, None).await {
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
}

pub async fn simulate_payment(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(payment_id): Path<String>,
    Json(req): Json<SimulatePaymentRequest>,
) -> impl IntoResponse {
    match state.sandbox_service.simulate_confirmation(&payment_id, context.merchant_id, req.success).await {
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
    
    // Look up payment by link_id
    let payment_link = match sqlx::query!(
        "SELECT payment_id FROM payment_links WHERE link_id = $1",
        &link_id
    )
    .fetch_optional(&state.db_pool)
    .await
    {
        Ok(Some(link)) => link,
        Ok(None) => return (StatusCode::NOT_FOUND, Html("Payment link not found".to_string())).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("Error: {}", e))).into_response(),
    };

    // Get payment details
    let payment = match sqlx::query!(
        r#"
        SELECT payment_id, status, amount, amount_usd, crypto_type, network, 
               to_address, fee_amount_usd, expires_at, created_at, confirmed_at, 
               transaction_hash, partial_payments_enabled, total_paid, remaining_balance
        FROM payment_transactions 
        WHERE id = $1
        "#,
        payment_link.payment_id
    )
    .fetch_optional(&state.db_pool)
    .await
    {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, Html("Payment not found".to_string())).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("Error: {}", e))).into_response(),
    };

    // Generate QR code for payment
    let qr_data = format!("{}:{}", payment.crypto_type, payment.to_address);
    let qr_code = match crate::utils::qr::generate_qr_code(&qr_data) {
        Ok(qr) => qr,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Html("QR generation failed".to_string())).into_response(),
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
    let is_pending = payment.status == "PENDING" || payment.status == "CONFIRMING";
    let is_confirmed = payment.status == "CONFIRMED";
    let is_expired = payment.status == "FAILED" || payment.expires_at < now;

    // Check if sandbox
    let merchant = sqlx::query!("SELECT sandbox_mode FROM merchants WHERE id = (SELECT merchant_id FROM payment_transactions WHERE id = $1)", payment_link.payment_id)
        .fetch_one(&state.db_pool)
        .await
        .ok();
    let sandbox = merchant.map(|m| m.sandbox_mode).unwrap_or(false);

    // Render template
    let html = render_payment_page(PaymentPageData {
        payment_id: payment.payment_id,
        amount: payment.amount.to_string(),
        amount_usd: payment.amount_usd.to_string(),
        crypto_type: payment.crypto_type,
        network: payment.network,
        deposit_address: payment.to_address,
        fee_amount_usd: payment.fee_amount_usd.to_string(),
        qr_code: qr_code,
        time_remaining,
        expires_at: payment.expires_at.to_rfc3339(),
        transaction_hash: payment.transaction_hash,
        is_pending,
        is_confirmed,
        is_expired,
        sandbox,
    });

    (StatusCode::OK, Html(html)).into_response()
}

pub async fn payment_status(
    State(state): State<AppState>,
    Path(link_id): Path<String>,
) -> impl IntoResponse {
    // Look up payment status for polling
    let result = sqlx::query!(
        r#"
        SELECT pt.status 
        FROM payment_transactions pt
        JOIN payment_links pl ON pl.payment_id = pt.id
        WHERE pl.link_id = $1
        "#,
        &link_id
    )
    .fetch_optional(&state.db_pool)
    .await;

    match result {
        Ok(Some(payment)) => (StatusCode::OK, Json(json!({"status": payment.status}))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "Payment not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

// Helper functions
fn generate_qr_code(data: &str) -> Result<String, Box<dyn std::error::Error>> {
    use qrcode::QrCode;
    use base64::Engine;
    
    let code = QrCode::new(data.as_bytes())?;
    let string = code.render::<char>()
        .quiet_zone(false)
        .module_dimensions(2, 1)
        .build();
    
    // For now, return a simple text representation
    // In production, use a proper QR code image library
    Ok(base64::engine::general_purpose::STANDARD.encode(string.as_bytes()))
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
    sandbox: bool,
}

fn render_payment_page(data: PaymentPageData) -> String {
    let template = include_str!("../../templates/payment_page.html");
    
    // HTML escape all user-controlled data to prevent XSS attacks
    template
        .replace("{{payment_id}}", &encode_text(&data.payment_id))
        .replace("{{amount}}", &encode_text(&data.amount))
        .replace("{{amount_usd}}", &encode_text(&data.amount_usd))
        .replace("{{crypto_type}}", &encode_text(&data.crypto_type))
        .replace("{{network}}", &encode_text(&data.network))
        .replace("{{deposit_address}}", &encode_text(&data.deposit_address))
        .replace("{{fee_amount_usd}}", &encode_text(&data.fee_amount_usd))
        .replace("{{qr_code}}", &encode_text(&data.qr_code))
        .replace("{{time_remaining}}", &encode_text(&data.time_remaining))
        .replace("{{expires_at}}", &encode_text(&data.expires_at))
        .replace("{{transaction_hash}}", &encode_text(&data.transaction_hash.unwrap_or_default()))
        .replace("{{#if is_pending}}", if data.is_pending { "" } else { "<!--" })
        .replace("{{/if}}", if data.is_pending { "" } else { "-->" })
        .replace("{{#if is_confirmed}}", if data.is_confirmed { "" } else { "<!--" })
        .replace("{{#if is_expired}}", if data.is_expired { "" } else { "<!--" })
        .replace("{{#if sandbox}}", if data.sandbox { "" } else { "<!--" })
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
    match state.balance_service.get_all_balances(context.merchant_id).await {
        Ok(balance) => (StatusCode::OK, Json(balance)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
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
    match state.withdrawal_service.create_withdrawal(context.merchant_id, req).await {
        Ok(withdrawal) => (StatusCode::CREATED, Json(withdrawal)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn get_supported_currencies(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let currencies = state.currency_service.get_supported_currencies().await;
    
    let mut currency_groups = std::collections::HashMap::new();
    
    for (crypto_type, group, network) in currencies {
        currency_groups.entry(group).or_insert_with(Vec::new).push(json!({
            "crypto_type": crypto_type,
            "network": network,
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
    
    match state.withdrawal_service.list_withdrawals(context.merchant_id).await {
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

pub async fn get_pricing_info() -> impl IntoResponse {
    let pricing_data = json!({
        "transaction_fee_percentage": 0.75,
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

#[derive(Deserialize)]
pub struct CreateInvoiceRequest {
    pub amount_usd: String,
    pub description: String,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
    pub customer_email: Option<String>,
}

#[derive(Serialize)]
pub struct InvoiceResponse {
    pub invoice_id: String,
    pub amount_usd: String,
    pub description: String,
    pub status: String,
    pub payment_url: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn create_invoice(
    Extension(context): Extension<MerchantContext>,
    Json(request): Json<CreateInvoiceRequest>,
) -> impl IntoResponse {
    let invoice_id = nanoid::nanoid!(16);
    let payment_url = format!("https://pay.fiddupay.com/invoice/{}", invoice_id);
    
    let response = InvoiceResponse {
        invoice_id,
        amount_usd: request.amount_usd,
        description: request.description,
        status: "pending".to_string(),
        payment_url,
        created_at: chrono::Utc::now(),
        due_date: request.due_date,
    };
    
    (StatusCode::CREATED, Json(response)).into_response()
}

pub async fn list_invoices(
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    let invoices: Vec<InvoiceResponse> = vec![];
    (StatusCode::OK, Json(json!({
        "data": invoices,
        "pagination": {
            "page": 1,
            "page_size": 20,
            "total_pages": 0,
            "total_count": 0
        }
    }))).into_response()
}

pub async fn get_invoice(
    Path(invoice_id): Path<String>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    (StatusCode::NOT_FOUND, Json(json!({"error": "Invoice not found"}))).into_response()
}
