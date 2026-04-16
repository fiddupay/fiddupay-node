use super::BlockchainMonitor;
use crate::payment::models::BlockchainTransaction;
use crate::utils::bitcoin_api::{get_with_failover, BitcoinApiConfig};
use async_trait::async_trait;
use rust_decimal::Decimal;
use tracing::{info, warn};

pub struct BtcMonitor {
    api_config: BitcoinApiConfig,
    network_name: &'static str,
    is_sandbox: bool,
}

impl BtcMonitor {
    pub fn new(api_url: String, is_sandbox: bool) -> Self {
        let network_name = if is_sandbox {
            "Bitcoin Testnet"
        } else {
            "Bitcoin Mainnet"
        };
        // Build a simple config using the given URL as both primary and backup.
        // When created from app Config via from_config(), both primary and backup are set properly.
        let api_config = BitcoinApiConfig {
            primary_url: api_url.trim_end_matches('/').to_string(),
            backup_url: if is_sandbox {
                "https://mempool.space/testnet/api".to_string()
            } else {
                "https://mempool.space/api".to_string()
            },
        };
        Self {
            api_config,
            network_name,
            is_sandbox,
        }
    }

    /// Create from full app config (uses primary + backup from env).
    pub fn from_config(config: &crate::config::Config, is_sandbox: bool) -> Self {
        let network_name = if is_sandbox {
            "Bitcoin Testnet"
        } else {
            "Bitcoin Mainnet"
        };
        let api_config = BitcoinApiConfig::from_config(config, is_sandbox);
        Self {
            api_config,
            network_name,
            is_sandbox,
        }
    }

    /// Check if a Bitcoin address is valid for the current network
    pub fn is_address_valid_for_network(&self, address: &str) -> bool {
        if self.is_sandbox {
            address.starts_with('m')
                || address.starts_with('n')
                || address.starts_with('2')
                || address.starts_with("tb1")
        } else {
            address.starts_with('1') || address.starts_with('3') || address.starts_with("bc1")
        }
    }
}

#[async_trait]
impl BlockchainMonitor for BtcMonitor {
    async fn get_transaction_details(
        &self,
        tx_hash: &str,
        target_address: Option<&str>,
    ) -> Result<BlockchainTransaction, Box<dyn std::error::Error + Send + Sync>> {
        // Validate target address if provided
        if let Some(target) = target_address {
            if !self.is_address_valid_for_network(target) {
                return Err(
                    format!("Address {} is invalid for {}", target, self.network_name).into(),
                );
            }
        }

        info!(" Fetching {} transaction: {}", self.network_name, tx_hash);

        let data = get_with_failover(&self.api_config, &format!("tx/{}", tx_hash))
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;

        // Blockstream API returns transaction details directly
        let status = data.get("status").ok_or("No status in response")?;
        let confirmed = status
            .get("confirmed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let block_height = status.get("block_height").and_then(|v| v.as_u64());
        let timestamp_secs = status.get("block_time").and_then(|v| v.as_i64());

        let mut from_address = "unknown".to_string();
        let mut to_address = "unknown".to_string();
        let mut amount = Decimal::ZERO;

        if let Some(vin) = data.get("vin").and_then(|v| v.as_array()) {
            if let Some(first_in) = vin.first() {
                from_address = first_in
                    .get("prevout")
                    .and_then(|p| p.get("scriptpubkey_address"))
                    .and_then(|a| a.as_str())
                    .unwrap_or("unknown")
                    .to_string();
            }
        }

        if let Some(vout) = data.get("vout").and_then(|v| v.as_array()) {
            let mut total_satoshis = 0u64;
            let mut found_target = false;

            for out in vout {
                let addr = out
                    .get("scriptpubkey_address")
                    .and_then(|a| a.as_str())
                    .unwrap_or("");

                if !addr.is_empty() {
                    let matches = if let Some(target) = target_address {
                        addr == target
                    } else {
                        // If no target provided, we pick the first one with a valid address
                        // that isn't the return address (simplified heuristic: first one we find)
                        !found_target
                    };

                    if matches {
                        if !found_target {
                            to_address = addr.to_string();
                            found_target = true;
                        }
                        total_satoshis += out.get("value").and_then(|v| v.as_u64()).unwrap_or(0);
                    }
                }
            }

            if found_target {
                amount = Decimal::from(total_satoshis) / Decimal::from(100_000_000u64);
                info!(" Found BTC transfer: to={}, amount={}", to_address, amount);
            }
        }

        let current_height = self.get_current_height().await?;
        let confirmations = match block_height {
            Some(h) if current_height >= h => (current_height - h + 1) as u32,
            _ => 0,
        };

        let timestamp = timestamp_secs
            .and_then(|s| chrono::DateTime::from_timestamp(s, 0))
            .unwrap_or_else(chrono::Utc::now);

        Ok(BlockchainTransaction {
            hash: tx_hash.to_string(),
            from_address,
            to_address,
            amount,
            confirmations,
            block_number: block_height,
            timestamp: Some(timestamp),
            success: confirmed,
            token_mint: None,
        })
    }

    async fn get_transactions_to_address(
        &self,
        address: &str,
        limit: usize,
        min_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<BlockchainTransaction>, Box<dyn std::error::Error + Send + Sync>> {
        // Validate address for network
        if !self.is_address_valid_for_network(address) {
            return Err(format!("Address {} is invalid for {}", address, self.network_name).into());
        }

        info!(
            " Fetching {} transactions for address: {}",
            self.network_name, address
        );

        let data = get_with_failover(&self.api_config, &format!("address/{}/txs", address))
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;

        let data: Vec<serde_json::Value> = data.as_array().cloned().unwrap_or_default();

        let mut transactions = Vec::new();
        for tx_data in data.iter().take(limit) {
            if let Some(txid) = tx_data.get("txid").and_then(|v| v.as_str()) {
                match self.get_transaction_details(txid, Some(address)).await {
                    Ok(tx) => {
                        if let Some(min_ts) = min_timestamp {
                            if let Some(ts) = tx.timestamp {
                                if ts < min_ts {
                                    continue;
                                }
                            }
                        }
                        transactions.push(tx);
                    }
                    Err(e) => warn!("Failed to get BTC transaction {}: {}", txid, e),
                }
            }
        }

        Ok(transactions)
    }

    fn blockchain_name(&self) -> &'static str {
        self.network_name
    }

    async fn listen_for_events(
        &self,
        _addresses: Vec<String>,
        _new_addresses_rx: tokio::sync::mpsc::UnboundedReceiver<String>,
        _callback: std::sync::Arc<dyn Fn(String, String) + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Bitcoin does not have a reliable public WebSocket for address monitoring.
        // For now, BtcMonitor only supports polling via get_transactions_to_address.
        Err("Bitcoin WebSocket monitoring is not yet implemented".into())
    }
}

impl BtcMonitor {
    async fn get_current_height(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let data = get_with_failover(&self.api_config, "blocks/tip/height")
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;

        // The response is a plain number (JSON number)
        data.as_u64()
            .ok_or_else(|| "Invalid block height response".into())
    }
}
