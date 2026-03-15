use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::fmt;

/// Application Error Type
#[derive(Debug)]
pub enum AppError {
    /// Internal Server Error
    InternalServerError(anyhow::Error),
    /// Invalid Input
    InvalidInput(String),
    /// Not Found
    NotFound(String),
}

// Convert anyhow::Error to AppError automatically
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        AppError::InternalServerError(err.into())
    }
}

// Implement Display
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InternalServerError(e) => write!(f, "Internal Server Error: {e}"),
            AppError::InvalidInput(msg) => write!(f, "Invalid Input: {msg}"),
            AppError::NotFound(msg) => write!(f, "Not Found: {msg}"),
        }
    }
}

// Implement IntoResponse for Axum
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InternalServerError(e) => {
                tracing::error!("Internal server error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AppError::InvalidInput(msg) => {
                tracing::warn!("Invalid input: {}", msg);
                (StatusCode::BAD_REQUEST, msg)
            }
            AppError::NotFound(msg) => {
                tracing::info!("Not found: {}", msg);
                (StatusCode::NOT_FOUND, msg)
            }
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}
