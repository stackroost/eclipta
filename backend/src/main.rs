use axum::{routing::get, Router};
use tower_http::services::{ServeDir, ServeFile};
use std::net::SocketAddr;
use std::path::PathBuf;

async fn hello_handler() -> &'static str {
    "Hello from API"
}

#[tokio::main]
async fn main() {
    // Absolute path to frontend/dist based on backend crate root
    let dist_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../frontend/dist");

    // Serve static files and fallback to index.html
    let frontend_service = ServeDir::new(&dist_path)
        .fallback(ServeFile::new(dist_path.join("index.html")));

    let app = Router::new()
        .route("/api/hello", get(hello_handler))
        .nest_service("/", frontend_service);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running: http://{addr}");

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
