use axum::{
    extract::{
        self,
        ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use chrono::{DateTime, Utc};

use crate::action_queue::SharedQueue;
use crate::AppState;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
// Configuration for the monitor connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    // Enable/disable different event types
    pub always_send_screen_updates: bool,
    pub always_send_cursor_updates: bool,
}

#[derive(Clone, Serialize)]
pub struct MonitorEvent {
    pub event_id: String,
    #[serde(flatten)]
    pub payload: MonitorEventPayload,
}

#[derive(Clone, Serialize)]
pub enum MonitorEventPayload {
    #[serde(rename = "action_request")]
    ActionRequest(crate::action_types::ActionRequest),
    #[serde(rename = "action_response")]
    ActionResponse(crate::action_types::ActionResponse),
    #[serde(rename = "screen_update")]
    ScreenUpdate {
        action_id: String, // ID of the action that triggered this screenshot
        image: String,     // Base64 encoded image
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "cursor_update")]
    CursorUpdate {
        action_id: String, // ID of the action that triggered this cursor update
        x: u32,
        y: u32,
        timestamp: DateTime<Utc>,
    },
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            always_send_screen_updates: true,
            always_send_cursor_updates: true,
        }
    }
}

pub async fn monitor_websocket(
    ws: WebSocketUpgrade,
    extract::State(state): extract::State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state.action_queue.clone()))
}

async fn handle_socket(mut socket: WebSocket, queue: SharedQueue) {
    // Subscribe to events from the action queue
    let mut action_rx = queue.subscribe_monitor();

    loop {
        tokio::select! {
            // Handle messages from client
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(_text))) => {
                        // Just send confirmation
                        let _ = socket.send(Message::Text(Utf8Bytes::from(
                            r#"{"status":"message_received"}"#
                        ))).await;
                    },
                    Some(Ok(_)) => {
                        // Ignore other message types
                    },
                    Some(Err(_)) | None => {
                        // Client disconnected
                        break;
                    }
                }
            },

            // Handle action events
            action_event = action_rx.recv() => {
                if let Ok(event) = action_event {
                    if let Ok(msg) = serde_json::to_string(&event) {
                        if socket.send(Message::Text(Utf8Bytes::from(msg))).await.is_err() {
                            break; // Client disconnected
                        }
                    }
                }
            },
        }
    }
}
