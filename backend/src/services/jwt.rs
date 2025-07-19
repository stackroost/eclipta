use jsonwebtoken::{encode, EncodingKey, Header};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims { // <- add `pub` here
    pub sub: String,
    pub is_admin: bool,
    pub exp: usize,
}

pub fn generate_token(username: &str, is_admin: bool) -> Result<String, String> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() + 3600; // 1 hour

    let claims = Claims {
        sub: username.to_owned(),
        is_admin,
        exp: expiration as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret("mysecret".as_ref()))
        .map_err(|e| e.to_string())
}
