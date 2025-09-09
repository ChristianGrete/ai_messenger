use axum::{Router, http::StatusCode, response::Json, routing::post};
use serde_json::{Value, json};

/// Build the message router
pub fn router() -> Router {
    Router::new().route("/send", post(send_message))
}

/// Send a message - dummy implementation
async fn send_message() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "success",
        "message_id": "msg_dummy_123",
        "response": "This is a dummy message endpoint - message sent!",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
