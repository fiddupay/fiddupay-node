// Webhook Service
// Business logic for webhook delivery

use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::Sha256;
use sqlx::PgPool;
use std::time::Duration;
use tracing::{info, warn};
use uuid::Uuid;
use url::Url;

use crate::error::ServiceError;
use crate::models::webhook::WebhookPayload;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct WebhookService {
    db_pool: PgPool,
    http_client: Client,
    signing_key: String,
}

impl WebhookService {
    pub fn new(db_pool: PgPool, signing_key: String) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            db_pool,
            http_client,
            signing_key,
        }
    }

    pub fn get_signing_key(&self) -> &str {
        &self.signing_key
    }

    /// Configure webhook URL and format for a merchant
    pub async fn set_webhook_url(
        &self,
        merchant_id: i64,
        url: Option<String>,
        payload_format: Option<String>,
    ) -> Result<(), ServiceError> {
        let format = payload_format.unwrap_or_else(|| "standard".to_string());
        
        if let Some(url_str) = url {
            // Validate URL format
            let parsed_url = Url::parse(&url_str)
                .map_err(|_| ServiceError::InvalidWebhookUrl("Invalid URL format".to_string()))?;

            // Validate HTTPS scheme
            if parsed_url.scheme() != "https" {
                return Err(ServiceError::InvalidWebhookUrl(
                    "Webhook URL must use HTTPS protocol".to_string()
                ));
            }

            sqlx::query(
                r#"
                INSERT INTO webhook_configs (merchant_id, url, payload_format, is_active, signing_secret)
                VALUES ($1, $2, $3, true, $4)
                ON CONFLICT (merchant_id) 
                DO UPDATE SET url = $2, payload_format = $3, is_active = true, updated_at = NOW()
                "#
            )
            .bind(merchant_id)
            .bind(&url_str)
            .bind(&format)
            .bind(hex::encode(rand::random::<[u8; 32]>()))
            .execute(&self.db_pool)
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to set webhook URL: {}", e)))?;
        } else {
             sqlx::query(
                r#"
                UPDATE webhook_configs 
                SET payload_format = $1, updated_at = NOW()
                WHERE merchant_id = $2
                "#
            )
            .bind(&format)
            .bind(merchant_id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to update webhook format: {}", e)))?;
        }

        Ok(())
    }

    /// Generate HMAC-SHA256 signature for webhook payload
    fn generate_signature(&self, payload: &str, timestamp: i64, secret: &str) -> String {
        let message = format!("{}.{}", timestamp, payload);
        
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(message.as_bytes());
        
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    /// Send webhook notification with optional signature
    pub async fn send_webhook(
        &self,
        url: &str,
        payload_value: &serde_json::Value,
        secret: &str,
        skip_signature: bool,
    ) -> Result<(u16, String), ServiceError> {
        let payload_json = payload_value.to_string();
        
        // Build request — add HMAC signature headers only for standard format
        let mut request = self.http_client
            .post(url)
            .header("Content-Type", "application/json");

        if !skip_signature {
            let timestamp = Utc::now().timestamp();
            let signature = self.generate_signature(&payload_json, timestamp, secret);
            let signature_header = format!("t={},v1={}", timestamp, signature);
            request = request.header("signature", signature_header);
        }

        let response = request
            .body(payload_json)
            .send()
            .await
            .map_err(|e| ServiceError::WebhookDeliveryFailed(format!("HTTP request failed: {}", e)))?;
        
        let status_code = response.status().as_u16();
        let response_body = response.text().await
            .unwrap_or_else(|_| "Failed to read response body".to_string());
        
        if status_code >= 200 && status_code < 300 {
            info!("Webhook delivered successfully to {}: {}", url, status_code);
            Ok((status_code, response_body))
        } else {
            warn!("Webhook delivery failed to {}: {} - {}", url, status_code, response_body);
            Err(ServiceError::WebhookDeliveryFailed(
                format!("HTTP {} - {}", status_code, response_body)
            ))
        }
    }

    /// Queue a webhook notification for delivery
    pub async fn queue_webhook(
        &self,
        merchant_id: i64,
        payment_id: Option<i64>,
        payload: WebhookPayload,
    ) -> Result<(), ServiceError> {
        // Get merchant's webhook configuration
        let config = sqlx::query(
            "SELECT url, payload_format, signing_secret FROM webhook_configs WHERE merchant_id = $1 AND is_active = true"
        )
        .bind(merchant_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| ServiceError::Internal(format!("Failed to fetch webhook config: {}", e)))?;

use sqlx::Row;

        if let Some(config) = config {
            let config_url: String = config.get("url");
            let config_payload_format: String = config.get("payload_format");
            let payload_value = if config_payload_format == "discord" {
                let payment_link = format!("https://pay.fiddupay.com/{}", payload.payment_id);
                let tx_hash_display = payload.transaction_hash.as_deref().unwrap_or("Pending/Unknown");
                
                let (color, title, body_text) = match payload.event_type.as_str() {
                    "payment.confirmed" | "merchant.deposit" | "customer.deposit" => {
                        let explorer_link = if payload.crypto_type.to_lowercase().contains("sol") {
                            format!("\n**[View on Explorer](https://explorer.solana.com/tx/{})**", tx_hash_display)
                        } else {
                            format!("\n**[View on Explorer](https://etherscan.io/tx/{})**", tx_hash_display)
                        };
                        
                        let is_deposit = payload.event_type.contains("deposit");
                        let view_link = if is_deposit {
                            String::new()
                        } else {
                            format!("\n\n**[View Payment Page]({})**", payment_link)
                        };

                        (
                            5763719, // Green
                            if payload.event_type == "merchant.deposit" {
                                "💰 Merchant Deposit"
                            } else if payload.event_type == "customer.deposit" {
                                "💰 Customer Deposit"
                            } else {
                                "✅ Payment Confirmed"
                            },
                            format!(
                                "**Amount:** `{} {}`\n**Payment ID:** `{}`\n**Tx Hash:** `{}`{}{}", 
                                payload.amount, payload.crypto_type, payload.payment_id, tx_hash_display, explorer_link, view_link
                            )
                        )
                    },
                    "payment.expired" => (
                        15548997, // Red
                        "❌ Payment Expired",
                        format!(
                            "**Amount:** `{} {}`\n**Payment ID:** `{}`\n\n**[View Payment Page]({})**", 
                            payload.amount, payload.crypto_type, payload.payment_id, payment_link
                        )
                    ),
                    _ => (
                        3447003, // Blue
                        "🔔 Webhook Alert",
                        format!(
                            "**Event:** `{}`\n**Amount:** `{} {}`\n**Payment ID:** `{}`\n\n**[View Payment Page]({})**", 
                            payload.event_type, payload.amount, payload.crypto_type, payload.payment_id, payment_link
                        )
                    ),
                };
                
                serde_json::json!({
                    "embeds": [{
                        "title": title,
                        "description": body_text,
                        "color": color,
                        "footer": {
                            "text": "Powered By FidduPay"
                        },
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }]
                })
            } else if config_payload_format == "slack" {
                let text = match payload.event_type.as_str() {
                    "payment.confirmed" => format!("✅ *Payment Confirmed*\nID: `{}`\nAmount: `{} {}`", 
                        payload.payment_id, payload.amount, payload.crypto_type),
                    "payment.expired" => format!("❌ *Payment Expired*\nID: `{}`", payload.payment_id),
                    _ => format!("🔔 *Webhook Alert*: `{}` for payment `{}`", payload.event_type, payload.payment_id),
                };
                serde_json::json!({ "text": text })
            } else {
                serde_json::to_value(&payload)
                    .map_err(|e| ServiceError::Internal(format!("Failed to serialize payload: {}", e)))?
            };

            // Queue notification for background delivery
            sqlx::query(
                r#"
                INSERT INTO webhook_deliveries (merchant_id, payment_id, event_type, url, payload, status, created_at, next_retry_at)
                VALUES ($1, $2, $3, $4, $5, 'pending', NOW(), NOW())
                "#
            )
            .bind(merchant_id)
            .bind(payment_id)
            .bind(&payload.event_type)
            .bind(&config_url)
            .bind(&payload_value)
            .execute(&self.db_pool)
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to queue webhook: {}", e)))?;
        }

        Ok(())
    }
}

