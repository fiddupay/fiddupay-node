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
        target_address: Option<&str>,
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
    token_address: Option<String>, // ERC20 token address if monitoring tokens
    chain_id: u64, // Chain ID required for Etherscan V2 API
}

impl EvmMonitor {
    pub fn new_bsc(config: &crate::config::Config, is_sandbox: bool, token_address: Option<String>, decimals: u32) -> Self {
        Self {
            client: Client::new(),
            api_url: config.etherscan_api_url.clone(),
            api_key: config.etherscan_api_key.clone(),
            chain_name: if is_sandbox { "BSC Testnet" } else { "BSC" },
            decimals,
            token_address,
            chain_id: if is_sandbox { config.bsc_testnet_chain_id } else { config.bsc_chain_id },
        }
    }

    pub fn new_arbitrum(config: &crate::config::Config, is_sandbox: bool, token_address: Option<String>, decimals: u32) -> Self {
        Self {
            client: Client::new(),
            api_url: config.etherscan_api_url.clone(),
            api_key: config.etherscan_api_key.clone(),
            chain_name: if is_sandbox { "Arbitrum Sepolia" } else { "Arbitrum" },
            decimals,
            token_address,
            chain_id: if is_sandbox { config.arbitrum_sepolia_chain_id } else { config.arbitrum_chain_id },
        }
    }

    pub fn new_polygon(config: &crate::config::Config, is_sandbox: bool, token_address: Option<String>, decimals: u32) -> Self {
        Self {
            client: Client::new(),
            api_url: config.etherscan_api_url.clone(),
            api_key: config.etherscan_api_key.clone(),
            chain_name: if is_sandbox { "Polygon Amoy" } else { "Polygon" },
            decimals,
            token_address,
            chain_id: if is_sandbox { config.polygon_amoy_chain_id } else { config.polygon_chain_id },
        }
    }

    pub fn new_ethereum(config: &crate::config::Config, is_sandbox: bool, token_address: Option<String>, decimals: u32) -> Self {
        Self {
            client: Client::new(),
            api_url: config.etherscan_api_url.clone(),
            api_key: config.etherscan_api_key.clone(),
            chain_name: if is_sandbox { "Ethereum Sepolia" } else { "Ethereum" },
            decimals,
            token_address,
            chain_id: if is_sandbox { config.ethereum_sepolia_chain_id } else { config.ethereum_chain_id },
        }
    }
}

#[async_trait]
impl BlockchainMonitor for EvmMonitor {
    async fn get_transaction_details(
        &self,
        tx_hash: &str,
        _target_address: Option<&str>,
    ) -> Result<BlockchainTransaction, Box<dyn std::error::Error + Send + Sync>> {
        info!(" Fetching {} transaction: {}", self.chain_name, tx_hash);

        // Build API request URL
        let mut url = format!(
            "{}?module=proxy&action=eth_getTransactionByHash&txhash={}&chainid={}",
            self.api_url, tx_hash, self.chain_id
        );

        if let Some(ref key) = self.api_key {
            url.push_str(&format!("&apikey={}", key));
        }

        let response = self.client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;

        // Check for API errors (Etherscan returns status="0" for errors with message in "result")
        if let Some(status) = data.get("status").and_then(|s| s.as_str()) {
            if status == "0" {
                let err_msg = data.get("result")
                    .map(|r| if r.is_string() { r.as_str().unwrap().to_string() } else { r.to_string() })
                    .unwrap_or_else(|| "Unknown EVM API Error".to_string());
                return Err(format!("EVM API Error: {}", err_msg).into());
            }
        }

        // Parse transaction data
        let result = data.get("result")
            .ok_or("No result in response")?;

        if result.is_null() {
            return Err("Transaction not found".into());
        }

        use crate::utils::api_keys::ApiKeyGenerator; // If needed, otherwise string manipulation

        let from_address = result.get("from")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut to_address = result.get("to")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let value_hex = result.get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("0x0");

        let input = result.get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("0x");

        let mut amount = Decimal::ZERO;

        // Check if this is an ERC20 token transfer
        if let Some(ref token) = self.token_address {
            // Verify 'to' is the contract address
            if to_address.trim().to_lowercase() == token.trim().to_lowercase() {
                if input.starts_with("0x") && input.len() >= 138 {
                    let sig = &input[2..10];
                    if sig == "a9059cbb" { // transfer(address,uint256)
                        let recipient_hex = &input[34..74]; // skip 2 (0x) + 8 (sig) + 24 (padding)
                        let amount_hex = &input[74..138];
                        
                        if let Ok(value_u128) = u128::from_str_radix(amount_hex, 16) {
                            amount = Decimal::from(value_u128) / Decimal::from(10u64.pow(self.decimals));
                            to_address = format!("0x{}", recipient_hex);
                            info!("Parsed ERC20 Transfer: to={}, amount={}", to_address, amount);
                        }
                    }
                }
            } else {
                 warn!("EVM ERC20 monitor expects to_address to be contract {}, got {}", token, to_address);
            }
        } else {
            // Native currency transfer
            // Convert hex value to decimal
            let value_u128 = u128::from_str_radix(value_hex.trim_start_matches("0x"), 16)
                .unwrap_or(0);
            amount = Decimal::from(value_u128) / Decimal::from(10u64.pow(self.decimals));
        }

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
            token_mint: self.token_address.clone(),
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
        let action = if self.token_address.is_some() { "tokentx" } else { "txlist" };
        let mut url = format!(
            "{}?module=account&action={}&address={}&startblock=0&endblock=99999999&page=1&offset={}&sort=desc&chainid={}",
            self.api_url, action, address, limit, self.chain_id
        );

        if let Some(ref token) = self.token_address {
            url.push_str(&format!("&contractaddress={}", token));
        }

        if let Some(ref key) = self.api_key {
            url.push_str(&format!("&apikey={}", key));
        }

        let response = self.client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;

        if let Some(status) = data.get("status").and_then(|s| s.as_str()) {
            if status == "0" {
                let err_msg = data.get("result")
                    .map(|r| if r.is_string() { r.as_str().unwrap().to_string() } else { r.to_string() })
                    .unwrap_or_else(|| "Unknown EVM API Error".to_string());
                return Err(format!("EVM API Error: {}", err_msg).into());
            }
        }

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

            // Get full transaction details, passing the address as target to help verifier if needed
            match self.get_transaction_details(&hash, Some(address)).await {
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
            "{}?module=proxy&action=eth_blockNumber&chainid={}",
            self.api_url, self.chain_id
        );

        if let Some(ref key) = self.api_key {
            url.push_str(&format!("&apikey={}", key));
        }

        let response = self.client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;

        if let Some(status) = data.get("status").and_then(|s| s.as_str()) {
            if status == "0" {
                let err_msg = data.get("result")
                    .map(|r| if r.is_string() { r.as_str().unwrap().to_string() } else { r.to_string() })
                    .unwrap_or_else(|| "Unknown EVM API Error".to_string());
                return Err(format!("EVM API Error: {}", err_msg).into());
            }
        }

        let result = data.get("result")
            .and_then(|v| v.as_str())
            .ok_or("No result in response")?;

        let block_number = u64::from_str_radix(result.trim_start_matches("0x"), 16)?;
        Ok(block_number)
    }

    /// Check if transaction succeeded
    async fn check_transaction_success(&self, tx_hash: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut url = format!(
            "{}?module=proxy&action=eth_getTransactionReceipt&txhash={}&chainid={}",
            self.api_url, tx_hash, self.chain_id
        );

        if let Some(ref key) = self.api_key {
            url.push_str(&format!("&apikey={}", key));
        }

        let response = self.client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;

        if let Some(status) = data.get("status").and_then(|s| s.as_str()) {
            if status == "0" {
                let err_msg = data.get("result")
                    .map(|r| if r.is_string() { r.as_str().unwrap().to_string() } else { r.to_string() })
                    .unwrap_or_else(|| "Unknown EVM API Error".to_string());
                return Err(format!("EVM API Error: {}", err_msg).into());
            }
        }

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
            "{}?module=proxy&action=eth_getBlockByNumber&tag=0x{:x}&boolean=false&chainid={}",
            self.api_url, block_number, self.chain_id
        );

        if let Some(ref key) = self.api_key {
            url.push_str(&format!("&apikey={}", key));
        }

        let response = self.client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;

        if let Some(status) = data.get("status").and_then(|s| s.as_str()) {
            if status == "0" {
                let err_msg = data.get("result")
                    .map(|r| if r.is_string() { r.as_str().unwrap().to_string() } else { r.to_string() })
                    .unwrap_or_else(|| "Unknown EVM API Error".to_string());
                return Err(format!("EVM API Error: {}", err_msg).into());
            }
        }

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
    let token_address = crypto_type.token_address().map(|s| s.to_string());
    let decimals = crypto_type.decimals();
    
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
        "BITCOIN" => Box::new(self::btc_monitor::BtcMonitor::from_config(&config, is_sandbox)),
        "ETHEREUM" => Box::new(EvmMonitor::new_ethereum(&config, is_sandbox, token_address, decimals)),
        "BEP20" => Box::new(EvmMonitor::new_bsc(&config, is_sandbox, token_address, decimals)),
        "POLYGON" => Box::new(EvmMonitor::new_polygon(&config, is_sandbox, token_address, decimals)),
        "ARBITRUM" => Box::new(EvmMonitor::new_arbitrum(&config, is_sandbox, token_address, decimals)),
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
