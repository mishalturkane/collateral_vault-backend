use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Authentication failed")]
    Unauthorized,
    
    #[error("Invalid request: {0}")]
    BadRequest(String),
    
    #[error("Resource not found")]
    NotFound,
    
    #[error("Internal server error")]
    InternalServerError,
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Solana RPC error: {0}")]
    SolanaError(String),
    
    #[error("Transaction failed: {0}")]
    TransactionError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(#[from] validator::ValidationErrors),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Resource not found"),
            ApiError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            ApiError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            ApiError::SolanaError(msg) => (StatusCode::BAD_GATEWAY, msg.as_str()),
            ApiError::TransactionError(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            ApiError::ValidationError(_) => (StatusCode::BAD_REQUEST, "Validation failed"),
        };

        let body = Json(json!({
            "error": error_message,
            "code": status.as_u16(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }));

        (status, body).into_response()
    }
}