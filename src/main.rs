use axum::{
    response::Json, routing::get, Router
};

use xcap::Monitor;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use image::ImageFormat;
use serde_json::{json, Value}; 

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

async fn root() -> &'static str {
    "Subspace is running"
}


#[tokio::main]
async fn main() {
    let app = Router::new()
        // REST API
        .route("/v1/screenshot", get(screenshot))
        .route("/", get(root));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}