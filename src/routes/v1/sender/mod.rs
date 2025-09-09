mod profile;

use axum::Router;

/// Build the sender router
pub fn router() -> Router {
    Router::new().nest("/profile", profile::router())
}
