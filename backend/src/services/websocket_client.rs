use futures_util::{SinkExt, StreamExt};

use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn};
use url::Url;

use crate::error::ServiceError;
use crate::models::webhook::WebhookPayload;

pub struct WebSocketClient {
    config: crate::config::Config,
}

impl WebSocketClient {
    pub fn new(
        config: crate::config::Config,
        _event_sender: broadcast::Sender<WebhookPayload>,
    ) -> Self {
        Self { config }
    }

    /// Start Solana WebSocket listener
    pub async fn start_solana_listener(&self) -> Result<(), ServiceError> {
        let rpc_ws_url = self.config.solana_rpc_url.replace("http", "ws");
        let url = Url::parse(&rpc_ws_url)
            .map_err(|e| ServiceError::Internal(format!("Invalid Solana WS URL: {}", e)))?;

        info!("Connecting to Solana WebSocket: {}", url);

        let (ws_stream, _) = connect_async(url.as_str()).await.map_err(|e| {
            ServiceError::Internal(format!("Failed to connect to Solana WS: {}", e))
        })?;

        let (mut write, mut read) = ws_stream.split();

        // Subscribe to program/account notifications (simplified example)
        // In production, we'd subscribe to specific program IDs or accounts we monitor
        let subscription_msg = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "programSubscribe",
            "params": [
                "11111111111111111111111111111111", // System Program (monitoring native SOL transfers)
                {
                    "encoding": "jsonParsed",
                    "commitment": "confirmed"
                }
            ]
        });

        write
            .send(Message::Text(subscription_msg.to_string()))
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to send sub request: {}", e)))?;

        // Process incoming messages
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(_text)) => {
                    // Parse text and trigger payment.detected if relevant
                    // This logic would need to filter for our addresses
                    // For now, we just log detection
                    // info!("Solana WS Message: {}", text);
                }
                Ok(Message::Close(_)) => {
                    warn!("Solana WS connection closed");
                    break;
                }
                Err(e) => {
                    error!("Solana WS error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Start EVM WebSocket listener (Generic)
    pub async fn start_evm_listener(
        &self,
        rpc_url: &str,
        network: &str,
    ) -> Result<(), ServiceError> {
        if !rpc_url.starts_with("ws") {
            warn!(
                "RPC URL for {} is not WebSocket enabled (skipping real-time)",
                network
            );
            return Ok(());
        }

        let url = Url::parse(rpc_url)
            .map_err(|e| ServiceError::Internal(format!("Invalid EVM WS URL: {}", e)))?;

        info!("Connecting to {} WebSocket: {}", network, url);

        // Similar connection logic...
        Ok(())
    }
}
