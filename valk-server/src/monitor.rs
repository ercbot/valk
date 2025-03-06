use axum::{
    extract::{
        self,
        ws::{Utf8Bytes, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};

use crate::action_queue::SharedQueue;
use crate::AppState;

use std::sync::Arc;

pub async fn monitor_websocket(
    ws: WebSocketUpgrade,
    extract::State(state): extract::State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state.action_queue.clone()))
}

async fn handle_socket(mut socket: WebSocket, queue: SharedQueue) {
    let mut rx = queue.subscribe_monitor();

    while let Ok(event) = rx.recv().await {
        if let Ok(msg) = serde_json::to_string(&event) {
            if socket
                .send(axum::extract::ws::Message::Text(Utf8Bytes::from(msg)))
                .await
                .is_err()
            {
                break; // Client disconnected
            }
        }
    }
}
