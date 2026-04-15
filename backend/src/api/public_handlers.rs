// Public Handlers
// Public-facing HTTP request handlers (no auth required)

use crate::api::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use validator::Validate;

// ============================================================================
// Root & Health
// ============================================================================

pub async fn root_handler() -> &'static str {
    "backend is running"
}

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "healthy"})))
}

// ============================================================================
// Supported Currencies
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CurrencyFilters {
    pub merchant_id: Option<i64>,
}

pub async fn get_supported_currencies(
    State(state): State<AppState>,
    Query(filters): Query<CurrencyFilters>,
) -> impl IntoResponse {
    let currencies = if let Some(merchant_id) = filters.merchant_id {
        state
            .currency_service
            .get_merchant_enabled_currencies(merchant_id)
            .await
    } else {
        state.currency_service.get_supported_currencies().await
    };

    let mut currency_groups = std::collections::HashMap::new();

    for (crypto_type, group, network, icon_url) in currencies {
        // Parse into CryptoType to fetch price
        let price = match crypto_type.parse::<crate::payment::models::CryptoType>() {
            Ok(ct) => state.price_service.get_price(ct).await.unwrap_or(1.0),
            Err(_) => 1.0,
        };

        currency_groups
            .entry(group)
            .or_insert_with(Vec::new)
            .push(json!({
                "crypto_type": crypto_type,
                "network": network,
                "icon_url": icon_url,
                "confirmations": state.currency_service.get_required_confirmations(crypto_type),
                "price_usd": price
            }));
    }

    (StatusCode::OK, Json(json!({
        "currency_groups": currency_groups,
        "description": "USDT can be accepted on multiple networks. Native currencies are network-specific."
    }))).into_response()
}

// ============================================================================
// Contact Form
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
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Validation failed",
                "details": validation_errors.to_string()
            })),
        )
            .into_response();
    }

    // Sanitize inputs to prevent XSS and injection attacks
    let sanitized_name = sanitize_input(&req.name);
    let sanitized_email = sanitize_input(&req.email);
    let sanitized_subject = sanitize_input(&req.subject);
    let sanitized_message = sanitize_input(&req.message);

    // Additional security checks
    if contains_malicious_content(&sanitized_name)
        || contains_malicious_content(&sanitized_subject)
        || contains_malicious_content(&sanitized_message)
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid content detected"
            })),
        )
            .into_response();
    }

    // Save to database
    match save_contact_message(
        &state.db_pool,
        &sanitized_name,
        &sanitized_email,
        &sanitized_subject,
        &sanitized_message,
    )
    .await
    {
        Ok(contact_id) => (
            StatusCode::OK,
            Json(json!({
                "message": "Contact form submitted successfully",
                "status": "received",
                "id": contact_id
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = ?e, "Failed to save contact message");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to process contact form"
                })),
            )
                .into_response()
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
        "javascript:",
        "data:",
        "vbscript:",
        "onload",
        "onerror",
        "onclick",
        "<script",
        "</script",
        "eval(",
        "alert(",
        "confirm(",
        "prompt(",
        "document.cookie",
        "window.location",
        "innerHTML",
        "outerHTML",
        "exec(",
        "system(",
        "cmd",
        "powershell",
        "bash",
        "sh",
        "drop table",
        "delete from",
        "insert into",
        "update set",
        "../",
        "..\\",
        "/etc/passwd",
        "c:\\windows",
    ];

    let input_lower = input.to_lowercase();
    malicious_patterns
        .iter()
        .any(|pattern| input_lower.contains(pattern))
}

async fn save_contact_message(
    pool: &PgPool,
    name: &str,
    email: &str,
    subject: &str,
    message: &str,
) -> Result<i64, sqlx::Error> {
    use sqlx::Row;
    let result = sqlx::query(
        r#"
        INSERT INTO contact_messages (name, email, subject, message, created_at, status)
        VALUES ($1, $2, $3, $4, NOW(), 'new')
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(email)
    .bind(subject)
    .bind(message)
    .fetch_one(pool)
    .await?;

    Ok(result.get("id"))
}

// ============================================================================
// Pricing
// ============================================================================

pub async fn get_pricing_info(State(state): State<AppState>) -> impl IntoResponse {
    let limit_str = state.config.daily_volume_limit_non_kyc_usd.to_string();
    let pricing_data = json!({
        "transaction_fee_percentage": state.config.default_fee_percentage,
        "daily_volume_limit_non_kyc_usd": limit_str,
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
                "daily_volume_limit": limit_str,
                "transaction_limit": limit_str
            }
        }
    });

    (StatusCode::OK, Json(pricing_data)).into_response()
}

// ============================================================================
// Public Cancel Payment
// ============================================================================

#[derive(Deserialize)]
pub struct CancelPaymentRequest {
    // No body needed for now, but kept for extensibility
}

pub async fn public_cancel_payment(
    State(state): State<AppState>,
    Path(payment_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    // 1. Fetch payment
    let payment_res = sqlx::query(
        "SELECT id, merchant_id, status::text as status FROM payment_transactions WHERE payment_id = $1"
    )
    .bind(&payment_id)
    .fetch_optional(&state.db_pool)
    .await;

    let payment = match payment_res {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Payment not found"})),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    };

    use sqlx::Row;
    let p_id: i64 = payment.get("id");
    let p_merchant_id: i64 = payment.get("merchant_id");

    // 2. Check status
    let status: Option<String> = payment.try_get("status").ok();
    let status = status.unwrap_or_default();

    if status != "PENDING" && status != "SELECTION_REQUIRED" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Cannot cancel payment in current status",
                "current_status": status
            })),
        )
            .into_response();
    }

    // 3. Update status to CANCELLED
    let update_res =
        sqlx::query("UPDATE payment_transactions SET status = 'CANCELLED' WHERE id = $1")
            .bind(p_id)
            .execute(&state.db_pool)
            .await;

    if let Err(e) = update_res {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response();
    }

    // 4. Get redirect URL
    let merchant_settings = sqlx::query("SELECT redirect_url FROM merchants WHERE id = $1")
        .bind(p_merchant_id)
        .fetch_optional(&state.db_pool)
        .await;

    let redirect_url: Option<String> = merchant_settings
        .ok()
        .flatten()
        .and_then(|m| m.get("redirect_url"));

    // 5. Return success with redirect info
    (
        StatusCode::OK,
        Json(json!({
            "status": "CANCELLED",
            "redirect_url": redirect_url.map(|url: String| {
                if url.contains('?') {
                    format!("{}&status=cancelled&payment_id={}", url, payment_id)
                } else {
                    format!("{}?status=cancelled&payment_id={}", url, payment_id)
                }
            })
        })),
    )
        .into_response()
}
