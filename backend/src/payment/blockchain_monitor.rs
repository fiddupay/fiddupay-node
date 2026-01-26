// Multi-Chain Blockchain Monitor
// Provides unified interface for monitoring payments across all supported blockchains

use async_trait::async_trait;
use reqwest::Client;
use rust_decimal::Decimal;
use std::str::FromStr;
use tracing::{info, warn, error};

use super::models::{BlockchainTransaction, CryptoType};

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
    pub fn new_bsc(api_key: Option<String>) -> Self {
        let api_url = std::env::var("BSCSCAN_API_URL")
            .unwrap_or_else(|_| "https://api.bscscan.com/api".to_string());

        Self {
            client: Client::new(),
            api_url,
            api_key,
            chain_name: "BSC",
            decimals: 18, // USDT on BSC has 18 decimals
        }
    }

    pub fn new_arbitrum(api_key: Option<String>) -> Self {
        let api_url = std::env::var("ARBISCAN_API_URL")
            .unwrap_or_else(|_| "https://api.arbiscan.io/api".to_string());

        Self {
            client: Client::new(),
            api_url,
            api_key,
            chain_name: "Arbitrum",
            decimals: 6, // USDT on Arbitrum has 6 decimals
        }
    }

    pub fn new_polygon(api_key: Option<String>) -> Self {
        let api_url = std::env::var("POLYGONSCAN_API_URL")
            .unwrap_or_else(|_| "https://api.polygonscan.com/api".to_string());

        Self {
            client: Client::new(),
            api_url,
            api_key,
            chain_name: "Polygon",
            decimals: 6, // USDT on Polygon has 6 decimals
        }
    }

    pub fn new_ethereum(api_key: Option<String>) -> Self {
        let api_url = std::env::var("ETHERSCAN_API_URL")
            .unwrap_or_else(|_| "https://api.etherscan.io/v2/api".to_string());

        Self {
            client: Client::new(),
            api_url,
            api_key,
            chain_name: "Ethereum",
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
        info!("🔍 Fetching {} transaction: {}", self.chain_name, tx_hash);

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
    ) -> Result<Vec<BlockchainTransaction>, Box<dyn std::error::Error + Send + Sync>> {
        info!("📡 Fetching {} transactions for address: {}", self.chain_name, address);

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
pub fn get_blockchain_monitor(crypto_type: &CryptoType, config: &crate::config::Config) -> Box<dyn BlockchainMonitor> {
    match crypto_type {
        CryptoType::Sol => Box::new(super::sol_monitor::SolanaMonitor::new(config, None)),
        CryptoType::UsdtSpl => Box::new(super::sol_monitor::SolanaMonitor::new(config, None)),
        CryptoType::UsdtBep20 => {
            let api_key = std::env::var("ETHERSCAN_API_KEY").ok();
            Box::new(EvmMonitor::new_bsc(api_key))
        }
        CryptoType::UsdtArbitrum => {
            let api_key = std::env::var("ETHERSCAN_API_KEY").ok();
            Box::new(EvmMonitor::new_arbitrum(api_key))
        }
        CryptoType::UsdtPolygon => {
            let api_key = std::env::var("ETHERSCAN_API_KEY").ok();
            Box::new(EvmMonitor::new_polygon(api_key))
        }
        CryptoType::UsdtEth => {
            let api_key = std::env::var("ETHERSCAN_API_KEY").ok();
            Box::new(EvmMonitor::new_ethereum(api_key))
        }
        CryptoType::Eth => {
            let api_key = std::env::var("ETHERSCAN_API_KEY").ok();
            Box::new(EvmMonitor::new_ethereum(api_key))
        }
        CryptoType::Arb => {
            let api_key = std::env::var("ETHERSCAN_API_KEY").ok();
            Box::new(EvmMonitor::new_arbitrum(api_key))
        }
        CryptoType::Matic => {
            let api_key = std::env::var("ETHERSCAN_API_KEY").ok();
            Box::new(EvmMonitor::new_polygon(api_key))
        }
        CryptoType::Bnb => {
            let api_key = std::env::var("ETHERSCAN_API_KEY").ok();
            Box::new(EvmMonitor::new_bsc(api_key))
        }
    }
}
