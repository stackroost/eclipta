use axum::{
    routing::post,
    Json, Router,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use crate::services::{pam_auth, jwt::generate_token};

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
    message: String,
}

async fn login_handler(Json(payload): Json<LoginRequest>) -> Response {
    match pam_auth(&payload.username, &payload.password) {
        Ok(true) => {
            let token = generate_token(&payload.username, true).unwrap_or_else(|_| "".into());
            Json(LoginResponse {
                token,
                message: "Authenticated: successfully".into(),
            }).into_response()
        }
        Ok(false) => Json(LoginResponse {
            token: "".into(),
            message: "Not root/sudo".into(),
        }).into_response(),
        Err(e) => {
            eprintln!("Auth error: {e}");
            Json(LoginResponse {
                token: "".into(),
                message: "Login failed".into(),
            }).into_response()
        }
    }
}

pub fn  routes() -> axum::Router {
    Router::new().route("/login", post(login_handler))
}