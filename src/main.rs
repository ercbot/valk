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
    Enigo,
    Settings,
    Mouse,
    Button,
    Direction::{Press, Release},
    Coordinate::Abs
};

const SCREENSHOT_DELAY: Duration = Duration::from_secs(2);
const ACTION_DELAY: Duration = Duration::from_millis(200);

async fn action_delay() {
    sleep(ACTION_DELAY).await;
}

async fn root() -> &'static str {
    "Subspace is running"
}

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

async fn left_click(
    extract::Json(click_request): extract::Json<ClickRequest>,
) -> Result<Json<Value>, ApiError> {
    // Initialize Enigo
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    action_delay().await;

    // Move the cursor to the specified position
    enigo.move_mouse(click_request.x as i32, click_request.y as i32, Abs).unwrap();

    action_delay().await;

    // Perform a left click
    enigo.button(Button::Left, Press).unwrap();

    action_delay().await;

    enigo.button(Button::Left, Release).unwrap();

    Ok(Json(json!({
        "status": "success"
    })))
}

async fn right_click(
    extract::Json(click_request): extract::Json<ClickRequest>,
) -> Result<Json<Value>, ApiError> {
    // Initialize Enigo
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    action_delay().await;

    enigo.move_mouse(click_request.x as i32, click_request.y as i32, Abs).unwrap();

    action_delay().await;

    enigo.button(Button::Right, Press).unwrap();

    action_delay().await;

    enigo.button(Button::Right, Release).unwrap();
    
    Ok(Json(json!({
        "status": "success"
    })))
}

async fn middle_click(
    extract::Json(click_request): extract::Json<ClickRequest>,
) -> Result<Json<Value>, ApiError> {
    // Initialize Enigo
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    action_delay().await;

    enigo.move_mouse(click_request.x as i32, click_request.y as i32, Abs).unwrap();

    action_delay().await;

    enigo.button(Button::Middle, Press).unwrap();

    action_delay().await;

    enigo.button(Button::Middle, Release).unwrap();

    Ok(Json(json!({
        "status": "success"
    })))
}

async fn double_click(
    extract::Json(click_request): extract::Json<ClickRequest>,
) -> Result<Json<Value>, ApiError> {
    // Initialize Enigo
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    // Move the cursor to the specified position
    enigo.move_mouse(click_request.x as i32, click_request.y as i32, Abs).unwrap();

    enigo.button(Button::Left, Press).unwrap();

    action_delay().await;

    enigo.button(Button::Left, Release).unwrap();

    // Perform second left click
    enigo.button(Button::Left, Press).unwrap();

    action_delay().await;

    enigo.button(Button::Left, Release).unwrap();

    Ok(Json(json!({
        "status": "success"
    })))
}

async fn cursor_position() -> Json<Value> {
    let enigo = Enigo::new(&Settings::default()).unwrap();
    let position = enigo.location().unwrap();
    
    Json(json!({
        "x": position.0,
        "y": position.1
    }))
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
        .route("/v1/actions/cursor_position", get(cursor_position));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}