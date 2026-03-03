use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::select;

use crate::api::state::AppState;

pub async fn p2p_trade_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(trade_id): Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_socket(socket, trade_id, state))
}

async fn handle_ws_socket(socket: WebSocket, trade_id: String, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let channel_name = format!("p2p_trade:{}", trade_id);

    if sender.send(Message::Text(format!("Connected to trade room: {}", trade_id))).await.is_err() {
        return;
    }

    // Attempt to get a multiplexed async connection for Subscribing
    let mut pubsub_conn = match state.redis_client.get_async_pubsub().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get Redis Pub/Sub connection: {}", e);
            let _ = sender.send(Message::Text("Error connecting to chat server".into())).await;
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
             // 1. Receive messages from Redis (published by other users/nodes) and send them to this WebSocket client
             Some(msg) = pubsub_stream.next() => {
                 if let Ok(payload) = msg.get_payload::<String>() {
                     if sender.send(Message::Text(payload)).await.is_err() {
                         break; // Client disconnected
                     }
                 }
             }

             // 2. Receive messages from THIS WebSocket client and publish them to Redis
             Some(Ok(msg)) = receiver.next() => {
                  if let Message::Text(text) = msg {
                      let mut publish_conn = match state.redis_client.get_multiplexed_async_connection().await {
                          Ok(conn) => conn,
                          Err(_) => continue, // Ignore fails, try again next message
                      };

                      let _ : redis::RedisResult<()> = redis::cmd("PUBLISH")
                          .arg(&channel_name)
                          .arg(text)
                          .query_async(&mut publish_conn)
                          .await;
                  } else if let Message::Close(_) = msg {
                      break; // Client requested close
                  }
             }
             else => {
                 break; // Socket closed or error
             }
        }
    }
}
