// Payment Handlers
// Payment CRUD, hosted payment page, QR code, sandbox, and refund endpoints

use crate::api::state::AppState;
use crate::middleware::auth::MerchantContext;
use crate::payment::models::{CreatePaymentRequest, PaymentFilters, CryptoType};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use serde_json::json;
use html_escape::encode_text;
use rust_decimal::Decimal;

// ============================================================================
// Payment CRUD
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
        let ct = CryptoType::from_string(ct_str).unwrap_or(CryptoType::Sol);
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
        qr_code,
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

// ============================================================================
// Payment Status (Public Polling)
// ============================================================================

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
    let payment_info = if link_id.starts_with("pay_") {
        sqlx::query_as!(
            PaymentStatusInfo,
            "SELECT payment_id, merchant_id, status FROM payment_transactions WHERE payment_id = $1",
            link_id
        )
        .fetch_optional(&state.db_pool)
        .await
    } else {
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
            
            if current_status == "PENDING" || current_status == "CONFIRMING" {
                 match state.payment_service.verify_payment_by_address(&payment.payment_id, payment.merchant_id).await {
                     Ok(true) => {
                         current_status = "CONFIRMED".to_string();
                     },
                     Ok(false) => {},
                     Err(e) => {
                         tracing::warn!("Failed to auto-verify payment {}: {}", link_id, e);
                     }
                 }
            }

            (StatusCode::OK, Json(json!({"status": current_status}))).into_response()
        },
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "Payment not found"}))).into_response(),
        Err(e) => {
            let error_msg = e.to_string();
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": error_msg}))).into_response()
        },
    }
}

// ============================================================================
// Currency Selection (Public)
// ============================================================================

#[derive(serde::Deserialize)]
pub struct SelectionRequest {
    pub crypto_type: CryptoType,
}

pub async fn finalize_payment_selection(
    State(state): State<AppState>,
    Path(link_id): Path<String>,
    Json(req): Json<SelectionRequest>,
) -> impl IntoResponse {
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

// ============================================================================
// Template Engine (Manual)
// ============================================================================

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

    let mut html = template
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
        .replace("{{status_display}}", status_html)
        .replace("{{redirect_url}}", &encode_text(&data.redirect_url.clone().unwrap_or_default()))
        .replace("{{status}}", &encode_text(if data.is_confirmed { "CONFIRMED" } else if data.is_cancelled { "CANCELLED" } else if data.is_expired { "EXPIRED" } else if data.is_selection_required { "SELECTION_REQUIRED" } else { "PENDING" }))
        .replace("{{is_confirmed_bool}}", if data.is_confirmed { "true" } else { "false" })
        .replace("{{is_expired_bool}}", if data.is_expired { "true" } else { "false" })
        .replace("{{is_selection_required_bool}}", if data.is_selection_required { "true" } else { "false" })
        .replace("{{supported_currencies_json}}", &supported_currencies_json);

    let status = if data.is_confirmed { "CONFIRMED" } else if data.is_cancelled { "CANCELLED" } else if data.is_expired { "EXPIRED" } else if data.is_selection_required { "SELECTION_REQUIRED" } else { "PENDING" };
    
    html = toggle_status_block(&html, "PENDING", status == "PENDING");
    html = toggle_status_block(&html, "CONFIRMED", status == "CONFIRMED");
    html = toggle_status_block(&html, "EXPIRED", status == "EXPIRED");
    html = toggle_status_block(&html, "CANCELLED", status == "CANCELLED");
    html = toggle_status_block(&html, "SELECTION_REQUIRED", status == "SELECTION_REQUIRED");
    
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
                result.push_str(&part[..end_index]);
                result.push_str(&part[end_index + end_tag.len()..]);
            } else {
                result.push_str(&part[end_index + end_tag.len()..]);
            }
        } else {
            result.push_str(&start_tag);
            result.push_str(part);
        }
    }
    result
}

// ============================================================================
// QR Code Helper (kept here as it's only used by payment page)
// ============================================================================

#[allow(dead_code)]
fn generate_qr_code(data: &str) -> Result<String, Box<dyn std::error::Error>> {
    use qrcode::QrCode;
    use base64::Engine;
    use image::{ImageBuffer, Luma};
    use std::io::Cursor;

    let code = QrCode::new(data.as_bytes())?;
    let size = code.width() as u32;
    let scale = 8;
    let mut image = ImageBuffer::new(size * scale, size * scale);

    for x in 0..size {
        for y in 0..size {
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
