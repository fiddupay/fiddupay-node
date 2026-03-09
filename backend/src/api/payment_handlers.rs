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
use sqlx::Row;

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
        Err(e) => (StatusCode::NOT_FOUND, Json(json!({"error": "Refund not found"}))).into_response(),
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
    let (internal_id, public_id) = match sqlx::query(
        "SELECT payment_id FROM payment_links WHERE link_id = $1"
    )
    .bind(&link_id)
    .fetch_optional(&state.db_pool)
    .await
    {
        Ok(Some(link)) => (Some(link.get::<i64, _>("payment_id")), None),
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
    let payment_res = sqlx::query(
        r#"
        SELECT merchant_id, payment_id, status, amount, amount_usd, crypto_type, network, 
               to_address, fee_amount_usd, expires_at, created_at, confirmed_at, 
               transaction_hash, partial_payments_enabled, total_paid, remaining_balance,
               last_verification_at
        FROM payment_transactions 
        WHERE id = $1 OR payment_id = $2
        "#
    )
    .bind(internal_id)
    .bind(&public_id)
    .fetch_optional(&state.db_pool)
    .await;

    let payment = match payment_res {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, Html("Payment not found".to_string())).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("Error: {}", e))).into_response(),
    };

    let p_merchant_id: i64 = payment.get("merchant_id");
    let p_payment_id: String = payment.get("payment_id");
    let p_status: String = payment.get("status");
    let p_amount: Option<Decimal> = payment.get("amount");
    let p_amount_usd: Decimal = payment.get("amount_usd");
    let p_crypto_type: Option<String> = payment.get("crypto_type");
    let p_network: Option<String> = payment.get("network");
    let p_to_address: Option<String> = payment.get("to_address");
    let p_fee_amount_usd: Decimal = payment.get("fee_amount_usd");
    let p_expires_at: chrono::DateTime<chrono::Utc> = payment.get("expires_at");
    let p_transaction_hash: Option<String> = payment.get("transaction_hash");
    let p_last_verification_at: Option<chrono::DateTime<chrono::Utc>> = payment.get("last_verification_at");

    // Generate QR code for payment (only if selection is finished)
    let qr_code = if let (Some(ct_str), Some(addr)) = (&p_crypto_type, &p_to_address) {
        let ct = CryptoType::from_string(ct_str).unwrap_or(CryptoType::Sol);
        let prefix = ct.uri_scheme();
        
        let qr_data = if let Some(amt) = p_amount {
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
    let time_remaining = if p_expires_at > now {
        let duration = p_expires_at - now;
        format!("{}m {}s", duration.num_minutes(), duration.num_seconds() % 60)
    } else {
        "Expired".to_string()
    };

    // Determine status flags
    let is_selection_required = p_status == "SELECTION_REQUIRED";
    let is_pending = p_status == "PENDING" || p_status == "CONFIRMING";
    let is_confirmed = p_status == "CONFIRMED";
    let is_cancelled = p_status == "CANCELLED";
    let is_expired = (p_status == "FAILED" || (p_expires_at < now)) && !is_confirmed && !is_cancelled;

    // Check if sandbox and get redirect_url
    let merchant_info_res = sqlx::query(
        "SELECT sandbox_mode, redirect_url, customer_pays_fee FROM merchants WHERE id = $1"
    )
    .bind(p_merchant_id)
    .fetch_optional(&state.db_pool)
    .await;
    
    let merchant_info = merchant_info_res.ok().flatten();
    
    let sandbox = merchant_info.as_ref().map(|m| m.get::<bool, _>("sandbox_mode")).unwrap_or(false);
    let redirect_url = merchant_info.as_ref().and_then(|m| m.get::<Option<String>, _>("redirect_url"));
    let customer_pays_fee = merchant_info.as_ref().map(|m| m.get::<bool, _>("customer_pays_fee")).unwrap_or(true);

    // Smart Verification: Trigger address scan in background if pending (respecting cooldown)
    if is_pending {
        let needs_verification = match p_last_verification_at {
            Some(last_v) => (chrono::Utc::now() - last_v) > chrono::Duration::seconds(20),
            None => true,
        };

        if needs_verification {
            tracing::info!("Triggering background smart verification for payment {}", link_id);
            let p_id_clone = p_payment_id.clone();
            let m_id_clone = p_merchant_id;
            let svc_clone = state.payment_service.clone();
            tokio::spawn(async move {
                if let Err(e) = svc_clone.verify_payment_by_address(&p_id_clone, m_id_clone).await {
                    tracing::error!("Background smart verification failed for payment {}: {}", p_id_clone, e);
                }
            });
        }
    }

    // Fetch supported currencies if needed
    let supported_currencies: Vec<(String, String)> = if is_selection_required {
        let currencies_res = sqlx::query(
             "SELECT crypto_type, network FROM merchant_wallets WHERE merchant_id = $1 AND is_active = true"
        )
        .bind(p_merchant_id)
        .fetch_all(&state.db_pool)
        .await;

        currencies_res.unwrap_or_default()
        .into_iter()
        .map(|r| (r.get::<String, _>("crypto_type"), r.get::<String, _>("network")))
        .collect()
    } else {
        vec![]
    };

    // Render logic
    let html = render_payment_page(PaymentPageData {
        payment_id: p_payment_id,
        amount: p_amount.unwrap_or_default().to_string(),
        amount_usd: p_amount_usd.to_string(),
        crypto_type: p_crypto_type.unwrap_or_default(),
        network: p_network.unwrap_or_default(),
        deposit_address: p_to_address.unwrap_or_default(),
        fee_amount_usd: p_fee_amount_usd.to_string(),
        qr_code,
        time_remaining,
        expires_at: p_expires_at.to_rfc3339(),
        transaction_hash: p_transaction_hash,
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
    last_verification_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn payment_status(
    State(state): State<AppState>,
    Path(link_id): Path<String>,
) -> impl IntoResponse {
    let payment_info: Result<Option<PaymentStatusInfo>, sqlx::Error> = if link_id.starts_with("pay_") {
        sqlx::query_as::<_, PaymentStatusInfo>(
            "SELECT payment_id, merchant_id, status, last_verification_at FROM payment_transactions WHERE payment_id = $1"
        )
        .bind(&link_id)
        .fetch_optional(&state.db_pool)
        .await
    } else {
        sqlx::query_as::<_, PaymentStatusInfo>(
            r#"
            SELECT pt.payment_id, pt.merchant_id, pt.status, pt.last_verification_at
            FROM payment_transactions pt
            JOIN payment_links pl ON pl.payment_id = pt.id
            WHERE pl.link_id = $1
            "#
        )
        .bind(&link_id)
        .fetch_optional(&state.db_pool)
        .await
    };

    match payment_info {
        Ok(Some(payment)) => {
            let current_status = payment.status.clone();
            
            if current_status == "PENDING" || current_status == "CONFIRMING" {
                 let needs_verification = match payment.last_verification_at {
                     Some(last_v) => (chrono::Utc::now() - last_v) > chrono::Duration::seconds(20),
                     None => true,
                 };

                  if needs_verification {
                      let p_id_clone = payment.payment_id.clone();
                      let m_id_clone = payment.merchant_id;
                      let svc_clone = state.payment_service.clone();
                      tokio::spawn(async move {
                          if let Err(e) = svc_clone.verify_payment_by_address(&p_id_clone, m_id_clone).await {
                              tracing::error!("Background status verification failed for payment {}: {}", p_id_clone, e);
                          }
                      });
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
    let payment_link_res = sqlx::query(
        "SELECT payment_id FROM payment_links WHERE link_id = $1"
    )
    .bind(&link_id)
    .fetch_optional(&pool)
    .await;

    let payment_link = match payment_link_res {
        Ok(Some(link)) => link,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error": "Payment link not found"}))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    };

    let pl_payment_id: i64 = payment_link.get("payment_id");

    // 2. Get payment details
    let payment_record_res = sqlx::query_as::<_, crate::models::payment::Payment>(
        r#"
        SELECT id, payment_id, merchant_id, amount, amount_usd, crypto_type, network,
               status, to_address, from_address, created_at, expires_at, confirmed_at,
               confirmations, required_confirmations, description, metadata,
               transaction_hash, webhook_url, fee_percentage, fee_amount, fee_amount_usd,
                user_id, subscription_id, block_number, partial_payments_enabled,
                total_paid, remaining_balance, is_non_custodial, last_verification_at
        FROM payment_transactions 
        WHERE id = $1
        "#
    )
    .bind(pl_payment_id)
    .fetch_optional(&pool)
    .await;

    let payment_record = match payment_record_res {
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
    use rust_decimal::prelude::FromPrimitive;
    let price_decimal = Decimal::from_f64(price).unwrap_or(Decimal::ONE);

    let amount_usd = payment_record.amount_usd;
    let amount_crypto = amount_usd / price_decimal;
    let fee_amount_crypto = payment_record.fee_amount_usd / price_decimal;
    let network = crypto_type.network();

    // 4. Update payment record
    if let Err(e) = sqlx::query(
        r#"
        UPDATE payment_transactions 
        SET crypto_type = $1, amount = $2, to_address = $3, network = $4, fee_amount = $5, status = 'PENDING'
        WHERE id = $6
        "#
    )
    .bind(crypto_type.to_string())
    .bind(amount_crypto)
    .bind(&to_address)
    .bind(network)
    .bind(fee_amount_crypto)
    .bind(payment_record.id)
    .execute(&pool)
    .await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response();
    }

    (StatusCode::OK, Json(json!({"message": "Selection finalized", "crypto_type": crypto_type.to_string()}))).into_response()
}

/// Lightweight trigger to start a background verification scan
pub async fn verify_payment_trigger(
    State(state): State<AppState>,
    Path(link_id): Path<String>,
) -> impl IntoResponse {
    // Look up payment_id from link_id
    let payment_res = if link_id.starts_with("pay_") {
        sqlx::query("SELECT id, payment_id, merchant_id, status, last_verification_at FROM payment_transactions WHERE payment_id = $1")
            .bind(&link_id)
            .fetch_optional(&state.db_pool)
            .await
    } else {
        sqlx::query(
            r#"
            SELECT pt.id, pt.payment_id, pt.merchant_id, pt.status, pt.last_verification_at
            FROM payment_transactions pt
            JOIN payment_links pl ON pl.payment_id = pt.id
            WHERE pl.link_id = $1
            "#
        )
        .bind(&link_id)
        .fetch_optional(&state.db_pool)
        .await
    };

    match payment_res {
        Ok(Some(row)) => {
            let p_payment_id: String = row.get("payment_id");
            let p_merchant_id: i64 = row.get("merchant_id");
            let p_status: String = row.get("status");

            if p_status == "PENDING" || p_status == "CONFIRMING" {
                let last_v_opt: Option<chrono::DateTime<chrono::Utc>> = row.get("last_verification_at");
                let needs_verification = match last_v_opt {
                    Some(last_v) => (chrono::Utc::now() - last_v) > chrono::Duration::seconds(20),
                    None => true,
                };

                if needs_verification {
                    let p_id_clone = p_payment_id.clone();
                    let m_id_clone = p_merchant_id;
                    let svc_clone = state.payment_service.clone();
                    tokio::spawn(async move {
                        if let Err(e) = svc_clone.verify_payment_by_address(&p_id_clone, m_id_clone).await {
                            tracing::error!("Manual trigger verification failed for payment {}: {}", p_id_clone, e);
                        }
                    });
                }
            }

            (StatusCode::ACCEPTED, Json(json!({"status": "verification_started"}))).into_response()
        },
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "Payment not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
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
