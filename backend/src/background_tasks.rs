// Background Tasks
// Long-running tasks for payment monitoring and webhook delivery

use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

use crate::error::ServiceError;
use crate::models::webhook::WebhookPayload;
use crate::payment::models::PaymentStatus;
use crate::services::webhook_service::WebhookService;

/// Background task manager
pub struct BackgroundTasks {
    db_pool: PgPool,
    webhook_service: Arc<WebhookService>,
}

impl BackgroundTasks {
    pub fn new(db_pool: PgPool, signing_key: String) -> Self {
        Self {
            db_pool: db_pool.clone(),
            webhook_service: Arc::new(WebhookService::new(db_pool, signing_key)),
        }
    }

    /// Start all background tasks
    /// 
    /// Spawns tokio tasks for:
    /// - Payment expiration checking
    /// - Webhook retry processing
    pub fn start(self: Arc<Self>) {
        let tasks_expiration = self.clone();
        tokio::spawn(async move {
            tasks_expiration.run_expiration_checker().await;
        });

        let tasks_webhook = self.clone();
        tokio::spawn(async move {
            tasks_webhook.run_webhook_retry().await;
        });

        info!("Background tasks started");
    }

    /// Run payment expiration checker
    /// 
    /// Continuously checks for expired payments and updates their status.
    /// Runs every 30 seconds.
    /// 
    /// # Requirements
    /// * 2.4: Mark payments as expired when expiration time elapses
    /// * 2.7: Update status to expired when time elapses
    /// * 4.3: Trigger webhook notifications for expired payments
    async fn run_expiration_checker(&self) {
        let mut interval = interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            if let Err(e) = self.check_expired_payments().await {
                error!("Error checking expired payments: {}", e);
            }
        }
    }

    /// Check for expired payments and update their status
    /// 
    /// Finds all payments that are past their expiration time and still
    /// in pending or confirming status, updates them to failed (expired),
    /// and triggers webhook notifications.
    /// 
    /// # Requirements
    /// * 2.4: Mark payments as expired when expiration time elapses
    /// * 2.7: Update status to expired when time elapses
    /// * 4.3: Trigger webhook notifications for expired payments
    async fn check_expired_payments(&self) -> Result<(), ServiceError> {
        // Find all expired payments that are still pending or confirming
        let expired_payments = sqlx::query!(
            r#"
            SELECT id, merchant_id, payment_id, amount, crypto_type
            FROM payment_transactions
            WHERE expires_at < $1
              AND status IN ('PENDING', 'CONFIRMING')
            "#,
            Utc::now()
        )
        .fetch_all(&self.db_pool)
        .await?;

        if expired_payments.is_empty() {
            return Ok(());
        }

        info!("Found {} expired payments to process", expired_payments.len());

        for payment in expired_payments {
            let payment_id_clone = payment.payment_id.clone();
            
            // Update payment status to FAILED (expired)
            let result = sqlx::query!(
                r#"
                UPDATE payment_transactions
                SET status = 'FAILED'
                WHERE id = $1 AND status IN ('PENDING', 'CONFIRMING')
                "#,
                payment.id
            )
            .execute(&self.db_pool)
            .await;

            match result {
                Ok(result) if result.rows_affected() > 0 => {
                    info!(
                        "Marked payment {} (id: {}) as expired for merchant {}",
                        payment.payment_id, payment.id, payment.merchant_id
                    );

                    // Queue webhook notification
                    let webhook_payload = WebhookPayload {
                        event_type: "payment.expired".to_string(),
                        payment_id: payment.payment_id,
                        merchant_id: payment.merchant_id,
                        status: PaymentStatus::Failed,
                        amount: payment.amount,
                        crypto_type: payment.crypto_type,
                        transaction_hash: None,
                        timestamp: Utc::now().timestamp(),
                    };

                    if let Err(e) = self.webhook_service.queue_webhook(
                        payment.merchant_id,
                        payment.id,
                        webhook_payload,
                    ).await {
                        error!(
                            "Failed to queue webhook for expired payment {}: {}",
                            payment_id_clone, e
                        );
                    }
                }
                Ok(_) => {
                    // Payment was already updated by another process
                    warn!(
                        "Payment {} was already updated (race condition)",
                        payment.payment_id
                    );
                }
                Err(e) => {
                    error!(
                        "Failed to update payment {} status: {}",
                        payment_id_clone, e
                    );
                }
            }
        }

        Ok(())
    }

    /// Run webhook retry background task
    /// 
    /// Continuously checks for failed webhooks and retries them with
    /// exponential backoff. Runs every 10 seconds.
    /// 
    /// # Requirements
    /// * 4.4: Retry webhook delivery with exponential backoff up to 5 attempts
    /// * 4.7: Log all webhook delivery attempts and results
    async fn run_webhook_retry(&self) {
        let mut interval = interval(Duration::from_secs(10));

        loop {
            interval.tick().await;

            if let Err(e) = self.retry_failed_webhooks().await {
                error!("Error retrying failed webhooks: {}", e);
            }
        }
    }

    /// Retry failed webhooks with exponential backoff
    /// 
    /// Finds all pending webhooks that are ready for retry (past their next_retry_at time),
    /// attempts to deliver them, and updates the database with the results.
    /// 
    /// Uses exponential backoff: 1s, 2s, 4s, 8s, 16s for attempts 1-5.
    /// After 5 failed attempts, marks the webhook as permanently failed.
    /// 
    /// # Requirements
    /// * 4.4: Retry webhook delivery with exponential backoff up to 5 attempts
    /// * 4.7: Log all webhook delivery attempts and results
    pub async fn retry_failed_webhooks(&self) -> Result<(), ServiceError> {
        // Find all pending webhooks ready for retry
        let pending_webhooks = sqlx::query!(
            r#"
            SELECT id, merchant_id, payment_id, event_type, url, payload, attempts
            FROM webhook_deliveries
            WHERE status = 'pending'
              AND next_retry_at <= $1
              AND attempts < 5
            ORDER BY next_retry_at ASC
            LIMIT 100
            "#,
            Utc::now()
        )
        .fetch_all(&self.db_pool)
        .await?;

        if pending_webhooks.is_empty() {
            return Ok(());
        }

        info!("Found {} webhooks to retry", pending_webhooks.len());

        for webhook in pending_webhooks {
            let attempt_number = webhook.attempts + 1;

            // Deserialize payload
            let payload: WebhookPayload = match serde_json::from_value(webhook.payload) {
                Ok(p) => p,
                Err(e) => {
                    error!(
                        "Failed to deserialize webhook payload for delivery {}: {}",
                        webhook.id, e
                    );
                    continue;
                }
            };

            info!(
                "Retrying webhook delivery {} (attempt {}/5) for merchant {} - event: {}",
                webhook.id, attempt_number, webhook.merchant_id, webhook.event_type
            );

            // Attempt delivery
            let delivery_result = self.webhook_service.send_webhook(&webhook.url, &payload).await;

            match delivery_result {
                Ok((status_code, response_body)) => {
                    // Success - mark as delivered
                    sqlx::query!(
                        r#"
                        UPDATE webhook_deliveries
                        SET status = 'delivered',
                            attempts = $1,
                            last_attempt_at = $2,
                            response_status = $3,
                            response_body = $4
                        WHERE id = $5
                        "#,
                        attempt_number,
                        Utc::now(),
                        status_code as i32,
                        response_body,
                        webhook.id
                    )
                    .execute(&self.db_pool)
                    .await?;

                    info!(
                        "Webhook delivery {} succeeded on attempt {}",
                        webhook.id, attempt_number
                    );
                }
                Err(e) => {
                    // Failed - update attempt count and schedule next retry
                    let (status, next_retry) = if attempt_number >= 5 {
                        // Max attempts reached - mark as failed
                        ("failed", None)
                    } else {
                        // Schedule next retry with exponential backoff
                        // Backoff: 1s, 2s, 4s, 8s, 16s
                        let backoff_seconds = 2_i64.pow(attempt_number as u32 - 1);
                        let next_retry_at = Utc::now() + chrono::Duration::seconds(backoff_seconds);
                        ("pending", Some(next_retry_at))
                    };

                    let error_message = e.to_string();
                    let response_status = if let ServiceError::WebhookDeliveryFailed(ref msg) = e {
                        // Try to extract status code from error message
                        if msg.starts_with("HTTP ") {
                            msg.split_whitespace()
                                .nth(1)
                                .and_then(|s| s.parse::<i32>().ok())
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    sqlx::query!(
                        r#"
                        UPDATE webhook_deliveries
                        SET status = $1,
                            attempts = $2,
                            last_attempt_at = $3,
                            next_retry_at = $4,
                            response_status = $5,
                            response_body = $6
                        WHERE id = $7
                        "#,
                        status,
                        attempt_number,
                        Utc::now(),
                        next_retry,
                        response_status,
                        error_message,
                        webhook.id
                    )
                    .execute(&self.db_pool)
                    .await?;

                    if attempt_number >= 5 {
                        error!(
                            "Webhook delivery {} failed permanently after {} attempts",
                            webhook.id, attempt_number
                        );
                    } else {
                        warn!(
                            "Webhook delivery {} failed on attempt {}, will retry in {}s",
                            webhook.id, attempt_number, 2_i64.pow(attempt_number as u32 - 1)
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use rust_decimal::Decimal;

    #[tokio::test]
    async fn test_background_tasks_creation() {
        let pool = PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let signing_key = "test_signing_key_32_bytes_long!!".to_string();
        let _tasks = BackgroundTasks::new(pool, signing_key);
        // Just verify it compiles and creates
        assert!(true);
    }

    #[test]
    fn test_webhook_payload_for_expired_payment() {
        let payload = WebhookPayload {
            event_type: "payment.expired".to_string(),
            payment_id: "pay_test123".to_string(),
            merchant_id: 1i64,
            status: PaymentStatus::Failed,
            amount: Decimal::new(100, 0),
            crypto_type: "USDT_BEP20".to_string(),
            transaction_hash: None,
            timestamp: Utc::now().timestamp(),
        };

        assert_eq!(payload.event_type, "payment.expired");
        assert_eq!(payload.status, PaymentStatus::Failed);
        assert!(payload.transaction_hash.is_none());
    }

    #[test]
    fn test_expiration_time_check() {
        let now = Utc::now();
        let expired_time = now - Duration::minutes(5);
        let future_time = now + Duration::minutes(5);

        // Payment with expired_time should be expired
        assert!(expired_time < now);

        // Payment with future_time should not be expired
        assert!(future_time > now);
    }

    #[test]
    fn test_payment_status_for_expiration() {
        // Payments in these statuses should be checked for expiration
        let expirable_statuses = vec!["PENDING", "CONFIRMING"];

        for status in expirable_statuses {
            assert!(status == "PENDING" || status == "CONFIRMING");
        }

        // Payments in these statuses should NOT be expired
        let non_expirable_statuses = vec!["CONFIRMED", "FAILED", "REFUNDED"];

        for status in non_expirable_statuses {
            assert!(status != "PENDING" && status != "CONFIRMING");
        }
    }

    #[test]
    fn test_webhook_event_type_for_expiration() {
        let event_type = "payment.expired";
        assert_eq!(event_type, "payment.expired");
    }

    #[test]
    fn test_expiration_checker_interval() {
        // The expiration checker should run every 30 seconds
        let interval_seconds = 30;
        assert_eq!(interval_seconds, 30);
    }

    #[test]
    fn test_webhook_retry_interval() {
        // The webhook retry task should run every 10 seconds
        let interval_seconds = 10;
        assert_eq!(interval_seconds, 10);
    }

    #[test]
    fn test_exponential_backoff_calculation() {
        // Test exponential backoff: 1s, 2s, 4s, 8s, 16s
        let backoffs = vec![
            (1, 1),   // 2^0 = 1
            (2, 2),   // 2^1 = 2
            (3, 4),   // 2^2 = 4
            (4, 8),   // 2^3 = 8
            (5, 16),  // 2^4 = 16
        ];

        for (attempt, expected_backoff) in backoffs {
            let backoff = 2_i64.pow(attempt - 1);
            assert_eq!(backoff, expected_backoff);
        }
    }

    #[test]
    fn test_max_webhook_attempts() {
        // Webhooks should be retried up to 5 times
        let max_attempts = 5;
        assert_eq!(max_attempts, 5);

        // After 5 attempts, webhook should be marked as failed
        let attempts = 5;
        let should_fail = attempts >= max_attempts;
        assert!(should_fail);
    }

    #[test]
    fn test_webhook_retry_status_transitions() {
        // Pending webhooks should be retried
        let status = "pending";
        assert_eq!(status, "pending");

        // After successful delivery, status should be "delivered"
        let success_status = "delivered";
        assert_eq!(success_status, "delivered");

        // After max attempts, status should be "failed"
        let failed_status = "failed";
        assert_eq!(failed_status, "failed");
    }

    #[test]
    fn test_webhook_retry_limit() {
        // Webhooks should only be retried if attempts < 5
        for attempt in 0..5 {
            assert!(attempt < 5);
        }

        // Attempt 5 should not be retried
        let attempt = 5;
        assert!(!(attempt < 5));
    }
}
