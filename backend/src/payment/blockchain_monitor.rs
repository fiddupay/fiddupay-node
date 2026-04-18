// Multi-Chain Blockchain Monitor
// Provides unified interface for monitoring payments across all supported blockchains

use async_trait::async_trait;
use reqwest::Client;
use rust_decimal::Decimal;
use tracing::{error, info, warn};

use super::models::{BlockchainTransaction, CryptoType};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};

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
}

impl EvmMonitor {
    fn build_rpc_urls(
        chain: &str,
        is_sandbox: bool,
        config: &crate::config::Config,
    ) -> Vec<String> {
        let mut urls = Vec::new();
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
        if chain == "ETH" && !is_sandbox {
            if let Some(ref url) = config.chainstack_eth_url {
                urls.push(url.clone());
            }
        }
        if chain == "BSC" && !is_sandbox {
            if let Some(ref url) = config.chainstack_bsc_url {
                urls.push(url.clone());
            }
        }
        for key in &config.ankr_api_keys {
            if let Some(url) = match (chain, is_sandbox) {
                ("ETH", false) => Some(format!("https://rpc.ankr.com/eth/{}", key)),
                ("ETH", true) => Some(format!("https://rpc.ankr.com/eth_sepolia/{}", key)),
                ("BSC", false) => Some(format!("https://rpc.ankr.com/bsc/{}", key)),
                ("BSC", true) => Some(format!("https://rpc.ankr.com/bsc_testnet_chapel/{}", key)),
                ("POLYGON", false) => Some(format!("https://rpc.ankr.com/polygon/{}", key)),
                ("POLYGON", true) => Some(format!("https://rpc.ankr.com/polygon_amoy/{}", key)),
                ("ARBITRUM", false) => Some(format!("https://rpc.ankr.com/arbitrum/{}", key)),
                ("ARBITRUM", true) => {
                    Some(format!("https://rpc.ankr.com/arbitrum_sepolia/{}", key))
                }
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
        for rpc in &rpc_urls {
            ws_urls.push(get_evm_ws_url(config, rpc));
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
            etherscan_api_url: "https://api.etherscan.io/v2/api".to_string(),
            etherscan_api_key: config.etherscan_api_key.clone(),
            internal_chain_identifier: "BSC",
            is_sandbox,
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
            etherscan_api_url: "https://api.etherscan.io/v2/api".to_string(),
            etherscan_api_key: config.etherscan_api_key.clone(),
            internal_chain_identifier: "ARBITRUM",
            is_sandbox,
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
            etherscan_api_url: "https://api.etherscan.io/v2/api".to_string(),
            etherscan_api_key: config.etherscan_api_key.clone(),
            internal_chain_identifier: "POLYGON",
            is_sandbox,
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
        for rpc in &rpc_urls {
            ws_urls.push(get_evm_ws_url(config, rpc));
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
        }
    }

    /// Listen for new transactions using WebSockets (Optimized Push Model)
    async fn listen_for_events(
        &self,
        addresses: Vec<String>,
        mut new_addresses_rx: tokio::sync::mpsc::UnboundedReceiver<String>,
        callback: std::sync::Arc<dyn Fn(String, String) + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut ws_stream_opt = None;
        let connected_ws_url;

        for url in &self.ws_urls {
            let safe_url = redact_url(url);
            info!(
                "🔌 Attempting connection to {} WebSocket: {}",
                self.chain_name, safe_url
            );
            match connect_async(url.as_str()).await {
                Ok((stream, _)) => {
                    ws_stream_opt = Some(stream);
                    connected_ws_url = url.clone();
                    info!(
                        "✅ Successfully connected to {} WebSocket: {}",
                        self.chain_name,
                        redact_url(&connected_ws_url)
                    );
                    break;
                }
                Err(e) => {
                    warn!(
                        "❌ Failed to connect to {} WebSocket {}: {}",
                        self.chain_name, safe_url, e
                    );
                    // Add a small delay between connection attempts to prevent hammering providers
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

        // Map of subscription_id -> monitored_address
        let mut subscription_map = std::collections::HashMap::new();
        let mut active_subscriptions = std::collections::HashMap::new();
        let mut next_request_id = 1u64;

        // 1. Subscribe to newHeads (to catch Native transfers)
        let request_id = next_request_id;
        next_request_id += 1;
        let head_sub_msg = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "eth_subscribe",
            "params": ["newHeads"]
        });
        write.send(Message::Text(head_sub_msg.to_string())).await?;
        subscription_map.insert(request_id, "NEW_HEADS".to_string());

        // 2. Subscribe to Tokens (via Logs) if we are monitoring a token
        if let Some(ref token) = self.token_address {
            for address in &addresses {
                let request_id = next_request_id;
                next_request_id += 1;

                // Transfer event signature
                let transfer_topic =
                    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
                let padded_address = pad_evm_address_to_32_bytes(address);

                let subscribe_msg = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "method": "eth_subscribe",
                    "params": [
                        "logs",
                        {
                            "address": token,
                            "topics": [transfer_topic, null, padded_address]
                        }
                    ]
                });
                write.send(Message::Text(subscribe_msg.to_string())).await?;
                subscription_map.insert(request_id, address.clone());
            }
            info!(
                "📡 {} WS: Sent {} token log subscription requests",
                self.chain_name,
                addresses.len()
            );
        }

        // --- CATCH-UP BACKFILL REMOVED (Per User Request - Manual Only) ---
        // History backfill is now handled manually by administrators to reduce RPC load.

        let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(15));
        let mut head_sub_id: Option<String> = None;

        loop {
            tokio::select! {
                _ = ping_interval.tick() => {
                    if let Err(e) = write.send(Message::Ping(Default::default())).await {
                        warn!("Failed to send {} WS ping: {}", self.chain_name, e);
                    }
                }
                Some(new_addr) = new_addresses_rx.recv() => {
                    // Subscribe to logs for new address if monitoring tokens
                    if let Some(ref token) = self.token_address {
                        let request_id = next_request_id;
                        next_request_id += 1;

                        let transfer_topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
                        let padded_address = pad_evm_address_to_32_bytes(&new_addr);

                        let subscribe_msg = serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": request_id,
                            "method": "eth_subscribe",
                            "params": [
                                "logs",
                                {
                                    "address": token,
                                    "topics": [transfer_topic, null, padded_address]
                                }
                            ]
                        });
                        if let Err(e) = write.send(Message::Text(subscribe_msg.to_string())).await {
                            warn!("Failed to send dynamic WS subscription for {}: {}", new_addr, e);
                        } else {
                            subscription_map.insert(request_id, new_addr.clone());
                        }
                    }

                    // Quick backfill for new address
                    let cb_clone = callback.clone();
                    let addr_clone = new_addr.clone();
                    let min_ts = Utc::now() - chrono::Duration::minutes(5);
                    let monitor_clone = self.clone_for_ws(); // Helper to clone without some fields if needed
                    tokio::spawn(async move {
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

                            // Handle subscription responses
                            if let Some(id) = v["id"].as_u64() {
                                if let Some(address) = subscription_map.remove(&id) {
                                    if let Some(sub_id) = v["result"].as_str() {
                                        if address == "NEW_HEADS" {
                                            head_sub_id = Some(sub_id.to_string());
                                        } else {
                                            active_subscriptions.insert(sub_id.to_string(), address.clone());
                                        }
                                    }
                                }
                            }

                            // Handle push notifications
                            if let Some(method) = v["method"].as_str() {
                                if method == "eth_subscription" {
                                    let params = &v["params"];
                                    let sub_id = params["subscription"].as_str().unwrap_or("");

                                    // Scenario A: Native Transfer (New Head)
                                    if Some(sub_id.to_string()) == head_sub_id {
                                        // A new block arrived. If monitoring native, we should check it.
                                        // Note: For native monitoring, it's more efficient to check addresses in run_evm_monitor
                                        // when a new block is detected. Here we just notify by triggering a "check".
                                        // For simplicity, we can just trigger a general scan or fetch block transactions.
                                        // Triggering a poll for all current addresses is a safe fallback.
                                        // (Actually, the background task will catch it on next poll if we skip WS native)
                                        // BUT we want it instant. So we should fetch the block.
                                        if self.token_address.is_none() {
                                            let block_hash = params["result"]["hash"].as_str().unwrap_or("");
                                            if !block_hash.is_empty() {
                                                // Fetch block with transactions
                                                if let Ok(block_data) = self.rpc_request("eth_getBlockByHash", serde_json::json!([block_hash, true])).await {
                                                    if let Some(txs) = block_data["result"]["transactions"].as_array() {
                                                        for tx in txs {
                                                            let to = tx["to"].as_str().unwrap_or("").to_lowercase();
                                                            let hash = tx["hash"].as_str().unwrap_or("").to_string();
                                                            // Check if 'to' is one of our addresses
                                                            // (This requires having the address list updated live)
                                                            // We'll pass the hash to the callback if matched
                                                            // For now, let's keep it simple: any transaction hash detected
                                                            // will be validated by the verifier anyway if we send it.
                                                            // But we need to know WHICH address it was for.
                                                            callback(hash, to);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    // Scenario B: Token Transfer (Log)
                                    else if let Some(address) = active_subscriptions.get(sub_id) {
                                        let tx_hash = params["result"]["transactionHash"].as_str().unwrap_or("").to_string();
                                        if !tx_hash.is_empty() {
                                            info!("🚀 {} WS: Detected token transfer for {} in tx {}", self.chain_name, address, tx_hash);
                                            callback(tx_hash, address.clone());
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
        match url.find("alchemy.com/v2/") {
            Some(idx) => format!("{}alchemy.com/v2/***REDACTED***", &url[..idx]),
            None => match url.find("infura.io/v3/") {
                Some(idx) => format!("{}infura.io/v3/***REDACTED***", &url[..idx]),
                None => url.to_string(),
            },
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

        for url in &self.rpc_urls {
            match self.client.post(url).json(&payload).send().await {
                Ok(response) => {
                    let status = response.status();
                    if status == 429 {
                        warn!(
                            "Rate limit (429) hit on {}, applying backoff delay (2s)...",
                            redact_url(url)
                        );
                        last_error = Some("Rate limit hit".to_string());
                        // Apply substantial backoff delay to allow provider throttle to clear
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        continue;
                    }
                    if status == 401 || status == 403 {
                        warn!(
                            "Auth error ({}) for {}, skipping provider...",
                            status,
                            redact_url(url)
                        );
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
                                        "Rate limit payload from {}, trying next RPC...",
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

        let url = if let Some(ref _token) = self.token_address {
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

        for key in &self.moralis_keys {
            if let Ok(response) = self.client.get(&url).header("X-API-Key", key).send().await {
                if response.status() == 429 {
                    continue;
                }
                if response.status().is_success() {
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
        let action = if self.token_address.is_some() {
            "tokentx"
        } else {
            "txlist"
        };
        let mut url = format!("{}?module=account&action={}&address={}&startblock=0&endblock=99999999&page=1&offset={}&sort=desc&chainid={}", 
            self.etherscan_api_url, action, address, limit, self.chain_id);
        if let Some(ref token) = self.token_address {
            url.push_str(&format!("&contractaddress={}", token));
        }
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

        let mut amount = Decimal::ZERO;

        // Check if this is an ERC20 token transfer
        if let Some(ref token) = self.token_address {
            // Verify 'to' is the contract address
            if to_address.trim().to_lowercase() == token.trim().to_lowercase() {
                if input.starts_with("0x") && input.len() >= 138 {
                    let sig = &input[2..10];
                    if sig == "a9059cbb" {
                        // transfer(address,uint256)
                        let recipient_hex = &input[34..74]; // skip 2 (0x) + 8 (sig) + 24 (padding)
                        let amount_hex = &input[74..138];

                        if let Ok(value_u128) = u128::from_str_radix(amount_hex, 16) {
                            amount =
                                Decimal::from(value_u128) / Decimal::from(10u64.pow(self.decimals));
                            to_address = format!("0x{}", recipient_hex);
                            info!(
                                "Parsed ERC20 Transfer: to={}, amount={}",
                                to_address, amount
                            );
                        }
                    }
                }
            } else {
                warn!(
                    "EVM ERC20 monitor expects to_address to be contract {}, got {}",
                    token, to_address
                );
            }
        } else {
            // Native currency transfer
            // Convert hex value to decimal
            let value_u128 =
                u128::from_str_radix(value_hex.trim_start_matches("0x"), 16).unwrap_or(0);
            amount = Decimal::from(value_u128) / Decimal::from(10u64.pow(self.decimals));
        }

        // Get transaction receipt for confirmation status
        let block_number = result
            .get("blockNumber")
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
            self.get_block_timestamp(block_num)
                .await
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
