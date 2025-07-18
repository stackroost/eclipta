mod services;
mod api;

use axum::{Router};
use tower_http::services::ServeDir;
use tower_http::cors::{CorsLayer, Any};
use axum::http::Method;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any) 
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api", api::routes())
        .nest_service("/", ServeDir::new("../frontend/dist"))
        .layer(cors); 

    println!("Server running at http://0.0.0.0:3000");
    axum::serve(
        tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap(),
        app
    ).await.unwrap();
}
