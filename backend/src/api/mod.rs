pub mod auth;

use axum::Router;

pub fn routes() -> Router {
    Router::new()
        .nest("/auth", auth::routes()) 
}
