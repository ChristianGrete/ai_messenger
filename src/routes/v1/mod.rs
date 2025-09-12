pub mod message;
pub mod sender;

use crate::adapter::services::AdapterRegistry;
use axum::Router;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Build the v1 API router
pub fn router(adapter_registry: Arc<RwLock<AdapterRegistry>>) -> Router {
    Router::new()
        .nest("/sender", sender::router())
        .nest("/message", message::router(adapter_registry))
}
