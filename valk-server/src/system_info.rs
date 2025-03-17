use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ComputerInfo {
    os_type: String,
    os_version: String,
    display_width: u32,
    display_height: u32,
}

/// Get information about the computer system
pub async fn system_info() -> Result<Json<ComputerInfo>, (StatusCode, String)> {
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
