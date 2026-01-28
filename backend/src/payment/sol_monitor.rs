// Solana Payment Monitor
// Monitors Solana blockchain for SOL and SPL token payments

use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::{info, warn, error};

use super::models::BlockchainTransaction;
use super::blockchain_monitor::BlockchainMonitor;

// Get Solana RPC URL from config
fn get_solana_rpc_url(config: &crate::config::Config) -> &str {
    &config.solana_rpc_url
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
}

impl SolanaMonitor {
    pub fn new(config: &crate::config::Config, rpc_url: Option<String>) -> Self {
        Self {
            client: Client::new(),
            rpc_url: rpc_url.unwrap_or_else(|| get_solana_rpc_url(config).to_string()),
        }
    }

    /// Get recent transactions for an address
    pub async fn get_transactions_to_address(
        &self,
        address: &str,
        limit: usize,
    ) -> Result<Vec<BlockchainTransaction>, Box<dyn std::error::Error + Send + Sync>> {
        info!(" Fetching Solana transactions for address: {}", address);

        // First, get signatures for address
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "getSignaturesForAddress".to_string(),
            params: serde_json::json!([
                address,
                { "limit": limit }
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
            match self.get_transaction_details(&sig.signature).await {
                Ok(tx) => blockchain_txs.push(tx),
                Err(e) => {
                    warn!("Failed to get transaction {}: {}", sig.signature, e);
                }
            }
        }

        info!(" Found {} SOL transactions", blockchain_txs.len());
        Ok(blockchain_txs)
    }

    /// Get transaction details
    pub async fn get_transaction_details(
        &self,
        signature: &str,
    ) -> Result<BlockchainTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "getTransaction".to_string(),
            params: serde_json::json!([
                signature,
                {
                    "encoding": "json",
                    "maxSupportedTransactionVersion": 0
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

        // Parse transaction amount (difference in balances)
        let amount = if let Some(ref meta) = tx_result.meta {
            if meta.preBalances.len() >= 2 && meta.postBalances.len() >= 2 {
                let sent = meta.preBalances[0] as i64 - meta.postBalances[0] as i64;
                // Convert from lamports to SOL (1 SOL = 1_000_000_000 lamports)
                Decimal::from(sent.abs()) / Decimal::from(1_000_000_000)
            } else {
                Decimal::ZERO
            }
        } else {
            Decimal::ZERO
        };

        // Get addresses from transaction
        let from_address = tx_result.transaction.message.accountKeys.get(0)
            .cloned()
            .unwrap_or_default();

        let to_address = tx_result.transaction.message.accountKeys.get(1)
            .cloned()
            .unwrap_or_default();

        // Check if transaction succeeded
        let success = tx_result.meta
            .as_ref()
            .map(|m| m.err.is_none())
            .unwrap_or(false);

        // Get current slot for confirmations
        let current_slot = self.get_current_slot().await?;
        let confirmations = if current_slot > tx_result.slot {
            (current_slot - tx_result.slot) as u32
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
    #[ignore] // Requires network access
    async fn test_get_current_slot() {
        // Use environment variable or default for testing
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
        let monitor = SolanaMonitor {
            client: reqwest::Client::new(),
            rpc_url,
        };
        match monitor.get_current_slot().await {
            Ok(slot) => println!("Current slot: {}", slot),
            Err(e) => println!("Error: {}", e),
        }
    }
}
