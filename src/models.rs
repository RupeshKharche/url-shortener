use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Url {
    pub id: String,
    pub original_url: String,
    pub code: String,
    pub created_at: String,
    pub expires_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUrlRequest {
    pub url: String,
    pub ttl_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct CreateUrlResponse {
    pub short_url: String,
    pub expires_at: String,
}
