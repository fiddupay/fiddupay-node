// Bitcoin Blockchain Monitor
// Uses Blockstream API for transaction verification

use async_trait::async_trait;
use reqwest::Client;
use rust_decimal::Decimal;
use tracing::{info, warn};
use crate::payment::models::{BlockchainTransaction, CryptoType};
use super::BlockchainMonitor;

pub struct BtcMonitor {
    client: Client,
    api_url: String,
    network_name: &'static str,
    is_sandbox: bool,
}

impl BtcMonitor {
    pub fn new(is_sandbox: bool) -> Self {
        let (api_url, network_name) = if is_sandbox {
            ("https://blockstream.info/testnet/api", "Bitcoin Testnet")
        } else {
            ("https://blockstream.info/api", "Bitcoin Mainnet")
        };

        Self {
            client: Client::new(),
            api_url: api_url.to_string(),
            network_name,
            is_sandbox,
        }
    }

    /// Check if a Bitcoin address is valid for the current network
    pub fn is_address_valid_for_network(&self, address: &str) -> bool {
        if self.is_sandbox {
            // Testnet: m, n, 2, tb1
            address.starts_with('m') || address.starts_with('n') || address.starts_with('2') || address.starts_with("tb1")
        } else {
            // Mainnet: 1, 3, bc1
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
                return Err(format!("Address {} is invalid for {}", target, self.network_name).into());
            }
        }

        info!(" Fetching {} transaction: {}", self.network_name, tx_hash);

        let url = format!("{}/tx/{}", self.api_url, tx_hash);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("{} API error: {} - {}", self.network_name, status, body).into());
        }

        let data: serde_json::Value = response.json().await?;

        // Blockstream API returns transaction details directly
        let status = data.get("status").ok_or("No status in response")?;
        let confirmed = status.get("confirmed").and_then(|v| v.as_bool()).unwrap_or(false);
        let block_height = status.get("block_height").and_then(|v| v.as_u64());
        let timestamp_secs = status.get("block_time").and_then(|v| v.as_i64());

        let mut from_address = "unknown".to_string();
        let mut to_address = "unknown".to_string();
        let mut amount = Decimal::ZERO;

        if let Some(vin) = data.get("vin").and_then(|v| v.as_array()) {
            if let Some(first_in) = vin.first() {
                from_address = first_in.get("prevout")
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
                let addr = out.get("scriptpubkey_address")
                    .and_then(|a| a.as_str())
                    .unwrap_or("");
                
                if !addr.is_empty() {
                    let matches = if let Some(target) = target_address {
                        addr == target
                    } else {
                        // If no target provided, we pick the first one with a valid address
                        // that isn't the return address (simplified heuristic: first one we find)
                        if !found_target { true } else { false }
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

        let timestamp = timestamp_secs.and_then(|s| chrono::DateTime::from_timestamp(s, 0))
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

        info!(" Fetching {} transactions for address: {}", self.network_name, address);

        let url = format!("{}/address/{}/txs", self.api_url, address);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            // If it's a 404, we can assume there are no transactions (or the address is unknown to this chain)
            if status == reqwest::StatusCode::NOT_FOUND {
                warn!("Address {} not found on {}. Returning empty list.", address, self.network_name);
                return Ok(Vec::new());
            }
            return Err(format!("{} API error: {} - {}", self.network_name, status, body).into());
        }

        let data: Vec<serde_json::Value> = response.json().await?;

        let mut transactions = Vec::new();
        for tx_data in data.iter().take(limit) {
            if let Some(txid) = tx_data.get("txid").and_then(|v| v.as_str()) {
                match self.get_transaction_details(txid, Some(address)).await {
                    Ok(tx) => {
                        if let Some(min_ts) = min_timestamp {
                             if let Some(ts) = tx.timestamp {
                                 if ts < min_ts { continue; }
                             }
                        }
                        transactions.push(tx);
                    },
                    Err(e) => warn!("Failed to get BTC transaction {}: {}", txid, e),
                }
            }
        }

        Ok(transactions)
    }

    fn blockchain_name(&self) -> &'static str {
        self.network_name
    }
}

impl BtcMonitor {
    async fn get_current_height(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/blocks/tip/height", self.api_url);
        let response = self.client.get(&url).send().await?;
        let height_str = response.text().await?;
        let height = height_str.trim().parse::<u64>()?;
        Ok(height)
    }
}
