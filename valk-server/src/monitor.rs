use axum::{
    extract::{
        self,
        ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};

use crate::action_queue::SharedQueue;
use crate::AppState;

use enigo::Mouse;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};

// Define different monitor event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "data")]
pub enum MonitorEvent {
    #[serde(rename = "action_request")]
    ActionRequest(crate::action_types::ActionRequest),

    #[serde(rename = "action_response")]
    ActionResponse(crate::action_types::ActionResponse),

    #[serde(rename = "screen_update")]
    ScreenUpdate {
        image: String,
        timestamp: i64,
        width: u32,
        height: u32,
    },

    #[serde(rename = "cursor_update")]
    CursorUpdate { x: u32, y: u32, timestamp: i64 },
}

// Configuration for the monitor connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    // Enable/disable different event types
    pub action_events: bool,
    pub screen_updates: bool,
    pub cursor_updates: bool,

    // Screen update frequency in ms (0 means disabled)
    pub screen_update_interval: u64,

    // Screen update quality (1-100, where 100 is highest quality)
    pub screen_update_quality: u8,

    // Screen update resolution scale (1.0 = full resolution, 0.5 = half resolution, etc.)
    pub screen_update_scale: f32,
}

impl MonitorConfig {
    // Check if a specific event type is enabled in this configuration
    pub fn is_enabled(&self, event: MonitorEvent) -> bool {
        match event {
            MonitorEvent::ActionRequest(_) | MonitorEvent::ActionResponse(_) => self.action_events,
            MonitorEvent::ScreenUpdate { .. } => self.screen_updates,
            MonitorEvent::CursorUpdate { .. } => self.cursor_updates,
        }
    }
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            action_events: true,
            screen_updates: true,
            cursor_updates: true,
            screen_update_interval: 500, // Default to 2 FPS
            screen_update_quality: 70,   // Default to medium quality JPEG
            screen_update_scale: 1.0,    // Default to full resolution
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
    // Create a channel for sending custom events to this specific client
    let (_client_tx, mut client_rx) = broadcast::channel::<MonitorEvent>(100);

    // Subscribe to action events
    let mut action_rx = queue.subscribe_monitor();

    // Clone queue for each task
    let screen_queue = queue.clone();
    let cursor_queue = queue.clone();

    // Start screen capture task if enabled
    let screen_task = {
        Some(tokio::spawn(async move {
            let mut interval_timer = interval(Duration::from_millis(500)); // Default 2 FPS

            loop {
                interval_timer.tick().await;

                // Capture screen and send the screen update event
                // This will use the shared screenshot functionality
                let _ = screen_queue.send_screen_update_event().await;
            }
        }))
    };

    // Start cursor position tracking
    let cursor_task = {
        Some(tokio::spawn(async move {
            let mut last_pos = (0, 0);
            let mut interval_timer = interval(Duration::from_millis(100)); // Update cursor at 10Hz

            loop {
                interval_timer.tick().await;

                // Get current position using enigo
                let enigo = enigo::Enigo::new(&enigo::Settings::default()).unwrap();
                if let Ok(pos) = enigo.location() {
                    // Only send updates when position changes
                    if pos != last_pos {
                        last_pos = pos;

                        let event = MonitorEvent::CursorUpdate {
                            x: pos.0 as u32,
                            y: pos.1 as u32,
                            timestamp: chrono::Utc::now().timestamp_millis(),
                        };

                        cursor_queue.send_cursor_update_event(event);
                    }
                }
            }
        }))
    };

    // Main event loop
    let mut tasks: Vec<JoinHandle<()>> = vec![];
    if let Some(task) = screen_task {
        tasks.push(task);
    }
    if let Some(task) = cursor_task {
        tasks.push(task);
    }

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

            // Handle custom client events (screen updates, cursor updates)
            client_event = client_rx.recv() => {
                if let Ok(event) = client_event {
                    if let Ok(msg) = serde_json::to_string(&event) {
                        if socket.send(Message::Text(Utf8Bytes::from(msg))).await.is_err() {
                            break; // Client disconnected
                        }
                    }
                }
            }
        }
    }

    // Cleanup on disconnect
    for task in tasks {
        task.abort();
    }
}
