use axum::{
    body::Body,
    extract::{self, Request},
    http::StatusCode,
    response::Response,
    routing::{delete, get, post},
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
use action_types::{ActionError, ActionRequest, ActionResponse, ActionResponseStatus};
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
    headers: axum::http::HeaderMap,
    Json(request): Json<ActionRequest>,
) -> Result<Json<ActionResponse>, (StatusCode, Json<ActionResponse>)> {
    let session_id = headers.get("X-Session-ID").and_then(|v| v.to_str().ok());

    match session_id {
        Some(session_id) if state.session_manager.validate_session_id(session_id) => {
            let response = state.action_queue.execute_action(request).await;

            // Convert application errors to appropriate HTTP status codes
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
        _ => Err((
            StatusCode::UNAUTHORIZED,
            Json(ActionResponse::error(
                request.id,
                request.action,
                ActionError::InvalidInput("Invalid or missing session ID".to_string()),
            )),
        )),
    }
}

#[derive(Clone)]
struct AppState {
    action_queue: SharedQueue,
    session_manager: Arc<SessionManager>,
}

#[derive(Debug, Serialize)]
struct SessionResponse {
    session_id: String,
}

#[derive(Debug, Deserialize)]
struct TokenRequest {
    clear_existing: Option<bool>,
}

//  Endpoint to get session ID
async fn create_session(
    extract::State(state): extract::State<Arc<AppState>>,
    Json(request): Json<TokenRequest>,
) -> Result<Json<SessionResponse>, (StatusCode, String)> {
    if request.clear_existing.unwrap_or(false) {
        state.session_manager.clear_session();
    }

    match state.session_manager.create_session() {
        Ok(session) => Ok(Json(SessionResponse {
            session_id: session.id,
        })),
        Err(msg) => Err((StatusCode::CONFLICT, msg.to_string())),
    }
}

async fn end_session(
    extract::State(state): extract::State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<(), StatusCode> {
    let session_id = headers.get("X-Session-ID").and_then(|v| v.to_str().ok());

    match session_id {
        Some(session_id) if state.session_manager.validate_session_id(session_id) => {
            state.session_manager.clear_session();
            Ok(())
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

// Middleware to validate session token
async fn validate_session<B>(
    extract::State(state): extract::State<Arc<AppState>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let session_id = request
        .headers()
        .get("X-Session-ID")
        .and_then(|v| v.to_str().ok());

    match session_id {
        Some(session_id) if state.session_manager.validate_session_id(session_id) => {
            Ok(next.run(request).await)
        }
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
        .route("/v1/session", delete(end_session))
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
