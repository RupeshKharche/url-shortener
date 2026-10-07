use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use sqlx::Error;

pub enum AppError {
    BadRequest(String),
    NotFound,
    Database(sqlx::Error),
    RateLimited
}

impl From<sqlx::Error> for AppError {
    fn from(error: Error) -> Self {
        Self::Database(error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": msg
                })),
            ).into_response(),

            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "Short Url not found"
                })),
            ).into_response(),

            AppError::Database(error) => {
                eprintln!("Database error: {}", error);

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Internal Server Error"
                    })),
                )
                    .into_response()
            }

            AppError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({
                    "error": "You have sent too many requests. Please try again in some time."
                })),
            ).into_response(),
        }
    }
}
