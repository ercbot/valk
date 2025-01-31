use std::str::FromStr;

use axum::{
    http::StatusCode,
    routing::{get, post}, Router, extract,
    response::{Json, IntoResponse}
};
use xcap::Monitor;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use image::ImageFormat;
use serde_json::{json, Value}; 
use serde::{Serialize, Deserialize};
use tokio::time::{sleep, Duration};
use enigo::{
    Button, Coordinate::Abs, Direction::{Press, Release}, Enigo, Keyboard, Mouse, Settings
};

mod key_press;

use key_press::KeyPress;

const SCREENSHOT_DELAY: Duration = Duration::from_secs(2);
const ACTION_DELAY: Duration = Duration::from_millis(200);

async fn action_delay() {
    sleep(ACTION_DELAY).await;
}

async fn root() -> &'static str {
    "Subspace is running"
}

/// Take a screenshot of the screen.
async fn screenshot() -> Json<Value> {
    // Delay to let things settle before taking a screenshot
    sleep(SCREENSHOT_DELAY).await;

    let monitors = Monitor::all().unwrap();

    // Only get the first monitor
    let monitor = monitors.first().unwrap();

    let image = monitor.capture_image().unwrap();
        
    // Convert image to base64
    let mut cursor = std::io::Cursor::new(Vec::new());
    image.write_to(&mut cursor, ImageFormat::Png).unwrap();
    let bytes = cursor.into_inner();
    let base64_image = BASE64.encode(bytes);

    // Create JSON object with "image" field
    let response_json = Json(json!({
        "image": base64_image
    }));
    
    response_json
}

// Request body structure
#[derive(Debug, Serialize, Deserialize)]
struct ClickRequest {
    x: u32,
    y: u32
}

// Custom error type for our API
#[derive(Debug)]
pub enum ApiError {
    ComputerInputError(String),
}

// Implement response conversion for our error type
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (_status, message) = match self {
            ApiError::ComputerInputError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        Json(json!({
            "status": "error",
            "message": message
        })).into_response()
    }
}

/// Click the left mouse button.
async fn left_click() -> Result<Json<Value>, ApiError> {
    // Initialize Enigo
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    // Perform a left click
    action_delay().await;
    enigo.button(Button::Left, Press).unwrap();
    action_delay().await;
    enigo.button(Button::Left, Release).unwrap();

    Ok(Json(json!({
        "status": "success"
    })))
}

/// Click the right mouse button.
async fn right_click() -> Result<Json<Value>, ApiError> {
    // Initialize Enigo
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    
    // Perform right click
    action_delay().await;
    enigo.button(Button::Right, Press).unwrap();
    action_delay().await;
    enigo.button(Button::Right, Release).unwrap();
    
    Ok(Json(json!({
        "status": "success"
    })))
}

/// Click the middle mouse button.
async fn middle_click() -> Result<Json<Value>, ApiError> {
    // Initialize Enigo
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    // Perform middle click
    action_delay().await;
    enigo.button(Button::Middle, Press).unwrap();
    action_delay().await;
    enigo.button(Button::Middle, Release).unwrap();

    Ok(Json(json!({
        "status": "success"
    })))
}

/// Double-click the left mouse button.
async fn double_click() -> Result<Json<Value>, ApiError> {
    // Initialize Enigo
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    // Perform first left click
    action_delay().await;
    enigo.button(Button::Left, Press).unwrap();
    action_delay().await;
    enigo.button(Button::Left, Release).unwrap();

    // Perform second left click
    action_delay().await;
    enigo.button(Button::Left, Press).unwrap();
    action_delay().await;
    enigo.button(Button::Left, Release).unwrap();

    Ok(Json(json!({
        "status": "success"
    })))
}

/// Get the current (x, y) pixel coordinate of the cursor on the screen.
async fn cursor_position() -> Json<Value> {
    let enigo = Enigo::new(&Settings::default()).unwrap();
    let position = enigo.location().unwrap();
    
    Json(json!({
        "x": position.0,
        "y": position.1
    }))
}

/// Move the cursor to a specified (x, y) pixel coordinate on the screen.
async fn mouse_move(
    extract::Json(click_request): extract::Json<ClickRequest>,
) -> Result<Json<Value>, ApiError> {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    action_delay().await;

    enigo.move_mouse(click_request.x as i32, click_request.y as i32, Abs).unwrap();

    Ok(Json(json!({
        "status": "success"
    })))
}

/// Click and drag the cursor to a specified (x, y) pixel coordinate on the screen.
async fn left_click_drag(
    extract::Json(click_request): extract::Json<ClickRequest>,
) -> Result<Json<Value>, ApiError> {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    // Click and hold
    action_delay().await;
    enigo.button(Button::Left, Press).unwrap();

    // Move the cursor to the specified position
    action_delay().await;
    enigo.move_mouse(click_request.x as i32, click_request.y as i32, Abs).unwrap();

    // Release the mouse button
    action_delay().await;
    enigo.button(Button::Left, Release).unwrap();

    Ok(Json(json!({
        "status": "success"
    })))
}

/// Type a string of text on the keyboard.
async fn type_text(
    extract::Json(text): extract::Json<String>,
) -> Result<Json<Value>, ApiError> {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    enigo.text(&text).unwrap();
    Ok(Json(json!({
        "status": "success",
        "text": text
    })))
}

/// Press a key or key-combination on the keyboard.
/// - This supports xdotool's `key` syntax.
/// - Examples: "a", "Return", "alt+Tab", "ctrl+s", "Up", "KP_0" (for the numpad 0 key).
async fn key(
    extract::Json(text): extract::Json<String>,
) -> Result<Json<Value>, ApiError> {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    
    let key_press = KeyPress::from_str(&text)
        .map_err(|e| ApiError::ComputerInputError(e))?;
    
    // Press modifiers
    for modifier in &key_press.modifiers {
        enigo.key(*modifier, Press).unwrap();
        action_delay().await;
    }
    
    // Press the main key
    enigo.key(key_press.key, Press).unwrap();
    action_delay().await;

    // Release the main key
    enigo.key(key_press.key, Release).unwrap();
    action_delay().await;
    // Release modifiers in reverse order
    for modifier in key_press.modifiers.iter().rev() {
        enigo.key(*modifier, Release).unwrap();
        action_delay().await;
    }

    Ok(Json(json!({
        "status": "success",
        "key": text
    })))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        // REST API
        .route("/", get(root))
        .route("/v1/actions/screenshot", get(screenshot))
        .route("/v1/actions/left_click", post(left_click))
        .route("/v1/actions/right_click", post(right_click))
        .route("/v1/actions/middle_click", post(middle_click))
        .route("/v1/actions/double_click", post(double_click))
        .route("/v1/actions/cursor_position", get(cursor_position))
        .route("/v1/actions/mouse_move", post(mouse_move))
        .route("/v1/actions/left_click_drag", post(left_click_drag))
        .route("/v1/actions/type", post(type_text))
        .route("/v1/actions/key", post(key));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}