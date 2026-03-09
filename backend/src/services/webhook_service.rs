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
            request = request
                .header("X-Signature", signature)
                .header("X-Timestamp", timestamp.to_string());
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
                    "payment.confirmed" => (
                        5763719, // Green
                        "✅ Payment Confirmed",
                        format!(
                            "**Amount:** `{} {}`\n**Payment ID:** `{}`\n**Tx Hash:** `{}`\n\n**[View Payment Page]({})**", 
                            payload.amount, payload.crypto_type, payload.payment_id, tx_hash_display, payment_link
                        )
                    ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    async fn setup_test_db() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://vibes:Soledayo%402001@localhost:5432/fiddupay_test".to_string());
        
        PgPool::connect(&database_url).await.unwrap()
    }

    async fn create_test_merchant(pool: &PgPool) -> i64 {
        let result = sqlx::query(
            r#"
            INSERT INTO merchants (email, business_name, test_api_key_hash, fee_percentage, is_active, sandbox_mode)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#
        )
        .bind(format!("test{}@example.com", uuid::Uuid::new_v4().simple()))
        .bind("Test Business")
        .bind(format!("test_hash_{}", Uuid::new_v4().simple()))
        .bind(rust_decimal::Decimal::new(150, 2))
        .bind(true)
        .bind(true)
        .fetch_one(pool)
        .await
        .unwrap();

        use sqlx::Row;
        result.get::<i64, _>("id")
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn test_set_webhook_url_valid_https() {
        let pool = setup_test_db().await;
        let service = WebhookService::new(pool.clone(), "test_signing_key".to_string());
        let merchant_id = create_test_merchant(&pool).await;

        let result = service.set_webhook_url(
            merchant_id,
            Some("https://example.com/webhook".to_string()),
            None
        ).await;

        assert!(result.is_ok());

        // Verify webhook was stored
        let config = sqlx::query(
            "SELECT url, is_active FROM webhook_configs WHERE merchant_id = $1"
        )
        .bind(merchant_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        use sqlx::Row;
        assert_eq!(config.get::<String, _>("url"), "https://example.com/webhook");
        assert!(config.get::<bool, _>("is_active"));

        // Cleanup
        sqlx::query("DELETE FROM webhook_configs WHERE merchant_id = $1")
            .bind(merchant_id)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM merchants WHERE id = $1")
            .bind(merchant_id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn test_set_webhook_url_rejects_http() {
        let pool = setup_test_db().await;
        let service = WebhookService::new(pool.clone(), "test_signing_key".to_string());
        let merchant_id = create_test_merchant(&pool).await;

        let result = service.set_webhook_url(
            merchant_id,
            Some("http://example.com/webhook".to_string()),
            None
        ).await;

        assert!(result.is_err());
        match result {
            Err(ServiceError::InvalidWebhookUrl(msg)) => {
                assert!(msg.contains("HTTPS"));
            }
            _ => panic!("Expected InvalidWebhookUrl error"),
        }

        // Cleanup
        sqlx::query("DELETE FROM merchants WHERE id = $1")
            .bind(merchant_id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn test_set_webhook_url_rejects_invalid_url() {
        let pool = setup_test_db().await;
        let service = WebhookService::new(pool.clone(), "test_signing_key".to_string());
        let merchant_id = create_test_merchant(&pool).await;

        let result = service.set_webhook_url(
            merchant_id,
            Some("not-a-valid-url".to_string()),
            None
        ).await;

        assert!(result.is_err());
        match result {
            Err(ServiceError::InvalidWebhookUrl(msg)) => {
                assert!(msg.contains("Invalid URL format"));
            }
            _ => panic!("Expected InvalidWebhookUrl error"),
        }

        // Cleanup
        sqlx::query("DELETE FROM merchants WHERE id = $1")
            .bind(merchant_id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn test_set_webhook_url_rejects_url_without_host() {
        let pool = setup_test_db().await;
        let service = WebhookService::new(pool.clone(), "test_signing_key".to_string());
        let merchant_id = create_test_merchant(&pool).await;

        let result = service.set_webhook_url(
            merchant_id,
            Some("https://".to_string()),
            None
        ).await;

        assert!(result.is_err());
        match result {
            Err(ServiceError::InvalidWebhookUrl(msg)) => {
                assert!(msg.contains("Invalid URL format"), "Expected 'Invalid URL format' in message: '{}'", msg);
            }
            _ => panic!("Expected InvalidWebhookUrl error"),
        }

        // Cleanup
        sqlx::query("DELETE FROM merchants WHERE id = $1")
            .bind(merchant_id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn test_set_webhook_url_updates_existing() {
        let pool = setup_test_db().await;
        let service = WebhookService::new(pool.clone(), "test_signing_key".to_string());
        let merchant_id = create_test_merchant(&pool).await;

        // Set initial webhook URL
        service.set_webhook_url(
            merchant_id,
            Some("https://example.com/webhook1".to_string()),
            None
        ).await.unwrap();

        // Update to new URL
        service.set_webhook_url(
            merchant_id,
            Some("https://example.com/webhook2".to_string()),
            None
        ).await.unwrap();

        // Verify only one config exists with the new URL
        let configs = sqlx::query(
            "SELECT url FROM webhook_configs WHERE merchant_id = $1"
        )
        .bind(merchant_id)
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(configs.len(), 1);
        use sqlx::Row;
        assert_eq!(configs[0].get::<String, _>("url"), "https://example.com/webhook2");

        // Cleanup
        sqlx::query("DELETE FROM webhook_configs WHERE merchant_id = $1")
            .bind(merchant_id)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM merchants WHERE id = $1")
            .bind(merchant_id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn test_set_webhook_url_with_path_and_query() {
        let pool = setup_test_db().await;
        let service = WebhookService::new(pool.clone(), "test_signing_key".to_string());
        let merchant_id = create_test_merchant(&pool).await;

        let result = service.set_webhook_url(
            merchant_id,
            Some("https://example.com/api/webhooks?token=abc123".to_string()),
            None
        ).await;

        assert!(result.is_ok());

        // Verify webhook was stored with full URL
        let config = sqlx::query(
            "SELECT url FROM webhook_configs WHERE merchant_id = $1"
        )
        .bind(merchant_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        use sqlx::Row;
        assert_eq!(config.get::<String, _>("url"), "https://example.com/api/webhooks?token=abc123");

        // Cleanup
        sqlx::query("DELETE FROM webhook_configs WHERE merchant_id = $1")
            .bind(merchant_id)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM merchants WHERE id = $1")
            .bind(merchant_id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn test_set_webhook_url_with_port() {
        let pool = setup_test_db().await;
        let service = WebhookService::new(pool.clone(), "test_signing_key".to_string());
        let merchant_id = create_test_merchant(&pool).await;

        let result = service.set_webhook_url(
            merchant_id,
            Some("https://example.com:8443/webhook".to_string()),
            None
        ).await;

        assert!(result.is_ok());

        // Verify webhook was stored
        let config = sqlx::query(
            "SELECT url FROM webhook_configs WHERE merchant_id = $1"
        )
        .bind(merchant_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        use sqlx::Row;
        assert_eq!(config.get::<String, _>("url"), "https://example.com:8443/webhook");

        // Cleanup
        sqlx::query("DELETE FROM webhook_configs WHERE merchant_id = $1")
            .bind(merchant_id)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM merchants WHERE id = $1")
            .bind(merchant_id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_generate_signature() {
        let pool = PgPool::connect_lazy("postgresql://vibes:Soledayo%402001@localhost:5432/fiddupay_test").unwrap();
        let service = WebhookService::new(pool, "test_signing_key".to_string());

        let payload = r#"{"event_type":"payment.confirmed","payment_id":"pay_123"}"#;
        let timestamp = 1234567890;

        let signature = service.generate_signature(payload, timestamp, "test_secret");

        // Signature should be a hex string
        assert_eq!(signature.len(), 64); // SHA256 produces 32 bytes = 64 hex chars
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[tokio::test]
    async fn test_generate_signature_consistency() {
        let pool = PgPool::connect_lazy("postgresql://vibes:Soledayo%402001@localhost:5432/fiddupay_test").unwrap();
        let service = WebhookService::new(pool, "test_signing_key".to_string());

        let payload = r#"{"event_type":"payment.confirmed"}"#;
        let timestamp = 1234567890;

        let sig1 = service.generate_signature(payload, timestamp, "test_secret");
        let sig2 = service.generate_signature(payload, timestamp, "test_secret");

        // Same input should produce same signature
        assert_eq!(sig1, sig2);
    }

    #[tokio::test]
    async fn test_generate_signature_different_payloads() {
        let pool = PgPool::connect_lazy("postgresql://vibes:Soledayo%402001@localhost:5432/fiddupay_test").unwrap();
        let service = WebhookService::new(pool, "test_signing_key".to_string());

        let payload1 = r#"{"event_type":"payment.confirmed"}"#;
        let payload2 = r#"{"event_type":"payment.expired"}"#;
        let timestamp = 1234567890;

        let sig1 = service.generate_signature(payload1, timestamp, "test_secret");
        let sig2 = service.generate_signature(payload2, timestamp, "test_secret");

        // Different payloads should produce different signatures
        assert_ne!(sig1, sig2);
    }

    #[tokio::test]
    async fn test_generate_signature_different_timestamps() {
        let pool = PgPool::connect_lazy("postgresql://vibes:Soledayo%402001@localhost:5432/fiddupay_test").unwrap();
        let service = WebhookService::new(pool, "test_signing_key".to_string());

        let payload = r#"{"event_type":"payment.confirmed"}"#;

        let sig1 = service.generate_signature(payload, 1234567890, "test_secret");
        let sig2 = service.generate_signature(payload, 1234567891, "test_secret");

        // Different timestamps should produce different signatures
        assert_ne!(sig1, sig2);
    }
}
