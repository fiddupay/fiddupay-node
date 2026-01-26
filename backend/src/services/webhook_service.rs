// Webhook Service
// Business logic for webhook delivery

use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::Sha256;
use sqlx::PgPool;
use std::time::Duration;
use tracing::{info, warn};
use url::Url;

use crate::error::ServiceError;
use crate::models::webhook::WebhookPayload;

type HmacSha256 = Hmac<Sha256>;

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

    /// Configure webhook URL for a merchant
    /// 
    /// Validates that the URL is a valid HTTPS endpoint and stores it in the database.
    /// If a webhook URL already exists for the merchant, it will be updated.
    /// 
    /// # Arguments
    /// * `merchant_id` - ID of the merchant
    /// * `url` - Webhook URL (must be HTTPS)
    /// 
    /// # Returns
    /// * `Ok(())` if the webhook URL was successfully configured
    /// * `Err(ServiceError::InvalidWebhookUrl)` if the URL is not HTTPS or invalid
    /// 
    /// # Requirements
    /// * 4.1: Validate webhook URL is a valid HTTPS endpoint
    pub async fn set_webhook_url(
        &self,
        merchant_id: i64,
        url: String,
    ) -> Result<(), ServiceError> {
        // Validate URL format
        let parsed_url = Url::parse(&url)
            .map_err(|_| ServiceError::InvalidWebhookUrl("Invalid URL format".to_string()))?;

        // Validate HTTPS scheme
        if parsed_url.scheme() != "https" {
            return Err(ServiceError::InvalidWebhookUrl(
                "Webhook URL must use HTTPS protocol".to_string()
            ));
        }

        // Validate host is present
        if parsed_url.host_str().is_none() {
            return Err(ServiceError::InvalidWebhookUrl(
                "Webhook URL must have a valid host".to_string()
            ));
        }

        // Insert or update webhook configuration
        sqlx::query!(
            r#"
            INSERT INTO webhook_configs (merchant_id, url, is_active, created_at, updated_at)
            VALUES ($1, $2, true, NOW(), NOW())
            ON CONFLICT (merchant_id)
            DO UPDATE SET
                url = EXCLUDED.url,
                is_active = true,
                updated_at = NOW()
            "#,
            merchant_id,
            &url
        )
        .execute(&self.db_pool)
        .await?;

        info!("Configured webhook URL for merchant {}: {}", merchant_id, url);

        Ok(())
    }

    /// Generate HMAC-SHA256 signature for webhook payload
    /// 
    /// Creates a signature using the webhook signing key to allow merchants
    /// to verify the authenticity of webhook requests.
    /// 
    /// # Arguments
    /// * `payload` - JSON string of the webhook payload
    /// * `timestamp` - Unix timestamp when the webhook is sent
    /// 
    /// # Returns
    /// Hex-encoded HMAC-SHA256 signature
    /// 
    /// # Requirements
    /// * 4.5: Include signature in webhook requests for verification
    fn generate_signature(&self, payload: &str, timestamp: i64) -> String {
        let message = format!("{}.{}", timestamp, payload);
        
        let mut mac = HmacSha256::new_from_slice(self.signing_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(message.as_bytes());
        
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    /// Send webhook notification with signature
    /// 
    /// Delivers a webhook to the merchant's configured URL with HMAC-SHA256
    /// signature for verification. Includes X-Signature and X-Timestamp headers.
    /// 
    /// # Arguments
    /// * `url` - Webhook URL to send to
    /// * `payload` - Webhook payload to send
    /// 
    /// # Returns
    /// * `Ok((status_code, response_body))` if the request was sent successfully
    /// * `Err(ServiceError)` if the request failed
    /// 
    /// # Requirements
    /// * 4.2: Send webhook notification on payment status change to confirmed
    /// * 4.3: Send webhook notification on payment status change to expired
    /// * 4.5: Include signature in webhook requests for verification
    /// * 4.6: Include payment details, status, and timestamp in payload
    pub async fn send_webhook(
        &self,
        url: &str,
        payload: &WebhookPayload,
    ) -> Result<(u16, String), ServiceError> {
        let timestamp = Utc::now().timestamp();
        
        // Serialize payload
        let payload_json = serde_json::to_string(payload)
            .map_err(|e| ServiceError::Internal(format!("Failed to serialize webhook payload: {}", e)))?;
        
        // Generate signature
        let signature = self.generate_signature(&payload_json, timestamp);
        
        // Send HTTP POST request with signature headers
        let response = self.http_client
            .post(url)
            .header("Content-Type", "application/json")
            .header("X-Signature", signature)
            .header("X-Timestamp", timestamp.to_string())
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
    /// 
    /// Creates a webhook delivery record in the database that will be
    /// processed by the webhook retry background task.
    /// 
    /// # Arguments
    /// * `merchant_id` - ID of the merchant to notify
    /// * `payment_id` - Database ID of the payment
    /// * `payload` - Webhook payload to send
    /// 
    /// # Requirements
    /// * 4.2: Send webhook notification on payment status change to confirmed
    /// * 4.3: Send webhook notification on payment status change to expired
    pub async fn queue_webhook(
        &self,
        merchant_id: i64,
        payment_id: i64,
        payload: WebhookPayload,
    ) -> Result<(), ServiceError> {
        // Get merchant's webhook URL
        let webhook_config = sqlx::query!(
            "SELECT url FROM webhook_configs WHERE merchant_id = $1 AND is_active = true",
            merchant_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        let url = match webhook_config {
            Some(config) => config.url,
            None => {
                // No webhook configured, skip
                info!("No webhook configured for merchant {}", merchant_id);
                return Ok(());
            }
        };

        // Serialize payload
        let payload_json = serde_json::to_value(&payload)
            .map_err(|e| ServiceError::Internal(format!("Failed to serialize webhook payload: {}", e)))?;

        // Insert webhook delivery record
        sqlx::query!(
            r#"
            INSERT INTO webhook_deliveries (
                merchant_id, payment_id, event_type, url, payload,
                status, attempts, next_retry_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            merchant_id,
            payment_id,
            &payload.event_type,
            &url,
            payload_json,
            "pending",
            0,
            Utc::now(), // Retry immediately
            Utc::now()
        )
        .execute(&self.db_pool)
        .await?;

        info!(
            "Queued webhook for merchant {} - event: {}",
            merchant_id, payload.event_type
        );

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
        let result = sqlx::query!(
            r#"
            INSERT INTO merchants (email, business_name, api_key_hash, fee_percentage, is_active, sandbox_mode)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
            "test@example.com",
            "Test Business",
            "test_hash",
            rust_decimal::Decimal::new(150, 2), // 1.50%
            true,
            false
        )
        .fetch_one(pool)
        .await
        .unwrap();

        result.id
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn test_set_webhook_url_valid_https() {
        let pool = setup_test_db().await;
        let service = WebhookService::new(pool.clone(), "test_signing_key".to_string());
        let merchant_id = create_test_merchant(&pool).await;

        let result = service.set_webhook_url(
            merchant_id,
            "https://example.com/webhook".to_string()
        ).await;

        assert!(result.is_ok());

        // Verify webhook was stored
        let config = sqlx::query!(
            "SELECT url, is_active FROM webhook_configs WHERE merchant_id = $1",
            merchant_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(config.url, "https://example.com/webhook");
        assert!(config.is_active);

        // Cleanup
        sqlx::query!("DELETE FROM webhook_configs WHERE merchant_id = $1", merchant_id)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query!("DELETE FROM merchants WHERE id = $1", merchant_id)
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
            "http://example.com/webhook".to_string()
        ).await;

        assert!(result.is_err());
        match result {
            Err(ServiceError::InvalidWebhookUrl(msg)) => {
                assert!(msg.contains("HTTPS"));
            }
            _ => panic!("Expected InvalidWebhookUrl error"),
        }

        // Cleanup
        sqlx::query!("DELETE FROM merchants WHERE id = $1", merchant_id)
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
            "not-a-valid-url".to_string()
        ).await;

        assert!(result.is_err());
        match result {
            Err(ServiceError::InvalidWebhookUrl(msg)) => {
                assert!(msg.contains("Invalid URL format"));
            }
            _ => panic!("Expected InvalidWebhookUrl error"),
        }

        // Cleanup
        sqlx::query!("DELETE FROM merchants WHERE id = $1", merchant_id)
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
            "https://".to_string()
        ).await;

        assert!(result.is_err());
        match result {
            Err(ServiceError::InvalidWebhookUrl(msg)) => {
                assert!(msg.contains("valid host"));
            }
            _ => panic!("Expected InvalidWebhookUrl error"),
        }

        // Cleanup
        sqlx::query!("DELETE FROM merchants WHERE id = $1", merchant_id)
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
            "https://example.com/webhook1".to_string()
        ).await.unwrap();

        // Update to new URL
        service.set_webhook_url(
            merchant_id,
            "https://example.com/webhook2".to_string()
        ).await.unwrap();

        // Verify only one config exists with the new URL
        let configs = sqlx::query!(
            "SELECT url FROM webhook_configs WHERE merchant_id = $1",
            merchant_id
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].url, "https://example.com/webhook2");

        // Cleanup
        sqlx::query!("DELETE FROM webhook_configs WHERE merchant_id = $1", merchant_id)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query!("DELETE FROM merchants WHERE id = $1", merchant_id)
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
            "https://example.com/api/webhooks?token=abc123".to_string()
        ).await;

        assert!(result.is_ok());

        // Verify webhook was stored with full URL
        let config = sqlx::query!(
            "SELECT url FROM webhook_configs WHERE merchant_id = $1",
            merchant_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(config.url, "https://example.com/api/webhooks?token=abc123");

        // Cleanup
        sqlx::query!("DELETE FROM webhook_configs WHERE merchant_id = $1", merchant_id)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query!("DELETE FROM merchants WHERE id = $1", merchant_id)
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
            "https://example.com:8443/webhook".to_string()
        ).await;

        assert!(result.is_ok());

        // Verify webhook was stored
        let config = sqlx::query!(
            "SELECT url FROM webhook_configs WHERE merchant_id = $1",
            merchant_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(config.url, "https://example.com:8443/webhook");

        // Cleanup
        sqlx::query!("DELETE FROM webhook_configs WHERE merchant_id = $1", merchant_id)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query!("DELETE FROM merchants WHERE id = $1", merchant_id)
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

        let signature = service.generate_signature(payload, timestamp);

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

        let sig1 = service.generate_signature(payload, timestamp);
        let sig2 = service.generate_signature(payload, timestamp);

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

        let sig1 = service.generate_signature(payload1, timestamp);
        let sig2 = service.generate_signature(payload2, timestamp);

        // Different payloads should produce different signatures
        assert_ne!(sig1, sig2);
    }

    #[tokio::test]
    async fn test_generate_signature_different_timestamps() {
        let pool = PgPool::connect_lazy("postgresql://vibes:Soledayo%402001@localhost:5432/fiddupay_test").unwrap();
        let service = WebhookService::new(pool, "test_signing_key".to_string());

        let payload = r#"{"event_type":"payment.confirmed"}"#;

        let sig1 = service.generate_signature(payload, 1234567890);
        let sig2 = service.generate_signature(payload, 1234567891);

        // Different timestamps should produce different signatures
        assert_ne!(sig1, sig2);
    }
}
