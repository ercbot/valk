use axum::{
    body::Body,
    extract::{self, Request},
    http::StatusCode,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use std::sync::Arc;

use tower_http::trace::{self, TraceLayer};
use tracing::{info, Level, Span};

use axum::middleware::{self, Next};

mod action_queue;
mod action_types;
mod config;
mod key_press;
mod monitor;
mod session;

use action_queue::{create_action_queue, SharedQueue};
use action_types::{ActionRequest, ActionResponse};
use config::Config;
use monitor::monitor_websocket;
use session::SessionManager;

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
    extract::Json(input): extract::Json<ActionRequest>,
) -> Json<ActionResponse> {
    Json(state.action_queue.execute_action(input).await)
}

#[derive(Clone)]
struct AppState {
    action_queue: SharedQueue,
    session_manager: Arc<SessionManager>,
}

#[derive(Debug, Serialize)]
struct SessionResponse {
    token: String,
}

#[derive(Debug, Deserialize)]
struct TokenRequest {
    clear_existing: Option<bool>,
}

// New endpoint to get session token
async fn create_session(
    extract::State(state): extract::State<Arc<AppState>>,
    Json(request): Json<TokenRequest>,
) -> Json<SessionResponse> {
    if request.clear_existing.unwrap_or(false) {
        state.session_manager.clear_session();
    }
    let session = state.session_manager.create_session();
    Json(SessionResponse {
        token: session.token,
    })
}

// Middleware to validate session token
async fn validate_session<B>(
    extract::State(state): extract::State<Arc<AppState>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = request
        .headers()
        .get("X-Session-Token")
        .and_then(|v| v.to_str().ok());

    match token {
        Some(token) if state.session_manager.validate_token(token) => Ok(next.run(request).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
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
    let session_manager = Arc::new(SessionManager::new());

    let state = Arc::new(AppState {
        action_queue,
        session_manager,
    });

    let protected_routes = Router::new()
        .route("/v1/action", post(action))
        .route("/v1/monitor", get(monitor_websocket))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            validate_session::<Body>,
        ));

    let app = Router::new()
        .route("/", get(root))
        .route("/v1/system/info", get(system_info))
        .route("/v1/session", post(create_session))
        .merge(protected_routes)
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
