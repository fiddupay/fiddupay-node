use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::select;

use crate::api::state::AppState;
use crate::middleware::auth::MerchantContext;
use axum::Extension;

/// WebSocket handler for Merchant dashboard notifications
pub async fn merchant_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_socket(socket, context.merchant_id, state))
}

async fn handle_ws_socket(socket: WebSocket, merchant_id: i64, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let channel_name = format!("merchant_notifications:{}", merchant_id);

    // Send connection acknowledgement
    if sender.send(Message::Text(r#"{"event":"connected"}"#.into())).await.is_err() {
        return;
    }

    // Subscribe to Redis Pub/Sub for merchant notifications
    let mut pubsub_conn = match state.redis_client.get_async_pubsub().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get Redis Pub/Sub connection for merchant {}: {}", merchant_id, e);
            return;
        }
    };

    if let Err(e) = pubsub_conn.subscribe(&channel_name).await {
         tracing::error!("Failed to subscribe to channel {}: {}", channel_name, e);
         return;
    }

    let mut pubsub_stream = pubsub_conn.on_message();

    loop {
        select! {
             // 1. Receive messages from Redis and send them to the WebSocket client
             Some(msg) = pubsub_stream.next() => {
                 if let Ok(payload) = msg.get_payload::<String>() {
                     if sender.send(Message::Text(payload)).await.is_err() {
                         break; // Client disconnected
                     }
                 }
             }

             // 2. Client closed connection
             Some(Ok(msg)) = receiver.next() => {
                  if let Message::Close(_) = msg {
                      break; 
                  }
             }
             else => {
                 break; 
             }
        }
    }
}
