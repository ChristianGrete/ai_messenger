use axum::{Router, http::StatusCode, response::Json, routing::get};
use serde_json::{Value, json};

/// Build the sender profile router
pub fn router() -> Router {
    Router::new().route("/", get(get_profile))
}

/// Get sender profile - dummy implementation
async fn get_profile() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "id": "sender",
        "name": "Default Sender",
        "status": "active",
        "message": "This is a dummy sender profile endpoint"
    })))
}
