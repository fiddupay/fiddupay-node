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

// ============================================================================
// Merchant Endpoints
// ============================================================================

#[derive(Deserialize)]
pub struct RegisterMerchantRequest {
    pub email: String,
    pub business_name: String,
}

pub async fn register_merchant(
    State(state): State<AppState>,
    Json(req): Json<RegisterMerchantRequest>,
) -> impl IntoResponse {
    match state.merchant_service.register_merchant(req.email, req.business_name).await {
        Ok(response) => (StatusCode::CREATED, Json(response)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn rotate_api_key(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    match state.merchant_service.rotate_api_key(context.merchant_id, context.api_key.clone()).await {
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
        "USDT_SPL" => CryptoType::UsdtSpl,
        "USDT_BEP20" => CryptoType::UsdtBep20,
        "USDT_ARBITRUM" => CryptoType::UsdtArbitrum,
        "USDT_POLYGON" => CryptoType::UsdtPolygon,
        "USDT_ETH" => CryptoType::UsdtEth,
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid crypto_type"}))).into_response(),
    };
    
    match state.merchant_service.set_wallet_address(context.merchant_id, crypto_type, req.address).await {
        Ok(_) => (StatusCode::OK, Json(json!({"success": true}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct SetWebhookRequest {
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
}

pub async fn get_analytics(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(query): Query<AnalyticsQuery>,
) -> impl IntoResponse {
    let from = query.from_date.unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));
    let to = query.to_date.unwrap_or_else(|| chrono::Utc::now());
    
    match state.analytics_service.get_analytics(context.merchant_id, from, to, None, None).await {
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
        Ok(_) => (StatusCode::OK, Json(json!({"success": true}))).into_response(),
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

    // Generate QR code
    let qr_data = format!("{}:{}?amount={}", 
        payment.network.to_lowercase(), 
        payment.to_address, 
        payment.amount
    );
    
    let qr_code_base64 = match generate_qr_code(&qr_data) {
        Ok(qr) => qr,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("QR code error: {}", e))).into_response(),
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
        qr_code: qr_code_base64,
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
    
    // Simple template replacement (in production, use a proper template engine)
    template
        .replace("{{payment_id}}", &data.payment_id)
        .replace("{{amount}}", &data.amount)
        .replace("{{amount_usd}}", &data.amount_usd)
        .replace("{{crypto_type}}", &data.crypto_type)
        .replace("{{network}}", &data.network)
        .replace("{{deposit_address}}", &data.deposit_address)
        .replace("{{fee_amount_usd}}", &data.fee_amount_usd)
        .replace("{{qr_code}}", &data.qr_code)
        .replace("{{time_remaining}}", &data.time_remaining)
        .replace("{{expires_at}}", &data.expires_at)
        .replace("{{transaction_hash}}", &data.transaction_hash.unwrap_or_default())
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
    Json(req): Json<SetIpWhitelistRequest>,
) -> impl IntoResponse {
    let merchant_id = 1; // TODO: Get from auth middleware
    
    match state.ip_whitelist_service.set_whitelist(merchant_id, req.ip_addresses).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "IP whitelist updated"}))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn get_ip_whitelist(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let merchant_id = 1; // TODO: Get from auth middleware
    
    match state.ip_whitelist_service.get_whitelist(merchant_id).await {
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
    Query(params): Query<AuditLogQueryParams>,
) -> impl IntoResponse {
    let merchant_id = 1; // TODO: Get from auth middleware
    
    let query = crate::services::audit_service::AuditLogQuery {
        from: params.from.and_then(|s| s.parse().ok()),
        to: params.to.and_then(|s| s.parse().ok()),
        action_type: params.action_type,
        limit: params.limit,
    };
    
    match state.audit_service.get_logs(merchant_id, query).await {
        Ok(logs) => (StatusCode::OK, Json(logs)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

// ============================================================================
// Balance Endpoints
// ============================================================================

pub async fn get_balance(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let merchant_id = 1; // TODO: Get from auth middleware
    
    match state.balance_service.get_balance(merchant_id).await {
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
    Query(params): Query<BalanceHistoryQuery>,
) -> impl IntoResponse {
    let merchant_id = 1; // TODO: Get from auth middleware
    let limit = params.limit.unwrap_or(100).min(1000);
    
    match state.balance_service.get_history(merchant_id, limit).await {
        Ok(history) => (StatusCode::OK, Json(history)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

// ============================================================================
// Withdrawal Endpoints
// ============================================================================

pub async fn create_withdrawal(
    State(state): State<AppState>,
    Json(req): Json<crate::services::withdrawal_service::WithdrawalRequest>,
) -> impl IntoResponse {
    let merchant_id = 1; // TODO: Get from auth middleware
    
    match state.withdrawal_service.create_withdrawal(merchant_id, req).await {
        Ok(withdrawal) => (StatusCode::CREATED, Json(withdrawal)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn get_withdrawal(
    State(state): State<AppState>,
    Path(withdrawal_id): Path<String>,
) -> impl IntoResponse {
    let merchant_id = 1; // TODO: Get from auth middleware
    
    match state.withdrawal_service.get_withdrawal(merchant_id, &withdrawal_id).await {
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
    Query(params): Query<WithdrawalListQuery>,
) -> impl IntoResponse {
    let merchant_id = 1; // TODO: Get from auth middleware
    let limit = params.limit.unwrap_or(100).min(1000);
    
    match state.withdrawal_service.list_withdrawals(merchant_id, limit).await {
        Ok(withdrawals) => (StatusCode::OK, Json(withdrawals)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn cancel_withdrawal(
    State(state): State<AppState>,
    Path(withdrawal_id): Path<String>,
) -> impl IntoResponse {
    let merchant_id = 1; // TODO: Get from auth middleware
    
    match state.withdrawal_service.cancel_withdrawal(merchant_id, &withdrawal_id).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "Withdrawal cancelled"}))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    }
}
