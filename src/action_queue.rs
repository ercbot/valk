use std::sync::Arc;
use serde_json::json;
use tokio::sync::{Mutex, oneshot};
use tokio::time::{sleep, Duration, timeout};
use serde::{Serialize, Deserialize};
use enigo::{Button, Coordinate::Abs, Direction::{Press, Release}, Enigo, Mouse, Keyboard};
use std::str::FromStr;
use crate::key_press::KeyPress;
use xcap::Monitor;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use image::ImageFormat;
use std::io::Cursor;
use axum::response::{IntoResponse, Json};
use axum::http::StatusCode;

const ACTION_DELAY: Duration = Duration::from_millis(500);
const ACTION_TIMEOUT: Duration = Duration::from_secs(10);
const SCREENSHOT_DELAY: Duration = Duration::from_secs(2);

/// Represents possible errors that can occur during action execution
#[derive(Debug)]
pub enum ActionError {
    /// Action took too long to complete
    Timeout,
    /// Action failed during execution
    ExecutionFailed(String),
    /// Invalid input parameters
    InvalidInput(String),
    /// Internal queue communication error
    ChannelError(String),
}

impl IntoResponse for ActionError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ActionError::Timeout => (StatusCode::REQUEST_TIMEOUT, "Action timed out".to_string()),
            ActionError::ExecutionFailed(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ActionError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            ActionError::ChannelError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };


        (status, message).into_response()
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    LeftClick,
    RightClick,
    MiddleClick,
    DoubleClick,
    MouseMove { x: u32, y: u32 },
    LeftClickDrag { x: u32, y: u32 },
    TypeText { text: String },
    KeyPress { key: String },
    Screenshot,
    CursorPosition,
}

/// Result of a successful action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    /// Optional data payload (e.g., screenshot data, cursor position)
    pub data: Option<serde_json::Value>,
}

impl IntoResponse for ActionResult {
    fn into_response(self) -> axum::response::Response {
        Json(json!({
            "status": "success",
            "data": self.data
        })).into_response()
    }
}

#[derive(Clone)]
pub struct ActionQueue {
    queue: Arc<Mutex<Vec<(Action, oneshot::Sender<Result<ActionResult, ActionError>>)>>>,
    enigo: Arc<Mutex<Enigo>>,
}

impl ActionQueue {
    pub fn new(enigo: Enigo) -> Self {
        ActionQueue {
            queue: Arc::new(Mutex::new(Vec::new())),
            enigo: Arc::new(Mutex::new(enigo)),
        }
    }

    // Add an action to the queue
    async fn queue_action(&self, action: Action) -> oneshot::Receiver<Result<ActionResult, ActionError>> {
        let (tx, rx) = oneshot::channel();
        let mut queue = self.queue.lock().await;
        queue.push((action, tx));
        rx
    }

    pub async fn execute_action(&self, action: Action) -> Result<ActionResult, ActionError> {
        let rx = self.queue_action(action.clone()).await;
        
        match timeout(ACTION_TIMEOUT, rx).await {
            Ok(result) => {
                result
                    .map_err(|e| ActionError::ChannelError(e.to_string()))  // Channel errors
                    .and_then(|r| Ok(r?)) 
            }
            Err(_) => {
                // Timeout occurred - remove action from queue if it's still there
                let mut queue = self.queue.lock().await;
                queue.retain(|(a, _)| !std::mem::discriminant(a).eq(&std::mem::discriminant(&action)));
                Err(ActionError::Timeout)
            }
        }
    }

    async fn action_delay() {
        sleep(ACTION_DELAY).await;
    }

    pub async fn start_processing(&self) {
        let queue_clone = self.queue.clone();
        let enigo_clone = self.enigo.clone();

        tokio::spawn(async move {
            loop {
                let action = {
                    let mut queue = queue_clone.lock().await;
                    queue.pop()
                };

                if let Some((action, tx)) = action {
                    let mut enigo = enigo_clone.lock().await;
                    Self::action_delay().await;
                    
                    let result = match action {
                        Action::LeftClick => {
                            let press_result = enigo.button(Button::Left, Press);
                            let release_result = if press_result.is_ok() {
                                Self::action_delay().await;
                                enigo.button(Button::Left, Release)
                            } else {
                                press_result
                            };
                            
                            release_result
                                .map(|_| ActionResult { data: None })
                                .map_err(|e| ActionError::ExecutionFailed(e.to_string()))
                        },

                        Action::RightClick => {
                            let press_result = enigo.button(Button::Right, Press);
                            let release_result = if press_result.is_ok() {
                                Self::action_delay().await;
                                enigo.button(Button::Right, Release)
                            } else {
                                press_result
                            };

                            release_result
                                .map(|_| ActionResult { data: None })
                                .map_err(|e| ActionError::ExecutionFailed(e.to_string()))
                        },

                        Action::MiddleClick => {
                            let press_result = enigo.button(Button::Middle, Press);
                            let release_result = if press_result.is_ok() {
                                Self::action_delay().await;
                                enigo.button(Button::Middle, Release)
                            } else {
                                press_result
                            };  

                            release_result
                                .map(|_| ActionResult { data: None })
                                .map_err(|e| ActionError::ExecutionFailed(e.to_string()))
                        },
                        Action::DoubleClick => {
                            // First click
                            let first_click = match (
                                enigo.button(Button::Left, Press),
                                Self::action_delay().await,
                                enigo.button(Button::Left, Release)
                            ) {
                                (Ok(_), _, Ok(_)) => true,
                                _ => false
                            };

                            if first_click {
                                // Second click
                                match (
                                    enigo.button(Button::Left, Press),
                                    Self::action_delay().await,
                                    enigo.button(Button::Left, Release)
                                ) {
                                    (Ok(_), _, Ok(_)) => Ok(ActionResult { data: None }),
                                    _ => Err(ActionError::ExecutionFailed("Failed to execute second click".into()))
                                }

                            } else {
                                Err(ActionError::ExecutionFailed("Failed to execute first click".into()))
                            }
                        },

                        Action::MouseMove { x, y } => {
                            enigo.move_mouse(x as i32, y as i32, Abs)
                                .map(|_| ActionResult { data: None })
                                .map_err(|e| ActionError::ExecutionFailed(e.to_string()))
                        },
                        Action::LeftClickDrag { x, y } => {
                            match (
                                enigo.button(Button::Left, Press),
                                Self::action_delay().await,
                                enigo.move_mouse(x as i32, y as i32, Abs),
                                Self::action_delay().await,
                                enigo.button(Button::Left, Release)
                            ) {
                                (Ok(_), _, Ok(_), _, Ok(_)) => Ok(ActionResult { data: None }),
                                _ => Err(ActionError::ExecutionFailed("Failed to execute left click drag".into()))
                            }
                        },

                        Action::TypeText { text } => {
                            enigo.text(&text)
                                .map(|_| ActionResult { data: None })
                                .map_err(|e| ActionError::ExecutionFailed(e.to_string()))
                        },
                        Action::KeyPress { key } => {
                            if let Ok(key_press) = KeyPress::from_str(&key) {
                                let result: Result<(), ActionError> = async {
                                    // Press modifiers
                                    for modifier in &key_press.modifiers {
                                        enigo.key(*modifier, Press).map_err(|e| ActionError::ExecutionFailed(e.to_string()))?;
                                        Self::action_delay().await;
                                    }

                                    // Press the main key
                                    enigo.key(key_press.key, Press).map_err(|e| ActionError::ExecutionFailed(e.to_string()))?;
                                    Self::action_delay().await;

                                    // Release modifiers in reverse order
                                    for modifier in key_press.modifiers.iter().rev() {
                                        enigo.key(*modifier, Release).map_err(|e| ActionError::ExecutionFailed(e.to_string()))?;
                                        Self::action_delay().await;
                                    }

                                    // Release the main key
                                    enigo.key(key_press.key, Release).map_err(|e| ActionError::ExecutionFailed(e.to_string()))?;
                                    Self::action_delay().await;
                                    Ok(())
                                }.await;
                                result.map(|()| ActionResult { data: None })
                            } else {
                                Err(ActionError::InvalidInput("Invalid key format".into()))
                            }
                        },
                        Action::CursorPosition => {
                            match enigo.location() {
                                Ok((x, y)) => Ok(ActionResult {
                                    data: Some(serde_json::json!({ "x": x, "y": y }))
                                }),
                                Err(e) => Err(ActionError::ExecutionFailed(e.to_string()))
                            }
                        },
                        Action::Screenshot => {
                            // Screenshot delay is slightly longer
                            sleep(SCREENSHOT_DELAY).await;

                            Monitor::all()
                                .map_err(|_| ActionError::ExecutionFailed("Failed to get monitors".to_string()))
                                .and_then(|monitors| {
                                    monitors.first()
                                        .cloned()
                                        .ok_or_else(|| ActionError::ExecutionFailed("No monitor found".to_string()))
                                })
                                .and_then(|monitor| {
                                    monitor.capture_image()
                                        .map_err(|_| ActionError::ExecutionFailed("Failed to capture image".to_string()))
                                })
                                .and_then(|image| {
                                    let mut cursor = Cursor::new(Vec::new());
                                    image.write_to(&mut cursor, ImageFormat::Png)
                                        .map_err(|_| ActionError::ExecutionFailed("Failed to encode image".to_string()))?;
                                    let bytes = cursor.into_inner();
                                    let base64_image = BASE64.encode(bytes);
                                    Ok(ActionResult {
                                        data: Some(serde_json::json!({ "image": base64_image }))
                                    })
                                })
                        },
                    };

                    // Notify completion with result
                    let _ = tx.send(result);
                }

                sleep(Duration::from_millis(10)).await;
            }
        });
    }
}
