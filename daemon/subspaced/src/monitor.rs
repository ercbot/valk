use axum::{
    extract::{self, ws::{Utf8Bytes, WebSocket, WebSocketUpgrade}},
    response::IntoResponse
};


use crate::action_queue::ActionQueue;

use std::sync::Arc;

pub async fn monitor_websocket(
    ws: WebSocketUpgrade,
    extract::State(queue): extract::State<Arc<ActionQueue>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, queue))
}

async fn handle_socket(mut socket: WebSocket, queue: Arc<ActionQueue>) {
    let mut rx = queue.subscribe_monitor();

    while let Ok(event) = rx.recv().await {
        if let Ok(msg) = serde_json::to_string(&event) {
            if socket.send(axum::extract::ws::Message::Text(Utf8Bytes::from(msg))).await.is_err() {
                break; // Client disconnected
            }

        }
    }
}