use crate::adapter::services::AdapterRegistry;
use axum::{Router, routing::post};
use std::sync::Arc;
use tokio::sync::RwLock;

mod handler;
mod request;
mod response;

pub use handler::send_message;

/// Build the message router
pub fn router(adapter_registry: Arc<RwLock<AdapterRegistry>>) -> Router {
    Router::new()
        .route("/:recipient_id", post(send_message))
        .with_state(adapter_registry)
}
