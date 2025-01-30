use axum::{
    http::StatusCode,
    routing::{get, post}, Router, extract,
    response::{Json, IntoResponse}
};
use rdev::{simulate, EventType, Button};
use xcap::Monitor;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use image::ImageFormat;
use serde_json::{json, Value}; 
use serde::{Serialize, Deserialize};
use tokio::time::{sleep, Duration};

const SCREENSHOT_DELAY: Duration = Duration::from_secs(2);

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

// Convert rdev::SimulateError into our ApiError
impl From<rdev::SimulateError> for ApiError {
    fn from(err: rdev::SimulateError) -> Self {
        ApiError::ComputerInputError(err.to_string())
    }
}

async fn left_click(
    extract::Json(click_request): extract::Json<ClickRequest>,
) -> Result<Json<Value>, ApiError> {
    simulate(&EventType::MouseMove { 
        x: click_request.x as f64, 
        y: click_request.y as f64 
    })?;
    simulate(&EventType::ButtonPress(Button::Left))?;
    simulate(&EventType::ButtonRelease(Button::Left))?;

    Ok(Json(json!({
        "status": "success"
    })))
}

async fn right_click(
    extract::Json(click_request): extract::Json<ClickRequest>,
) -> Result<Json<Value>, ApiError> {
    simulate(&EventType::MouseMove { 
        x: click_request.x as f64, 
        y: click_request.y as f64 
    })?;
    simulate(&EventType::ButtonPress(Button::Right))?;
    simulate(&EventType::ButtonRelease(Button::Right))?;

    Ok(Json(json!({
        "status": "success"
    })))
}

async fn middle_click(
    extract::Json(click_request): extract::Json<ClickRequest>,
) -> Result<Json<Value>, ApiError> {
    simulate(&EventType::MouseMove { 
        x: click_request.x as f64, 
        y: click_request.y as f64 
    })?;
    simulate(&EventType::ButtonPress(Button::Middle))?;
    simulate(&EventType::ButtonRelease(Button::Middle))?;

    Ok(Json(json!({
        "status": "success"
    })))
}

async fn double_click(
    extract::Json(click_request): extract::Json<ClickRequest>,
) -> Result<Json<Value>, ApiError> {
    simulate(&EventType::MouseMove { x: click_request.x as f64, y: click_request.y as f64 })?;
    simulate(&EventType::ButtonPress(Button::Left))?;
    simulate(&EventType::ButtonRelease(Button::Left))?;
    simulate(&EventType::ButtonPress(Button::Left))?;
    simulate(&EventType::ButtonRelease(Button::Left))?;

    Ok(Json(json!({
        "status": "success"
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
        .route("/v1/actions/double_click", post(double_click));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}