use axum::{
    routing::post,
    Json, Router,
    response::IntoResponse,
};
use serde::Deserialize;
use crate::services::pam_auth;

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

async fn login_handler(Json(payload): Json<LoginRequest>) -> impl IntoResponse {
    match pam_auth(&payload.username, &payload.password) {
        Ok(true) => "Authenticated: root/sudo",
        Ok(false) => "Not root/sudo",
        Err(e) => {
            eprintln!("Auth error: {e}");
            "Login failed"
        }
    }
}

pub fn routes() -> Router {
    Router::new().route("/login", post(login_handler)) // /api/auth/login
}
