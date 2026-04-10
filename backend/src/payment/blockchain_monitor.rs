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
    chain_name: &'static str,
    decimals: u32,
    token_address: Option<String>,
    chain_id: u64,
    rpc_urls: Vec<String>,
    moralis_keys: Vec<String>,
    etherscan_api_url: String,
    etherscan_api_key: Option<String>,
    internal_chain_identifier: &'static str,
}

impl EvmMonitor {
    fn build_rpc_urls(chain: &str, is_sandbox: bool, config: &crate::config::Config) -> Vec<String> {
        let mut urls = Vec::new();
        for key in &config.alchemy_api_keys {
            if let Some(url) = match (chain, is_sandbox) {
                ("ETH", false) => Some(format!("https://eth-mainnet.g.alchemy.com/v2/{}", key)),
                ("ETH", true) => Some(format!("https://eth-sepolia.g.alchemy.com/v2/{}", key)),
                ("BSC", false) => Some(format!("https://bnb-mainnet.g.alchemy.com/v2/{}", key)),
                ("BSC", true) => Some(format!("https://bnb-testnet.g.alchemy.com/v2/{}", key)),
                ("POLYGON", false) => Some(format!("https://polygon-mainnet.g.alchemy.com/v2/{}", key)),
                ("POLYGON", true) => Some(format!("https://polygon-amoy.g.alchemy.com/v2/{}", key)),
                ("ARBITRUM", false) => Some(format!("https://arb-mainnet.g.alchemy.com/v2/{}", key)),
                ("ARBITRUM", true) => Some(format!("https://arb-sepolia.g.alchemy.com/v2/{}", key)),
                _ => None,
            } { urls.push(url); }
        }
        for key in &config.infura_api_keys {
            if let Some(url) = match (chain, is_sandbox) {
                ("ETH", false) => Some(format!("https://mainnet.infura.io/v3/{}", key)),
                ("ETH", true) => Some(format!("https://sepolia.infura.io/v3/{}", key)),
                ("POLYGON", false) => Some(format!("https://polygon-mainnet.infura.io/v3/{}", key)),
                ("POLYGON", true) => Some(format!("https://polygon-amoy.infura.io/v3/{}", key)),
                ("ARBITRUM", false) => Some(format!("https://arbitrum-mainnet.infura.io/v3/{}", key)),
                ("ARBITRUM", true) => Some(format!("https://arbitrum-sepolia.infura.io/v3/{}", key)),
                _ => None,
            } { urls.push(url); }
        }
        if chain == "ETH" && !is_sandbox { if let Some(ref url) = config.chainstack_eth_url { urls.push(url.clone()); } }
        if chain == "BSC" && !is_sandbox { if let Some(ref url) = config.chainstack_bsc_url { urls.push(url.clone()); } }
        for key in &config.ankr_api_keys {
            if let Some(url) = match (chain, is_sandbox) {
                ("ETH", false) => Some(format!("https://rpc.ankr.com/eth/{}", key)),
                ("ETH", true) => Some(format!("https://rpc.ankr.com/eth_sepolia/{}", key)),
                ("BSC", false) => Some(format!("https://rpc.ankr.com/bsc/{}", key)),
                ("BSC", true) => Some(format!("https://rpc.ankr.com/bsc_testnet_chapel/{}", key)),
                ("POLYGON", false) => Some(format!("https://rpc.ankr.com/polygon/{}", key)),
                ("POLYGON", true) => Some(format!("https://rpc.ankr.com/polygon_amoy/{}", key)),
                ("ARBITRUM", false) => Some(format!("https://rpc.ankr.com/arbitrum/{}", key)),
                ("ARBITRUM", true) => Some(format!("https://rpc.ankr.com/arbitrum_sepolia/{}", key)),
                _ => None,
            } { urls.push(url); }
        }
        if chain == "ETH" && !is_sandbox { urls.extend(config.getblock_eth_keys.iter().map(|k| format!("https://go.getblock.io/{}", k))); }
        if chain == "BSC" && !is_sandbox { urls.extend(config.getblock_bsc_keys.iter().map(|k| format!("https://go.getblock.io/{}", k))); }

        if urls.is_empty() {
             urls.push(match (chain, is_sandbox) {
                ("ETH", false) => config.ethereum_rpc_url.clone(),
                ("ETH", true) => config.ethereum_sepolia_rpc_url.clone(),
                ("BSC", false) => config.bsc_rpc_url.clone(),
                ("BSC", true) => config.bsc_testnet_rpc_url.clone(),
                ("POLYGON", false) => config.polygon_rpc_url.clone(),
                ("POLYGON", true) => config.polygon_amoy_rpc_url.clone(),
                ("ARBITRUM", false) => config.arbitrum_rpc_url.clone(),
                ("ARBITRUM", true) => config.arbitrum_sepolia_rpc_url.clone(),
                _ => "".to_string(),
             });
        }
        urls
    }
}

impl EvmMonitor {
    pub fn new_bsc(config: &crate::config::Config, is_sandbox: bool, token_address: Option<String>, decimals: u32) -> Self {
        Self {
            client: Client::new(),
            chain_name: if is_sandbox { "BSC Testnet" } else { "BSC" },
            decimals,
            token_address,
            chain_id: if is_sandbox { config.bsc_testnet_chain_id } else { config.bsc_chain_id },
            rpc_urls: Self::build_rpc_urls("BSC", is_sandbox, config),
            moralis_keys: config.moralis_api_keys.clone(),
            etherscan_api_url: "https://api.etherscan.io/v2/api".to_string(),
            etherscan_api_key: config.etherscan_api_key.clone(),
            internal_chain_identifier: "BSC",
        }
    }

    pub fn new_arbitrum(config: &crate::config::Config, is_sandbox: bool, token_address: Option<String>, decimals: u32) -> Self {
        Self {
            client: Client::new(),
            chain_name: if is_sandbox { "Arbitrum Sepolia" } else { "Arbitrum" },
            decimals,
            token_address,
            chain_id: if is_sandbox { config.arbitrum_sepolia_chain_id } else { config.arbitrum_chain_id },
            rpc_urls: Self::build_rpc_urls("ARBITRUM", is_sandbox, config),
            moralis_keys: config.moralis_api_keys.clone(),
            etherscan_api_url: "https://api.etherscan.io/v2/api".to_string(),
            etherscan_api_key: config.etherscan_api_key.clone(),
            internal_chain_identifier: "ARBITRUM",
        }
    }

    pub fn new_polygon(config: &crate::config::Config, is_sandbox: bool, token_address: Option<String>, decimals: u32) -> Self {
        let chain_id = if is_sandbox { config.polygon_amoy_chain_id } else { config.polygon_chain_id };
        Self {
            client: Client::new(),
            chain_name: if is_sandbox { "Polygon Amoy" } else { "Polygon" },
            decimals,
            token_address,
            chain_id,
            rpc_urls: Self::build_rpc_urls("POLYGON", is_sandbox, config),
            moralis_keys: config.moralis_api_keys.clone(),
            etherscan_api_url: "https://api.etherscan.io/v2/api".to_string(),
            etherscan_api_key: config.etherscan_api_key.clone(),
            internal_chain_identifier: "POLYGON",
        }
    }

    pub fn new_ethereum(config: &crate::config::Config, is_sandbox: bool, token_address: Option<String>, decimals: u32) -> Self {
        let chain_id = if is_sandbox { config.ethereum_sepolia_chain_id } else { config.ethereum_chain_id };
        Self {
            client: Client::new(),
            chain_name: if is_sandbox { "Ethereum Sepolia" } else { "Ethereum" },
            decimals,
            token_address,
            chain_id,
            rpc_urls: Self::build_rpc_urls("ETH", is_sandbox, config),
            moralis_keys: config.moralis_api_keys.clone(),
            etherscan_api_url: "https://api.etherscan.io/v2/api".to_string(),
            etherscan_api_key: config.etherscan_api_key.clone(),
            internal_chain_identifier: "ETH",
        }
    }
}

// Redact queries from URLs to prevent leaking API keys
fn redact_url(url: &str) -> String {
    if let Some(idx) = url.find('?') {
        format!("{}?***REDACTED***", &url[..idx])
    } else {
        match url.find("alchemy.com/v2/") {
            Some(idx) => format!("{}alchemy.com/v2/***REDACTED***", &url[..idx]),
            None => {
                match url.find("infura.io/v3/") {
                    Some(idx) => format!("{}infura.io/v3/***REDACTED***", &url[..idx]),
                    None => url.to_string()
                }
            }
        }
    }
}

impl EvmMonitor {
    async fn rpc_request(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let mut last_error = None;

        for url in &self.rpc_urls {
            match self.client.post(url).json(&payload).send().await {
                Ok(response) => {
                    let status = response.status();
                    if status == 429 {
                        warn!("Rate limit (429) hit on {}, trying next RPC...", redact_url(url));
                        last_error = Some("Rate limit hit".to_string());
                        continue;
                    }
                    match response.json::<serde_json::Value>().await {
                        Ok(data) => {
                            if data.get("error").is_some() {
                                let err_msg = data["error"]["message"].as_str().unwrap_or("Unknown RPC error");
                                if err_msg.to_lowercase().contains("rate limit") || err_msg.to_lowercase().contains("too many") {
                                    warn!("Rate limit payload from {}, trying next RPC...", redact_url(url));
                                    last_error = Some(format!("Rate limit: {}", err_msg));
                                    continue;
                                }
                                return Err(format!("RPC Error: {}", err_msg).into());
                            }
                            return Ok(data);
                        },
                        Err(e) => last_error = Some(e.to_string()),
                    }
                },
                Err(e) => {
                    warn!("Network error connecting to {}: {}", redact_url(url), e);
                    last_error = Some(e.to_string());
                }
            }
        }

        if !self.etherscan_api_url.is_empty() {
             let mut url = format!("{}?module=proxy&action={}&chainid={}", self.etherscan_api_url, method, self.chain_id);
             if let Some(ref key) = self.etherscan_api_key { url.push_str(&format!("&apikey={}", key)); }
             let params_mapped = match method {
                 "eth_getTransactionByHash" | "eth_getTransactionReceipt" => if let Some(arr) = params.as_array() {
                     format!("&txhash={}", arr.get(0).and_then(|v| v.as_str()).unwrap_or(""))
                 } else { "".to_string() },
                 "eth_getBlockByNumber" => if let Some(arr) = params.as_array() {
                     format!("&tag={}&boolean=false", arr.get(0).and_then(|v| v.as_str()).unwrap_or(""))
                 } else { "".to_string() },
                 _ => "".to_string(),
             };
             url.push_str(&params_mapped);

             if let Ok(res) = self.client.get(&url).send().await {
                 if let Ok(data) = res.json::<serde_json::Value>().await {
                     if let Some(status) = data.get("status").and_then(|s| s.as_str()) {
                         if status == "0" && !data.get("result").and_then(|r| r.as_str()).unwrap_or("").contains("rate limit") {
                             return Err(format!("EVM API Error: {:?}", data.get("result")).into());
                         }
                     }
                     return Ok(data);
                 }
             }
        }

        Err(format!("All RPC nodes failed. Last error: {}", last_error.unwrap_or_else(|| "Unknown".to_string())).into())
    }

    async fn get_moralis_transactions(&self, address: &str, limit: usize) -> Result<Vec<BlockchainTransaction>, Box<dyn std::error::Error + Send + Sync>> {
        let moralis_chain = match self.internal_chain_identifier {
            "ETH" => "eth",
            "BSC" => "bsc",
            "POLYGON" => "polygon",
            "ARBITRUM" => "arbitrum",
            _ => "eth"
        };
        let chain_str = if self.chain_name.contains("Sepolia") { "sepolia" } else if self.chain_name.contains("Testnet") { "bsc testnet" } else { moralis_chain };

        let url = if let Some(ref token) = self.token_address {
            format!("https://deep-index.moralis.io/api/v2.2/{}/erc20/transfers?chain={}&limit={}", address, chain_str, limit)
        } else {
            format!("https://deep-index.moralis.io/api/v2.2/{}/verbose?chain={}&limit={}", address, chain_str, limit)
        };

        for key in &self.moralis_keys {
            if let Ok(response) = self.client.get(&url).header("X-API-Key", key).send().await {
                if response.status() == 429 { continue; }
                if response.status().is_success() {
                    let data = response.json::<serde_json::Value>().await?;
                    let mut transactions = Vec::new();
                    if let Some(result_arr) = data.get("result").and_then(|v| v.as_array()) {
                        for tx in result_arr {
                            let hash = tx.get("transaction_hash").or_else(|| tx.get("hash")).and_then(|v| v.as_str()).unwrap_or("");
                            if !hash.is_empty() {
                                if let Ok(tx_dets) = self.get_transaction_details(hash, Some(address)).await {
                                    transactions.push(tx_dets);
                                }
                            }
                        }
                    }
                    return Ok(transactions);
                }
            }
        }
        Err("All Moralis keys exhausted or failed".into())
    }

    async fn get_etherscan_transactions(&self, address: &str, limit: usize, min_timestamp: Option<chrono::DateTime<chrono::Utc>>) -> Result<Vec<BlockchainTransaction>, Box<dyn std::error::Error + Send + Sync>> {
        let action = if self.token_address.is_some() { "tokentx" } else { "txlist" };
        let mut url = format!("{}?module=account&action={}&address={}&startblock=0&endblock=99999999&page=1&offset={}&sort=desc&chainid={}", 
            self.etherscan_api_url, action, address, limit, self.chain_id);
        if let Some(ref token) = self.token_address { url.push_str(&format!("&contractaddress={}", token)); }
        if let Some(ref key) = self.etherscan_api_key { url.push_str(&format!("&apikey={}", key)); }

        let response = self.client.get(&url).send().await?.json::<serde_json::Value>().await?;
        if let Some(status) = response.get("status").and_then(|s| s.as_str()) {
            if status == "0" {
                if let Some(res_arr) = response.get("result").and_then(|r| r.as_array()) {
                    if res_arr.is_empty() { return Ok(Vec::new()); }
                }
                return Err(format!("EVM API Error: {:?}", response.get("result")).into());
            }
        }
        let result = response.get("result").and_then(|v| v.as_array()).ok_or("Invalid response format")?;
        let mut transactions = Vec::new();
        for tx in result.iter().take(limit) {
            let hash = tx.get("hash").and_then(|v| v.as_str()).unwrap_or("").to_string();
            
            if let (Some(min_ts), Some(tx_ts_str)) = (min_timestamp, tx.get("timeStamp").and_then(|v| v.as_str())) {
                if let Ok(ts_secs) = tx_ts_str.parse::<i64>() {
                    if let Some(ts) = chrono::DateTime::from_timestamp(ts_secs, 0) {
                        if ts < min_ts - chrono::Duration::seconds(60) { continue; }
                    }
                }
            }
            if let Ok(tx_dets) = self.get_transaction_details(&hash, Some(address)).await { transactions.push(tx_dets); }
        }
        Ok(transactions)
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

        let data = self.rpc_request("eth_getTransactionByHash", serde_json::json!([tx_hash])).await?;
        let result = data.get("result").filter(|v| !v.is_null()).ok_or("Transaction not found")?;

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

        if !self.moralis_keys.is_empty() {
             match self.get_moralis_transactions(address, limit).await {
                 Ok(txs) => return Ok(txs),
                 Err(e) => warn!("Moralis parsing/fetching failed: {}, falling back to Etherscan...", e)
             }
        }
        
        self.get_etherscan_transactions(address, limit, min_timestamp).await
    }

    fn blockchain_name(&self) -> &'static str {
        self.chain_name
    }
}

impl EvmMonitor {
    /// Get current block number
    async fn get_current_block(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let data = self.rpc_request("eth_blockNumber", serde_json::json!([])).await?;
        let result = data.get("result").and_then(|v| v.as_str()).ok_or("No result in response")?;
        let block_number = u64::from_str_radix(result.trim_start_matches("0x"), 16)?;
        Ok(block_number)
    }

    /// Check if transaction succeeded
    async fn check_transaction_success(&self, tx_hash: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let data = self.rpc_request("eth_getTransactionReceipt", serde_json::json!([tx_hash])).await?;
        let result = data.get("result").ok_or("No result in response")?;
        if result.is_null() { return Ok(false); }
        let status = result.get("status").and_then(|v| v.as_str()).unwrap_or("0x0");
        Ok(status == "0x1")
    }

    /// Get block timestamp by block number
    async fn get_block_timestamp(&self, block_number: u64) -> Result<chrono::DateTime<chrono::Utc>, Box<dyn std::error::Error + Send + Sync>> {
        let tag = format!("0x{:x}", block_number);
        let data = self.rpc_request("eth_getBlockByNumber", serde_json::json!([tag, false])).await?;
        let result = data.get("result").ok_or("No result in response")?;
        if result.is_null() { return Err("Block not found".into()); }
        let timestamp_hex = result.get("timestamp").and_then(|v| v.as_str()).ok_or("No timestamp in block")?;
        let timestamp_secs = u64::from_str_radix(timestamp_hex.trim_start_matches("0x"), 16)?;
        chrono::DateTime::from_timestamp(timestamp_secs as i64, 0).ok_or_else(|| "Invalid timestamp".into())
    }
}

/// Factory function to create appropriate blockchain monitor
pub fn get_blockchain_monitor(crypto_type: &CryptoType, config: crate::config::Config, is_sandbox: bool) -> Box<dyn BlockchainMonitor> {
    let token_address = crypto_type.token_address().map(|s| s.to_string());
    let decimals = crypto_type.decimals();
    
    match crypto_type.network() {
        "SOLANA" | "SOLANA_SPL" => {
            let expected_mint = crypto_type.token_address().map(|s| s.to_string());
            Box::new(crate::payment::sol_monitor::SolanaMonitor::new(&config, is_sandbox, expected_mint))
        },
        "BITCOIN" => Box::new(self::btc_monitor::BtcMonitor::from_config(&config, is_sandbox)),
        "ETHEREUM" => Box::new(EvmMonitor::new_ethereum(&config, is_sandbox, token_address, decimals)),
        "BINANCE" => Box::new(EvmMonitor::new_bsc(&config, is_sandbox, token_address, decimals)),
        "POLYGON" => Box::new(EvmMonitor::new_polygon(&config, is_sandbox, token_address, decimals)),
        "ARBITRUM" => Box::new(EvmMonitor::new_arbitrum(&config, is_sandbox, token_address, decimals)),
        _ => {
             // Default to Solana for unknown types (fallback)
            let expected_mint = crypto_type.token_address().map(|s| s.to_string());
            Box::new(crate::payment::sol_monitor::SolanaMonitor::new(&config, is_sandbox, expected_mint))
        },
    }
}
