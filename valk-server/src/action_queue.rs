use crate::key_press::KeyPress;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use enigo::InputError;
use enigo::{
    Button,
    Coordinate::{Abs, Rel},
    Direction::{Press, Release},
    Enigo, Keyboard, Mouse, Settings,
};
use image::ImageFormat;
use serde::Serialize;
use std::env;
use std::io::Cursor;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{broadcast, oneshot, Mutex};
use tokio::time::{sleep, timeout, Duration};
use xcap::Monitor;

use crate::action_types::*;

const ACTION_DELAY: Duration = Duration::from_millis(500);
const ACTION_TIMEOUT: Duration = Duration::from_secs(10);
const SCREENSHOT_DELAY: Duration = Duration::from_secs(2);
const DOUBLE_CLICK_DELAY: Duration = Duration::from_millis(100);

pub trait InputDriver: Mouse + Keyboard + Send + 'static {}
impl<T: Mouse + Keyboard + Send + 'static> InputDriver for T {}

pub async fn create_action_queue() -> Arc<ActionQueue<Enigo>> {
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

// Define type aliases for the complex parts
type ActionSender = oneshot::Sender<Result<ActionOutput, ActionError>>;
type QueueItem = (Action, ActionSender);

#[derive(Clone, Serialize)]
pub enum MonitorEvent {
    Request(ActionRequest),
    Response(ActionResponse),
}

/// The GenericActionQueue for testing
/// Can be used to test with a mock input driver
/// Or a real input driver as ActionQueue
#[derive(Clone)]
pub struct ActionQueue<T: InputDriver> {
    queue: Arc<Mutex<Vec<QueueItem>>>,
    input_driver: Arc<Mutex<T>>,
    monitor_tx: broadcast::Sender<MonitorEvent>,
}

// Implementation stays on the generic type
impl<T: InputDriver> ActionQueue<T> {
    pub fn new(input_driver: T) -> Self {
        let (monitor_tx, _) = broadcast::channel(100);
        ActionQueue {
            queue: Arc::new(Mutex::new(Vec::new())),
            input_driver: Arc::new(Mutex::new(input_driver)),
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
    ) -> oneshot::Receiver<Result<ActionOutput, ActionError>> {
        let (tx, rx) = oneshot::channel();
        let mut queue = self.queue.lock().await;
        queue.push((action, tx));
        rx
    }

    pub async fn execute_action(&self, request: ActionRequest) -> ActionResponse {
        // Send request event
        let _ = self.monitor_tx.send(MonitorEvent::Request(request.clone()));

        let rx = self.queue_action(request.action.clone()).await;

        let response = match timeout(ACTION_TIMEOUT, rx).await {
            Ok(result) => match result {
                Ok(Ok(output)) => ActionResponse::success(request.id, request.action, output),
                Ok(Err(error)) => ActionResponse::error(request.id, request.action, error),
                Err(e) => ActionResponse::error(
                    request.id,
                    request.action,
                    ActionError::ChannelError(e.to_string()),
                ),
            },
            Err(_) => {
                // Timeout occurred - remove action from queue if it's still there
                let mut queue = self.queue.lock().await;
                queue.retain(|(a, _)| {
                    !std::mem::discriminant(a).eq(&std::mem::discriminant(&request.action))
                });
                ActionResponse::error(request.id, request.action, ActionError::Timeout)
            }
        };

        // Send response event
        let _ = self
            .monitor_tx
            .send(MonitorEvent::Response(response.clone()));

        response
    }

    async fn action_delay() {
        sleep(ACTION_DELAY).await;
    }

    async fn handle_action(
        input_driver: &mut T,
        action: &Action,
    ) -> Result<ActionOutput, ActionError> {
        match action {
            Action::LeftClick => {
                let press_result = input_driver.button(Button::Left, Press);
                let release_result = if press_result.is_ok() {
                    Self::action_delay().await;
                    input_driver.button(Button::Left, Release)
                } else {
                    press_result
                };

                release_result
                    .map(|_| ActionOutput::NoData)
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
                    .map(|_| ActionOutput::NoData)
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
                    .map(|_| ActionOutput::NoData)
                    .map_err(|e| ActionError::ExecutionFailed(e.to_string()))
            }
            Action::DoubleClick => {
                // First click
                let first_click = matches!(
                    (
                        input_driver.button(Button::Left, Press),
                        sleep(DOUBLE_CLICK_DELAY).await,
                        input_driver.button(Button::Left, Release),
                    ),
                    (Ok(_), _, Ok(_))
                );

                sleep(DOUBLE_CLICK_DELAY).await;

                if first_click {
                    // Second click
                    match (
                        input_driver.button(Button::Left, Press),
                        sleep(DOUBLE_CLICK_DELAY).await,
                        input_driver.button(Button::Left, Release),
                    ) {
                        (Ok(_), _, Ok(_)) => Ok(ActionOutput::NoData),
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
            Action::MouseMove { input } => input_driver
                .move_mouse(input.x as i32, input.y as i32, Abs)
                .map(|_| ActionOutput::NoData)
                .map_err(|e| ActionError::ExecutionFailed(e.to_string())),
            Action::LeftClickDrag { input } => {
                // First press and hold the left button
                if let Err(e) = input_driver.button(Button::Left, Press) {
                    return Err(ActionError::ExecutionFailed(e.to_string()))
                        as Result<ActionOutput, ActionError>;
                }

                sleep(DOUBLE_CLICK_DELAY).await;

                // We need to use interpolation to drag the mouse
                let current_pos = input_driver.location().unwrap();
                let target_pos = (input.x as i32, input.y as i32);

                let distance = (((current_pos.0 - target_pos.0).pow(2)
                    + (current_pos.1 - target_pos.1).pow(2))
                    as f64)
                    .sqrt();
                let steps = (distance / 10.0).max(1.0); // One step for every 10 euclidan px traveled
                let step_x = (target_pos.0 - current_pos.0) as f64 / steps;
                let step_y = (target_pos.1 - current_pos.1) as f64 / steps;

                // Do relative moves for all but last step
                for i in 0..steps as u32 {
                    if i == steps as u32 - 1 {
                        // Last step - do absolute move to target
                        match input_driver.move_mouse(target_pos.0, target_pos.1, Abs) {
                            Ok(_) => (),
                            Err(e) => {
                                // Cleanup: release button if move fails
                                let _ = input_driver.button(Button::Left, Release);
                                return Err(ActionError::ExecutionFailed(e.to_string()));
                            }
                        };
                    } else {
                        // Regular relative move for intermediate steps
                        match input_driver.move_mouse(step_x as i32, step_y as i32, Rel) {
                            Ok(_) => (),
                            Err(e) => {
                                // Cleanup: release button if move fails
                                let _ = input_driver.button(Button::Left, Release);
                                return Err(ActionError::ExecutionFailed(e.to_string()));
                            }
                        };
                        sleep(Duration::from_millis(10)).await;
                    }
                }

                sleep(DOUBLE_CLICK_DELAY).await;

                // Release button
                match input_driver.button(Button::Left, Release) {
                    Ok(_) => Ok(ActionOutput::NoData),
                    Err(e) => Err(ActionError::ExecutionFailed(e.to_string())),
                }
            }
            Action::TypeText { input } => {
                // First check if text is empty
                if input.text.is_empty() {
                    return Err(ActionError::InvalidInput(
                        "Text cannot be empty".to_string(),
                    ));
                }

                // Attempt to type the text with detailed error handling
                match input_driver.text(&input.text) {
                    Ok(_) => Ok(ActionOutput::NoData),
                    Err(e) => {
                        // Log the specific type of InputError
                        match e {
                            InputError::Simulate(msg) => {
                                eprintln!("Simulation error: {}", msg);
                                let non_ascii_chars: Vec<char> =
                                    input.text.chars().filter(|c| !c.is_ascii()).collect();
                                let has_non_ascii = !non_ascii_chars.is_empty();

                                if has_non_ascii {
                                    Err(ActionError::ExecutionFailed(format!(
                                        "Input simulation failed. This might be because the text contains non-ASCII characters ({:?}) which may not be supported by your system. Original error: {}",
                                        non_ascii_chars, msg
                                    )))
                                } else {
                                    Err(ActionError::ExecutionFailed(format!(
                                        "Input simulation failed: {}",
                                        msg
                                    )))
                                }
                            }
                            _ => Err(ActionError::ExecutionFailed(e.to_string())),
                        }
                    }
                }
            }
            Action::KeyPress { input } => {
                if let Ok(key_press) = KeyPress::from_str(&input.key) {
                    let result: Result<(), ActionError> = async {
                        // Press modifiers
                        for modifier in &key_press.modifiers {
                            input_driver
                                .key(*modifier, Press)
                                .map_err(|e| ActionError::ExecutionFailed(e.to_string()))?;
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
                            input_driver
                                .key(*modifier, Release)
                                .map_err(|e| ActionError::ExecutionFailed(e.to_string()))?;
                            Self::action_delay().await;
                        }

                        Ok(())
                    }
                    .await;
                    result.map(|_| ActionOutput::NoData)
                } else {
                    Err(ActionError::InvalidInput(format!(
                        "Invalid key format or key not found: {}",
                        input.key
                    )))
                }
            }
            Action::CursorPosition => match input_driver.location() {
                Ok((x, y)) => Ok(ActionOutput::CursorPosition {
                    x: x as u32,
                    y: y as u32,
                }),
                Err(e) => Err(ActionError::ExecutionFailed(e.to_string())),
            },
            Action::Screenshot => {
                // Screenshot delay is slightly longer
                sleep(SCREENSHOT_DELAY).await;

                Monitor::all()
                    .map_err(|_| ActionError::ExecutionFailed("Failed to get monitors".to_string()))
                    .and_then(|monitors| {
                        monitors.first().cloned().ok_or_else(|| {
                            ActionError::ExecutionFailed("No monitor found".to_string())
                        })
                    })
                    .and_then(|monitor| {
                        monitor.capture_image().map_err(|_| {
                            ActionError::ExecutionFailed("Failed to capture image".to_string())
                        })
                    })
                    .and_then(|image| {
                        let mut cursor = Cursor::new(Vec::new());
                        image.write_to(&mut cursor, ImageFormat::Png).map_err(|_| {
                            ActionError::ExecutionFailed("Failed to encode image".to_string())
                        })?;
                        let bytes = cursor.into_inner();
                        let base64_image = BASE64.encode(bytes);
                        Ok(ActionOutput::Screenshot {
                            image: base64_image,
                        })
                    })
            }
        }
    }

    pub async fn start_processing(&self) {
        let queue_clone = self.queue.clone();
        let input_driver_clone = self.input_driver.clone();

        tokio::spawn(async move {
            loop {
                let action = {
                    let mut queue = queue_clone.lock().await;
                    queue.pop()
                };

                if let Some((action, tx)) = action {
                    let mut input_driver = input_driver_clone.lock().await;
                    Self::action_delay().await;

                    let result = Self::handle_action(&mut input_driver, &action).await;

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
pub mod tests {
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

    // Make the helper function public
    pub async fn create_test_action_queue() -> Arc<ActionQueue<MockEnigo>> {
        let mock_enigo = MockEnigo::new();
        let action_queue = ActionQueue::new(mock_enigo);
        let action_queue = Arc::new(action_queue);
        action_queue.start_processing().await;
        action_queue
    }

    #[tokio::test]
    async fn test_mouse_move() {
        let queue = create_test_action_queue().await;

        let result = queue
            .execute_action(ActionRequest {
                id: "test_mouse_move".to_string(),
                action: Action::MouseMove {
                    input: MouseMoveInput { x: 100, y: 200 },
                },
            })
            .await;
        assert!(matches!(result.status, ActionResponseStatus::Success));

        let enigo = queue.input_driver.lock().await;
        assert_eq!(enigo.mouse_pos, (100, 200));
        assert_eq!(enigo.last_action, "move_mouse_100,200");
    }

    #[tokio::test]
    async fn test_left_click() {
        let queue = create_test_action_queue().await;

        let result = queue
            .execute_action(ActionRequest {
                id: "test_left_click".to_string(),
                action: Action::LeftClick,
            })
            .await;
        assert!(matches!(result.status, ActionResponseStatus::Success));

        let enigo = queue.input_driver.lock().await;
        assert!(enigo.last_action.contains("button_Left"));
    }

    #[tokio::test]
    async fn test_type_text() {
        let queue = create_test_action_queue().await;

        let test_texts = ["Hello, World!", "1234567890", "Special chars: !@#$%^&*()"];

        for text in test_texts {
            let response = queue
                .execute_action(ActionRequest {
                    id: "test_type_text".to_string(),
                    action: Action::TypeText {
                        input: TypeTextInput {
                            text: text.to_string(),
                        },
                    },
                })
                .await;

            match response.status {
                ActionResponseStatus::Success => {
                    let enigo = queue.input_driver.lock().await;
                    assert_eq!(
                        enigo.last_action,
                        format!("text_{}", text),
                        "Failed to verify text input for: {}",
                        text
                    );
                }
                ActionResponseStatus::Error => {
                    panic!("Failed to type text '{}': {:?}", text, response.error);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_type_unicode() {
        let queue = create_test_action_queue().await;

        let test_texts = [
            "Unicode: ñáéíóú",
            "Emojis: 😊🚀🌟",
            "Cyrillic: Привет, мир!",
            "Japanese: こんにちは世界",
            "Korean: 안녕하세요 세계",
            "Arabic: مرحبا بالعالم",
            "Hebrew: שלום עולם",
            "Greek: Γειά σου κόσμε",
            "Turkish: Merhaba dünya",
            "Vietnamese: Chào thế giới",
            "Thai: สวัสดีโลก",
            "Russian: Привет, мир!",
            "Chinese: 你好，世界",
        ];

        for text in test_texts {
            let response = queue
                .execute_action(ActionRequest {
                    id: "test_type_unicode".to_string(),
                    action: Action::TypeText {
                        input: TypeTextInput {
                            text: text.to_string(),
                        },
                    },
                })
                .await;

            match response.status {
                ActionResponseStatus::Success => {
                    let enigo = queue.input_driver.lock().await;
                    assert_eq!(
                        enigo.last_action,
                        format!("text_{}", text),
                        "Failed to verify text input for: {}",
                        text
                    );
                }
                ActionResponseStatus::Error => {
                    panic!("Failed to type text '{}': {:?}", text, response.error);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_type_text_empty() {
        let queue = create_test_action_queue().await;

        let response = queue
            .execute_action(ActionRequest {
                id: "test_type_text_empty".to_string(),
                action: Action::TypeText {
                    input: TypeTextInput {
                        text: "".to_string(),
                    },
                },
            })
            .await;
        assert!(matches!(response.status, ActionResponseStatus::Error));
    }

    #[tokio::test]
    async fn test_key_press() {
        let queue = create_test_action_queue().await;

        let response = queue
            .execute_action(ActionRequest {
                id: "test_key_press".to_string(),
                action: Action::KeyPress {
                    input: KeyPressInput {
                        key: "ctrl+c".to_string(),
                    },
                },
            })
            .await;
        assert!(matches!(response.status, ActionResponseStatus::Success));

        let enigo = queue.input_driver.lock().await;
        // The last action should be releasing the ctrl key
        assert!(enigo.last_action.contains("key_Control_Release"));
    }

    #[tokio::test]
    async fn test_cursor_position() {
        let queue = create_test_action_queue().await;

        // First move the cursor
        let _ = queue
            .execute_action(ActionRequest {
                id: "test_cursor_position".to_string(),
                action: Action::MouseMove {
                    input: MouseMoveInput { x: 150, y: 250 },
                },
            })
            .await;

        // Then get position
        let response = queue
            .execute_action(ActionRequest {
                id: "test_cursor_position".to_string(),
                action: Action::CursorPosition,
            })
            .await;
        assert!(matches!(response.status, ActionResponseStatus::Success));

        if let Some(ActionOutput::CursorPosition { x, y }) = response.data {
            assert_eq!(x, 150);
            assert_eq!(y, 250);
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
        let result = timeout(
            short_timeout,
            queue.execute_action(ActionRequest {
                id: "test_action_timeout".to_string(),
                action: Action::LeftClick,
            }),
        )
        .await;

        // The timeout should occur
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_double_click() {
        let queue = create_test_action_queue().await;

        let response = queue
            .execute_action(ActionRequest {
                id: "test_double_click".to_string(),
                action: Action::DoubleClick,
            })
            .await;
        assert!(matches!(response.status, ActionResponseStatus::Success));

        let enigo = queue.input_driver.lock().await;
        // Should end with a release of left button
        assert!(enigo.last_action.contains("button_Left_Release"));
    }

    #[tokio::test]
    async fn test_left_click_drag() {
        let queue = create_test_action_queue().await;

        let response = queue
            .execute_action(ActionRequest {
                id: "test_left_click_drag".to_string(),
                action: Action::LeftClickDrag {
                    input: MouseMoveInput { x: 300, y: 400 },
                },
            })
            .await;
        assert!(matches!(response.status, ActionResponseStatus::Success));

        let enigo = queue.input_driver.lock().await;
        assert_eq!(enigo.mouse_pos, (300, 400));
        // Should end with a release
        assert!(enigo.last_action.contains("button_Left_Release"));
    }
}
