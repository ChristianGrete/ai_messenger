use axum::{Router, routing::post};

mod handler;
mod request;
mod response;

pub use handler::send_message;

/// Build the message router
pub fn router() -> Router {
    Router::new().route("/:recipient_id", post(send_message))
}
