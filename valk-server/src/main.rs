use axum::{
    extract::{self, Request},
    http::StatusCode,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::time::Duration;

use std::sync::Arc;

use tower_http::trace::{self, TraceLayer};
use tracing::{info, Level, Span};

mod action_queue;
mod action_types;
mod config;
mod key_press;
mod monitor;

use action_queue::{create_action_queue, SharedQueue};
use action_types::{ActionError, ActionRequest, ActionResponse, ActionResponseStatus};
use config::Config;
use monitor::monitor_websocket;

async fn root() -> &'static str {
    "Valk is running"
}

#[derive(Debug, Serialize)]
struct ComputerInfo {
    os_type: String,
    os_version: String,
    display_width: u32,
    display_height: u32,
}

/// Get information about the computer system
async fn system_info() -> Result<Json<ComputerInfo>, (StatusCode, String)> {
    let monitor = xcap::Monitor::all()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get display info: {}", e),
            )
        })?
        .first()
        .cloned()
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "No monitor found".to_string(),
        ))?;

    let os_info = os_info::get();

    Ok(Json(ComputerInfo {
        os_type: os_info.os_type().to_string(),
        os_version: os_info.version().to_string(),
        display_width: monitor.width(),
        display_height: monitor.height(),
    }))
}

/// A single RCP style action request.
async fn action(
    extract::State(state): extract::State<Arc<AppState>>,
    Json(request): Json<ActionRequest>,
) -> Result<Json<ActionResponse>, (StatusCode, Json<ActionResponse>)> {
    // Convert application errors to appropriate HTTP status codes
    let response = state.action_queue.execute_action(request).await;

    match response.status {
        ActionResponseStatus::Success => Ok(Json(response)),
        ActionResponseStatus::Error => {
            let status_code = match &response.error {
                Some(ActionError::InvalidInput(_)) => StatusCode::UNPROCESSABLE_ENTITY,
                Some(ActionError::Timeout) => StatusCode::REQUEST_TIMEOUT,
                Some(ActionError::ExecutionFailed(_)) => StatusCode::INTERNAL_SERVER_ERROR,
                Some(ActionError::ChannelError(_)) => StatusCode::INTERNAL_SERVER_ERROR,
                None => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status_code, Json(response)))
        }
    }
}

#[derive(Clone)]
struct AppState {
    action_queue: SharedQueue,
}

#[tokio::main]
async fn main() {
    let config = Config::new();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    let action_queue: SharedQueue = create_action_queue().await;

    let state = Arc::new(AppState { action_queue });

    let app = Router::new()
        .route("/", get(root))
        .route("/v1/system/info", get(system_info))
        .route("/v1/action", post(action))
        .route("/v1/monitor", get(monitor_websocket))
        .with_state(state)
        // Trace layer
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_request(|request: &Request<_>, _span: &Span| {
                    info!("Request: {} {}", request.method(), request.uri());
                })
                .on_response(|response: &Response<_>, latency: Duration, _span: &Span| {
                    info!("Response: {} ({:?})", response.status(), latency);
                }),
        );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
