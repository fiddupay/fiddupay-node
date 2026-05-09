// Background Tasks
// Long-running tasks for payment monitoring and webhook delivery

use chrono::Utc;
use sqlx::PgPool;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{error, info, warn};

use crate::error::ServiceError;
use crate::models::webhook::WebhookPayload;
use crate::payment::blockchain_monitor::BlockchainMonitor;
use crate::payment::models::CryptoType;
use crate::payment::models::PaymentStatus;
use crate::payment::verifier::PaymentVerifier;
use crate::services::balance_monitoring_service::BalanceMonitoringService;
use crate::services::blockchain_transaction_sender::BlockchainTransactionSender;
use crate::services::webhook_notification_service::WebhookNotificationService;
use crate::services::webhook_service::WebhookService;

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
    notification_service: Arc<crate::services::notification_service::NotificationService>,
    blockchain_sender: Arc<BlockchainTransactionSender>,
    balance_service: Arc<crate::services::balance_service::BalanceService>,
}

impl BackgroundTasks {
    pub fn new(
        db_pool: PgPool,
        config: crate::config::Config,
        price_service: Arc<crate::services::price_service::PriceService>,
        redis_client: redis::Client,
        notification_service: Arc<crate::services::notification_service::NotificationService>,
        blockchain_sender: Arc<BlockchainTransactionSender>,
        balance_service: Arc<crate::services::balance_service::BalanceService>,
    ) -> Self {
        let webhook_service = Arc::new(WebhookService::new(
            db_pool.clone(),
            config.webhook_signing_key.clone(),
        ));
        Self {
            db_pool,
            webhook_service,
            config,
            price_service,
            redis_client,
            notification_service,
            blockchain_sender,
            balance_service,
        }
    }

    /// Start all background tasks
    ///
    /// Spawns tokio tasks for:
    /// - Payment expiration checking
    /// - Webhook retry processing
    pub fn start(self: Arc<Self>) {
        // Consolidated Monitor Status Summary
        info!(
            "[MONITOR CONFIG] SOL: {}, BTC: {}, ETH: {}, BNB: {}, MATIC: {}, ARB: {}",
            if self.config.solana_enabled {
                "ENABLED"
            } else {
                "DISABLED"
            },
            if self.config.bitcoin_enabled {
                "ENABLED"
            } else {
                "DISABLED"
            },
            if self.config.ethereum_enabled {
                "ENABLED"
            } else {
                "DISABLED"
            },
            if self.config.bsc_enabled {
                "ENABLED"
            } else {
                "DISABLED"
            },
            if self.config.polygon_enabled {
                "ENABLED"
            } else {
                "DISABLED"
            },
            if self.config.arbitrum_enabled {
                "ENABLED"
            } else {
                "DISABLED"
            }
        );

        let tasks_expiration = self.clone();
        tokio::spawn(async move {
            tasks_expiration.run_expiration_checker().await;
        });

        let tasks_maintenance = self.clone();
        tokio::spawn(async move {
            tasks_maintenance.run_database_maintenance_cron().await;
        });

        let tasks_webhook = self.clone();
        tokio::spawn(async move {
            tasks_webhook.run_webhook_retry().await;
        });

        let enable_sandbox = self.config.enable_sandbox_monitors;

        if self.config.solana_enabled {
            let tasks_solana_prod = self.clone();
            tokio::spawn(async move {
                tasks_solana_prod.run_solana_monitor(false).await;
            });
        }

        if self.config.solana_enabled && self.config.solana_sandbox_enabled {
            let tasks_solana_sandbox = self.clone();
            tokio::spawn(async move {
                tasks_solana_sandbox.run_solana_monitor(true).await;
            });
        }

        if self.config.bitcoin_enabled {
            let tasks_btc_prod = self.clone();
            tokio::spawn(async move {
                tasks_btc_prod.run_btc_monitor(false).await;
            });
        }

        let _tasks_btc_sandbox = self.clone();
        tokio::spawn(async move {
            // Disabled BTC Testnet monitor to reduce API usage as requested
            // _tasks_btc_sandbox.run_btc_monitor(true).await;
        });

        // --- EVM Production Monitors ---
        // Monitor both native currencies and major tokens for each network
        let evm_configs = [
            ("ETH", vec!["ETH", "USDT_ETH"]),
            ("BNB", vec!["BNB", "USDT_BEP20", "BUSD_BEP20"]),
            ("MATIC", vec!["MATIC", "USDT_POLYGON"]),
            ("ARB", vec!["ARB", "USDT_ARBITRUM"]),
        ];

        let evm_configs_prod = evm_configs.clone();
        let tasks_prod = self.clone();
        tokio::spawn(async move {
            for (net, cryptos) in evm_configs_prod {
                let is_enabled = match net {
                    "ETH" => tasks_prod.config.ethereum_enabled,
                    "BNB" => tasks_prod.config.bsc_enabled,
                    "MATIC" => tasks_prod.config.polygon_enabled,
                    "ARB" => tasks_prod.config.arbitrum_enabled,
                    _ => true,
                };

                if is_enabled {
                    for crypto_id in cryptos {
                        let tasks = tasks_prod.clone();
                        tokio::spawn(async move {
                            tasks.run_evm_monitor(crypto_id, false).await;
                        });
                        // Small stagger between tokens
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    }
                }
                // Stagger network starts
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        });

        // --- EVM Sandbox Monitors ---

        if enable_sandbox {
            let tasks_sandbox = self.clone();
            let evm_configs_sandbox = evm_configs; // Last use, can be moved
            tokio::spawn(async move {
                for (net, cryptos) in evm_configs_sandbox {
                    let is_enabled = match net {
                        "ETH" => {
                            tasks_sandbox.config.ethereum_enabled
                                && tasks_sandbox.config.eth_sandbox_enabled
                        }
                        "BNB" => {
                            tasks_sandbox.config.bsc_enabled
                                && tasks_sandbox.config.bnb_sandbox_enabled
                        }
                        "MATIC" => {
                            tasks_sandbox.config.polygon_enabled
                                && tasks_sandbox.config.matic_sandbox_enabled
                        }
                        "ARB" => {
                            tasks_sandbox.config.arbitrum_enabled
                                && tasks_sandbox.config.arb_sandbox_enabled
                        }
                        _ => true,
                    };

                    if is_enabled {
                        for crypto_id in cryptos {
                            let tasks = tasks_sandbox.clone();
                            tokio::spawn(async move {
                                tasks.run_evm_monitor(crypto_id, true).await;
                            });
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        }
                    }
                    // Stagger sandbox network starts
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            });
        } else {
            info!("All EVM Sandbox monitors are globally disabled via ENABLE_SANDBOX_MONITORS");
        }

        // Initialize Gas Monitor and Auto-Sweeper for platform fees
        let monitor = crate::services::gas_monitor_service::GasMonitorService::new(
            self.db_pool.clone(),
            self.config.clone(),
        );
        tokio::spawn(async move {
            monitor.start_monitoring().await;
        });

        // On-demand balance monitoring is handled via API triggers in AppState

        let fee_service = crate::services::fee_collection_service::FeeCollectionService::new(
            self.db_pool.clone(),
            self.config.clone(),
        );
        tokio::spawn(async move {
            fee_service.start_auto_sweeper().await;
        });

        let withdraw_tasks = self.clone();
        tokio::spawn(async move {
            withdraw_tasks.run_withdrawal_processor().await;
        });

        let balance_tasks = self.clone();
        tokio::spawn(async move {
            balance_tasks.run_balance_monitor().await;
        });

        let idempotency_pool = self.db_pool.clone();
        tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(3600)); // Every hour
            loop {
                cleanup_interval.tick().await;
                match sqlx::query("DELETE FROM idempotency_keys WHERE expires_at < NOW()")
                    .execute(&idempotency_pool)
                    .await
                {
                    Ok(result) => {
                        if result.rows_affected() > 0 {
                            info!(
                                "🧹 Pruned {} expired idempotency keys",
                                result.rows_affected()
                            );
                        }
                    }
                    Err(e) => {
                        warn!("Failed to prune idempotency keys: {}", e);
                    }
                }
            }
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
            "#,
        )
        .bind(Utc::now())
        .fetch_all(&self.db_pool)
        .await;

        let expired_payments = match expired_payments_res {
            Ok(rows) => {
                use sqlx::Row;
                rows.into_iter()
                    .map(|r| {
                        let id: i64 = r.get("id");
                        let merchant_id: i64 = r.get("merchant_id");
                        let payment_id: String = r.get("payment_id");
                        let amount: Option<rust_decimal::Decimal> = r.get("amount");
                        let crypto_type: Option<String> = r.get("crypto_type");
                        ExpiredPaymentRow {
                            id,
                            merchant_id,
                            payment_id,
                            amount,
                            crypto_type,
                        }
                    })
                    .collect::<Vec<_>>()
            }
            Err(e) => return Err(ServiceError::InternalError(e.to_string())),
        };

        if expired_payments.is_empty() {
            return Ok(());
        }

        info!(
            "Found {} expired payments to process",
            expired_payments.len()
        );

        for payment in expired_payments {
            let payment_id_clone = payment.payment_id.clone();

            // Update payment status to FAILED (expired)
            let result_res: Result<sqlx::postgres::PgQueryResult, sqlx::Error> = sqlx::query(
                r#"
                UPDATE payment_transactions
                SET status = 'FAILED'
                WHERE id = $1 AND status IN ('PENDING', 'CONFIRMING')
                "#,
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
                        customer_external_id: None,
                        timestamp: Utc::now().timestamp(),
                    };

                    if let Err(e) = self
                        .webhook_service
                        .queue_webhook(payment.merchant_id, Some(payment.id), webhook_payload)
                        .await
                    {
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
            "#,
        )
        .bind(Utc::now())
        .fetch_all(&self.db_pool)
        .await;

        let pending_webhooks = match pending_webhooks_res {
            Ok(rows) => {
                use sqlx::Row;
                rows.into_iter()
                    .map(|r| PendingWebhookRow {
                        id: r.get("id"),
                        merchant_id: r.get("merchant_id"),

                        event_type: r.get("event_type"),
                        url: r.get("url"),
                        payload: r.get("payload"),
                        attempts: r.get("attempts"),
                    })
                    .collect::<Vec<_>>()
            }
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
                "SELECT signing_secret, payload_format FROM webhook_configs WHERE merchant_id = $1",
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
                }
                Err(_) => (
                    self.webhook_service.get_signing_key().to_string(),
                    "standard".to_string(),
                ),
            };

            // Attempt delivery — skip signature for Discord/Slack
            let skip_signature = payload_format == "discord" || payload_format == "slack";
            let payload_value: serde_json::Value = serde_json::from_str(&webhook.payload)
                .unwrap_or(serde_json::json!({"raw": webhook.payload}));
            let delivery_result = self
                .webhook_service
                .send_webhook(&webhook.url, &payload_value, &secret, skip_signature)
                .await;

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
                        "#,
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
                        "#,
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
                            webhook.id,
                            attempt_number,
                            2_i64.pow(attempt_number as u32 - 1)
                        );
                    }
                }
            }
        }

        Ok(())
    }

    async fn fetch_solana_addresses(pool: &sqlx::PgPool, sandbox_mode: bool) -> Vec<String> {
        let addresses_res = sqlx::query(
            r#"
            SELECT DISTINCT to_address 
            FROM payment_transactions 
            WHERE status IN ('PENDING', 'CONFIRMING')
              AND to_address IS NOT NULL
              AND sandbox_mode = $1
              AND crypto_type IN ('SOL', 'WSOL', 'USDT_SPL')
            UNION
            SELECT DISTINCT address as to_address
            FROM merchant_customer_wallets
            WHERE sandbox_mode = $1
              AND crypto_type IN ('SOL', 'WSOL', 'USDT_SPL')
            UNION
            SELECT DISTINCT address as to_address
            FROM merchant_wallets
            WHERE sandbox_mode = $1 AND is_active = true
              AND crypto_type IN ('SOL', 'WSOL', 'USDT_SPL')
            "#,
        )
        .bind(sandbox_mode)
        .fetch_all(pool)
        .await;

        match addresses_res {
            Ok(rows) => {
                use sqlx::Row;
                rows.into_iter()
                    .filter_map(|r| r.get::<Option<String>, _>("to_address"))
                    // Filter out EVM addresses that might have been mislabeled (starts with 0x)
                    // and ensure length matches Solana base58 expectations
                    .filter(|addr| {
                        let trimmed = addr.trim().to_lowercase();
                        !trimmed.starts_with("0x") && addr.len() >= 32 && addr.len() <= 44
                    })
                    .collect()
            }
            Err(e) => {
                error!("Failed to fetch pending Solana addresses: {}", e);
                Vec::new()
            }
        }
    }

    /// Tier 1: Addresses with active/pending payments — polled every cycle
    async fn fetch_btc_active_addresses(pool: &sqlx::PgPool, sandbox_mode: bool) -> Vec<String> {
        let result = sqlx::query(
            r#"
            SELECT DISTINCT to_address 
            FROM payment_transactions 
            WHERE status IN ('PENDING', 'CONFIRMING')
              AND to_address IS NOT NULL
              AND sandbox_mode = $1
              AND crypto_type = 'BTC'
            "#,
        )
        .bind(sandbox_mode)
        .fetch_all(pool)
        .await;

        match result {
            Ok(rows) => {
                use sqlx::Row;
                rows.into_iter()
                    .filter_map(|r| r.get::<Option<String>, _>("to_address"))
                    .collect()
            }
            Err(e) => {
                error!("Failed to fetch active BTC payment addresses: {}", e);
                Vec::new()
            }
        }
    }

    /// Tier 2: Static merchant & customer wallets — polled every Nth cycle
    async fn fetch_btc_static_addresses(pool: &sqlx::PgPool, sandbox_mode: bool) -> Vec<String> {
        let result = sqlx::query(
            r#"
            SELECT DISTINCT address as to_address
            FROM merchant_customer_wallets
            WHERE sandbox_mode = $1
              AND crypto_type = 'BTC'
            UNION
            SELECT DISTINCT address as to_address
            FROM merchant_wallets
            WHERE sandbox_mode = $1 AND is_active = true
              AND crypto_type = 'BTC'
            "#,
        )
        .bind(sandbox_mode)
        .fetch_all(pool)
        .await;

        match result {
            Ok(rows) => {
                use sqlx::Row;
                rows.into_iter()
                    .filter_map(|r| r.get::<Option<String>, _>("to_address"))
                    .collect()
            }
            Err(e) => {
                error!("Failed to fetch static BTC wallet addresses: {}", e);
                Vec::new()
            }
        }
    }

    async fn fetch_evm_addresses(
        pool: &sqlx::PgPool,
        network_prefix: &str,
        sandbox_mode: bool,
    ) -> Vec<String> {
        // Map network prefix to exact crypto types and networks
        let (networks, cryptos): (Vec<String>, Vec<String>) =
            match network_prefix.to_uppercase().as_str() {
                "ETH" => (
                    vec![
                        "ethereum".to_string(),
                        "goerli".to_string(),
                        "sepolia".to_string(),
                    ],
                    vec!["ETH".to_string(), "USDT_ETH".to_string()],
                ),
                "BNB" | "BSC" => (
                    vec![
                        "bsc".to_string(),
                        "bsc_testnet".to_string(),
                        "binance".to_string(),
                    ],
                    vec![
                        "BNB".to_string(),
                        "USDT_BEP20".to_string(),
                        "BUSD_BEP20".to_string(),
                    ],
                ),
                "MATIC" | "POLYGON" => (
                    vec![
                        "polygon".to_string(),
                        "amoy".to_string(),
                        "mumbai".to_string(),
                    ],
                    vec!["MATIC".to_string(), "USDT_POLYGON".to_string()],
                ),
                "ARB" | "ARBITRUM" => (
                    vec!["arbitrum".to_string(), "arbitrum_sepolia".to_string()],
                    vec!["ARB".to_string(), "USDT_ARBITRUM".to_string()],
                ),
                _ => (
                    vec![network_prefix.to_lowercase()],
                    vec![network_prefix.to_uppercase()],
                ),
            };

        let addresses_res = sqlx::query(
            r#"
            SELECT DISTINCT to_address FROM (
                SELECT to_address 
                FROM payment_transactions 
                WHERE status IN ('PENDING', 'CONFIRMING')
                  AND to_address IS NOT NULL
                  AND sandbox_mode = $1
                  AND (network = ANY($2) OR crypto_type = ANY($3))
                UNION ALL
                SELECT address as to_address
                FROM merchant_customer_wallets
                WHERE sandbox_mode = $1
                  AND crypto_type = ANY($3)
                UNION ALL
                SELECT address as to_address
                FROM merchant_wallets
                WHERE sandbox_mode = $1 AND is_active = true
                  AND crypto_type = ANY($3)
            ) as discovery_pool
            "#,
        )
        .bind(sandbox_mode)
        .bind(&networks)
        .bind(&cryptos)
        .fetch_all(pool)
        .await;

        match addresses_res {
            Ok(rows) => {
                use sqlx::Row;
                rows.into_iter()
                    .filter_map(|r| r.get::<Option<String>, _>("to_address"))
                    // Ensure we only pull EVM-style addresses
                    .filter(|addr| addr.starts_with("0x") && addr.len() >= 40)
                    .collect()
            }
            Err(e) => {
                error!(
                    "Failed to fetch pending {} addresses: {}",
                    network_prefix, e
                );
                Vec::new()
            }
        }
    }

    /// Run EVM real-time monitor
    async fn run_evm_monitor(&self, crypto_id: &'static str, sandbox_mode: bool) {
        let cluster_name = if sandbox_mode { "Sandbox" } else { "Mainnet" };
        info!(
            "Starting EVM {} ({}) real-time WebSocket monitor...",
            crypto_id, cluster_name
        );

        loop {
            // 1. Initial address fetch
            let addresses = Self::fetch_evm_addresses(&self.db_pool, crypto_id, sandbox_mode).await;

            // Initial monitored address set
            let monitored_addresses = Arc::new(RwLock::new(
                addresses.iter().cloned().collect::<HashSet<String>>(),
            ));

            // Setup dynamic address channel
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();
            let db_clone = self.db_pool.clone();
            let monitored_addresses_sync = monitored_addresses.clone();

            // Background task to discovery newly added wallets or payments
            tokio::spawn(async move {
                use rand::Rng;

                loop {
                    // Jittered sleep: 5 minutes +/- 30s (To save RPC/Moralis/Alchemy credits)
                    let sleep_secs = rand::thread_rng().gen_range(270..330);
                    tokio::time::sleep(Duration::from_secs(sleep_secs)).await;

                    let current =
                        BackgroundTasks::fetch_evm_addresses(&db_clone, crypto_id, sandbox_mode)
                            .await;

                    let mut lock = monitored_addresses_sync.write().await;
                    for a in current {
                        if !lock.contains(&a) {
                            lock.insert(a.clone());
                            let _ = tx.send(a);
                        }
                    }
                }
            });

            // 3. Initialize monitor and verifier
            let crypto_type = CryptoType::from_string(crypto_id).unwrap_or(CryptoType::Eth);

            let monitor = crate::payment::blockchain_monitor::get_blockchain_monitor(
                &crypto_type,
                self.config.clone(),
                sandbox_mode,
            );

            let verifier = Arc::new(PaymentVerifier::new(
                self.db_pool.clone(),
                (*self.webhook_service).clone(),
                self.price_service.clone(),
                self.config.clone(),
                self.redis_client.clone(),
                self.notification_service.clone(),
                self.balance_service.clone(),
            ));

            let db_clone = self.db_pool.clone();
            let verifier_clone = verifier.clone();
            let net_name = crypto_id;

            // 4. Verification Callback
            let monitored_addresses_callback = monitored_addresses.clone();
            let callback = Arc::new(move |hash: String, address: String| {
                let db = db_clone.clone();
                let v = verifier_clone.clone();
                let addr_lwr = address.to_lowercase();
                let monitored = monitored_addresses_callback.clone();

                tokio::spawn(async move {
                    // Optimized: Only hit DB if the address is in our monitored set
                    {
                        let lock = monitored.read().await;
                        // For EVM, to_address in DB is usually stored lowercase or compared lowercase
                        // Most internal addresses are added to the list in their original format,
                        // but the monitor callback provides the address from the blockchain.
                        if !lock.contains(&address) && !lock.contains(&addr_lwr) {
                            return;
                        }
                    }

                    // Try to find if this belongs to a specific pending payment
                    let pending_res = sqlx::query(
                        "SELECT id, merchant_id FROM payment_transactions WHERE LOWER(to_address) = $1 AND status IN ('PENDING', 'CONFIRMING')"
                    )
                    .bind(&addr_lwr)
                    .fetch_all(&db)
                    .await;

                    match pending_res {
                        Ok(rows) => {
                            use sqlx::Row;
                            if rows.is_empty() {
                                // No pending payment? Check if it's a static CUSTOMER wallet
                                let wallet_res = sqlx::query(
                                    "SELECT customer_id, merchant_id, crypto_type FROM merchant_customer_wallets WHERE LOWER(address) = $1 AND sandbox_mode = $2"
                                )
                                .bind(&addr_lwr)
                                .bind(sandbox_mode)
                                .fetch_optional(&db)
                                .await;

                                if let Ok(Some(wallet)) = wallet_res {
                                    let c_id: i64 = wallet.get("customer_id");
                                    let m_id: i64 = wallet.get("merchant_id");
                                    let c_str: String = wallet.get("crypto_type");
                                    if let Err(e) = v
                                        .verify_customer_deposit(
                                            c_id,
                                            &hash,
                                            m_id,
                                            &c_str,
                                            sandbox_mode,
                                        )
                                        .await
                                    {
                                        error!(
                                            "[EVM-{}] Static customer verification failed: {}",
                                            net_name, e
                                        );
                                    }
                                } else {
                                    // Check if it's a static MERCHANT wallet
                                    let m_wallet_res = sqlx::query(
                                        "SELECT merchant_id, crypto_type FROM merchant_wallets WHERE LOWER(address) = $1 AND sandbox_mode = $2 AND is_active = true"
                                    )
                                    .bind(&addr_lwr)
                                    .bind(sandbox_mode)
                                    .fetch_optional(&db)
                                    .await;

                                    if let Ok(Some(m_wallet)) = m_wallet_res {
                                        let m_id: i64 = m_wallet.get("merchant_id");
                                        let c_str: String = m_wallet.get("crypto_type");
                                        if let Err(e) = v
                                            .verify_merchant_deposit(
                                                m_id,
                                                &hash,
                                                &c_str,
                                                sandbox_mode,
                                            )
                                            .await
                                        {
                                            error!(
                                                "[EVM-{}] Static merchant verification failed: {}",
                                                net_name, e
                                            );
                                        }
                                    }
                                }
                            } else {
                                for row in rows {
                                    let p_id: i64 = row.get("id");
                                    let m_id: i64 = row.get("merchant_id");
                                    if let Err(e) =
                                        v.verify_payment_by_hash(p_id, &hash, m_id).await
                                    {
                                        tracing::trace!(
                                            "[EVM-{}] Verification error for {}: {}",
                                            net_name,
                                            hash,
                                            e
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => error!("[EVM-{}] DB error in WS callback: {}", net_name, e),
                    }
                });
            });

            // 5. Start listener (blocks until error or disconnect)
            if let Err(e) = monitor.listen_for_events(addresses, rx, callback).await {
                error!(
                    "[EVM-{}] WebSocket listener crashed: {}. Restarting in 10s...",
                    crypto_id, e
                );
                tokio::time::sleep(Duration::from_secs(10)).await;
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

            // Initial monitored address set (in-memory lock to avoid DB pressure)
            let monitored_addresses = Arc::new(RwLock::new(
                addresses.iter().cloned().collect::<HashSet<String>>(),
            ));

            let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();
            let db_clone = self.db_pool.clone();
            let sandbox_clone = sandbox_mode;
            let monitored_addresses_sync = monitored_addresses.clone();

            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    let current =
                        BackgroundTasks::fetch_solana_addresses(&db_clone, sandbox_clone).await;

                    let mut lock = monitored_addresses_sync.write().await;
                    for a in current {
                        if !lock.contains(&a) {
                            lock.insert(a.clone());
                            let _ = tx.send(a);
                        }
                    }
                }
            });

            info!(
                "Monitoring {} active Solana {} addresses via WebSocket",
                addresses.len(),
                cluster_name
            );

            // Initialize monitor and verifier
            let monitor = crate::payment::blockchain_monitor::get_blockchain_monitor(
                &CryptoType::Sol,
                self.config.clone(),
                sandbox_mode,
            );
            let verifier = Arc::new(PaymentVerifier::new(
                self.db_pool.clone(),
                (*self.webhook_service).clone(),
                self.price_service.clone(),
                self.config.clone(),
                self.redis_client.clone(),
                self.notification_service.clone(),
                self.balance_service.clone(),
            ));

            let db_clone = self.db_pool.clone();
            let verifier_clone = verifier.clone();

            let monitored_addresses_callback = monitored_addresses.clone();

            // Callback for new signatures
            let callback = Arc::new(move |signature: String, address: String| {
                let db = db_clone.clone();
                let v = verifier_clone.clone();
                let addr_clone = address.clone();
                let monitored = monitored_addresses_callback.clone();

                tokio::spawn(async move {
                    // Optimized: Only hit DB if the address is in our monitored set
                    {
                        let lock = monitored.read().await;
                        if !lock.contains(&addr_clone) {
                            return;
                        }
                    }

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
                                // Find active customer wallet across any environment for identification
                                let customer_wallet_res = sqlx::query(
                                    r#"
                                    SELECT w.customer_id, w.merchant_id, w.crypto_type 
                                    FROM merchant_customer_wallets w
                                    JOIN merchant_customers mc ON mc.id = w.customer_id
                                    WHERE w.address = $1 AND mc.is_active = true
                                    "#,
                                )
                                .bind(&addr_clone)
                                .fetch_optional(&db)
                                .await;

                                match customer_wallet_res {
                                    Ok(Some(wallet)) => {
                                        let c_id: i64 = wallet.get("customer_id");
                                        let m_id: i64 = wallet.get("merchant_id");
                                        let crypto_str: String = wallet.get("crypto_type");
                                        info!("WebSocket detected static deposit for customer {} on address {}", c_id, addr_clone);
                                        if let Err(e) = v
                                            .verify_customer_deposit(
                                                c_id,
                                                &signature,
                                                m_id,
                                                &crypto_str,
                                                sandbox_mode,
                                            )
                                            .await
                                        {
                                            error!("Static deposit verification failed for customer {}: {}", c_id, e);
                                        }
                                    }
                                    Ok(None) => {
                                        // Check if this is a static merchant address
                                        // Check if this is a static merchant address across any environment (Must be active)
                                        let merchant_wallet_res = sqlx::query(
                                            "SELECT merchant_id, crypto_type FROM merchant_wallets WHERE address = $1 AND is_active = true"
                                        )
                                        .bind(&addr_clone)
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
                                    }
                                    Err(e) => error!(
                                        "Failed to query customer wallet for address {}: {}",
                                        addr_clone, e
                                    ),
                                }
                                return;
                            }
                            for row in rows {
                                let p_id: i64 = row.get("id");
                                let m_id: i64 = row.get("merchant_id");
                                info!(
                                    "Verifying payment {} for signature {} on address {}",
                                    p_id, signature, addr_clone
                                );
                                if let Err(e) =
                                    v.verify_payment_by_hash(p_id, &signature, m_id).await
                                {
                                    error!(
                                        "WebSocket verification failed for payment {}: {}",
                                        p_id, e
                                    );
                                }
                            }
                        }
                        Err(e) => error!(
                            "Failed to query pending payments for address {}: {}",
                            addr_clone, e
                        ),
                    }
                });
            });

            // Start listening (this block is long-running)
            if let Err(e) = monitor.listen_for_events(addresses, rx, callback).await {
                error!(
                    "Solana WebSocket monitor crashed: {}. Reconnecting in 2s...",
                    e
                );
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }

    /// Run Bitcoin real-time monitor (Tiered polling)
    async fn run_btc_monitor(&self, sandbox_mode: bool) {
        let cluster_name = if sandbox_mode { "Testnet" } else { "Mainnet" };
        info!(
            "Starting Bitcoin {} monitor (tiered polling)...",
            cluster_name
        );

        // Blockstream free tier: 700 req/hour = ~1 req every 5.14s.
        // With 6s between addresses we stay safely within limits.
        const POLL_INTERVAL_SECS: u64 = 180; // full-batch every 3 minutes
        const INTER_ADDRESS_DELAY_MS: u64 = 6_000; // 6 seconds between addresses
        const STATIC_POLL_EVERY_N_CYCLES: u64 = 5; // Static wallets polled every 5th cycle (~15 min)

        let mut poll_interval = interval(Duration::from_secs(POLL_INTERVAL_SECS));
        let mut cycle_count: u64 = 0;

        loop {
            poll_interval.tick().await;
            cycle_count += 1;

            // Tier 1: Always fetch active payment addresses
            let mut addresses = Self::fetch_btc_active_addresses(&self.db_pool, sandbox_mode).await;
            let active_count = addresses.len();

            // Tier 2: Include static wallets every Nth cycle
            let is_full_cycle = cycle_count.is_multiple_of(STATIC_POLL_EVERY_N_CYCLES);
            if is_full_cycle {
                let static_addrs =
                    Self::fetch_btc_static_addresses(&self.db_pool, sandbox_mode).await;
                let static_count = static_addrs.len();
                // Merge, dedup (active payment addresses may overlap with static wallets)
                for addr in static_addrs {
                    if !addresses.contains(&addr) {
                        addresses.push(addr);
                    }
                }
                tracing::debug!(
                    "[BTC] Full cycle #{}: {} active + {} static = {} total addresses",
                    cycle_count,
                    active_count,
                    static_count,
                    addresses.len()
                );
            } else {
                tracing::debug!(
                    "[BTC] Fast cycle #{}: {} active payment addresses only",
                    cycle_count,
                    active_count
                );
            }

            if addresses.is_empty() {
                continue;
            }

            let verifier = PaymentVerifier::new(
                self.db_pool.clone(),
                (*self.webhook_service).clone(),
                self.price_service.clone(),
                self.config.clone(),
                self.redis_client.clone(),
                self.notification_service.clone(),
                self.balance_service.clone(),
            );

            let monitor = crate::payment::blockchain_monitor::btc_monitor::BtcMonitor::from_config(
                &self.config,
                sandbox_mode,
            );

            let mut primary_rate_limited = false;
            let batch_start = tokio::time::Instant::now();

            for address in &addresses {
                // Respect inter-address delay to stay within rate limits
                tokio::time::sleep(Duration::from_millis(INTER_ADDRESS_DELAY_MS)).await;

                if primary_rate_limited {
                    tracing::debug!(
                        "[BTC] Primary rate-limited this batch — using backup for: {}",
                        &address[..12]
                    );
                }

                match monitor.get_transactions_to_address(address, 5, None).await {
                    Ok(transactions) => {
                        for tx in transactions {
                            // Check for pending invoice payments
                            let pending_res = sqlx::query(
                                "SELECT id, merchant_id FROM payment_transactions WHERE to_address = $1 AND status IN ('PENDING', 'CONFIRMING')"
                            )
                            .bind(address)
                            .fetch_all(&self.db_pool)
                            .await;

                            match pending_res {
                                Ok(rows) => {
                                    use sqlx::Row;
                                    // If there are pending payments, verify them
                                    if !rows.is_empty() {
                                        for row in rows {
                                            let p_id: i64 = row.get("id");
                                            let m_id: i64 = row.get("merchant_id");
                                            if let Err(e) = verifier
                                                .verify_payment_by_hash(p_id, &tx.hash, m_id)
                                                .await
                                            {
                                                tracing::trace!(
                                                    "BTC verify payment {} hash {}: {}",
                                                    p_id,
                                                    tx.hash,
                                                    e
                                                );
                                            }
                                        }
                                    } else if is_full_cycle {
                                        // No pending payments — this is a static wallet
                                        // Check if it's a merchant static deposit
                                        let merchant_wallet = sqlx::query(
                                            "SELECT merchant_id FROM merchant_wallets WHERE address = $1 AND crypto_type = 'BTC' AND sandbox_mode = $2 AND is_active = true"
                                        )
                                        .bind(address)
                                        .bind(sandbox_mode)
                                        .fetch_optional(&self.db_pool)
                                        .await;

                                        if let Ok(Some(row)) = merchant_wallet {
                                            use sqlx::Row;
                                            let m_id: i64 = row.get("merchant_id");
                                            if let Err(e) = verifier
                                                .verify_merchant_deposit(
                                                    m_id,
                                                    &tx.hash,
                                                    "BTC",
                                                    sandbox_mode,
                                                )
                                                .await
                                            {
                                                tracing::trace!(
                                                    "BTC static merchant deposit {}: {}",
                                                    tx.hash,
                                                    e
                                                );
                                            }
                                        } else {
                                            // Check if it's a customer static deposit
                                            let customer_wallet = sqlx::query(
                                                "SELECT customer_id, merchant_id FROM merchant_customer_wallets WHERE address = $1 AND crypto_type = 'BTC' AND sandbox_mode = $2"
                                            )
                                            .bind(address)
                                            .bind(sandbox_mode)
                                            .fetch_optional(&self.db_pool)
                                            .await;

                                            if let Ok(Some(row)) = customer_wallet {
                                                use sqlx::Row;
                                                let c_id: i64 = row.get("customer_id");
                                                let m_id: i64 = row.get("merchant_id");
                                                if let Err(e) = verifier
                                                    .verify_customer_deposit(
                                                        c_id,
                                                        &tx.hash,
                                                        m_id,
                                                        "BTC",
                                                        sandbox_mode,
                                                    )
                                                    .await
                                                {
                                                    tracing::trace!(
                                                        "BTC static customer deposit {}: {}",
                                                        tx.hash,
                                                        e
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => error!(
                                    "Failed to query pending BTC payments for {}: {}",
                                    address, e
                                ),
                            }
                        }
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        if err_msg.contains("429") || err_msg.contains("Too Many Requests") {
                            if !primary_rate_limited {
                                warn!(
                                    "[BTC] Primary API rate-limited (429). Will use backup-only for the remaining {} addresses in this batch.",
                                    addresses.len()
                                );
                                primary_rate_limited = true;
                            }
                            warn!(
                                "[BTC] Both providers failed for address {}: {}",
                                &address[..12],
                                err_msg
                            );
                        } else if err_msg.contains("is invalid for") {
                            warn!(
                                "[BTC] Network mismatch for address {}: {}",
                                address, err_msg
                            );
                        } else {
                            warn!(
                                "[BTC] Failed to fetch transactions for {}: {}",
                                &address[..12],
                                err_msg
                            );
                        }
                    }
                }
            }

            // If the batch finished quickly, sleep the remainder to avoid double-polling
            let elapsed = batch_start.elapsed();
            let min_batch_duration = Duration::from_secs(POLL_INTERVAL_SECS / 2);
            if elapsed < min_batch_duration {
                let remaining = min_batch_duration - elapsed;
                tracing::debug!(
                    "[BTC] Batch done in {:?}. Sleeping {:?} before next poll.",
                    elapsed,
                    remaining
                );
                tokio::time::sleep(remaining).await;
            }
        }
    }

    /// Background task to process pending withdrawals
    async fn run_withdrawal_processor(&self) {
        let mut interval = interval(Duration::from_secs(15));
        let processor = crate::services::withdrawal_processor::WithdrawalProcessor::new(
            self.db_pool.clone(),
            self.config.clone(),
            self.notification_service.clone(),
            self.blockchain_sender.clone(),
            self.balance_service.clone(),
        );

        loop {
            interval.tick().await;

            // Find PENDING withdrawals (or stuck PROCESSING rows older than 15 mins)
            let pending_res = sqlx::query(
                "SELECT withdrawal_id FROM withdrawals WHERE status = 'PENDING' OR (status = 'PROCESSING' AND updated_at < NOW() - INTERVAL '15 minutes') ORDER BY created_at ASC LIMIT 10"
            )
            .fetch_all(&self.db_pool)
            .await;

            match pending_res {
                Ok(rows) => {
                    use sqlx::Row;
                    for row in rows {
                        let wd_id: String = row.get("withdrawal_id");
                        if let Err(e) = processor.process_withdrawal(&wd_id).await {
                            error!("Error processing withdrawal {}: {}", wd_id, e);
                        }
                    }
                }
                Err(e) => error!("Error fetching pending withdrawals: {}", e),
            }
        }
    }

    /// Background task to monitor merchant balances and send alerts
    async fn run_balance_monitor(&self) {
        let mut interval = interval(Duration::from_secs(3600)); // Check every hour
        let webhook_notif = Arc::new(WebhookNotificationService::new(self.db_pool.clone()));
        let monitor = BalanceMonitoringService::new(
            self.db_pool.clone(),
            self.notification_service.clone(),
            self.price_service.clone(),
            webhook_notif,
        );

        loop {
            interval.tick().await;

            if let Err(e) = monitor.check_low_balances().await {
                error!("Error checking low balances: {}", e);
            }
        }
    }

    /// Background task to prune obsolete metrics and emit webhook telemetries
    async fn run_database_maintenance_cron(&self) {
        // Run completely once a day (86400 seconds)
        let mut interval = interval(Duration::from_secs(86400));

        loop {
            interval.tick().await;

            tracing::info!("Starting scheduled database maintenance cron...");

            // Aggressively flush old metric payloads
            let result = sqlx::query(
                "DELETE FROM system_health_history WHERE timestamp < NOW() - INTERVAL '30 days'",
            )
            .execute(&self.db_pool)
            .await;

            match result {
                Ok(query_result) => {
                    let rows_affected = query_result.rows_affected();

                    if rows_affected > 0 {
                        tracing::info!("Maintenance cron pruned {} obsolete metric logs. Dispatching webhooks.", rows_affected);

                        // 1. Ledger as a public System Incident (resolved Maintenance)
                        let incident_desc = format!("Routine automated database maintenance successfully purged {} obsolete system health logs.", rows_affected);
                        let _ = sqlx::query(
                            "INSERT INTO system_incidents (title, description, status, severity) VALUES ($1, $2, 'resolved', 'maintenance')"
                        )
                        .bind("Scheduled Storage Optimization")
                        .bind(&incident_desc)
                        .execute(&self.db_pool)
                        .await;

                        // 2. Transmit Discord Payload precisely if defined
                        let settings_res = sqlx::query(
                            "SELECT discord_webhook_url FROM fee_sweep_settings LIMIT 1",
                        )
                        .fetch_optional(&self.db_pool)
                        .await;

                        if let Ok(Some(row)) = settings_res {
                            use sqlx::Row;
                            if let Ok(Some(discord_url)) =
                                row.try_get::<Option<String>, _>("discord_webhook_url")
                            {
                                let client = reqwest::Client::new();
                                let msg = serde_json::json!({
                                    "content": format!("🧹 **Database Optimization Complete**\n`{}`", incident_desc)
                                });
                                let _ = client.post(&discord_url).json(&msg).send().await;
                            }
                        }
                    } else {
                        tracing::info!("Maintenance cron completed perfectly - no records old enough to purge yet.");
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "FATAL: Maintenance cron failed to clean metrics table: {}",
                        e
                    );
                }
            }
        }
    }
}
