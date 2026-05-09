// Multi-Chain Blockchain Monitor
// Provides unified interface for monitoring payments across all supported blockchains

use async_trait::async_trait;
use reqwest::Client;
use rust_decimal::Decimal;
use tracing::{error, info, warn};

use super::models::{BlockchainTransaction, CryptoType};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};

use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use rand::Rng;
use std::num::NonZeroU32;
use std::sync::Arc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

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

    /// Listen for new transactions using WebSockets (Optimized Push Model)
    async fn listen_for_events(
        &self,
        addresses: Vec<String>,
        new_addresses_rx: tokio::sync::mpsc::UnboundedReceiver<String>,
        callback: std::sync::Arc<dyn Fn(String, String) + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Get current balance of an address
    async fn get_balance(
        &self,
        address: &str,
    ) -> Result<Decimal, Box<dyn std::error::Error + Send + Sync>>;
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
    ws_urls: Vec<String>,
    moralis_keys: Vec<String>,
    etherscan_api_url: String,
    etherscan_api_key: Option<String>,
    internal_chain_identifier: &'static str,
    is_sandbox: bool,
    rate_limiter: Arc<DefaultDirectRateLimiter>,
    rpc_blacklist: Arc<tokio::sync::RwLock<std::collections::HashMap<String, std::time::Instant>>>,
    moralis_blacklist:
        Arc<tokio::sync::RwLock<std::collections::HashMap<String, std::time::Instant>>>,
}

impl EvmMonitor {
    fn build_rpc_urls(
        chain: &str,
        is_sandbox: bool,
        config: &crate::config::Config,
    ) -> Vec<String> {
        let mut urls = Vec::new();
        // --- 1. PRIMARY PROVIDER: Chainstack (User's preferred high-limit provider) ---
        if chain == "ETH" && !is_sandbox {
            urls.extend(config.chainstack_eth_keys.iter().map(|k| {
                if k.starts_with("http") {
                    k.clone()
                } else {
                    format!("https://ethereum-mainnet.core.chainstack.com/{}", k)
                }
            }));
        }
        if chain == "BSC" && !is_sandbox {
            urls.extend(config.chainstack_bsc_keys.iter().map(|k| {
                if k.starts_with("http") {
                    k.clone()
                } else {
                    format!("https://bsc-mainnet.core.chainstack.com/{}", k)
                }
            }));
        }

        // --- 2. SECONDARY PROVIDERS: Ankr, Infura (Solid backups) ---
        for key in &config.ankr_api_keys {
            if let Some(url) = match (chain, is_sandbox) {
                ("ETH", false) => Some(format!("https://rpc.ankr.com/eth/{}", key)),
                ("BSC", false) => Some(format!("https://rpc.ankr.com/bsc/{}", key)),
                ("POLYGON", false) => Some(format!("https://rpc.ankr.com/polygon/{}", key)),
                ("ARBITRUM", false) => Some(format!("https://rpc.ankr.com/arbitrum/{}", key)),
                _ => None,
            } {
                urls.push(url);
            }
        }

        for key in &config.infura_api_keys {
            if let Some(url) = match (chain, is_sandbox) {
                ("ETH", false) => Some(format!("https://mainnet.infura.io/v3/{}", key)),
                ("ETH", true) => Some(format!("https://sepolia.infura.io/v3/{}", key)),
                ("POLYGON", false) => Some(format!("https://polygon-mainnet.infura.io/v3/{}", key)),
                ("POLYGON", true) => Some(format!("https://polygon-amoy.infura.io/v3/{}", key)),
                ("ARBITRUM", false) => {
                    Some(format!("https://arbitrum-mainnet.infura.io/v3/{}", key))
                }
                ("ARBITRUM", true) => {
                    Some(format!("https://arbitrum-sepolia.infura.io/v3/{}", key))
                }
                _ => None,
            } {
                urls.push(url);
            }
        }

        // --- 3. LAST RESORT: Alchemy (Prone to 429s, moved to bottom) ---
        for key in &config.alchemy_api_keys {
            if let Some(url) = match (chain, is_sandbox) {
                ("ETH", false) => Some(format!("https://eth-mainnet.g.alchemy.com/v2/{}", key)),
                ("ETH", true) => Some(format!("https://eth-sepolia.g.alchemy.com/v2/{}", key)),
                ("BSC", false) => Some(format!("https://bnb-mainnet.g.alchemy.com/v2/{}", key)),
                ("BSC", true) => Some(format!("https://bnb-testnet.g.alchemy.com/v2/{}", key)),
                ("POLYGON", false) => {
                    Some(format!("https://polygon-mainnet.g.alchemy.com/v2/{}", key))
                }
                ("POLYGON", true) => Some(format!("https://polygon-amoy.g.alchemy.com/v2/{}", key)),
                ("ARBITRUM", false) => {
                    Some(format!("https://arb-mainnet.g.alchemy.com/v2/{}", key))
                }
                ("ARBITRUM", true) => Some(format!("https://arb-sepolia.g.alchemy.com/v2/{}", key)),
                _ => None,
            } {
                urls.push(url);
            }
        }

        // --- PUBLIC FALLBACKS (LlamaNodes & Keyless Ankr) ---
        // If we are in production, add high-performance public fallbacks to escape Alchemy/Infura rate limits
        if !is_sandbox {
            // LlamaNodes (Excellent limits for public use)
            match chain {
                "ETH" => urls.push("https://eth.llamarpc.com".to_string()),
                "BSC" => urls.push("https://binance.llamarpc.com".to_string()),
                "POLYGON" => urls.push("https://polygon.llamarpc.com".to_string()),
                "ARBITRUM" => urls.push("https://arbitrum.llamarpc.com".to_string()),
                _ => {}
            }

            // Ankr Public (Keyless) - solid backup
            match chain {
                "ETH" => urls.push("https://rpc.ankr.com/eth".to_string()),
                "BSC" => urls.push("https://rpc.ankr.com/bsc".to_string()),
                "POLYGON" => urls.push("https://rpc.ankr.com/polygon".to_string()),
                "ARBITRUM" => urls.push("https://rpc.ankr.com/arbitrum".to_string()),
                _ => {}
            }
        }

        if chain == "ETH" && !is_sandbox {
            urls.extend(
                config
                    .getblock_eth_keys
                    .iter()
                    .map(|k| format!("https://go.getblock.io/{}", k)),
            );
        }
        if chain == "BSC" && !is_sandbox {
            urls.extend(
                config
                    .getblock_bsc_keys
                    .iter()
                    .map(|k| format!("https://go.getblock.io/{}", k)),
            );
        }

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
    pub fn new_bsc(
        config: &crate::config::Config,
        is_sandbox: bool,
        token_address: Option<String>,
        decimals: u32,
    ) -> Self {
        let rpc_urls = Self::build_rpc_urls("BSC", is_sandbox, config);
        let mut ws_urls = Vec::new();

        // Prioritize Chainstack WSS if available
        if !is_sandbox {
            for key in &config.chainstack_bsc_keys {
                let url = if key.starts_with("http") {
                    key.replace("https://", "wss://")
                } else {
                    format!("wss://bsc-mainnet.core.chainstack.com/{}", key)
                };
                ws_urls.push(url);
            }
        }

        for rpc in &rpc_urls {
            let ws = get_evm_ws_url(config, rpc);
            if !ws_urls.contains(&ws) {
                ws_urls.push(ws);
            }
        }

        Self {
            client: Client::new(),
            chain_name: if is_sandbox { "BSC Testnet" } else { "BSC" },
            decimals,
            token_address,
            chain_id: if is_sandbox {
                config.bsc_testnet_chain_id
            } else {
                config.bsc_chain_id
            },
            rpc_urls,
            ws_urls,
            moralis_keys: config.moralis_api_keys.clone(),
            etherscan_api_url: if is_sandbox {
                "https://api-testnet.bscscan.com/api".to_string()
            } else {
                "https://api.bscscan.com/api".to_string()
            },
            etherscan_api_key: config.etherscan_api_key.clone(),
            internal_chain_identifier: "BSC",
            is_sandbox,
            // 25 Requests per second limit as requested for Chainstack
            rate_limiter: Arc::new(RateLimiter::direct(Quota::per_second(
                NonZeroU32::new(25).unwrap(),
            ))),
            rpc_blacklist: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            moralis_blacklist: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub fn new_arbitrum(
        config: &crate::config::Config,
        is_sandbox: bool,
        token_address: Option<String>,
        decimals: u32,
    ) -> Self {
        let rpc_urls = Self::build_rpc_urls("ARBITRUM", is_sandbox, config);
        let mut ws_urls = Vec::new();
        for rpc in &rpc_urls {
            ws_urls.push(get_evm_ws_url(config, rpc));
        }

        Self {
            client: Client::new(),
            chain_name: if is_sandbox {
                "Arbitrum Sepolia"
            } else {
                "Arbitrum"
            },
            decimals,
            token_address,
            chain_id: if is_sandbox {
                config.arbitrum_sepolia_chain_id
            } else {
                config.arbitrum_chain_id
            },
            rpc_urls,
            ws_urls,
            moralis_keys: config.moralis_api_keys.clone(),
            etherscan_api_url: if is_sandbox {
                "https://api-sepolia.arbiscan.io/api".to_string()
            } else {
                "https://api.arbiscan.io/api".to_string()
            },
            etherscan_api_key: config.etherscan_api_key.clone(),
            internal_chain_identifier: "ARBITRUM",
            is_sandbox,
            rate_limiter: Arc::new(RateLimiter::direct(Quota::per_second(
                NonZeroU32::new(25).unwrap(),
            ))),
            rpc_blacklist: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            moralis_blacklist: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub fn new_polygon(
        config: &crate::config::Config,
        is_sandbox: bool,
        token_address: Option<String>,
        decimals: u32,
    ) -> Self {
        let chain_id = if is_sandbox {
            config.polygon_amoy_chain_id
        } else {
            config.polygon_chain_id
        };
        let rpc_urls = Self::build_rpc_urls("POLYGON", is_sandbox, config);
        let mut ws_urls = Vec::new();
        for rpc in &rpc_urls {
            ws_urls.push(get_evm_ws_url(config, rpc));
        }

        Self {
            client: Client::new(),
            chain_name: if is_sandbox {
                "Polygon Amoy"
            } else {
                "Polygon"
            },
            decimals,
            token_address,
            chain_id,
            rpc_urls,
            ws_urls,
            moralis_keys: config.moralis_api_keys.clone(),
            etherscan_api_url: if is_sandbox {
                "https://api-amoy.polygonscan.com/api".to_string()
            } else {
                "https://api.polygonscan.com/api".to_string()
            },
            etherscan_api_key: config.etherscan_api_key.clone(),
            internal_chain_identifier: "POLYGON",
            is_sandbox,
            rate_limiter: Arc::new(RateLimiter::direct(Quota::per_second(
                NonZeroU32::new(25).unwrap(),
            ))),
            rpc_blacklist: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            moralis_blacklist: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub fn new_ethereum(
        config: &crate::config::Config,
        is_sandbox: bool,
        token_address: Option<String>,
        decimals: u32,
    ) -> Self {
        let chain_id = if is_sandbox {
            config.ethereum_sepolia_chain_id
        } else {
            config.ethereum_chain_id
        };
        let rpc_urls = Self::build_rpc_urls("ETH", is_sandbox, config);
        let mut ws_urls = Vec::new();

        // Prioritize Chainstack WSS if available
        if !is_sandbox {
            for key in &config.chainstack_eth_keys {
                let url = if key.starts_with("http") {
                    key.replace("https://", "wss://")
                } else {
                    format!("wss://ethereum-mainnet.core.chainstack.com/{}", key)
                };
                ws_urls.push(url);
            }
        }

        for rpc in &rpc_urls {
            let ws = get_evm_ws_url(config, rpc);
            if !ws_urls.contains(&ws) {
                ws_urls.push(ws);
            }
        }

        Self {
            client: Client::new(),
            chain_name: if is_sandbox {
                "Ethereum Sepolia"
            } else {
                "Ethereum"
            },
            decimals,
            token_address,
            chain_id,
            rpc_urls,
            ws_urls,
            moralis_keys: config.moralis_api_keys.clone(),
            etherscan_api_url: "https://api.etherscan.io/v2/api".to_string(),
            etherscan_api_key: config.etherscan_api_key.clone(),
            internal_chain_identifier: "ETH",
            is_sandbox,
            rate_limiter: Arc::new(RateLimiter::direct(Quota::per_second(
                NonZeroU32::new(25).unwrap(),
            ))),
            rpc_blacklist: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            moralis_blacklist: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Listen for new transactions using WebSockets (Global Filter Architecture)
    ///
    /// Instead of creating N per-address subscriptions (which hits provider limits at ~100-1000),
    /// we use ONE global subscription for all Transfer events on the token contract and filter
    /// incoming events locally against a HashSet of monitored addresses.
    ///
    /// Architecture:
    ///   - 1x `eth_subscribe("newHeads")` for native transfers (BNB/ETH/etc.)
    ///   - 1x `eth_subscribe("logs")` for ALL token Transfer events (global, unfiltered by recipient)
    ///   - Local `HashSet<String>` for O(1) address matching per event
    ///
    /// This scales to unlimited addresses on a single WebSocket connection.
    async fn listen_for_events(
        &self,
        addresses: Vec<String>,
        mut new_addresses_rx: tokio::sync::mpsc::UnboundedReceiver<String>,
        callback: std::sync::Arc<dyn Fn(String, String) + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // === Global Filter: Local address set for O(1) lookups ===
        let mut monitored_set: std::collections::HashSet<String> =
            addresses.iter().map(|a| a.to_lowercase()).collect();

        info!(
            "📡 {} WS: Initializing Global Filter with {} monitored addresses",
            self.chain_name,
            monitored_set.len()
        );

        let mut ws_stream_opt = None;

        for (i, url) in self.ws_urls.iter().enumerate() {
            let safe_url = redact_url(url);
            info!(
                "🔌 [Key #{}] Attempting connection to {} WebSocket: {}",
                i + 1,
                self.chain_name,
                safe_url
            );
            match connect_async(url.as_str()).await {
                Ok((stream, _)) => {
                    ws_stream_opt = Some(stream);
                    info!(
                        "✅ [Key #{}] Successfully connected to {} WebSocket: {}",
                        i + 1,
                        self.chain_name,
                        safe_url
                    );
                    break;
                }
                Err(e) => {
                    warn!(
                        "❌ [Key #{}] Failed to connect to {} WebSocket {}: {}",
                        i + 1,
                        self.chain_name,
                        safe_url,
                        e
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }
            }
        }

        let ws_stream = match ws_stream_opt {
            Some(stream) => stream,
            None => {
                let err_msg = format!("All {} WebSocket nodes failed to connect", self.chain_name);
                error!("{}", err_msg);
                return Err(err_msg.into());
            }
        };

        let (mut write, mut read) = ws_stream.split();

        let mut next_request_id = 1u64;

        // 1. Subscribe to newHeads (catches native transfers like BNB/ETH)
        let head_request_id = next_request_id;
        next_request_id += 1;
        let head_sub_msg = serde_json::json!({
            "jsonrpc": "2.0",
            "id": head_request_id,
            "method": "eth_subscribe",
            "params": ["newHeads"]
        });
        write.send(Message::Text(head_sub_msg.to_string())).await?;

        // 2. ONE global subscription for ALL Transfer events on the token contract
        //    No per-address topic filtering — we filter locally against `monitored_set`.
        let mut token_request_id: Option<u64> = None;
        if let Some(ref token) = self.token_address {
            let rid = next_request_id;
            next_request_id += 1;
            token_request_id = Some(rid);

            let transfer_topic =
                "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

            let subscribe_msg = serde_json::json!({
                "jsonrpc": "2.0",
                "id": rid,
                "method": "eth_subscribe",
                "params": [
                    "logs",
                    {
                        "address": token,
                        "topics": [transfer_topic]
                    }
                ]
            });
            write.send(Message::Text(subscribe_msg.to_string())).await?;
            info!(
                "📡 {} WS: Sent 1 global token log subscription for {} (filtering {} addresses locally)",
                self.chain_name, token, monitored_set.len()
            );
        }

        // Suppress unused warning — next_request_id is kept available for future subscription types
        let _ = next_request_id;

        let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(15));
        let mut head_sub_id: Option<String> = None;
        let mut token_sub_id: Option<String> = None;

        loop {
            tokio::select! {
                _ = ping_interval.tick() => {
                    if let Err(e) = write.send(Message::Ping(Default::default())).await {
                        warn!("Failed to send {} WS ping: {}", self.chain_name, e);
                    }
                }
                Some(new_addr) = new_addresses_rx.recv() => {
                    // Global Filter: just insert into local HashSet — NO new WS subscription needed
                    let lower = new_addr.to_lowercase();
                    if monitored_set.insert(lower) {
                        info!(
                            "📡 {} WS: Added address to local filter (total: {})",
                            self.chain_name, monitored_set.len()
                        );
                    }

                    // Quick backfill for the new address (check last 5 minutes of history)
                    let cb_clone = callback.clone();
                    let addr_clone = new_addr.clone();
                    let min_ts = Utc::now() - chrono::Duration::minutes(5);
                    let monitor_clone = self.clone_for_ws();
                    tokio::spawn(async move {
                        // Random delay (0-5s) to avoid "thundering herd" on RPC/API keys
                        let delay = rand::thread_rng().gen_range(0..5000);
                        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;

                        if let Ok(txs) = monitor_clone.get_transactions_to_address(&addr_clone, 5, Some(min_ts)).await {
                            for tx in txs {
                                cb_clone(tx.hash.clone(), addr_clone.clone());
                            }
                        }
                    });
                }
                message = read.next() => {
                    let message = match message {
                        Some(m) => m,
                        None => break, // Stream closed
                    };

                    match message {
                        Ok(Message::Text(text)) => {
                            let v: serde_json::Value = serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);

                            // Handle subscription confirmations
                            if let Some(id) = v["id"].as_u64() {
                                if let Some(sub_id) = v["result"].as_str() {
                                    if id == head_request_id {
                                        head_sub_id = Some(sub_id.to_string());
                                        info!("✅ {} WS: newHeads subscription confirmed ({})", self.chain_name, sub_id);
                                    } else if token_request_id == Some(id) {
                                        token_sub_id = Some(sub_id.to_string());
                                        info!("✅ {} WS: Global token logs subscription confirmed ({})", self.chain_name, sub_id);
                                    }
                                }
                            }

                            // Handle push notifications
                            if let Some(method) = v["method"].as_str() {
                                if method == "eth_subscription" {
                                    let params = &v["params"];
                                    let sub_id = params["subscription"].as_str().unwrap_or("");

                                    // Scenario A: Native Transfer (New Block Head)
                                    // Fetch block transactions and check tx.to against our monitored set
                                    if head_sub_id.as_deref() == Some(sub_id) {
                                        if self.token_address.is_none() {
                                            let block_hash = params["result"]["hash"].as_str().unwrap_or("");
                                            if !block_hash.is_empty() {
                                                if let Ok(block_data) = self.rpc_request("eth_getBlockByHash", serde_json::json!([block_hash, true])).await {
                                                    if let Some(txs) = block_data["result"]["transactions"].as_array() {
                                                        for tx in txs {
                                                            let to = tx["to"].as_str().unwrap_or("").to_lowercase();
                                                            let hash = tx["hash"].as_str().unwrap_or("").to_string();
                                                            let input = tx["input"].as_str().unwrap_or("0x");

                                                            // 1. Check for Native Transfer
                                                            if monitored_set.contains(&to) {
                                                                callback(hash.clone(), to);
                                                            }

                                                            // 2. Check for Token Transfer inside this block (Universal Detection)
                                                            else if input.starts_with("0x") && input.len() >= 138 && &input[2..10] == "a9059cbb" {
                                                                let recipient_hex = &input[34..74];
                                                                let token_recipient = format!("0x{}", recipient_hex).to_lowercase();
                                                                if monitored_set.contains(&token_recipient) {
                                                                    info!("🚀 {} WS: Detected token transfer in native block head for {} in tx {}", self.chain_name, token_recipient, hash);
                                                                    callback(hash, token_recipient);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    // Scenario B: Token Transfer (Global Log — extract recipient from topic[2])
                                    // Transfer(address indexed from, address indexed to, uint256 value)
                                    // topic[0] = event signature, topic[1] = from, topic[2] = to
                                    else if token_sub_id.as_deref() == Some(sub_id) {
                                        let result = &params["result"];
                                        let tx_hash = result["transactionHash"].as_str().unwrap_or("").to_string();

                                        if !tx_hash.is_empty() {
                                            if let Some(topics) = result["topics"].as_array() {
                                                if topics.len() >= 3 {
                                                    if let Some(to_topic) = topics[2].as_str() {
                                                        // topic[2] is 32-byte padded: 0x + 24 zero-chars + 20-byte address
                                                        // e.g. "0x000000000000000000000000abcdef1234567890abcdef1234567890abcdef12"
                                                        if to_topic.len() >= 42 {
                                                            let to_addr = format!("0x{}", &to_topic[26..]).to_lowercase();
                                                            if monitored_set.contains(&to_addr) {
                                                                info!(
                                                                    "🚀 {} WS: Detected token transfer for {} in tx {}",
                                                                    self.chain_name, to_addr, tx_hash
                                                                );
                                                                callback(tx_hash, to_addr);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Ok(Message::Close(_)) => break,
                        Err(e) => {
                            warn!("{} WS error: {}", self.chain_name, e);
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    /// Internal clone for spawn move
    fn clone_for_ws(&self) -> Self {
        Self {
            client: self.client.clone(),
            chain_name: self.chain_name,
            decimals: self.decimals,
            token_address: self.token_address.clone(),
            chain_id: self.chain_id,
            rpc_urls: self.rpc_urls.clone(),
            ws_urls: self.ws_urls.clone(),
            moralis_keys: self.moralis_keys.clone(),
            etherscan_api_url: self.etherscan_api_url.clone(),
            etherscan_api_key: self.etherscan_api_key.clone(),
            internal_chain_identifier: self.internal_chain_identifier,
            is_sandbox: self.is_sandbox,
            rate_limiter: self.rate_limiter.clone(),
            rpc_blacklist: self.rpc_blacklist.clone(),
            moralis_blacklist: self.moralis_blacklist.clone(),
        }
    }
}

/// Helper to pad EVM address for log topics (20 bytes -> 32 bytes)
fn pad_evm_address_to_32_bytes(address: &str) -> String {
    let clean_addr = address.trim_start_matches("0x");
    format!("0x000000000000000000000000{}", clean_addr.to_lowercase())
}

/// Dynamic conversion of HTTP RPC URLs to WebSocket WSS URLs for EVM
fn get_evm_ws_url(config: &crate::config::Config, rpc_url: &str) -> String {
    if rpc_url.starts_with("https://") {
        rpc_url.replace("https://", "wss://")
    } else if config.environment == "production" {
        // Fallback or custom logic if needed, but replace is standard for major providers
        rpc_url.to_string()
    } else {
        rpc_url.to_string()
    }
}

// Redact queries from URLs to prevent leaking API keys
fn redact_url(url: &str) -> String {
    if let Some(idx) = url.find('?') {
        format!("{}?***REDACTED***", &url[..idx])
    } else {
        // Redact key in path for Alchemy, Infura, Chainstack, Ankr
        if url.contains("alchemy.com/v2/") {
            let parts: Vec<&str> = url.split("alchemy.com/v2/").collect();
            format!("{}alchemy.com/v2/***REDACTED***", parts[0])
        } else if url.contains("infura.io/v3/") {
            let parts: Vec<&str> = url.split("infura.io/v3/").collect();
            format!("{}infura.io/v3/***REDACTED***", parts[0])
        } else if url.contains("chainstack.com/") {
            // Chainstack usually has the key as the last part of the path
            if let Some(last_slash) = url.rfind('/') {
                format!("{}/[REDACTED]", &url[..last_slash])
            } else {
                url.to_string()
            }
        } else if url.contains("rpc.ankr.com/") {
            // Ankr: https://rpc.ankr.com/eth/KEY or https://rpc.ankr.com/eth (public)
            let parts: Vec<&str> = url.split('/').collect();
            if parts.len() > 4 {
                // Has a key
                format!(
                    "{}/{}/{}/{}/[REDACTED]",
                    parts[0], parts[1], parts[2], parts[3]
                )
            } else {
                url.to_string()
            }
        } else {
            url.to_string()
        }
    }
}

impl EvmMonitor {
    async fn rpc_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let mut last_error = None;

        // Apply global rate limit before any RPC request
        self.rate_limiter.until_ready().await;

        for (i, url) in self.rpc_urls.iter().enumerate() {
            // Check if node is blacklisted
            {
                let blacklist = self.rpc_blacklist.read().await;
                if let Some(expiry) = blacklist.get(url) {
                    if std::time::Instant::now() < *expiry {
                        continue;
                    }
                }
            }

            match self.client.post(url).json(&payload).send().await {
                Ok(response) => {
                    let status = response.status();
                    if status == 429 {
                        warn!(
                            "Rate limit (429) hit on [Key #{} - {}], blacklisting for 5 mins...",
                            i + 1,
                            redact_url(url)
                        );
                        {
                            let mut blacklist = self.rpc_blacklist.write().await;
                            blacklist.insert(
                                url.clone(),
                                std::time::Instant::now() + std::time::Duration::from_secs(300),
                            );
                        }
                        last_error = Some("Rate limit hit".to_string());
                        continue;
                    }
                    if status == 401 || status == 403 {
                        warn!(
                            "Auth error ({}) for [Key #{} - {}], blacklisting for 10 mins...",
                            status,
                            i + 1,
                            redact_url(url)
                        );
                        {
                            let mut blacklist = self.rpc_blacklist.write().await;
                            blacklist.insert(
                                url.clone(),
                                std::time::Instant::now() + std::time::Duration::from_secs(600),
                            );
                        }
                        last_error = Some(format!("Auth error: {}", status));
                        continue;
                    }
                    match response.json::<serde_json::Value>().await {
                        Ok(data) => {
                            if data.get("error").is_some() {
                                let err_msg = data["error"]["message"]
                                    .as_str()
                                    .unwrap_or("Unknown RPC error");
                                if err_msg.to_lowercase().contains("rate limit")
                                    || err_msg.to_lowercase().contains("too many")
                                {
                                    warn!(
                                        "Rate limit payload from [Key #{} - {}], trying next RPC...",
                                        i + 1,
                                        redact_url(url)
                                    );
                                    last_error = Some(format!("Rate limit: {}", err_msg));
                                    continue;
                                }
                                return Err(format!("RPC Error: {}", err_msg).into());
                            }
                            return Ok(data);
                        }
                        Err(e) => last_error = Some(e.to_string()),
                    }
                }
                Err(e) => {
                    warn!("Network error connecting to {}: {}", redact_url(url), e);
                    last_error = Some(e.to_string());
                }
            }
        }

        if !self.etherscan_api_url.is_empty() {
            let mut url = format!(
                "{}?module=proxy&action={}&chainid={}",
                self.etherscan_api_url, method, self.chain_id
            );
            if let Some(ref key) = self.etherscan_api_key {
                url.push_str(&format!("&apikey={}", key));
            }
            let params_mapped = match method {
                "eth_getTransactionByHash" | "eth_getTransactionReceipt" => {
                    if let Some(arr) = params.as_array() {
                        format!(
                            "&txhash={}",
                            arr.first().and_then(|v| v.as_str()).unwrap_or("")
                        )
                    } else {
                        "".to_string()
                    }
                }
                "eth_getBlockByNumber" => {
                    if let Some(arr) = params.as_array() {
                        format!(
                            "&tag={}&boolean=false",
                            arr.first().and_then(|v| v.as_str()).unwrap_or("")
                        )
                    } else {
                        "".to_string()
                    }
                }
                _ => "".to_string(),
            };
            url.push_str(&params_mapped);

            if let Ok(res) = self.client.get(&url).send().await {
                if let Ok(data) = res.json::<serde_json::Value>().await {
                    if let Some(status) = data.get("status").and_then(|s| s.as_str()) {
                        if status == "0"
                            && !data
                                .get("result")
                                .and_then(|r| r.as_str())
                                .unwrap_or("")
                                .contains("rate limit")
                        {
                            return Err(format!("EVM API Error: {:?}", data.get("result")).into());
                        }
                    }
                    return Ok(data);
                }
            }
        }

        Err(format!(
            "All RPC nodes failed. Last error: {}",
            last_error.unwrap_or_else(|| "Unknown".to_string())
        )
        .into())
    }

    async fn get_moralis_transactions(
        &self,
        address: &str,
        limit: usize,
    ) -> Result<Vec<BlockchainTransaction>, Box<dyn std::error::Error + Send + Sync>> {
        let moralis_chain = match self.internal_chain_identifier {
            "ETH" => "eth",
            "BSC" => "bsc",
            "POLYGON" => "polygon",
            "ARBITRUM" => "arbitrum",
            _ => "eth",
        };
        let chain_str = if self.chain_name.contains("Sepolia") {
            "sepolia"
        } else if self.chain_name.contains("Testnet") {
            "bsc testnet"
        } else {
            moralis_chain
        };

        // Universal Scan: Fetch 'verbose' history which includes native transactions.
        // For token-specific monitors, we still fetch all ERC20 transfers.
        let url = if self.token_address.is_some() {
            format!(
                "https://deep-index.moralis.io/api/v2.2/{}/erc20/transfers?chain={}&limit={}",
                address, chain_str, limit
            )
        } else {
            format!(
                "https://deep-index.moralis.io/api/v2.2/{}/verbose?chain={}&limit={}",
                address, chain_str, limit
            )
        };

        for (i, key) in self.moralis_keys.iter().enumerate() {
            // Check blacklist
            {
                let blacklist = self.moralis_blacklist.read().await;
                if let Some(expiry) = blacklist.get(key) {
                    if std::time::Instant::now() < *expiry {
                        continue;
                    }
                }
            }

            if let Ok(response) = self.client.get(&url).header("X-API-Key", key).send().await {
                let status = response.status();
                if status == 429 || status == 401 || status == 403 {
                    warn!(
                        "Moralis key error ({}) on [Key #{}], blacklisting for 10 mins...",
                        status,
                        i + 1
                    );
                    {
                        let mut blacklist = self.moralis_blacklist.write().await;
                        blacklist.insert(
                            key.clone(),
                            std::time::Instant::now() + std::time::Duration::from_secs(600),
                        );
                    }
                    continue;
                }
                if status.is_success() {
                    let data = response.json::<serde_json::Value>().await?;
                    let mut transactions = Vec::new();
                    if let Some(result_arr) = data.get("result").and_then(|v| v.as_array()) {
                        for tx in result_arr {
                            let hash = tx
                                .get("transaction_hash")
                                .or_else(|| tx.get("hash"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            if !hash.is_empty() {
                                if let Ok(tx_dets) =
                                    self.get_transaction_details(hash, Some(address)).await
                                {
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

    async fn get_alchemy_transactions(
        &self,
        address: &str,
        limit: usize,
    ) -> Result<Vec<BlockchainTransaction>, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "alchemy_getAssetTransfers",
            "params": [
                {
                    "toAddress": address,
                    "category": ["external", "erc20"],
                    "withMetadata": true,
                    "maxCount": format!("0x{:x}", limit),
                    "excludeZeroValue": true,
                }
            ]
        });

        for rpc_url in &self.rpc_urls {
            // Only try Asset Transfers on Alchemy nodes
            if !rpc_url.contains("alchemy.com") {
                continue;
            }

            if let Ok(response) = self.client.post(rpc_url).json(&payload).send().await {
                if response.status().is_success() {
                    let data = response.json::<serde_json::Value>().await?;
                    if let Some(result) = data.get("result") {
                        if let Some(transfers) = result.get("transfers").and_then(|t| t.as_array())
                        {
                            let mut transactions = Vec::new();
                            for transfer in transfers {
                                let hash =
                                    transfer.get("hash").and_then(|h| h.as_str()).unwrap_or("");
                                if !hash.is_empty() {
                                    // Fetch full details to ensure consistency
                                    if let Ok(tx_dets) =
                                        self.get_transaction_details(hash, Some(address)).await
                                    {
                                        transactions.push(tx_dets);
                                    }
                                }
                            }
                            return Ok(transactions);
                        }
                    }
                }
            }
        }
        Err("Alchemy Asset Transfers failed or no results".into())
    }

    async fn get_etherscan_transactions(
        &self,
        address: &str,
        limit: usize,
        min_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<BlockchainTransaction>, Box<dyn std::error::Error + Send + Sync>> {
        let action = "tokentx"; // Always fetch token transfers for comprehensive audit
        let mut url = format!("{}?module=account&action={}&address={}&startblock=0&endblock=99999999&page=1&offset={}&sort=desc&chainid={}", 
            self.etherscan_api_url, action, address, limit, self.chain_id);

        if let Some(ref key) = self.etherscan_api_key {
            url.push_str(&format!("&apikey={}", key));
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        if let Some(status) = response.get("status").and_then(|s| s.as_str()) {
            if status == "0" {
                if let Some(res_arr) = response.get("result").and_then(|r| r.as_array()) {
                    if res_arr.is_empty() {
                        return Ok(Vec::new());
                    }
                }
                return Err(format!("EVM API Error: {:?}", response.get("result")).into());
            }
        }
        let result = response
            .get("result")
            .and_then(|v| v.as_array())
            .ok_or("Invalid response format")?;
        let mut transactions = Vec::new();
        for tx in result.iter().take(limit) {
            let hash = tx
                .get("hash")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if let (Some(min_ts), Some(tx_ts_str)) =
                (min_timestamp, tx.get("timeStamp").and_then(|v| v.as_str()))
            {
                if let Ok(ts_secs) = tx_ts_str.parse::<i64>() {
                    if let Some(ts) = chrono::DateTime::from_timestamp(ts_secs, 0) {
                        if ts < min_ts - chrono::Duration::seconds(60) {
                            continue;
                        }
                    }
                }
            }
            if let Ok(tx_dets) = self.get_transaction_details(&hash, Some(address)).await {
                transactions.push(tx_dets);
            }
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

        let data = self
            .rpc_request("eth_getTransactionByHash", serde_json::json!([tx_hash]))
            .await?;
        let result = data
            .get("result")
            .filter(|v| !v.is_null())
            .ok_or("Transaction not found")?;

        if result.is_null() {
            return Err("Transaction not found".into());
        }

        let from_address = result
            .get("from")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut to_address = result
            .get("to")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let value_hex = result
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("0x0");

        let input = result.get("input").and_then(|v| v.as_str()).unwrap_or("0x");

        let block_number = result
            .get("blockNumber")
            .and_then(|v| v.as_str())
            .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok());

        let amount;

        // Check if this is an ERC20/BEP20 token transfer by inspecting function signature
        let is_token_call =
            input.starts_with("0x") && input.len() >= 138 && &input[2..10] == "a9059cbb"; // transfer(address,uint256)

        if is_token_call {
            let recipient_hex = &input[34..74]; // skip 2 (0x) + 8 (sig) + 24 (padding)
            let amount_hex = &input[74..138];

            if let Ok(value_u128) = u128::from_str_radix(amount_hex, 16) {
                // Determine decimals: use configured decimals if contract matches, otherwise default to 18 for BEP20/ERC20
                let actual_decimals = if let Some(ref expected_token) = self.token_address {
                    if to_address.trim().to_lowercase() == expected_token.trim().to_lowercase() {
                        self.decimals
                    } else {
                        // For cross-token detection, we assume standard 18 decimals for most BEP20/ERC20
                        // (USDT is 6 on ETH but 18 on BSC, so this is generally safe for BSC)
                        18
                    }
                } else {
                    18
                };

                amount = Decimal::from(value_u128) / Decimal::from(10u64.pow(actual_decimals));
                let actual_recipient = format!("0x{}", recipient_hex);

                info!(
                    "Parsed Universal ERC20 Transfer: contract={}, recipient={}, amount={}",
                    to_address, actual_recipient, amount
                );

                // In token transfers, the 'to_address' in our model should be the RECIPIENT,
                // and 'token_mint' should be the CONTRACT.
                let token_contract = to_address;
                to_address = actual_recipient;

                return Ok(BlockchainTransaction {
                    hash: tx_hash.to_string(),
                    from_address,
                    to_address,
                    amount,
                    confirmations: self.get_confirmations(block_number).await?,
                    block_number,
                    timestamp: Some(self.get_timestamp_or_now(block_number).await),
                    success: self.check_transaction_success(tx_hash).await?,
                    token_mint: Some(token_contract),
                });
            }
        }

        // Native currency transfer
        let value_u128 = u128::from_str_radix(value_hex.trim_start_matches("0x"), 16).unwrap_or(0);
        amount = Decimal::from(value_u128) / Decimal::from(10u64.pow(self.decimals));

        // Get current block to calculate confirmations
        let confirmations = self.get_confirmations(block_number).await?;

        // Check if transaction succeeded
        let success = self.check_transaction_success(tx_hash).await?;

        // Get actual block timestamp
        let timestamp = self.get_timestamp_or_now(block_number).await;

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
        info!(
            " Fetching {} transactions for address: {}",
            self.chain_name, address
        );

        // 1. Try Moralis (Primary)
        if !self.moralis_keys.is_empty() {
            match self.get_moralis_transactions(address, limit).await {
                Ok(txs) => return Ok(txs),
                Err(e) => warn!(
                    "Moralis parsing/fetching failed: {}, falling back to Alchemy...",
                    e
                ),
            }
        }

        // 2. Try Alchemy Asset Transfers (Robust Fallback - Free Multichain support)
        match self.get_alchemy_transactions(address, limit).await {
            Ok(txs) => return Ok(txs),
            Err(e) => warn!(
                "Alchemy Asset Transfers failed: {}, falling back to Etherscan...",
                e
            ),
        }

        // 3. Try Etherscan V2 (Best Effort / Legacy fallback)
        self.get_etherscan_transactions(address, limit, min_timestamp)
            .await
    }

    fn blockchain_name(&self) -> &'static str {
        self.chain_name
    }

    async fn listen_for_events(
        &self,
        addresses: Vec<String>,
        new_addresses_rx: tokio::sync::mpsc::UnboundedReceiver<String>,
        callback: std::sync::Arc<dyn Fn(String, String) + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.listen_for_events(addresses, new_addresses_rx, callback)
            .await
    }

    async fn get_balance(
        &self,
        address: &str,
    ) -> Result<Decimal, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref token) = self.token_address {
            // ERC20 Token Balance
            let payload = serde_json::json!([
                {
                    "to": token,
                    "data": format!("0x70a08231{}", pad_evm_address_to_32_bytes(address).trim_start_matches("0x"))
                },
                "latest"
            ]);
            let data = self.rpc_request("eth_call", payload).await?;
            let result_hex = data.get("result").and_then(|v| v.as_str()).unwrap_or("0x0");
            let value_u128 =
                u128::from_str_radix(result_hex.trim_start_matches("0x"), 16).unwrap_or(0);
            Ok(Decimal::from(value_u128) / Decimal::from(10u64.pow(self.decimals)))
        } else {
            // Native Balance
            let data = self
                .rpc_request("eth_getBalance", serde_json::json!([address, "latest"]))
                .await?;
            let result_hex = data
                .get("result")
                .and_then(|v| v.as_str())
                .ok_or("No result in eth_getBalance")?;
            let value_u128 =
                u128::from_str_radix(result_hex.trim_start_matches("0x"), 16).unwrap_or(0);
            Ok(Decimal::from(value_u128) / Decimal::from(10u64.pow(self.decimals)))
        }
    }
}

impl EvmMonitor {
    /// Get current block number
    async fn get_current_block(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let data = self
            .rpc_request("eth_blockNumber", serde_json::json!([]))
            .await?;
        let result = data
            .get("result")
            .and_then(|v| v.as_str())
            .ok_or("No result in response")?;
        let block_number = u64::from_str_radix(result.trim_start_matches("0x"), 16)?;
        Ok(block_number)
    }

    /// Check if transaction succeeded
    async fn check_transaction_success(
        &self,
        tx_hash: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let data = self
            .rpc_request("eth_getTransactionReceipt", serde_json::json!([tx_hash]))
            .await?;
        let result = data.get("result").ok_or("No result in response")?;
        if result.is_null() {
            return Ok(false);
        }
        let status = result
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("0x0");
        Ok(status == "0x1")
    }

    /// Get block timestamp by block number
    async fn get_block_timestamp(
        &self,
        block_number: u64,
    ) -> Result<chrono::DateTime<chrono::Utc>, Box<dyn std::error::Error + Send + Sync>> {
        let tag = format!("0x{:x}", block_number);
        let data = self
            .rpc_request("eth_getBlockByNumber", serde_json::json!([tag, false]))
            .await?;
        let result = data.get("result").ok_or("No result in response")?;
        if result.is_null() {
            return Err("Block not found".into());
        }
        let timestamp_hex = result
            .get("timestamp")
            .and_then(|v| v.as_str())
            .ok_or("No timestamp in block")?;
        let timestamp_secs = u64::from_str_radix(timestamp_hex.trim_start_matches("0x"), 16)?;
        chrono::DateTime::from_timestamp(timestamp_secs as i64, 0)
            .ok_or_else(|| "Invalid timestamp".into())
    }

    async fn get_confirmations(
        &self,
        block_number: Option<u64>,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        let current_block = self.get_current_block().await?;
        if let Some(tx_block) = block_number {
            if current_block > tx_block {
                Ok((current_block - tx_block) as u32)
            } else {
                Ok(0)
            }
        } else {
            Ok(0)
        }
    }

    async fn get_timestamp_or_now(
        &self,
        block_number: Option<u64>,
    ) -> chrono::DateTime<chrono::Utc> {
        if let Some(block_num) = block_number {
            self.get_block_timestamp(block_num)
                .await
                .unwrap_or_else(|e| {
                    warn!("Failed to get block timestamp: {}, using current time", e);
                    chrono::Utc::now()
                })
        } else {
            chrono::Utc::now()
        }
    }
}

/// Factory function to create appropriate blockchain monitor
pub fn get_blockchain_monitor(
    crypto_type: &CryptoType,
    config: crate::config::Config,
    is_sandbox: bool,
) -> Box<dyn BlockchainMonitor> {
    let token_address = crypto_type.token_address().map(|s| s.to_string());
    let decimals = crypto_type.decimals();

    match crypto_type.network() {
        "SOLANA" | "SOLANA_SPL" => {
            let expected_mint = crypto_type.token_address().map(|s| s.to_string());
            Box::new(crate::payment::sol_monitor::SolanaMonitor::new(
                &config,
                is_sandbox,
                expected_mint,
            ))
        }
        "BITCOIN" => Box::new(self::btc_monitor::BtcMonitor::from_config(
            &config, is_sandbox,
        )),
        "ETHEREUM" => Box::new(EvmMonitor::new_ethereum(
            &config,
            is_sandbox,
            token_address,
            decimals,
        )),
        "BINANCE" => Box::new(EvmMonitor::new_bsc(
            &config,
            is_sandbox,
            token_address,
            decimals,
        )),
        "POLYGON" => Box::new(EvmMonitor::new_polygon(
            &config,
            is_sandbox,
            token_address,
            decimals,
        )),
        "ARBITRUM" => Box::new(EvmMonitor::new_arbitrum(
            &config,
            is_sandbox,
            token_address,
            decimals,
        )),
        _ => {
            // Default to Solana for unknown types (fallback)
            let expected_mint = crypto_type.token_address().map(|s| s.to_string());
            Box::new(crate::payment::sol_monitor::SolanaMonitor::new(
                &config,
                is_sandbox,
                expected_mint,
            ))
        }
    }
}
