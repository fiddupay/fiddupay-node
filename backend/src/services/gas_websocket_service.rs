// Real-time Gas Fee WebSocket Service (2026)
// Provides live gas price updates via WebSocket subscriptions

use crate::error::ServiceError;
use crate::services::gas_fee_service::GasFeeEstimate;
use futures_util::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::info;

pub struct GasWebSocketService {
    config: crate::config::Config,
    gas_updates: broadcast::Sender<HashMap<String, GasFeeEstimate>>,
}

impl GasWebSocketService {
    pub fn new(config: crate::config::Config) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            config,
            gas_updates: tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<HashMap<String, GasFeeEstimate>> {
        self.gas_updates.subscribe()
    }

    /// Start WebSocket connections for real-time gas price updates
    pub async fn start_gas_monitoring(&self) -> Result<(), ServiceError> {
        info!("Starting Gas WebSocket monitoring service...");
        // Start Ethereum monitoring
        self.monitor_ethereum_gas().await?;
        Ok(())
    }

    /// Monitor Ethereum gas prices via WebSocket
    #[allow(dead_code)]
    async fn monitor_ethereum_gas(&self) -> Result<(), ServiceError> {
        let ws_url = self
            .config
            .ethereum_rpc_url
            .replace("https://", "wss://")
            .replace("http://", "ws://");
        let (ws_stream, _) = match connect_async(ws_url.as_str()).await {
            Ok(stream) => stream,
            Err(e) => {
                return Err(ServiceError::Internal(format!(
                    "ETH WebSocket connection failed: {}",
                    e
                )))
            }
        };

        let (mut write, mut read) = ws_stream.split();

        // Subscribe to new block headers for base fee updates
        let subscribe_msg = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_subscribe",
            "params": ["newHeads"]
        });

        write
            .send(Message::Text(subscribe_msg.to_string()))
            .await
            .map_err(|e| ServiceError::Internal(format!("ETH subscription failed: {}", e)))?;

        let gas_updates = self.gas_updates.clone();

        tokio::spawn(async move {
            info!("ETH Gas WebSocket: Monitoring loop started");
            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(text)) = msg {
                    if let Ok(data) = serde_json::from_str::<Value>(&text) {
                        if let Some(params) = data.get("params") {
                            if let Some(result) = params.get("result") {
                                if let Some(base_fee) =
                                    result.get("baseFeePerGas").and_then(|v| v.as_str())
                                {
                                    if let Ok(base_fee_wei) =
                                        u64::from_str_radix(&base_fee[2..], 16)
                                    {
                                        let gas_limit = 21000u64;
                                        let base_fee_eth = Decimal::new(
                                            base_fee_wei as i64 * gas_limit as i64,
                                            18,
                                        );
                                        let priority_fee_eth =
                                            Decimal::new(2000000000i64 * gas_limit as i64, 18); // 2 gwei default

                                        let estimate = GasFeeEstimate {
                                            network: "ethereum".to_string(),
                                            native_currency: "ETH".to_string(),
                                            standard_fee: base_fee_eth + priority_fee_eth,
                                            fast_fee: (base_fee_eth + priority_fee_eth)
                                                * Decimal::new(15, 1),
                                            estimated_withdrawal_cost: base_fee_eth
                                                + priority_fee_eth,
                                            base_fee: Some(base_fee_eth),
                                            priority_fee: Some(priority_fee_eth),
                                        };

                                        let mut updates = HashMap::new();
                                        updates.insert("ethereum".to_string(), estimate);
                                        let _ = gas_updates.send(updates);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }
}
