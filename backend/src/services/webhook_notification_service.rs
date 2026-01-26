// Webhook Notification Service for Address-Only Payments
// Sends status updates to merchants via webhooks

use crate::error::ServiceError;
use crate::services::address_only_service::{AddressOnlyPayment, AddressOnlyStatus};
use reqwest::Client;
use serde_json::json;
use sqlx::PgPool;

pub struct WebhookNotificationService {
    client: Client,
    db_pool: PgPool,
}

impl WebhookNotificationService {
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            client: Client::new(),
            db_pool,
        }
    }

    /// Send payment status webhook to merchant
    pub async fn send_payment_status_webhook(
        &self,
        payment: &AddressOnlyPayment,
        webhook_url: &str,
    ) -> Result<(), ServiceError> {
        let payload = json!({
            "event": "address_only_payment_status",
            "payment_id": payment.payment_id,
            "merchant_id": payment.merchant_id,
            "status": payment.status,
            "crypto_type": payment.crypto_type,
            "requested_amount": payment.requested_amount,
            "customer_amount": payment.customer_amount,
            "processing_fee": payment.processing_fee,
            "forwarding_amount": payment.forwarding_amount,
            "customer_pays_fee": payment.customer_amount > payment.requested_amount,
            "gateway_deposit_address": payment.gateway_deposit_address,
            "merchant_destination_address": payment.merchant_destination_address,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        let response = self.client
            .post(webhook_url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| ServiceError::Internal(format!("Webhook request failed: {}", e)))?;

        if response.status().is_success() {
            tracing::info!("Webhook sent successfully for payment {}", payment.payment_id);
        } else {
            tracing::warn!("Webhook failed with status {} for payment {}", response.status(), payment.payment_id);
        }

        // Log webhook attempt
        self.log_webhook_attempt(&payment.payment_id, webhook_url, response.status().as_u16()).await?;

        Ok(())
    }

    /// Get merchant webhook URL
    pub async fn get_merchant_webhook_url(&self, merchant_id: i64) -> Result<Option<String>, ServiceError> {
        let record: Option<_> = sqlx::query!(
            "SELECT webhook_url FROM merchants WHERE id = $1 AND webhook_url IS NOT NULL",
            merchant_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(record.map(|r| r.webhook_url).flatten())
    }

    /// Log webhook attempt for debugging
    async fn log_webhook_attempt(
        &self,
        payment_id: &str,
        webhook_url: &str,
        status_code: u16,
    ) -> Result<(), ServiceError> {
        sqlx::query!(
            r#"
            INSERT INTO webhook_logs (payment_id, webhook_url, status_code, attempted_at)
            VALUES ($1, $2, $3, NOW())
            "#,
            payment_id,
            webhook_url,
            status_code as i32
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Send webhook for all status changes
    pub async fn notify_status_change(
        &self,
        payment: &AddressOnlyPayment,
    ) -> Result<(), ServiceError> {
        if let Some(webhook_url) = self.get_merchant_webhook_url(payment.merchant_id).await? {
            self.send_payment_status_webhook(payment, &webhook_url).await?;
        }

        Ok(())
    }
}
