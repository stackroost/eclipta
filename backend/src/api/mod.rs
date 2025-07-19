pub mod auth;
pub mod agents;
use axum::Router;


pub fn routes() -> Router {
    Router::new()
        .nest("/auth", auth::routes())
        .nest("/agents", agents::routes())
}
