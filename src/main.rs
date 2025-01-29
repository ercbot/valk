use axum::{
    http::StatusCode,
    routing::{get, post}, Router, response::Json, extract
};
use rdev::{simulate, EventType, Button};
use xcap::Monitor;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use image::ImageFormat;
use serde_json::{json, Value}; 
use serde::{Serialize, Deserialize};

async fn root() -> &'static str {
    "Subspace is running"
}

async fn screenshot() -> Json<Value> {
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

async fn left_click(
    extract::Json(click_request): extract::Json<ClickRequest>
) -> Result<Json<Value>, (StatusCode, String)> {
    simulate(&EventType::MouseMove { x: click_request.x as f64, y: click_request.y as f64 })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    simulate(&EventType::ButtonPress(Button::Left))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    simulate(&EventType::ButtonRelease(Button::Left))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
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
        .route("/v1/actions/left_click", post(left_click));
        

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}