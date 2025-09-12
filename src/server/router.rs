use crate::adapter::services::AdapterRegistry;
use crate::routes;
use axum::Router;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Build the main application router
pub fn build_router(base_path: &str, adapter_registry: Arc<RwLock<AdapterRegistry>>) -> Router {
    let app = Router::new()
        // Health endpoint (always unversioned at root)
        .route("/", axum::routing::get(routes::health::health_check));

    // If base_path is empty, mount v1 directly at /v1
    // If base_path is set (e.g., "api"), mount v1 at /{base_path}/v1
    if base_path.is_empty() {
        app.nest("/v1", routes::v1::router(adapter_registry))
    } else {
        app.nest(
            &format!("/{}/v1", base_path),
            routes::v1::router(adapter_registry),
        )
    }
}
