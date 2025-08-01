use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Session error: {0}")]
    SessionError(String),
    
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
    
    #[error("Audio streaming error: {0}")]
    AudioError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::SessionError(msg) => (StatusCode::CONFLICT, msg),
            AppError::WebSocketError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::AudioError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Internal(err) => {
                tracing::error!("Internal error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        let body = Json(json!({
            "error": error_message,
            "success": false
        }));

        (status, body).into_response()
    }
}

// Convenience conversion from std::io::Error
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Internal(err.into())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;