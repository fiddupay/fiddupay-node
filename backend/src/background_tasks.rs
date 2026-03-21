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
use crate::payment::sol_monitor::SolanaMonitor;
use crate::payment::verifier::PaymentVerifier;
use crate::payment::models::CryptoType;

struct ExpiredPaymentRow {
    id: i64,
    merchant_id: i64,
    payment_id: String,
    amount: Option<rust_decimal::Decimal>,
    crypto_type: Option<String>,
}

struct PendingWebhookRow {
    id: i64,
    merchant_id: i64,
    payment_id: Option<i64>,
    event_type: String,
    url: String,
    payload: String,
    attempts: i32,
}

/// Background task manager
pub struct BackgroundTasks {
    db_pool: PgPool,
    webhook_service: Arc<WebhookService>,
    config: crate::config::Config,
    price_service: Arc<crate::services::price_service::PriceService>,
    redis_client: redis::Client,
}

impl BackgroundTasks {
    pub fn new(
        db_pool: PgPool, 
        config: crate::config::Config, 
        price_service: Arc<crate::services::price_service::PriceService>,
        redis_client: redis::Client,
    ) -> Self {
        let webhook_service = Arc::new(WebhookService::new(db_pool.clone(), config.webhook_signing_key.clone()));
        Self {
            db_pool,
            webhook_service,
            config,
            price_service,
            redis_client,
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

        let tasks_solana_prod = self.clone();
        tokio::spawn(async move {
            tasks_solana_prod.run_solana_monitor(false).await;
        });

        let tasks_solana_sandbox = self.clone();
        tokio::spawn(async move {
            tasks_solana_sandbox.run_solana_monitor(true).await;
        });

        // Initialize Gas Monitor and Auto-Sweeper for platform fees
        let monitor = crate::services::gas_monitor_service::GasMonitorService::new(self.db_pool.clone(), self.config.clone());
        tokio::spawn(async move {
            monitor.start_monitoring().await;
        });

        let fee_service = crate::services::fee_collection_service::FeeCollectionService::new(self.db_pool.clone(), self.config.clone());
        tokio::spawn(async move {
            fee_service.start_auto_sweeper().await;
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
        let expired_payments_res = sqlx::query(
            r#"
            SELECT id, merchant_id, payment_id, amount, crypto_type
            FROM payment_transactions
            WHERE expires_at < $1
              AND status IN ('PENDING', 'CONFIRMING')
            "#
        )
        .bind(Utc::now())
        .fetch_all(&self.db_pool)
        .await;
        
        let expired_payments = match expired_payments_res {
            Ok(rows) => {
                use sqlx::Row;
                rows.into_iter().map(|r| {
                    let id: i64 = r.get("id");
                    let merchant_id: i64 = r.get("merchant_id");
                    let payment_id: String = r.get("payment_id");
                    let amount: Option<rust_decimal::Decimal> = r.get("amount");
                    let crypto_type: Option<String> = r.get("crypto_type");
                    ExpiredPaymentRow { id, merchant_id, payment_id, amount, crypto_type }
                }).collect::<Vec<_>>()
            },
            Err(e) => return Err(ServiceError::InternalError(e.to_string())),
        };

        if expired_payments.is_empty() {
            return Ok(());
        }

        info!("Found {} expired payments to process", expired_payments.len());

        for payment in expired_payments {
            let payment_id_clone = payment.payment_id.clone();
            
            // Update payment status to FAILED (expired)
            let result_res: Result<sqlx::postgres::PgQueryResult, sqlx::Error> = sqlx::query(
                r#"
                UPDATE payment_transactions
                SET status = 'FAILED'
                WHERE id = $1 AND status IN ('PENDING', 'CONFIRMING')
                "#
            )
            .bind(payment.id)
            .execute(&self.db_pool)
            .await;

            match result_res {
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
                        amount: payment.amount.unwrap_or_default(),
                        crypto_type: payment.crypto_type.unwrap_or_else(|| "UNKNOWN".to_string()),
                        transaction_hash: None,
                        timestamp: Utc::now().timestamp(),
                    };

                    if let Err(e) = self.webhook_service.queue_webhook(
                        payment.merchant_id,
                        Some(payment.id),
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
        let pending_webhooks_res = sqlx::query(
            r#"
            SELECT id, merchant_id, payment_id, event_type, url, payload::text, attempts
            FROM webhook_deliveries
            WHERE status = 'pending'
            AND COALESCE(next_retry_at, '1970-01-01'::timestamptz) <= $1
            AND attempts < 12
            ORDER BY next_retry_at ASC NULLS FIRST
            LIMIT 100
            "#
        )
        .bind(Utc::now())
        .fetch_all(&self.db_pool)
        .await;
        
        let pending_webhooks = match pending_webhooks_res {
            Ok(rows) => {
                use sqlx::Row;
                rows.into_iter().map(|r| {
                    PendingWebhookRow {
                        id: r.get("id"),
                        merchant_id: r.get("merchant_id"),
                        payment_id: r.get("payment_id"),
                        event_type: r.get("event_type"),
                        url: r.get("url"),
                        payload: r.get("payload"),
                        attempts: r.get("attempts"),
                    }
                }).collect::<Vec<_>>()
            },
            Err(e) => return Err(ServiceError::InternalError(e.to_string())),
        };

        if pending_webhooks.is_empty() {
            return Ok(());
        }

        info!("Found {} webhooks to retry", pending_webhooks.len());

        for webhook in pending_webhooks {
            let attempt_number = webhook.attempts + 1;

            info!(
                "Retrying webhook delivery {} (attempt {}/12) for merchant {} - event: {}",
                webhook.id, attempt_number, webhook.merchant_id, webhook.event_type
            );

            // Fetch merchant-specific signing secret and format
            let config_res = sqlx::query(
                "SELECT signing_secret, payload_format FROM webhook_configs WHERE merchant_id = $1"
            )
            .bind(webhook.merchant_id)
            .fetch_one(&self.db_pool)
            .await;

            let (secret, payload_format) = match config_res {
                Ok(row) => {
                    use sqlx::Row;
                    let ss: String = row.get("signing_secret");
                    let pf: String = row.get("payload_format");
                    (ss, pf)
                },
                Err(_) => (self.webhook_service.get_signing_key().to_string(), "standard".to_string()),
            };

            // Attempt delivery — skip signature for Discord/Slack
            let skip_signature = payload_format == "discord" || payload_format == "slack";
            let payload_value: serde_json::Value = serde_json::from_str(&webhook.payload).unwrap_or(serde_json::json!({"raw": webhook.payload}));
            let delivery_result = self.webhook_service.send_webhook(&webhook.url, &payload_value, &secret, skip_signature).await;

            match delivery_result {
                Ok((status_code, response_body)) => {
                    // Success - mark as delivered
                    sqlx::query(
                        r#"
                        UPDATE webhook_deliveries
                        SET status = 'delivered',
                            attempts = $1,
                            last_attempt_at = $2,
                            response_status = $3,
                            response_body = $4
                        WHERE id = $5
                        "#
                    )
                    .bind(attempt_number)
                    .bind(Utc::now())
                    .bind(status_code as i32)
                    .bind(&response_body)
                    .bind(webhook.id)
                    .execute(&self.db_pool)
                    .await?;

                    info!(
                        "Webhook delivery {} succeeded on attempt {}",
                        webhook.id, attempt_number
                    );
                }
                Err(e) => {
                    // Failed - update attempt count and schedule next retry
                    let (status, next_retry) = if attempt_number >= 12 {
                        // Max attempts reached - mark as failed
                        ("failed", None)
                    } else {
                        // Schedule next retry with exponential backoff
                        // Backoff: 1s, 2s, 4s, 8s, 16s, 32s, 64s, 128s, 256s, 512s, 1024s, 2048s...
                        // We'll cap the backoff at 2 hours for later attempts
                        let backoff_seconds = if attempt_number <= 10 {
                            2_i64.pow(attempt_number as u32 - 1)
                        } else {
                            7200 // 2 hours cap
                        };
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

                    sqlx::query(
                        r#"
                        UPDATE webhook_deliveries
                        SET status = $1,
                            attempts = $2,
                            last_attempt_at = $3,
                            next_retry_at = $4,
                            response_status = $5,
                            response_body = $6
                        WHERE id = $7
                        "#
                    )
                    .bind(status)
                    .bind(attempt_number)
                    .bind(Utc::now())
                    .bind(next_retry)
                    .bind(response_status)
                    .bind(&error_message)
                    .bind(webhook.id)
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
    }    async fn fetch_solana_addresses(pool: &sqlx::PgPool, sandbox_mode: bool) -> Vec<String> {
        let addresses_res = sqlx::query(
            r#"
            SELECT DISTINCT to_address 
            FROM payment_transactions 
            WHERE status IN ('PENDING', 'CONFIRMING')
              AND to_address IS NOT NULL
              AND sandbox_mode = $1
              AND (network ILIKE '%solana%' OR crypto_type ILIKE '%sol%')
            UNION
            SELECT DISTINCT address as to_address
            FROM merchant_customer_wallets
            WHERE sandbox_mode = $1
              AND (crypto_type ILIKE '%sol%')
            UNION
            SELECT DISTINCT address as to_address
            FROM merchant_wallets
            WHERE sandbox_mode = $1 AND is_active = true
              AND (crypto_type ILIKE '%sol%')
            "#
        )
        .bind(sandbox_mode)
        .fetch_all(pool)
        .await;

        match addresses_res {
            Ok(rows) => {
                use sqlx::Row;
                rows.into_iter()
                    .filter_map(|r| r.get::<Option<String>, _>("to_address"))
                    .collect()
            }
            Err(e) => {
                error!("Failed to fetch pending Solana addresses: {}", e);
                Vec::new()
            }
        }
    }

    /// Run Solana real-time monitor (WebSocket logsSubscribe)
    async fn run_solana_monitor(&self, sandbox_mode: bool) {
        let cluster_name = if sandbox_mode { "Devnet" } else { "Mainnet" };
        info!("Starting Solana {} real-time monitor...", cluster_name);
        
        loop {
            // Get all unique addresses for pending Solana payments
            let addresses = Self::fetch_solana_addresses(&self.db_pool, sandbox_mode).await;
            if addresses.is_empty() {
                // No pending payments, wait and check again later
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }

            let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();
            let db_clone = self.db_pool.clone();
            let sandbox_clone = sandbox_mode;
            let mut known = addresses.iter().cloned().collect::<std::collections::HashSet<String>>();

            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    let current = BackgroundTasks::fetch_solana_addresses(&db_clone, sandbox_clone).await;
                    for a in current {
                        if !known.contains(&a) {
                            known.insert(a.clone());
                            let _ = tx.send(a);
                        }
                    }
                }
            });

            info!("Monitoring {} active Solana {} addresses via WebSocket", addresses.len(), cluster_name);

            // Initialize monitor and verifier
            let rpc_url = if sandbox_mode {
                Some(self.config.solana_devnet_rpc_url.clone())
            } else {
                Some(self.config.solana_rpc_url.clone())
            };
            
            let monitor = SolanaMonitor::new(&self.config, rpc_url, None);
            let verifier = Arc::new(PaymentVerifier::new(
                self.db_pool.clone(),
                (*self.webhook_service).clone(),
                self.price_service.clone(),
                self.config.clone(),
                self.redis_client.clone(),
            ));

            let db_clone = self.db_pool.clone();
            let verifier_clone = verifier.clone();

            // Callback for new signatures
            let callback = Arc::new(move |signature: String, address: String| {
                let db = db_clone.clone();
                let v = verifier_clone.clone();
                let addr_clone = address.clone();
                tokio::spawn(async move {
                    // Optimized: Only check payments for the specific address that received the transaction
                    let pending_res = sqlx::query(
                        "SELECT id, merchant_id FROM payment_transactions WHERE to_address = $1 AND status IN ('PENDING', 'CONFIRMING')"
                    )
                    .bind(&addr_clone)
                    .fetch_all(&db)
                    .await;

                    match pending_res {
                        Ok(rows) => {
                            use sqlx::Row;
                            if rows.is_empty() {
                                // Check if this is a static customer address
                                let customer_wallet_res = sqlx::query(
                                    "SELECT customer_id, merchant_id, crypto_type FROM merchant_customer_wallets WHERE address = $1 AND sandbox_mode = $2"
                                )
                                .bind(&addr_clone)
                                .bind(sandbox_mode)
                                .fetch_optional(&db)
                                .await;

                                match customer_wallet_res {
                                    Ok(Some(wallet)) => {
                                        let c_id: i64 = wallet.get("customer_id");
                                        let m_id: i64 = wallet.get("merchant_id");
                                        let crypto_str: String = wallet.get("crypto_type");
                                        info!("WebSocket detected static deposit for customer {} on address {}", c_id, addr_clone);
                                        if let Err(e) = v.verify_customer_deposit(c_id, &signature, m_id, &crypto_str, sandbox_mode).await {
                                            error!("Static deposit verification failed for customer {}: {}", c_id, e);
                                        }
                                    },
                                    Ok(None) => {
                                        // Check if this is a static merchant address
                                        let merchant_wallet_res = sqlx::query(
                                            "SELECT merchant_id, crypto_type FROM merchant_wallets WHERE address = $1 AND sandbox_mode = $2 AND is_active = true"
                                        )
                                        .bind(&addr_clone)
                                        .bind(sandbox_mode)
                                        .fetch_optional(&db)
                                        .await;

                                        match merchant_wallet_res {
                                            Ok(Some(m_wallet)) => {
                                                let m_id: i64 = m_wallet.get("merchant_id");
                                                let crypto_str: String = m_wallet.get("crypto_type");
                                                info!("WebSocket detected static deposit for merchant {} on address {}", m_id, addr_clone);
                                                if let Err(e) = v.verify_merchant_deposit(m_id, &signature, &crypto_str, sandbox_mode).await {
                                                    error!("Static deposit verification failed for merchant {}: {}", m_id, e);
                                                }
                                            },
                                            Ok(None) => {
                                                warn!("WebSocket detected transaction on {} (Signature: {}) but found no pending payments, customer, or merchant wallets in DB", addr_clone, signature);
                                            },
                                            Err(e) => error!("Failed to query merchant wallet for address {}: {}", addr_clone, e),
                                        }
                                    },
                                    Err(e) => error!("Failed to query customer wallet for address {}: {}", addr_clone, e),
                                }
                                return;
                            }
                            for row in rows {
                                let p_id: i64 = row.get("id");
                                let m_id: i64 = row.get("merchant_id");
                                info!("Verifying payment {} for signature {} on address {}", p_id, signature, addr_clone);
                                if let Err(e) = v.verify_payment_by_hash(p_id, &signature, m_id).await {
                                    error!("WebSocket verification failed for payment {}: {}", p_id, e);
                                }
                            }
                        }
                        Err(e) => error!("Failed to query pending payments for address {}: {}", addr_clone, e),
                    }
                });
            });

            // Start listening (this block is long-running)
            if let Err(e) = monitor.listen_for_signatures(addresses, rx, callback).await {
                error!("Solana WebSocket monitor crashed: {}. Reconnecting in 2s...", e);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}

