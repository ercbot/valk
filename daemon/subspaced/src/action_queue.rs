use crate::key_press::KeyPress;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use enigo::{
    Button,
    Coordinate::Abs,
    Direction::{Press, Release},
    Enigo, Keyboard, Mouse, Settings,
};
use image::ImageFormat;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::io::Cursor;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, oneshot, Mutex};
use tokio::time::{sleep, timeout, Duration};
use xcap::Monitor;

const ACTION_DELAY: Duration = Duration::from_millis(500);
const ACTION_TIMEOUT: Duration = Duration::from_secs(10);
const SCREENSHOT_DELAY: Duration = Duration::from_secs(2);
const DOUBLE_CLICK_DELAY: Duration = Duration::from_millis(100);

/// Represents possible errors that can occur during action execution
#[derive(Debug, Clone, Serialize)]
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
        }))
        .into_response()
    }
}

pub async fn create_action_queue() -> Arc<ActionQueue> {
    let settings = Settings {
        x11_display: Some(env::var("DISPLAY").unwrap()),
        ..Settings::default()
    };
    let enigo = Enigo::new(&settings).unwrap();
    let queue = ActionQueue::new(enigo);
    let queue = Arc::new(queue);
    queue.start_processing().await;
    queue
}

/// The GenericActionQueue for testing
/// Can be used to test with a mock input driver
/// Or a real input driver as ActionQueue
#[derive(Clone)]
pub struct GenericActionQueue<T: Mouse + Keyboard + Send + 'static> {
    queue: Arc<Mutex<Vec<(Action, oneshot::Sender<Result<ActionResult, ActionError>>)>>>,
    input_driver: Arc<Mutex<T>>,
    monitor_tx: broadcast::Sender<MonitorEvent>,
}

// Type alias for the "real" production ActionQueue
pub type ActionQueue = GenericActionQueue<Enigo>;

#[derive(Clone, Debug, Serialize)]
pub struct MonitorEvent {
    pub timestamp: u64,
    pub action: Action,
    pub result: Result<ActionResult, ActionError>,
}

// Implementation stays on the generic type
impl<T: Mouse + Keyboard + Send + 'static> GenericActionQueue<T> {
    pub fn new(enigo: T) -> Self {
        let (monitor_tx, _) = broadcast::channel(100); // Buffer size of 100 event

        GenericActionQueue {
            queue: Arc::new(Mutex::new(Vec::new())),
            input_driver: Arc::new(Mutex::new(enigo)),
            monitor_tx,
        }
    }

    pub fn subscribe_monitor(&self) -> broadcast::Receiver<MonitorEvent> {
        self.monitor_tx.subscribe()
    }

    // Add an action to the queue
    async fn queue_action(
        &self,
        action: Action,
    ) -> oneshot::Receiver<Result<ActionResult, ActionError>> {
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
                    .map_err(|e| ActionError::ChannelError(e.to_string())) // Channel errors
                    .and_then(|r| Ok(r?))
            }
            Err(_) => {
                // Timeout occurred - remove action from queue if it's still there
                let mut queue = self.queue.lock().await;
                queue.retain(|(a, _)| {
                    !std::mem::discriminant(a).eq(&std::mem::discriminant(&action))
                });
                Err(ActionError::Timeout)
            }
        }
    }

    async fn action_delay() {
        sleep(ACTION_DELAY).await;
    }

    pub async fn start_processing(&self) {
        let queue_clone = self.queue.clone();
        let input_driver_clone = self.input_driver.clone();
        let monitor_tx = self.monitor_tx.clone();

        tokio::spawn(async move {
            loop {
                let action = {
                    let mut queue = queue_clone.lock().await;
                    queue.pop()
                };

                if let Some((action, tx)) = action {
                    let mut input_driver = input_driver_clone.lock().await;
                    Self::action_delay().await;

                    let result = match &action {
                        Action::LeftClick => {
                            let press_result = input_driver.button(Button::Left, Press);
                            let release_result = if press_result.is_ok() {
                                Self::action_delay().await;
                                input_driver.button(Button::Left, Release)
                            } else {
                                press_result
                            };

                            release_result
                                .map(|_| ActionResult { data: None })
                                .map_err(|e| ActionError::ExecutionFailed(e.to_string()))
                        }

                        Action::RightClick => {
                            let press_result = input_driver.button(Button::Right, Press);
                            let release_result = if press_result.is_ok() {
                                Self::action_delay().await;
                                input_driver.button(Button::Right, Release)
                            } else {
                                press_result
                            };

                            release_result
                                .map(|_| ActionResult { data: None })
                                .map_err(|e| ActionError::ExecutionFailed(e.to_string()))
                        }

                        Action::MiddleClick => {
                            let press_result = input_driver.button(Button::Middle, Press);
                            let release_result = if press_result.is_ok() {
                                Self::action_delay().await;
                                input_driver.button(Button::Middle, Release)
                            } else {
                                press_result
                            };

                            release_result
                                .map(|_| ActionResult { data: None })
                                .map_err(|e| ActionError::ExecutionFailed(e.to_string()))
                        }
                        Action::DoubleClick => {
                            // First click
                            let first_click = match (
                                input_driver.button(Button::Left, Press),
                                sleep(DOUBLE_CLICK_DELAY).await,
                                input_driver.button(Button::Left, Release),
                            ) {
                                (Ok(_), _, Ok(_)) => true,
                                _ => false,
                            };

                            sleep(DOUBLE_CLICK_DELAY).await;

                            if first_click {
                                // Second click
                                match (
                                    input_driver.button(Button::Left, Press),
                                    sleep(DOUBLE_CLICK_DELAY).await,
                                    input_driver.button(Button::Left, Release),
                                ) {
                                    (Ok(_), _, Ok(_)) => Ok(ActionResult { data: None }),
                                    _ => Err(ActionError::ExecutionFailed(
                                        "Failed to execute second click".into(),
                                    )),
                                }
                            } else {
                                Err(ActionError::ExecutionFailed(
                                    "Failed to execute first click".into(),
                                ))
                            }
                        }

                        Action::MouseMove { x, y } => input_driver
                            .move_mouse(*x as i32, *y as i32, Abs)
                            .map(|_| ActionResult { data: None })
                            .map_err(|e| ActionError::ExecutionFailed(e.to_string())),
                        Action::LeftClickDrag { x, y } => {
                            match (
                                input_driver.button(Button::Left, Press),
                                Self::action_delay().await,
                                input_driver.move_mouse(*x as i32, *y as i32, Abs),
                                Self::action_delay().await,
                                input_driver.button(Button::Left, Release),
                            ) {
                                (Ok(_), _, Ok(_), _, Ok(_)) => Ok(ActionResult { data: None }),
                                _ => Err(ActionError::ExecutionFailed(
                                    "Failed to execute left click drag".into(),
                                )),
                            }
                        }

                        Action::TypeText { text } => input_driver
                            .text(&text)
                            .map(|_| ActionResult { data: None })
                            .map_err(|e| ActionError::ExecutionFailed(e.to_string())),
                        Action::KeyPress { key } => {
                            if let Ok(key_press) = KeyPress::from_str(&key) {
                                let result: Result<(), ActionError> = async {
                                    // Press modifiers
                                    for modifier in &key_press.modifiers {
                                        input_driver.key(*modifier, Press).map_err(|e| {
                                            ActionError::ExecutionFailed(e.to_string())
                                        })?;
                                        Self::action_delay().await;
                                    }

                                    // Press the main key
                                    input_driver
                                        .key(key_press.key, Press)
                                        .map_err(|e| ActionError::ExecutionFailed(e.to_string()))?;
                                    Self::action_delay().await;

                                    // Release the main key
                                    input_driver
                                        .key(key_press.key, Release)
                                        .map_err(|e| ActionError::ExecutionFailed(e.to_string()))?;
                                    Self::action_delay().await;

                                    // Release modifiers in reverse order
                                    for modifier in key_press.modifiers.iter().rev() {
                                        input_driver.key(*modifier, Release).map_err(|e| {
                                            ActionError::ExecutionFailed(e.to_string())
                                        })?;
                                        Self::action_delay().await;
                                    }

                                    Ok(())
                                }
                                .await;
                                result.map(|()| ActionResult { data: None })
                            } else {
                                Err(ActionError::InvalidInput("Invalid key format".into()))
                            }
                        }
                        Action::CursorPosition => match input_driver.location() {
                            Ok((x, y)) => Ok(ActionResult {
                                data: Some(serde_json::json!({ "x": x, "y": y })),
                            }),
                            Err(e) => Err(ActionError::ExecutionFailed(e.to_string())),
                        },
                        Action::Screenshot => {
                            // Screenshot delay is slightly longer
                            sleep(SCREENSHOT_DELAY).await;

                            Monitor::all()
                                .map_err(|_| {
                                    ActionError::ExecutionFailed(
                                        "Failed to get monitors".to_string(),
                                    )
                                })
                                .and_then(|monitors| {
                                    monitors.first().cloned().ok_or_else(|| {
                                        ActionError::ExecutionFailed("No monitor found".to_string())
                                    })
                                })
                                .and_then(|monitor| {
                                    monitor.capture_image().map_err(|_| {
                                        ActionError::ExecutionFailed(
                                            "Failed to capture image".to_string(),
                                        )
                                    })
                                })
                                .and_then(|image| {
                                    let mut cursor = Cursor::new(Vec::new());
                                    image.write_to(&mut cursor, ImageFormat::Png).map_err(
                                        |_| {
                                            ActionError::ExecutionFailed(
                                                "Failed to encode image".to_string(),
                                            )
                                        },
                                    )?;
                                    let bytes = cursor.into_inner();
                                    let base64_image = BASE64.encode(bytes);
                                    Ok(ActionResult {
                                        data: Some(serde_json::json!({ "image": base64_image })),
                                    })
                                })
                        }
                    };

                    // Send monitor event
                    let monitor_event = MonitorEvent {
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                        action: action.clone(),
                        result: result.clone(),
                    };
                    let _ = monitor_tx.send(monitor_event);

                    // Notify completion with result
                    let _ = tx.send(result);
                }

                sleep(Duration::from_millis(10)).await;
            }
        });
    }
}

/// Mock driver for testing
#[cfg(test)]
mod tests {
    use super::*;

    use enigo::{Axis, Coordinate, Direction, InputResult, Key};

    pub struct MockEnigo {
        pub mouse_pos: (i32, i32),
        pub last_action: String,
    }

    impl MockEnigo {
        pub fn new() -> Self {
            MockEnigo {
                mouse_pos: (0, 0),
                last_action: String::new(),
            }
        }
    }

    impl Keyboard for MockEnigo {
        fn key(&mut self, key: Key, direction: Direction) -> InputResult<()> {
            self.last_action = format!("key_{:?}_{:?}", key, direction);
            Ok(())
        }

        fn raw(&mut self, keycode: u16, direction: Direction) -> InputResult<()> {
            self.last_action = format!("raw_key_{:?}_{:?}", keycode, direction);
            Ok(())
        }

        fn text(&mut self, text: &str) -> InputResult<()> {
            self.last_action = format!("text_{}", text);
            Ok(())
        }

        fn fast_text(&mut self, text: &str) -> InputResult<Option<()>> {
            self.last_action = format!("fast_text_{}", text);
            Ok(Some(()))
        }
    }

    impl Mouse for MockEnigo {
        fn button(&mut self, button: Button, direction: Direction) -> InputResult<()> {
            self.last_action = format!("button_{:?}_{:?}", button, direction);
            Ok(())
        }

        fn move_mouse(&mut self, x: i32, y: i32, _coordinate: Coordinate) -> InputResult<()> {
            self.mouse_pos = (x, y);
            self.last_action = format!("move_mouse_{},{}", x, y);
            Ok(())
        }

        fn scroll(&mut self, length: i32, axis: Axis) -> InputResult<()> {
            self.last_action = format!("scroll_{}_{:?}", length, axis);
            Ok(())
        }

        fn main_display(&self) -> InputResult<(i32, i32)> {
            Ok((1920, 1080)) // Mock display size
        }

        fn location(&self) -> InputResult<(i32, i32)> {
            Ok(self.mouse_pos)
        }
    }

    // Helper function to create an action queue with a mock enigo
    async fn create_test_action_queue() -> Arc<GenericActionQueue<MockEnigo>> {
        let mock_enigo = MockEnigo::new();
        let action_queue = GenericActionQueue::new(mock_enigo);
        let action_queue = Arc::new(action_queue);
        action_queue.start_processing().await;
        action_queue
    }

    #[tokio::test]
    async fn test_mouse_move() {
        let queue = create_test_action_queue().await;

        let result = queue
            .execute_action(Action::MouseMove { x: 100, y: 200 })
            .await;
        assert!(result.is_ok());

        let enigo = queue.input_driver.lock().await;
        assert_eq!(enigo.mouse_pos, (100, 200));
        assert_eq!(enigo.last_action, "move_mouse_100,200");
    }

    #[tokio::test]
    async fn test_left_click() {
        let queue = create_test_action_queue().await;

        let result = queue.execute_action(Action::LeftClick).await;
        assert!(result.is_ok());

        let enigo = queue.input_driver.lock().await;
        assert!(enigo.last_action.contains("button_Left"));
    }

    #[tokio::test]
    async fn test_type_text() {
        let queue = create_test_action_queue().await;
        let test_text = "Hello, World!";

        let result = queue
            .execute_action(Action::TypeText {
                text: test_text.to_string(),
            })
            .await;
        assert!(result.is_ok());

        let enigo = queue.input_driver.lock().await;
        assert_eq!(enigo.last_action, format!("text_{}", test_text));
    }

    #[tokio::test]
    async fn test_key_press() {
        let queue = create_test_action_queue().await;

        let result = queue
            .execute_action(Action::KeyPress {
                key: "ctrl+c".to_string(),
            })
            .await;
        assert!(result.is_ok());

        let enigo = queue.input_driver.lock().await;
        // The last action should be releasing the ctrl key
        assert!(enigo.last_action.contains("key_Control_Release"));
    }

    #[tokio::test]
    async fn test_cursor_position() {
        let queue = create_test_action_queue().await;

        // First move the cursor
        let _ = queue
            .execute_action(Action::MouseMove { x: 150, y: 250 })
            .await;

        // Then get position
        let result = queue.execute_action(Action::CursorPosition).await;
        assert!(result.is_ok());

        if let Ok(ActionResult { data: Some(data) }) = result {
            assert_eq!(data["x"], 150);
            assert_eq!(data["y"], 250);
        } else {
            panic!("Expected cursor position data");
        }
    }

    #[tokio::test]
    async fn test_action_timeout() {
        let queue = create_test_action_queue().await;

        // Create a very short timeout for testing
        let short_timeout = Duration::from_millis(10);

        // Attempt to execute an action with a short timeout
        let result = timeout(short_timeout, queue.execute_action(Action::LeftClick)).await;

        // The timeout should occur
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_double_click() {
        let queue = create_test_action_queue().await;

        let result = queue.execute_action(Action::DoubleClick).await;
        assert!(result.is_ok());

        let enigo = queue.input_driver.lock().await;
        // Should end with a release of left button
        assert!(enigo.last_action.contains("button_Left_Release"));
    }

    #[tokio::test]
    async fn test_left_click_drag() {
        let queue = create_test_action_queue().await;

        let result = queue
            .execute_action(Action::LeftClickDrag { x: 300, y: 400 })
            .await;
        assert!(result.is_ok());

        let enigo = queue.input_driver.lock().await;
        assert_eq!(enigo.mouse_pos, (300, 400));
        // Should end with a release
        assert!(enigo.last_action.contains("button_Left_Release"));
    }
}
