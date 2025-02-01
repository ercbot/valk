use std::time::Duration;
use axum::{
    routing::{get, post}, Router, extract,
    response::Response,
    extract::Request
};
use serde::{Serialize, Deserialize};

use std::sync::Arc;

use tower_http::trace::{self, TraceLayer};
use tracing::{info, Level, Span};

mod key_press;
mod action_queue;
mod config;

use config::Config;
use action_queue::{ActionQueue, Action, ActionError, ActionResult, create_action_queue};

async fn root() -> &'static str {
    "Subspace is running"
}

/// Take a screenshot of the screen.
async fn screenshot(
    extract::State(queue): extract::State<Arc<ActionQueue>>
) -> Result<ActionResult, ActionError> {
    queue.execute_action(Action::Screenshot).await
}

// Request body structure
#[derive(Debug, Serialize, Deserialize)]
struct ClickRequest {
    x: u32,
    y: u32
}

/// Click the left mouse button.
async fn left_click(
    extract::State(queue): extract::State<Arc<ActionQueue>>
) -> Result<ActionResult, ActionError> {
    queue.execute_action(Action::LeftClick).await
}

/// Click the right mouse button.
async fn right_click(
    extract::State(queue): extract::State<Arc<ActionQueue>>
) -> Result<ActionResult, ActionError> {
    queue.execute_action(Action::RightClick).await
}

/// Click the middle mouse button.
async fn middle_click(
    extract::State(queue): extract::State<Arc<ActionQueue>>
) -> Result<ActionResult, ActionError> {
    queue.execute_action(Action::MiddleClick).await
}

/// Double-click the left mouse button.
async fn double_click(
    extract::State(queue): extract::State<Arc<ActionQueue>>
) -> Result<ActionResult, ActionError> {
    queue.execute_action(Action::DoubleClick).await
}

/// Get the current (x, y) pixel coordinate of the cursor on the screen.
async fn cursor_position(
    extract::State(queue): extract::State<Arc<ActionQueue>>
) -> Result<ActionResult, ActionError> {
    queue.execute_action(Action::CursorPosition).await
}

/// Move the cursor to a specified (x, y) pixel coordinate on the screen.
async fn mouse_move(
    extract::State(queue): extract::State<Arc<ActionQueue>>,
    extract::Json(click_request): extract::Json<ClickRequest>,
) -> Result<ActionResult, ActionError> {
    queue.execute_action(Action::MouseMove {
        x: click_request.x,
        y: click_request.y,
    }).await
}

/// Click and drag the cursor to a specified (x, y) pixel coordinate on the screen.
async fn left_click_drag(
    extract::State(queue): extract::State<Arc<ActionQueue>>,
    extract::Json(click_request): extract::Json<ClickRequest>,
) -> Result<ActionResult, ActionError> {
    queue.execute_action(Action::LeftClickDrag {
        x: click_request.x,
        y: click_request.y,
    }).await
}

#[derive(Deserialize)]
struct TextInput {
    text: String
}

/// Type a string of text on the keyboard.
async fn type_text(
    extract::State(queue): extract::State<Arc<ActionQueue>>,
    extract::Json(input): extract::Json<TextInput>,
) -> Result<ActionResult, ActionError> {
    queue.execute_action(Action::TypeText {
        text: input.text,
    }).await
}

/// Press a key or key-combination on the keyboard.
/// - This supports xdotool's `key` syntax.
/// - Examples: "a", "Return", "alt+Tab", "ctrl+s", "Up", "KP_0" (for the numpad 0 key).
async fn key(
    extract::State(queue): extract::State<Arc<ActionQueue>>,
    extract::Json(input): extract::Json<TextInput>,
) -> Result<ActionResult, ActionError> {
    queue.execute_action(Action::KeyPress {
        key: input.text,
    }).await
}

#[tokio::main]
async fn main() {
    let config = Config::new();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    let action_queue = create_action_queue().await;

    let app = Router::new()
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
        .route("/v1/actions/key", post(key))
        .with_state(action_queue)
        // Trace layer
        .layer(TraceLayer::new_for_http()
            .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
            .on_request(|request: &Request<_>, _span: &Span| {
                info!("Request: {} {}", request.method(), request.uri());
            })
            .on_response(|response: &Response<_>, latency: Duration, _span: &Span| {
                info!(
                    "Response: {} ({:?})",
                    response.status(),
                    latency
                );
            }),
    );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}