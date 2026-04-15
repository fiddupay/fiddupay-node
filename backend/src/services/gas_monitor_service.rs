use crate::payment::models::CryptoType;
use crate::services::blockchain_transaction_sender::BlockchainTransactionSender;
use chrono::{DateTime, Utc};
use reqwest::Client;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde_json::json;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{error, info};

pub struct GasMonitorService {
    db_pool: PgPool,
    config: crate::config::Config,
}

impl GasMonitorService {
    pub fn new(db_pool: PgPool, config: crate::config::Config) -> Self {
        Self { db_pool, config }
    }

    /// Run the gas monitoring loop
    pub async fn start_monitoring(&self) {
        let mut last_alert_times: HashMap<CryptoType, DateTime<Utc>> = HashMap::new();
        let http_client = Client::new();

        loop {
            // Check every 30 minutes
            tokio::time::sleep(Duration::from_secs(1800)).await;

            if let Err(e) = self
                .check_gas_prices(&http_client, &mut last_alert_times)
                .await
            {
                error!("Error in gas monitoring loop: {}", e);
            }
        }
    }

    async fn check_gas_prices(
        &self,
        http_client: &Client,
        last_alert_times: &mut HashMap<CryptoType, DateTime<Utc>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 1. Get Settings
        let settings = match sqlx::query(
            "SELECT discord_webhook_url, gas_alert_threshold_gwei, gas_alert_threshold_lamports FROM fee_sweep_settings LIMIT 1"
        )
        .fetch_optional(&self.db_pool)
        .await? {
            Some(row) => row,
            None => return Ok(()),
        };

        let webhook_url: Option<String> = settings.try_get("discord_webhook_url").ok().flatten();
        let threshold_gwei: Option<Decimal> =
            settings.try_get("gas_alert_threshold_gwei").ok().flatten();
        let threshold_lamports: Option<i64> = settings
            .try_get("gas_alert_threshold_lamports")
            .ok()
            .flatten();

        if webhook_url.is_none() || (threshold_gwei.is_none() && threshold_lamports.is_none()) {
            return Ok(());
        }

        let sender = BlockchainTransactionSender::new(self.config.clone());
        let evm_networks = vec![
            CryptoType::Eth,
            CryptoType::Bnb,
            CryptoType::Matic,
            CryptoType::Arb,
        ];

        // 2. Check EVM networks
        if let Some(target_gwei) = threshold_gwei {
            for crypto in evm_networks {
                if let Ok(gas_wei) = sender.get_current_gas_price(crypto.clone(), false).await {
                    let gas_gwei = Decimal::new(gas_wei.as_u128() as i64, 9); // Wei to Gwei

                    // Save to history
                    let network_str = match crypto.clone() {
                        CryptoType::Eth => "ETHEREUM",
                        CryptoType::Bnb => "BSC",
                        CryptoType::Matic => "POLYGON",
                        CryptoType::Arb => "ARBITRUM",
                        _ => "UNKNOWN",
                    };

                    let _ = sqlx::query(
                        "INSERT INTO gas_history (network, base_fee_gwei) VALUES ($1, $2)",
                    )
                    .bind(network_str)
                    .bind(gas_gwei)
                    .execute(&self.db_pool)
                    .await;

                    // Alert logic (cooldown: 4 hours per network)
                    if gas_gwei <= target_gwei {
                        let now = Utc::now();
                        let last_alert = last_alert_times
                            .get(&crypto)
                            .copied()
                            .unwrap_or(now - chrono::Duration::days(1));

                        if now.signed_duration_since(last_alert).num_hours() >= 4 {
                            if let Some(url) = &webhook_url {
                                let msg = format!("🟢 **Low Gas Alert**: {} gas is currently at {:.2} Gwei (Threshold: {} Gwei). Good time to trigger manual fee sweeps!", network_str, gas_gwei, target_gwei);
                                let _ = self.send_discord_alert(http_client, url, &msg).await;
                            }
                            last_alert_times.insert(crypto.clone(), now);
                        }
                    }
                }
            }
        }

        // 3. Check Solana
        if let Some(target_lamports) = threshold_lamports {
            if let Ok(lamports) = sender.get_solana_fee(false).await {
                // Save to history
                let _ = sqlx::query(
                    "INSERT INTO gas_history (network, base_fee_lamports) VALUES ($1, $2)",
                )
                .bind("SOLANA")
                .bind(lamports as i64)
                .execute(&self.db_pool)
                .await;

                // Alert logic
                if (lamports as i64) <= target_lamports {
                    let now = Utc::now();
                    let last_alert = last_alert_times
                        .get(&CryptoType::Sol)
                        .copied()
                        .unwrap_or(now - chrono::Duration::days(1));

                    if now.signed_duration_since(last_alert).num_hours() >= 4 {
                        if let Some(url) = &webhook_url {
                            let msg = format!("🟢 **Low Gas Alert**: SOLANA base fee is currently at {} lamports (Threshold: {} lamports). Good time to trigger manual fee sweeps!", lamports, target_lamports);
                            let _ = self.send_discord_alert(http_client, url, &msg).await;
                        }
                        last_alert_times.insert(CryptoType::Sol, now);
                    }
                }
            }
        }

        Ok(())
    }

    async fn send_discord_alert(
        &self,
        client: &Client,
        webhook_url: &str,
        message: &str,
    ) -> Result<(), reqwest::Error> {
        let payload = json!({
            "content": message
        });

        client.post(webhook_url).json(&payload).send().await?;

        Ok(())
    }
}
