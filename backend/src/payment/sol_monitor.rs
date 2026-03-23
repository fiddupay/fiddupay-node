// Solana Payment Monitor
// Monitors Solana blockchain for SOL and SPL token payments

use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::{info, warn, error};
use std::sync::Arc;
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use super::models::BlockchainTransaction;
use super::blockchain_monitor::BlockchainMonitor;

// Get Solana RPC URL from config
fn get_solana_rpc_url(config: &crate::config::Config) -> &str {
    &config.solana_rpc_url
}

// Get Solana WS URL from config
fn get_solana_ws_url(config: &crate::config::Config, rpc_url: &str) -> String {
    if rpc_url.contains("devnet") {
        config.solana_devnet_ws_url.clone()
    } else {
        config.solana_ws_url.clone()
    }
}

#[derive(Debug, Serialize)]
struct RpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    jsonrpc: String,
    result: Option<T>,
    error: Option<RpcError>,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct GetSignaturesResult {
    signature: String,
    slot: u64,
    #[serde(rename = "blockTime")]
    block_time: Option<i64>,
    confirmationStatus: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TransactionResult {
    slot: u64,
    #[serde(rename = "blockTime")]
    block_time: Option<i64>,
    transaction: SolanaTransaction,
    meta: Option<TransactionMeta>,
}

#[derive(Debug, Deserialize)]
struct SolanaTransaction {
    message: TransactionMessage,
    signatures: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TransactionMessage {
    accountKeys: Vec<String>,
    instructions: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct TokenBalanceInfo {
    #[allow(non_snake_case)]
    accountIndex: u64,
    mint: Option<String>,
    owner: Option<String>,
    #[allow(non_snake_case)]
    uiTokenAmount: Option<UiTokenAmount>,
}

#[derive(Debug, Deserialize)]
struct UiTokenAmount {
    amount: Option<String>,
    decimals: Option<u32>,
    #[allow(non_snake_case)]
    uiAmount: Option<f64>,
    #[allow(non_snake_case)]
    uiAmountString: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TransactionMeta {
    err: Option<serde_json::Value>,
    #[allow(non_snake_case)]
    preBalances: Vec<u64>,
    #[allow(non_snake_case)]
    postBalances: Vec<u64>,
    #[allow(non_snake_case)]
    #[serde(default)]
    preTokenBalances: Vec<TokenBalanceInfo>,
    #[allow(non_snake_case)]
    #[serde(default)]
    postTokenBalances: Vec<TokenBalanceInfo>,
}

pub struct SolanaMonitor {
    client: Client,
    rpc_url: String,
    ws_url: String,
    expected_mint: Option<String>,
}

impl SolanaMonitor {
    pub fn new(config: &crate::config::Config, custom_rpc_url: Option<String>, expected_mint: Option<String>) -> Self {
        let rpc_url = custom_rpc_url.unwrap_or_else(|| config.solana_rpc_url.clone());
        let ws_url = get_solana_ws_url(config, &rpc_url);
        
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| Client::new()),
            rpc_url,
            ws_url,
            expected_mint,
        }
    }

    /// Get recent transactions for an address
    pub async fn get_transactions_to_address(
        &self,
        address: &str,
        limit: usize,
        min_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<BlockchainTransaction>, Box<dyn std::error::Error + Send + Sync>> {
        info!(" Fetching Solana transactions for address: {}", address);

        // Fetch current slot once to avoid N+1 RPC calls for confirmation calculations
        let current_slot = self.get_current_slot().await.unwrap_or(0);

        // First, get signatures for address
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "getSignaturesForAddress".to_string(),
            params: serde_json::json!([
                address,
                { "limit": limit, "commitment": "confirmed" }
            ]),
        };

        let response = self.client
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                error!("Solana RPC getSignaturesForAddress network error: {}", e);
                e
            })?;

        let rpc_response: RpcResponse<Vec<GetSignaturesResult>> = response.json().await
            .map_err(|e| {
                error!("Solana RPC getSignaturesForAddress JSON error: {}", e);
                e
            })?;

        if let Some(err) = rpc_response.error {
            error!("Solana RPC getSignaturesForAddress error {}: {}", err.code, err.message);
            return Err(format!("RPC Error: {}", err.message).into());
        }

        let signatures = rpc_response.result.unwrap_or_default();

        let mut blockchain_txs = Vec::new();

        // Get details for each transaction
        for sig in signatures {
            // Optimization: Skip transactions older than min_timestamp (Requirement 3.8 protection)
            if let (Some(min_ts), Some(block_time)) = (min_timestamp, sig.block_time) {
                if let Some(ts) = chrono::DateTime::from_timestamp(block_time, 0) {
                    // Allow 60s buffer for clock skew
                    if ts < min_ts - chrono::Duration::seconds(60) {
                        continue;
                    }
                }
            }

            match self.get_transaction_details_with_slot(&sig.signature, Some(current_slot)).await {
                Ok(tx) => blockchain_txs.push(tx),
                Err(e) => {
                    warn!("Failed to get transaction {}: {}", sig.signature, e);
                }
            }
        }

        info!(" Found {} SOL transactions", blockchain_txs.len());
        Ok(blockchain_txs)
    }

    /// Get transaction details with optional current slot for optimization
    pub async fn get_transaction_details_with_slot(
        &self,
        signature: &str,
        current_slot: Option<u64>,
    ) -> Result<BlockchainTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "getTransaction".to_string(),
            params: serde_json::json!([
                signature,
                {
                    "encoding": "json",
                    "maxSupportedTransactionVersion": 0,
                    "commitment": "confirmed"
                }
            ]),
        };

        let mut tx_result: Option<TransactionResult> = None;
        let mut retry_count = 0;
        let max_retries = 3;

        while retry_count <= max_retries {
            let response = self.client
                .post(&self.rpc_url)
                .json(&request)
                .send()
                .await
                .map_err(|e| {
                    error!("Solana RPC getTransaction network error for {}: {}", signature, e);
                    e
                })?;

            let text = response.text().await?;
            let rpc_response: RpcResponse<Option<TransactionResult>> = match serde_json::from_str(&text) {
                Ok(res) => res,
                Err(e) => {
                    error!("Solana RPC getTransaction JSON error for {}: {} | Raw response: {}", signature, e, text);
                    return Err(e.into());
                }
            };

            if let Some(err) = rpc_response.error {
                error!("Solana RPC getTransaction error for {}: {}: {}", signature, err.code, err.message);
                return Err(format!("RPC Error: {}", err.message).into());
            }

            if let Some(Some(res)) = rpc_response.result {
                tx_result = Some(res);
                break;
            }

            if retry_count < max_retries {
                retry_count += 1;
                warn!("Solana Transaction {} not found yet (indexed status lag). Retrying in 2s... (Attempt {}/{})", signature, retry_count, max_retries);
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            } else {
                break;
            }
        }

        let tx_result = tx_result.ok_or_else(|| {
            warn!("Solana Transaction {} not found after {} retries. This is likely due to RPC node indexing lag.", signature, max_retries);
            "Transaction not found"
        })?;

        // Parse transaction recipient and amount (Requirement 3.2, 3.3)
        // Strategy:
        //   1. If postTokenBalances exist → SPL token transfer (e.g. USDT)
        //      → Use the token account OWNER as to_address, and the token amount
        //   2. Otherwise → Native SOL transfer
        //      → Use lamport balance diffs (preBalances/postBalances)
        let (to_address, amount) = if let Some(ref meta) = tx_result.meta {
            // --- SPL Token Transfer Detection ---
            if !meta.postTokenBalances.is_empty() {
                // Find the token account that received tokens by comparing pre vs post
                let mut best_recipient_owner: Option<String> = None;
                let mut best_amount = Decimal::ZERO;

                for post_tb in &meta.postTokenBalances {
                    // MINT VALIDATION: skip if we expect a specific mint and this doesn't match
                    if let (Some(expected), Some(actual_mint)) = (&self.expected_mint, &post_tb.mint) {
                        if expected != actual_mint {
                            continue;
                        }
                    }

                    let post_raw = post_tb.uiTokenAmount.as_ref()
                        .and_then(|u| u.amount.as_ref())
                        .and_then(|a| a.parse::<u128>().ok())
                        .unwrap_or(0);

                    let decimals = post_tb.uiTokenAmount.as_ref()
                        .and_then(|u| u.decimals)
                        .unwrap_or(6);

                    // Find matching pre-balance for same account index AND mint
                    let pre_raw = meta.preTokenBalances.iter()
                        .find(|pre| pre.accountIndex == post_tb.accountIndex && pre.mint == post_tb.mint)
                        .and_then(|pre| pre.uiTokenAmount.as_ref())
                        .and_then(|u| u.amount.as_ref())
                        .and_then(|a| a.parse::<u128>().ok())
                        .unwrap_or(0);

                    if post_raw > pre_raw {
                        let increase = post_raw - pre_raw;
                        let token_amount = Decimal::from(increase) / Decimal::from(10u64.pow(decimals));

                        if token_amount > best_amount {
                            best_amount = token_amount;
                            // Use the OWNER of the token account — this is the merchant wallet
                            best_recipient_owner = post_tb.owner.clone();
                        }
                    }
                }

                if let Some(owner) = best_recipient_owner {
                    info!("[SOL-MONITOR] SPL token transfer detected: {} tokens to owner {}", best_amount, owner);
                    (owner, best_amount)
                } else if self.expected_mint.is_none() {
                    // Fall back to SOL diff only if we aren't looking for a specific token
                    Self::parse_sol_balance_diff(meta, &tx_result.transaction.message.accountKeys)
                } else {
                    // We expected a token but didn't find a matching increase
                    (tx_result.transaction.message.accountKeys.get(1).cloned().unwrap_or_default(), Decimal::ZERO)
                }
            } else if self.expected_mint.is_none() {
                // --- Native SOL Transfer ---
                Self::parse_sol_balance_diff(meta, &tx_result.transaction.message.accountKeys)
            } else {
                // Not an SPL transfer and we were expecting one
                (tx_result.transaction.message.accountKeys.get(1).cloned().unwrap_or_default(), Decimal::ZERO)
            }
        } else {
            (tx_result.transaction.message.accountKeys.get(1).cloned().unwrap_or_default(), Decimal::ZERO)
        };

        // Get sender address (the account that paid for the transaction or the first account)
        let from_address = tx_result.transaction.message.accountKeys.get(0)
            .cloned()
            .unwrap_or_default();

        // Check if transaction succeeded
        let success = tx_result.meta
            .as_ref()
            .map(|m| m.err.is_none())
            .unwrap_or(false);

        // Get current slot for confirmations (optimization: use passed slot if available)
        let current_slot = if let Some(s) = current_slot {
            s
        } else {
            self.get_current_slot().await.unwrap_or(tx_result.slot)
        };

        let confirmations = if current_slot > tx_result.slot {
            let count = (current_slot - tx_result.slot) as u32;
            // Cap at 32 (standard finalization) to avoid confusingly high numbers for users
            if count > 32 { 32 } else { count }
        } else {
            0
        };

        Ok(BlockchainTransaction {
            hash: signature.to_string(),
            from_address,
            to_address,
            amount,
            confirmations,
            block_number: Some(tx_result.slot),
            timestamp: chrono::DateTime::from_timestamp(
                tx_result.block_time.unwrap_or(0) as i64,
                0
            ),
            success,
        })
    }

    /// Public wrapper for get_transaction_details (Requirement 3.1)
    pub async fn get_transaction_details(
        &self,
        signature: &str,
    ) -> Result<BlockchainTransaction, Box<dyn std::error::Error + Send + Sync>> {
        self.get_transaction_details_with_slot(signature, None).await
    }

    /// Get current slot number
    async fn get_current_slot(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "getSlot".to_string(),
            params: serde_json::json!([]),
        };

        let response = self.client
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .await?;

        let rpc_response: RpcResponse<u64> = response.json().await
            .map_err(|e| {
                error!("Solana RPC getSlot JSON error: {}", e);
                e
            })?;

        if let Some(err) = rpc_response.error {
            error!("Solana RPC getSlot error {}: {}", err.code, err.message);
            return Err(format!("RPC Error: {}", err.message).into());
        }

        Ok(rpc_response.result.unwrap_or(0))
    }

    /// Listen for new transactions using WebSockets (Requirement: Push-based)
    pub async fn listen_for_signatures(
        &self,
        addresses: Vec<String>,
        mut new_addresses_rx: tokio::sync::mpsc::UnboundedReceiver<String>,
        callback: Arc<dyn Fn(String, String) + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("🔌 Connecting to Solana WebSocket: {}", self.ws_url);
        
        let (ws_stream, _) = connect_async(&self.ws_url).await?;
        let (mut write, mut read) = ws_stream.split();

        let mut subscription_map = std::collections::HashMap::new();
        let mut next_request_id = (addresses.len() + 1) as u64;

        // Subscribe to logs for each address
        for (i, address) in addresses.iter().enumerate() {
            let request_id = i as u64 + 1;
            let subscribe_msg = serde_json::json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "method": "logsSubscribe",
                "params": [
                    { "mentions": [address] },
                    { "commitment": "confirmed" }
                ]
            });
            write.send(Message::Text(subscribe_msg.to_string())).await?;
            // We'll map the request_id to address temporarily, 
            // then map the result subscription_id to address once we get the response.
            subscription_map.insert(request_id, address.clone());
            info!("✅ Subscription request sent for: {}", address);
        }

        // --- START CATCH-UP BACKFILL ---
        info!("⏳ Performing Solana history backfill catch-up for {} address(es)...", addresses.len());
        for addr in &addresses {
            let min_ts = Utc::now() - chrono::Duration::minutes(10);
            match self.get_transactions_to_address(addr, 10, Some(min_ts)).await {
                Ok(txs) => {
                    info!(" Found {} historical transactions for {}. Triggering verification backfill...", txs.len(), addr);
                    for tx in txs {
                        callback(tx.hash.clone(), addr.clone());
                    }
                }
                Err(e) => warn!("Solana history catch-up backfill failed for {}: {}", addr, e),
            }
            // Throttle requests to avoid RPC Rate Limiting (429) on free nodes
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        // --- END CATCH-UP BACKFILL ---

        let mut active_subscriptions = std::collections::HashMap::new();

        // Handle incoming messages
        loop {
            tokio::select! {
                Some(new_addr) = new_addresses_rx.recv() => {
                    let request_id = next_request_id;
                    next_request_id += 1;
                    
                    let subscribe_msg = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": request_id,
                        "method": "logsSubscribe",
                        "params": [
                            { "mentions": [new_addr] },
                            { "commitment": "confirmed" }
                        ]
                    });
                    if let Err(e) = write.send(Message::Text(subscribe_msg.to_string())).await {
                        warn!("Failed to send dynamic subscription for {}: {}", new_addr, e);
                        continue;
                    }
                    subscription_map.insert(request_id, new_addr.clone());
                    info!("✅ Dynamic Subscription request sent for: {}", new_addr);

                    // --- START DYNAMIC CATCH-UP ---
                    let addr_clone = new_addr.clone();
                    let cb_clone = callback.clone();
                    let min_ts = Utc::now() - chrono::Duration::minutes(10);
                    match self.get_transactions_to_address(&addr_clone, 5, Some(min_ts)).await {
                        Ok(txs) => {
                            for tx in txs {
                                cb_clone(tx.hash.clone(), addr_clone.clone());
                            }
                        }
                        Err(e) => warn!("Solana dynamic catch-up backfill failed for {}: {}", addr_clone, e),
                    }
                    // --- END DYNAMIC CATCH-UP ---
                }
                message = read.next() => {
                    let message = match message {
                        Some(m) => m,
                        None => break, // Stream closed
                    };

                    match message {
                        Ok(Message::Text(text)) => {
                            let v: serde_json::Value = serde_json::from_str(&text)?;
                            
                            // 1. Check if it's a response to a subscription request
                            if let Some(id) = v["id"].as_u64() {
                                if let Some(address) = subscription_map.remove(&id) {
                                    if let Some(sub_id) = v["result"].as_u64() {
                                        active_subscriptions.insert(sub_id, address.clone());
                                        info!("📡 Active subscription ID {} for address {}", sub_id, address);
                                    }
                                }
                            }

                            // 2. Check if it's a notification
                            if v["method"] == "logsNotification" {
                                if let Some(sub_id) = v["params"]["subscription"].as_u64() {
                                    if let Some(address) = active_subscriptions.get(&sub_id) {
                                        if let Some(signature) = v["params"]["result"]["value"]["signature"].as_str() {
                                            let err = &v["params"]["result"]["value"]["err"];
                                            if err.is_null() {
                                                info!("🚀 Solana WebSocket: New transaction for {}: {}", address, signature);
                                                callback(signature.to_string(), address.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Ok(Message::Close(_)) => {
                            warn!("Solana WebSocket closed");
                            break;
                        }
                        Err(e) => {
                            error!("Solana WebSocket error: {}", e);
                            return Err(e.into());
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    /// Monitor address for new payments
    pub async fn monitor_address(
        &self,
        address: &str,
        callback: impl Fn(BlockchainTransaction) + Send + Sync,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(" Monitoring Solana address: {}", address);

        let mut known_txs = std::collections::HashSet::new();

        loop {
            let min_ts = Utc::now() - chrono::Duration::minutes(10);
            match self.get_transactions_to_address(address, 50, Some(min_ts)).await {
                Ok(transactions) => {
                    for tx in transactions {
                        if !known_txs.contains(&tx.hash) {
                            info!(" New Solana transaction detected: {}", tx.hash);
                            callback(tx.clone());
                            known_txs.insert(tx.hash);
                        }
                    }
                }
                Err(e) => {
                    error!(" Error fetching transactions: {}", e);
                }
            }

            // Poll every 2 seconds (Solana slot time is ~400ms)
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }

    /// Helper method to parse native SOL transfers from pre/post lamport balances
    fn parse_sol_balance_diff(meta: &TransactionMeta, account_keys: &[String]) -> (String, Decimal) {
        let mut max_increase = 0u64;
        let mut recipient_idx = None;

        if meta.preBalances.len() == meta.postBalances.len() {
            for i in 0..meta.postBalances.len() {
                let post = meta.postBalances[i];
                let pre = meta.preBalances[i];
                if post > pre {
                    let increase = post - pre;
                    if increase > max_increase {
                        max_increase = increase;
                        recipient_idx = Some(i);
                    }
                }
            }
        }

        match recipient_idx {
            Some(idx) => {
                let addr = account_keys.get(idx)
                    .cloned()
                    .unwrap_or_default();
                // Convert from lamports to SOL (1 SOL = 1_000_000_000 lamports)
                let decimal_amount = Decimal::from(max_increase) / Decimal::from(1_000_000_000u64);
                (addr, decimal_amount)
            }
            None => {
                // Fallback to second account if no increase found
                (account_keys.get(1).cloned().unwrap_or_default(), Decimal::ZERO)
            }
        }
    }
}

// Implement BlockchainMonitor trait for Solana
#[async_trait]
impl BlockchainMonitor for SolanaMonitor {
    async fn get_transaction_details(
        &self,
        tx_hash: &str,
        _target_address: Option<&str>,
    ) -> Result<BlockchainTransaction, Box<dyn std::error::Error + Send + Sync>> {
        self.get_transaction_details(tx_hash).await
    }

    async fn get_transactions_to_address(
        &self,
        address: &str,
        limit: usize,
        min_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<BlockchainTransaction>, Box<dyn std::error::Error + Send + Sync>> {
        self.get_transactions_to_address(address, limit, min_timestamp).await
    }

    fn blockchain_name(&self) -> &'static str {
        "Solana"
    }
}

