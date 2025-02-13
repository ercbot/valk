use axum::{
    extract::{
        self,
        ws::{Utf8Bytes, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};

use crate::action_queue::{ActionQueue, InputDriver};

use std::sync::Arc;

pub async fn monitor_websocket<D: InputDriver>(
    ws: WebSocketUpgrade,
    extract::State(queue): extract::State<Arc<ActionQueue<D>>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, queue))
}

async fn handle_socket<D: InputDriver>(mut socket: WebSocket, queue: Arc<ActionQueue<D>>) {
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
