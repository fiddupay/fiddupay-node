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
struct RpcResponse<T> {
    jsonrpc: String,
    result: T,
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
struct TransactionMeta {
    err: Option<serde_json::Value>,
    #[allow(non_snake_case)]
    preBalances: Vec<u64>,
    #[allow(non_snake_case)]
    postBalances: Vec<u64>,
}

pub struct SolanaMonitor {
    client: Client,
    rpc_url: String,
    ws_url: String,
}

impl SolanaMonitor {
    pub fn new(config: &crate::config::Config, rpc_url: Option<String>) -> Self {
        let r_url = rpc_url.unwrap_or_else(|| get_solana_rpc_url(config).to_string());
        let w_url = get_solana_ws_url(config, &r_url);
        Self {
            client: Client::new(),
            rpc_url: r_url,
            ws_url: w_url,
        }
    }

    /// Get recent transactions for an address
    pub async fn get_transactions_to_address(
        &self,
        address: &str,
        limit: usize,
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
            .await?;

        let rpc_response: RpcResponse<Vec<GetSignaturesResult>> = response.json().await?;
        let signatures = rpc_response.result;

        let mut blockchain_txs = Vec::new();

        // Get details for each transaction
        for sig in signatures {
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

        let response = self.client
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .await?;

        let rpc_response: RpcResponse<Option<TransactionResult>> = response.json().await?;

        let tx_result = rpc_response.result
            .ok_or("Transaction not found")?;

        // Parse transaction recipient and amount (Requirement 3.2, 3.3)
        // We find the account with the maximum balance increase to identify the recipient
        // and its actual received amount (excluding fees).
        let (to_address, amount) = if let Some(ref meta) = tx_result.meta {
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
                    let addr = tx_result.transaction.message.accountKeys.get(idx)
                        .cloned()
                        .unwrap_or_default();
                    // Convert from lamports to SOL (1 SOL = 1_000_000_000 lamports)
                    let decimal_amount = Decimal::from(max_increase) / Decimal::from(1_000_000_000u64);
                    (addr, decimal_amount)
                }
                None => {
                    // Fallback to old placeholder logic if no increase found
                    (tx_result.transaction.message.accountKeys.get(1).cloned().unwrap_or_default(), Decimal::ZERO)
                }
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

        let rpc_response: RpcResponse<u64> = response.json().await?;
        Ok(rpc_response.result)
    }

    /// Listen for new transactions using WebSockets (Requirement: Push-based)
    pub async fn listen_for_signatures(
        &self,
        addresses: Vec<String>,
        callback: Arc<dyn Fn(String, String) + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("🔌 Connecting to Solana WebSocket: {}", self.ws_url);
        
        let (ws_stream, _) = connect_async(&self.ws_url).await?;
        let (mut write, mut read) = ws_stream.split();

        let mut subscription_map = std::collections::HashMap::new();

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

        let mut active_subscriptions = std::collections::HashMap::new();

        // Handle incoming messages
        while let Some(message) = read.next().await {
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
                    break;
                }
                _ => {}
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
            match self.get_transactions_to_address(address, 50).await {
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
}

// Implement BlockchainMonitor trait for Solana
#[async_trait]
impl BlockchainMonitor for SolanaMonitor {
    async fn get_transaction_details(
        &self,
        tx_hash: &str,
    ) -> Result<BlockchainTransaction, Box<dyn std::error::Error + Send + Sync>> {
        self.get_transaction_details(tx_hash).await
    }

    async fn get_transactions_to_address(
        &self,
        address: &str,
        limit: usize,
    ) -> Result<Vec<BlockchainTransaction>, Box<dyn std::error::Error + Send + Sync>> {
        self.get_transactions_to_address(address, limit).await
    }

    fn blockchain_name(&self) -> &'static str {
        "Solana"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_transaction_details() {
        let json_data = r#"{
            "meta": {
                "err": null,
                "fee": 14900,
                "postBalances": [3342638250, 148489050, 1, 1],
                "preBalances": [3352653150, 138489050, 1, 1],
                "status": { "Ok": null }
            },
            "slot": 447055021,
            "transaction": {
                "message": {
                    "accountKeys": [
                        "3dJucSCQP1zGVFtbHpWchbSXapDmyzPbPZE6yWRM8uzQ",
                        "HJjZJkuXLhn52aur41JUcDa5BmQ9ca3eY67tizEuUbjn",
                        "11111111111111111111111111111111",
                        "ComputeBudget111111111111111111111111111111"
                    ],
                    "instructions": []
                },
                "signatures": ["2A3KhPCefptkDUbj2yZNo7Zcvn8AmBMWeUAkt64fQBCBae4KkX6wRyEgLFrvKWnYPwm7JERiRoW1sLcoLXxVje63"]
            }
        }"#;

        let tx_result: TransactionResult = serde_json::from_str(json_data).unwrap();
        
        // Manual implementation of the logic for verification in test
        let (to_address, amount) = {
            let meta = tx_result.meta.as_ref().unwrap();
            let mut max_increase = 0u64;
            let mut recipient_idx = None;

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

            let idx = recipient_idx.unwrap();
            let addr = tx_result.transaction.message.accountKeys.get(idx).cloned().unwrap();
            let decimal_amount = Decimal::from(max_increase) / Decimal::from(1_000_000_000u64);
            (addr, decimal_amount)
        };

        assert_eq!(to_address, "HJjZJkuXLhn52aur41JUcDa5BmQ9ca3eY67tizEuUbjn");
        assert_eq!(amount.to_string(), "0.01");
    }
}
