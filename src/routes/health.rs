use axum::{http::StatusCode, response::Json};
use serde_json::{Value, json};

/// Health check endpoint - always available at /
pub async fn health_check() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "ok",
        "message": "AI Messenger is running",
        "version": env!("CARGO_PKG_VERSION")
    })))
}
