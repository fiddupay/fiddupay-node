// Multi-Chain Blockchain Monitor
// Provides unified interface for monitoring payments across all supported blockchains

use async_trait::async_trait;
use reqwest::Client;
use rust_decimal::Decimal;
use std::str::FromStr;
use tracing::{info, warn, error};

use super::models::{BlockchainTransaction, CryptoType};

pub mod btc_monitor;

/// Trait for blockchain monitoring across different chains
#[async_trait]
pub trait BlockchainMonitor: Send + Sync {
    /// Get transaction details by hash
    async fn get_transaction_details(
        &self,
        tx_hash: &str,
    ) -> Result<BlockchainTransaction, Box<dyn std::error::Error + Send + Sync>>;

    /// Get recent transactions for an address
    async fn get_transactions_to_address(
        &self,
        address: &str,
        limit: usize,
        min_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<BlockchainTransaction>, Box<dyn std::error::Error + Send + Sync>>;

    /// Get blockchain name
    fn blockchain_name(&self) -> &'static str;
}

/// EVM-based blockchain monitor (BSC, Arbitrum, Polygon)
/// Uses Etherscan-like API for transaction fetching
pub struct EvmMonitor {
    client: Client,
    api_url: String,
    api_key: Option<String>,
    chain_name: &'static str,
    decimals: u32, // Token decimals (18 for most ERC20)
}

impl EvmMonitor {
    pub fn new_bsc(config: &crate::config::Config, is_sandbox: bool) -> Self {
        Self {
            client: Client::new(),
            api_url: if is_sandbox { config.bscscan_testnet_api_url.clone() } else { config.bscscan_api_url.clone() },
            api_key: config.etherscan_api_key.clone(),
            chain_name: if is_sandbox { "BSC Testnet" } else { "BSC" },
            decimals: 18, // USDT on BSC has 18 decimals
        }
    }

    pub fn new_arbitrum(config: &crate::config::Config, is_sandbox: bool) -> Self {
        Self {
            client: Client::new(),
            api_url: if is_sandbox { config.arbiscan_sepolia_api_url.clone() } else { config.arbiscan_api_url.clone() },
            api_key: config.etherscan_api_key.clone(),
            chain_name: if is_sandbox { "Arbitrum Sepolia" } else { "Arbitrum" },
            decimals: 6, // USDT on Arbitrum has 6 decimals
        }
    }

    pub fn new_polygon(config: &crate::config::Config, is_sandbox: bool) -> Self {
        Self {
            client: Client::new(),
            api_url: if is_sandbox { config.polygonscan_mumbai_api_url.clone() } else { config.polygonscan_api_url.clone() },
            api_key: config.etherscan_api_key.clone(),
            chain_name: if is_sandbox { "Polygon Mumbai" } else { "Polygon" },
            decimals: 6, // USDT on Polygon has 6 decimals
        }
    }

    pub fn new_ethereum(config: &crate::config::Config, is_sandbox: bool) -> Self {
        Self {
            client: Client::new(),
            api_url: if is_sandbox { config.etherscan_sepolia_api_url.clone() } else { config.etherscan_api_url.clone() },
            api_key: config.etherscan_api_key.clone(),
            chain_name: if is_sandbox { "Ethereum Sepolia" } else { "Ethereum" },
            decimals: 6, // USDT on Ethereum has 6 decimals
        }
    }
}

#[async_trait]
impl BlockchainMonitor for EvmMonitor {
    async fn get_transaction_details(
        &self,
        tx_hash: &str,
    ) -> Result<BlockchainTransaction, Box<dyn std::error::Error + Send + Sync>> {
        info!(" Fetching {} transaction: {}", self.chain_name, tx_hash);

        // Build API request URL
        let mut url = format!(
            "{}?module=proxy&action=eth_getTransactionByHash&txhash={}",
            self.api_url, tx_hash
        );

        if let Some(ref key) = self.api_key {
            url.push_str(&format!("&apikey={}", key));
        }

        let response = self.client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;

        // Parse transaction data
        let result = data.get("result")
            .ok_or("No result in response")?;

        if result.is_null() {
            return Err("Transaction not found".into());
        }

        let from_address = result.get("from")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let to_address = result.get("to")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let value_hex = result.get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("0x0");

        // Convert hex value to decimal
        let value_u128 = u128::from_str_radix(value_hex.trim_start_matches("0x"), 16)
            .unwrap_or(0);

        let amount = Decimal::from(value_u128) / Decimal::from(10u64.pow(self.decimals));

        // Get transaction receipt for confirmation status
        let block_number = result.get("blockNumber")
            .and_then(|v| v.as_str())
            .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok());

        // Get current block to calculate confirmations
        let current_block = self.get_current_block().await?;
        let confirmations = if let Some(tx_block) = block_number {
            if current_block > tx_block {
                (current_block - tx_block) as u32
            } else {
                0
            }
        } else {
            0
        };

        // Check if transaction succeeded
        let success = self.check_transaction_success(tx_hash).await?;

        // Get actual block timestamp if block number is available
        let timestamp = if let Some(block_num) = block_number {
            self.get_block_timestamp(block_num).await
                .unwrap_or_else(|e| {
                    warn!("Failed to get block timestamp: {}, using current time", e);
                    chrono::Utc::now()
                })
        } else {
            chrono::Utc::now()
        };

        Ok(BlockchainTransaction {
            hash: tx_hash.to_string(),
            from_address,
            to_address,
            amount,
            confirmations,
            block_number,
            timestamp: Some(timestamp),
            success,
        })
    }

    async fn get_transactions_to_address(
        &self,
        address: &str,
        limit: usize,
        min_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<BlockchainTransaction>, Box<dyn std::error::Error + Send + Sync>> {
        info!(" Fetching {} transactions for address: {}", self.chain_name, address);

        // Build API request URL for transaction list
        let mut url = format!(
            "{}?module=account&action=txlist&address={}&startblock=0&endblock=99999999&page=1&offset={}&sort=desc",
            self.api_url, address, limit
        );

        if let Some(ref key) = self.api_key {
            url.push_str(&format!("&apikey={}", key));
        }

        let response = self.client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;

        let result = data.get("result")
            .and_then(|v| v.as_array())
            .ok_or("Invalid response format")?;

        let mut transactions = Vec::new();

        for tx in result.iter().take(limit) {
            let hash = tx.get("hash")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // Optimization: skip if transaction is obviously too old (EVM APIs often provide timestamp in list)
            if let (Some(min_ts), Some(tx_ts_str)) = (min_timestamp, tx.get("timeStamp").and_then(|v| v.as_str())) {
                if let Ok(ts_secs) = tx_ts_str.parse::<i64>() {
                    if let Some(ts) = chrono::DateTime::from_timestamp(ts_secs, 0) {
                        if ts < min_ts - chrono::Duration::seconds(60) {
                            continue;
                        }
                    }
                }
            }

            // Get full transaction details
            match self.get_transaction_details(&hash).await {
                Ok(blockchain_tx) => transactions.push(blockchain_tx),
                Err(e) => warn!("Failed to get transaction {}: {}", hash, e),
            }
        }

        info!(" Found {} {} transactions", transactions.len(), self.chain_name);
        Ok(transactions)
    }

    fn blockchain_name(&self) -> &'static str {
        self.chain_name
    }
}

impl EvmMonitor {
    /// Get current block number
    async fn get_current_block(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let mut url = format!(
            "{}?module=proxy&action=eth_blockNumber",
            self.api_url
        );

        if let Some(ref key) = self.api_key {
            url.push_str(&format!("&apikey={}", key));
        }

        let response = self.client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;

        let result = data.get("result")
            .and_then(|v| v.as_str())
            .ok_or("No result in response")?;

        let block_number = u64::from_str_radix(result.trim_start_matches("0x"), 16)?;
        Ok(block_number)
    }

    /// Check if transaction succeeded
    async fn check_transaction_success(&self, tx_hash: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut url = format!(
            "{}?module=proxy&action=eth_getTransactionReceipt&txhash={}",
            self.api_url, tx_hash
        );

        if let Some(ref key) = self.api_key {
            url.push_str(&format!("&apikey={}", key));
        }

        let response = self.client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;

        let result = data.get("result")
            .ok_or("No result in response")?;

        if result.is_null() {
            return Ok(false);
        }

        // Status "0x1" means success
        let status = result.get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("0x0");

        Ok(status == "0x1")
    }

    /// Get block timestamp by block number
    async fn get_block_timestamp(&self, block_number: u64) -> Result<chrono::DateTime<chrono::Utc>, Box<dyn std::error::Error + Send + Sync>> {
        let mut url = format!(
            "{}?module=proxy&action=eth_getBlockByNumber&tag=0x{:x}&boolean=false",
            self.api_url, block_number
        );

        if let Some(ref key) = self.api_key {
            url.push_str(&format!("&apikey={}", key));
        }

        let response = self.client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;

        let result = data.get("result")
            .ok_or("No result in response")?;

        if result.is_null() {
            return Err("Block not found".into());
        }

        // Get timestamp from block (hex string)
        let timestamp_hex = result.get("timestamp")
            .and_then(|v| v.as_str())
            .ok_or("No timestamp in block")?;

        // Convert hex timestamp to u64
        let timestamp_secs = u64::from_str_radix(timestamp_hex.trim_start_matches("0x"), 16)?;

        // Convert to DateTime
        chrono::DateTime::from_timestamp(timestamp_secs as i64, 0)
            .ok_or_else(|| "Invalid timestamp".into())
    }
}

/// Factory function to create appropriate blockchain monitor
pub fn get_blockchain_monitor(crypto_type: &CryptoType, config: crate::config::Config, is_sandbox: bool) -> Box<dyn BlockchainMonitor> {
    match crypto_type.network() {
        "SOLANA" | "SOLANA_SPL" => {
            let rpc_url = if is_sandbox {
                Some(config.solana_devnet_rpc_url.clone())
            } else {
                None // Uses default from config (mainnet)
            };
            let expected_mint = crypto_type.token_address().map(|s| s.to_string());
            Box::new(crate::payment::sol_monitor::SolanaMonitor::new(&config, rpc_url, expected_mint))
        },
        "BITCOIN" => Box::new(crate::payment::btc_monitor::BtcMonitor::new(is_sandbox)),
        "ETHEREUM" => Box::new(EvmMonitor::new_ethereum(&config, is_sandbox)),
        "BEP20" => Box::new(EvmMonitor::new_bsc(&config, is_sandbox)),
        "POLYGON" => Box::new(EvmMonitor::new_polygon(&config, is_sandbox)),
        "ARBITRUM" => Box::new(EvmMonitor::new_arbitrum(&config, is_sandbox)),
        _ => {
             // Default to Solana for unknown types (fallback)
            let rpc_url = if is_sandbox {
                Some(config.solana_devnet_rpc_url.clone())
            } else {
                None
            };
            let expected_mint = crypto_type.token_address().map(|s| s.to_string());
            Box::new(crate::payment::sol_monitor::SolanaMonitor::new(&config, rpc_url, expected_mint))
        },
    }
}
