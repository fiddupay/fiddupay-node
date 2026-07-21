// Solana Payment Monitor
// Monitors Solana blockchain for SOL and SPL token payments

use async_trait::async_trait;
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;

use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn};

use super::blockchain_monitor::BlockchainMonitor;
use super::models::{BlockchainTransaction, CryptoType};

// Redact queries from URLs to prevent leaking API keys
fn redact_url(url: &str) -> String {
    if let Some(idx) = url.find('?') {
        return format!("{}?***REDACTED***", &url[..idx]);
    }
    if let Some(idx) = url.find("alchemy.com/v2/") {
        return format!("{}alchemy.com/v2/***REDACTED***", &url[..idx]);
    }
    url.to_string()
}

// Get Solana WS URL from config based on the selected RPC node
fn get_solana_ws_url(config: &crate::config::Config, rpc_url: &str) -> String {
    if rpc_url == config.solana_rpc_url {
        return config.solana_ws_url.clone();
    }
    if rpc_url == config.solana_devnet_rpc_url {
        return config.solana_devnet_ws_url.clone();
    }

    // Convert https:// -> wss:// and http:// -> ws://
    if rpc_url.starts_with("https://") {
        rpc_url.replace("https://", "wss://")
    } else if rpc_url.starts_with("http://") {
        rpc_url.replace("http://", "ws://")
    } else {
        if rpc_url.contains("devnet") {
            config.solana_devnet_ws_url.clone()
        } else {
            config.solana_ws_url.clone()
        }
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
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct GetSignaturesResult {
    signature: String,
    #[serde(rename = "blockTime")]
    block_time: Option<i64>,
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
}

#[derive(Debug, Deserialize)]
struct TransactionMessage {
    #[serde(rename = "accountKeys")]
    account_keys: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TokenBalanceInfo {
    #[serde(rename = "accountIndex")]
    account_index: u64,
    mint: Option<String>,
    owner: Option<String>,
    #[serde(rename = "uiTokenAmount")]
    ui_token_amount: Option<UiTokenAmount>,
}

#[derive(Debug, Deserialize)]
struct UiTokenAmount {
    amount: Option<String>,
    decimals: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct TransactionMeta {
    err: Option<serde_json::Value>,
    #[serde(rename = "preBalances")]
    pre_balances: Vec<u64>,
    #[serde(rename = "postBalances")]
    post_balances: Vec<u64>,
    #[serde(rename = "preTokenBalances")]
    #[serde(default)]
    pre_token_balances: Vec<TokenBalanceInfo>,
    #[serde(rename = "postTokenBalances")]
    #[serde(default)]
    post_token_balances: Vec<TokenBalanceInfo>,
}

#[derive(Debug, Clone)]
pub struct SolanaMonitor {
    client: Client,
    pub rpc_urls: Vec<String>,
    pub historical_rpc_urls: Vec<String>,
    pub ws_urls: Vec<String>,
    expected_mint: Option<String>,
}

impl SolanaMonitor {
    pub fn new(
        config: &crate::config::Config,
        is_sandbox: bool,
        expected_mint: Option<String>,
    ) -> Self {
        let mut rpc_urls = Vec::new();
        let mut historical_rpc_urls = Vec::new();

        if is_sandbox {
            // Helius - Priority 1 for Devnet (Requested by user for better RPC reliability/limits)
            if let Some(ref key) = config.helius_api_key {
                let helius_devnet_url = format!("https://devnet.helius-rpc.com/?api-key={}", key);
                rpc_urls.push(helius_devnet_url.clone());
                historical_rpc_urls.push(helius_devnet_url);
            }
            // Fallback default
            rpc_urls.push(config.solana_devnet_rpc_url.clone());
            historical_rpc_urls.push(config.solana_devnet_rpc_url.clone());
        } else {
            // SVS - Priority 1 (Primary for Mainnet)
            for key in &config.svs_api_keys {
                let svs_live_url =
                    format!("https://basic.rpc.solanavibestation.com/?api_key={}", key);
                let svs_historical_url = format!(
                    "https://basic.rpc.solanavibestation.com/historical?api_key={}",
                    key
                );
                // Live endpoint
                rpc_urls.push(svs_live_url);
                // Historical endpoint for deep queries / getTransaction / getSignaturesForAddress
                historical_rpc_urls.push(svs_historical_url);
            }

            // Helius - Priority 2 (Backup)
            if let Some(ref key) = config.helius_api_key {
                let helius_url = format!("https://mainnet.helius-rpc.com/?api-key={}", key);
                rpc_urls.push(helius_url.clone());
                historical_rpc_urls.push(helius_url);
            }

            // Alchemy - Priority 3
            for key in &config.alchemy_api_keys {
                let alchemy_url = format!("https://solana-mainnet.g.alchemy.com/v2/{}", key);
                rpc_urls.push(alchemy_url.clone());
                historical_rpc_urls.push(alchemy_url);
            }

            // Fallback default
            rpc_urls.push(config.solana_rpc_url.clone());
            historical_rpc_urls.push(config.solana_rpc_url.clone());
        }

        // We store all possible WS URLs to fail over if needed
        let mut ws_urls = Vec::new();
        for rpc in &rpc_urls {
            ws_urls.push(get_solana_ws_url(config, rpc));
        }

        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| Client::new()),
            rpc_urls,
            historical_rpc_urls,
            ws_urls,
            expected_mint,
        }
    }

    /// Derives the Associated Token Account (ATA) for a given owner and mint.
    fn get_ata_address(owner: &str, mint: &str) -> Option<String> {
        let owner_pubkey = Pubkey::from_str(owner).ok()?;
        let mint_pubkey = Pubkey::from_str(mint).ok()?;
        let ata_pubkey = get_associated_token_address(&owner_pubkey, &mint_pubkey);
        Some(ata_pubkey.to_string())
    }

    async fn rpc_request<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: serde_json::Value,
        is_historical: bool,
    ) -> Result<RpcResponse<T>, Box<dyn std::error::Error + Send + Sync>> {
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: method.to_string(),
            params,
        };
        let mut last_error = None;

        let urls = if is_historical {
            &self.historical_rpc_urls
        } else {
            &self.rpc_urls
        };

        for url in urls {
            match self.client.post(url).json(&request).send().await {
                Ok(response) => {
                    if response.status() == 429 {
                        warn!(
                            "Solana rate limit hit on {}, trying next RPC...",
                            redact_url(url)
                        );
                        last_error = Some("Rate limit hit".to_string());
                        continue;
                    }
                    match response.json::<RpcResponse<T>>().await {
                        Ok(data) => return Ok(data),
                        Err(e) => {
                            warn!("Solana RPC JSON parse error on {}: {}", redact_url(url), e);
                            last_error = Some(e.to_string());
                        }
                    }
                }
                Err(e) => {
                    warn!("Network error connecting to {}: {}", redact_url(url), e);
                    last_error = Some(e.to_string());
                }
            }
        }
        Err(format!("All Solana RPC nodes failed. Last error: {:?}", last_error).into())
    }

    /// Get recent transactions for an address
    pub async fn get_transactions_to_address(
        &self,
        address: &str,
        limit: usize,
        min_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<BlockchainTransaction>, Box<dyn std::error::Error + Send + Sync>> {
        if address.starts_with("0x") || address.len() < 32 || Pubkey::from_str(address).is_err() {
            return Ok(Vec::new());
        }

        info!(" Fetching Solana transactions for address: {}", address);

        let current_slot = self.get_current_slot().await.unwrap_or(0);

        let mut addresses_to_check = vec![address.to_string()];
        let mut ata_address: Option<String> = None;
        if let Some(ref mint) = self.expected_mint {
            if let Some(ata) = Self::get_ata_address(address, mint) {
                if ata != address {
                    info!(" Monitoring ATA: {} for mint: {}", ata, mint);
                    addresses_to_check.push(ata.clone());
                    ata_address = Some(ata);
                }
            }
        }

        let mut all_signatures = std::collections::HashSet::new();

        for addr in addresses_to_check {
            let rpc_response: RpcResponse<Vec<GetSignaturesResult>> = match self
                .rpc_request(
                    "getSignaturesForAddress",
                    serde_json::json!([addr, { "limit": limit, "commitment": "finalized" }]),
                    true,
                )
                .await
            {
                Ok(res) => res,
                Err(e) => {
                    error!(
                        "Solana RPC getSignaturesForAddress error for {}: {}",
                        addr, e
                    );
                    continue;
                }
            };

            if let Some(err) = rpc_response.error {
                error!(
                    "Solana RPC getSignaturesForAddress error for {} {}: {}",
                    addr, err.code, err.message
                );
                continue;
            }

            if let Some(signatures) = rpc_response.result {
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
                    all_signatures.insert(sig.signature);
                }
            }
        }

        let mut blockchain_txs = Vec::new();

        // Get details for each transaction
        for signature in all_signatures {
            match self
                .get_transaction_details_with_slot(&signature, Some(current_slot))
                .await
            {
                Ok(tx) => {
                    // Include if it's a successful transaction involving THIS exact address or ATA
                    let is_incoming =
                        tx.to_address == address || ata_address.as_ref() == Some(&tx.to_address);
                    let is_outgoing = tx.from_address == address
                        || ata_address.as_ref() == Some(&tx.from_address);

                    if tx.success
                        && (tx.amount > Decimal::ZERO || self.expected_mint.is_none())
                        && (is_incoming || is_outgoing)
                    {
                        blockchain_txs.push(tx);
                    }
                }
                Err(e) => {
                    warn!("Failed to get transaction {}: {}", signature, e);
                }
            }
        }

        info!(" Found {} SOL transactions", blockchain_txs.len());
        Ok(blockchain_txs)
    }

    async fn rpc_request_transaction(
        &self,
        signature: &str,
    ) -> Result<RpcResponse<Option<TransactionResult>>, Box<dyn std::error::Error + Send + Sync>>
    {
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
        let mut last_error = None;
        let mut last_response = None;

        for url in &self.historical_rpc_urls {
            match self.client.post(url).json(&request).send().await {
                Ok(response) => {
                    if response.status() == 429 {
                        warn!(
                            "Solana rate limit hit on {}, trying next RPC...",
                            redact_url(url)
                        );
                        last_error = Some("Rate limit hit".to_string());
                        continue;
                    }
                    match response
                        .json::<RpcResponse<Option<TransactionResult>>>()
                        .await
                    {
                        Ok(data) => {
                            if data.error.is_none() && matches!(data.result, Some(Some(_))) {
                                return Ok(data);
                            }
                            last_response = Some(data);
                            warn!(
                                "Solana transaction {} returned null on {}, trying next fallback RPC...",
                                signature,
                                redact_url(url)
                            );
                        }
                        Err(e) => {
                            warn!("Solana RPC JSON parse error on {}: {}", redact_url(url), e);
                            last_error = Some(e.to_string());
                        }
                    }
                }
                Err(e) => {
                    warn!("Network error connecting to {}: {}", redact_url(url), e);
                    last_error = Some(e.to_string());
                }
            }
        }

        if let Some(resp) = last_response {
            Ok(resp)
        } else {
            Err(format!("All Solana RPC nodes failed. Last error: {:?}", last_error).into())
        }
    }

    /// Get transaction details with optional current slot for optimization
    pub async fn get_transaction_details_with_slot(
        &self,
        signature: &str,
        current_slot: Option<u64>,
    ) -> Result<BlockchainTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let mut tx_result: Option<TransactionResult> = None;
        let mut retry_count = 0;
        let max_retries = 3;

        while retry_count <= max_retries {
            let rpc_response = self.rpc_request_transaction(signature).await;

            match rpc_response {
                Ok(res) => {
                    if let Some(err) = res.error {
                        error!(
                            "Solana RPC getTransaction error for {}: {}: {}",
                            signature, err.code, err.message
                        );
                        return Err(format!("RPC Error: {}", err.message).into());
                    }
                    if let Some(Some(tx)) = res.result {
                        tx_result = Some(tx);
                        break;
                    }
                }
                Err(e) => {
                    error!("Solana RPC getTransaction error for {}: {}", signature, e);
                    return Err(e);
                }
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
        let (from_opt, to_address, amount, token_mint) = if let Some(ref meta) = tx_result.meta {
            // --- SPL Token Transfer Detection ---
            if !meta.post_token_balances.is_empty() {
                // Find the token account that received tokens by comparing pre vs post
                let mut best_recipient_owner: Option<String> = None;
                let mut best_sender_owner: Option<String> = None;
                let mut best_mint: Option<String> = None;
                let mut best_amount = Decimal::ZERO;

                for post_tb in &meta.post_token_balances {
                    // MINT VALIDATION: skip if we expect a specific mint and this doesn't match
                    if let (Some(expected), Some(actual_mint)) =
                        (&self.expected_mint, &post_tb.mint)
                    {
                        if expected != actual_mint {
                            continue;
                        }
                    }

                    let post_raw = post_tb
                        .ui_token_amount
                        .as_ref()
                        .and_then(|u| u.amount.as_ref())
                        .and_then(|a| a.parse::<u128>().ok())
                        .unwrap_or(0);

                    let decimals = post_tb
                        .ui_token_amount
                        .as_ref()
                        .and_then(|u| u.decimals)
                        .unwrap_or(6);

                    // Find matching pre-balance for same account index AND mint
                    let pre_raw = meta
                        .pre_token_balances
                        .iter()
                        .find(|pre| {
                            pre.account_index == post_tb.account_index && pre.mint == post_tb.mint
                        })
                        .and_then(|pre| pre.ui_token_amount.as_ref())
                        .and_then(|u| u.amount.as_ref())
                        .and_then(|a| a.parse::<u128>().ok())
                        .unwrap_or(0);

                    if post_raw > pre_raw {
                        let increase = post_raw - pre_raw;
                        let token_amount =
                            Decimal::from(increase) / Decimal::from(10u64.pow(decimals));

                        if token_amount > best_amount {
                            best_amount = token_amount;
                            best_recipient_owner = post_tb.owner.clone().or_else(|| {
                                tx_result
                                    .transaction
                                    .message
                                    .account_keys
                                    .get(post_tb.account_index as usize)
                                    .cloned()
                            });
                            best_mint = post_tb.mint.clone();
                        }
                    } else if pre_raw > post_raw {
                        let decrease = pre_raw - post_raw;
                        let decrease_amount =
                            Decimal::from(decrease) / Decimal::from(10u64.pow(decimals));
                        if decrease_amount > Decimal::ZERO {
                            best_sender_owner = post_tb.owner.clone().or_else(|| {
                                tx_result
                                    .transaction
                                    .message
                                    .account_keys
                                    .get(post_tb.account_index as usize)
                                    .cloned()
                            });
                        }
                    }
                }

                if let Some(owner) = best_recipient_owner {
                    info!(
                        "[SOL-MONITOR] SPL token transfer detected: {} (mint: {:?}) from {:?} to owner {}",
                        best_amount, best_mint, best_sender_owner, owner
                    );
                    (best_sender_owner, owner, best_amount, best_mint)
                } else {
                    // No matching token transfer found, fall back to native SOL
                    let (addr, amt) = Self::parse_sol_balance_diff(
                        meta,
                        &tx_result.transaction.message.account_keys,
                    );
                    (None, addr, amt, None)
                }
            } else {
                // --- Native SOL Transfer ---
                let (addr, amt) =
                    Self::parse_sol_balance_diff(meta, &tx_result.transaction.message.account_keys);
                (None, addr, amt, None)
            }
        } else {
            // No metadata available, can't parse balances
            (
                None,
                tx_result
                    .transaction
                    .message
                    .account_keys
                    .get(1)
                    .cloned()
                    .unwrap_or_default(),
                Decimal::ZERO,
                None,
            )
        };

        // Get sender address (the account that lost tokens, or the fee payer)
        let from_address = from_opt.unwrap_or_else(|| {
            tx_result
                .transaction
                .message
                .account_keys
                .first()
                .cloned()
                .unwrap_or_default()
        });

        // Check if transaction succeeded
        let success = tx_result
            .meta
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
            if count > 32 {
                32
            } else {
                count
            }
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
            timestamp: chrono::DateTime::from_timestamp(tx_result.block_time.unwrap_or(0), 0),
            success,
            token_mint,
        })
    }

    /// Public wrapper for get_transaction_details (Requirement 3.1)
    pub async fn get_transaction_details(
        &self,
        signature: &str,
    ) -> Result<BlockchainTransaction, Box<dyn std::error::Error + Send + Sync>> {
        self.get_transaction_details_with_slot(signature, None)
            .await
    }

    /// Get current slot number
    async fn get_current_slot(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let rpc_response: RpcResponse<u64> = self
            .rpc_request("getSlot", serde_json::json!([]), false)
            .await?;
        if let Some(err) = rpc_response.error {
            error!("Solana RPC getSlot error {}: {}", err.code, err.message);
            return Err(format!("RPC Error: {}", err.message).into());
        }
        Ok(rpc_response.result.unwrap_or(0))
    }

    /// Listen for new transactions using WebSockets (Requirement: Push-based)
    async fn listen_for_events(
        &self,
        addresses: Vec<String>,
        mut new_addresses_rx: tokio::sync::mpsc::UnboundedReceiver<String>,
        callback: std::sync::Arc<dyn Fn(String, String) + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut ws_stream_opt = None;

        for url in &self.ws_urls {
            let safe_url = redact_url(url);
            info!("🔌 Attempting connection to Solana WebSocket: {}", safe_url);
            match connect_async(url.as_str()).await {
                Ok((stream, _)) => {
                    ws_stream_opt = Some(stream);
                    info!(
                        "✅ Successfully connected to Solana WebSocket: {}",
                        redact_url(url)
                    );
                    break;
                }
                Err(e) => {
                    warn!(
                        "❌ Failed to connect to Solana WebSocket {}: {}",
                        safe_url, e
                    );
                    continue;
                }
            }
        }

        let ws_stream = match ws_stream_opt {
            Some(stream) => stream,
            None => {
                let err_msg = "All Solana WebSocket nodes failed to connect".to_string();
                error!("{}", err_msg);
                return Err(err_msg.into());
            }
        };

        let (mut write, mut read) = ws_stream.split();

        // Map of subscription_id -> monitored_address (monitored_address could be an ATA)
        let mut subscription_map = std::collections::HashMap::new();
        // Map of monitored_address -> owner_address (for ATAs, owner_address is the wallet; for native, both are same)
        let mut owner_map = std::collections::HashMap::new();

        let mut next_request_id = 1u64;

        // Combine owner addresses and their ATAs for monitoring
        let mut initial_monitor_addresses = Vec::new();
        let tokens_to_monitor = vec![CryptoType::UsdtSpl, CryptoType::WSol];

        for address in &addresses {
            // Always monitor the owner address (for native SOL)
            owner_map.insert(address.clone(), address.clone());
            initial_monitor_addresses.push(address.clone());

            // Also monitor ATAs for supported tokens
            for token in &tokens_to_monitor {
                if let Some(mint) = token.token_address() {
                    if let Some(ata) = Self::get_ata_address(address, mint) {
                        if ata != *address {
                            owner_map.insert(ata.clone(), address.clone());
                            initial_monitor_addresses.push(ata);
                        }
                    }
                }
            }
        }

        // Subscribe to logs for each address
        let mut wallet_count = 0;
        let mut ata_count = 0;

        for address in &initial_monitor_addresses {
            let request_id = next_request_id;
            next_request_id += 1;

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
            subscription_map.insert(request_id, address.clone());

            if addresses.contains(address) {
                wallet_count += 1;
            } else {
                ata_count += 1;
            }
        }
        info!(
            "✅ Solana WS: Sent {} wallet and {} ATA subscription requests (Total: {})",
            wallet_count,
            ata_count,
            initial_monitor_addresses.len()
        );

        // ✅ NOTE: Automatic history backfill for existing addresses has been disabled.
        // It should now be triggered manually by an admin using the /historical endpoint.

        let mut active_subscriptions = std::collections::HashMap::new();
        let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(10));

        // Handle incoming messages
        loop {
            tokio::select! {
                _ = ping_interval.tick() => {
                    if let Err(e) = write.send(Message::Ping(Default::default())).await {
                        warn!("Failed to send Solana WS ping: {}", e);
                    }
                }
                Some(new_owner) = new_addresses_rx.recv() => {
                    // Monitor new owner and its ATAs
                    let mut to_add = vec![new_owner.clone()];
                    owner_map.insert(new_owner.clone(), new_owner.clone());

                    let tokens_to_monitor = vec![CryptoType::UsdtSpl, CryptoType::WSol];
                    for token in &tokens_to_monitor {
                        if let Some(mint) = token.token_address() {
                            if let Some(ata) = Self::get_ata_address(&new_owner, mint) {
                                if ata != new_owner {
                                    owner_map.insert(ata.clone(), new_owner.clone());
                                    to_add.push(ata);
                                }
                            }
                        }
                    }

                    for addr in &to_add {
                        let request_id = next_request_id;
                        next_request_id += 1;

                        let subscribe_msg = serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": request_id,
                            "method": "logsSubscribe",
                            "params": [
                                { "mentions": [addr] },
                                { "commitment": "confirmed" }
                            ]
                        });
                        if let Err(e) = write.send(Message::Text(subscribe_msg.to_string())).await {
                            warn!("Failed to send dynamic subscription for {}: {}", addr, e);
                            continue;
                        }
                        subscription_map.insert(request_id, addr.clone());
                    }
                    info!("✅ Solana WS: Sent dynamic subscriptions for wallet {} + {} ATAs", new_owner, to_add.len() - 1);

                    // ✅ NOTE: Automatic history backfill for dynamic addresses has been disabled.
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
                                        // info!("📡 Active subscription ID {} for address/ATA {}", sub_id, address);
                                    }
                                }
                            }

                            // 2. Check if it's a notification
                            if v["method"] == "logsNotification" {
                                if let Some(sub_id) = v["params"]["subscription"].as_u64() {
                                    if let Some(monitored_addr) = active_subscriptions.get(&sub_id) {
                                        if let Some(signature) = v["params"]["result"]["value"]["signature"].as_str() {
                                            let err = &v["params"]["result"]["value"]["err"];
                                            if err.is_null() {
                                                // Resolve the OWNER address for the triggered monitored_addr (ATA or owner)
                                                let owner_address = owner_map.get(monitored_addr).unwrap_or(monitored_addr);
                                                info!("🚀 Solana WebSocket: New transaction for {} (Monitored: {}): {}", owner_address, monitored_addr, signature);
                                                callback(signature.to_string(), owner_address.clone());
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
            match self
                .get_transactions_to_address(address, 50, Some(min_ts))
                .await
            {
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
    fn parse_sol_balance_diff(
        meta: &TransactionMeta,
        account_keys: &[String],
    ) -> (String, Decimal) {
        let mut max_increase = 0u64;
        let mut recipient_idx = None;

        if meta.pre_balances.len() == meta.post_balances.len() {
            for i in 0..meta.post_balances.len() {
                let post = meta.post_balances[i];
                let pre = meta.pre_balances[i];
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
                let addr = account_keys.get(idx).cloned().unwrap_or_default();
                // Convert from lamports to SOL (1 SOL = 1_000_000_000 lamports)
                let decimal_amount = Decimal::from(max_increase) / Decimal::from(1_000_000_000u64);
                (addr, decimal_amount)
            }
            None => {
                // Fallback to second account if no increase found
                (
                    account_keys.get(1).cloned().unwrap_or_default(),
                    Decimal::ZERO,
                )
            }
        }
    }
}

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
        self.get_transactions_to_address(address, limit, min_timestamp)
            .await
    }

    fn blockchain_name(&self) -> &'static str {
        "Solana"
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
        if let Some(ref mint) = self.expected_mint {
            // Fetch SPL Token Balance (USDC, WSOL, etc.) via ATA
            let ata = Self::get_ata_address(address, mint)
                .ok_or_else(|| format!("Could not derive ATA for {} and mint {}", address, mint))?;

            let rpc_response: RpcResponse<serde_json::Value> = self
                .rpc_request(
                    "getTokenAccountBalance",
                    serde_json::json!([ata, { "commitment": "confirmed" }]),
                    false,
                )
                .await?;

            if let Some(err) = rpc_response.error {
                // If it's 404/not found, it likely just means the ATA hasn't been created (0 balance)
                if err.code == -32602 || err.message.contains("not found") {
                    return Ok(Decimal::ZERO);
                }
                return Err(
                    format!("Solana RPC getTokenAccountBalance error: {}", err.message).into(),
                );
            }

            let amount_str = rpc_response
                .result
                .and_then(|r| r["value"]["uiAmountString"].as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "0".to_string());

            return Ok(Decimal::from_str(&amount_str).unwrap_or(Decimal::ZERO));
        }

        // Fetch Native SOL Balance
        let rpc_response: RpcResponse<serde_json::Value> = self
            .rpc_request(
                "getBalance",
                serde_json::json!([address, { "commitment": "confirmed" }]),
                false,
            )
            .await?;

        if let Some(err) = rpc_response.error {
            return Err(format!("Solana RPC getBalance error: {}", err.message).into());
        }

        let lamports = rpc_response
            .result
            .and_then(|r| r["value"].as_u64())
            .unwrap_or(0);

        // Convert lamports -> SOL
        Ok(Decimal::from(lamports) / Decimal::from(1_000_000_000u64))
    }
}
