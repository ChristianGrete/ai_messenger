pub mod message;
pub mod sender;

use axum::Router;

/// Build the v1 API router
pub fn router() -> Router {
    Router::new()
        .nest("/sender", sender::router())
        .nest("/message", message::router())
}
