use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Rate limited, retry after {retry_after} seconds")]
    RateLimit { retry_after: u64 },

    #[error("Invalid amount: {amount}, must be between {min} and {max}")]
    InvalidAmount { 
        amount: rust_decimal::Decimal, 
        min: rust_decimal::Decimal, 
        max: rust_decimal::Decimal 
    },

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Cloudflare block detected")]
    CloudflareBlock,

    #[error("No available accounts")]
    NoAvailableAccounts,

    #[error("Invalid receipt: {0}")]
    InvalidReceipt(String),

    #[error("Amount mismatch - expected: {expected}, received: {received}")]
    AmountMismatch {
        expected: rust_decimal::Decimal,
        received: rust_decimal::Decimal,
    },

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Session expired")]
    SessionExpired,

    #[error("Order not found: {0}")]
    OrderNotFound(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("AI service error: {0}")]
    AIService(String),

    #[error("Email service error: {0}")]
    EmailService(String),

    #[error("OCR processing error: {0}")]
    OCRProcessing(String),
    
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    
    #[error("Gmail API error: {0}")]
    Gmail(String),
    
    #[error("File system error: {0}")]
    FileSystem(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Email error: {0}")]
    EmailError(String),
    
    #[error("OCR error: {0}")]
    OcrError(String),
    
    #[error("OCR error: {0}")]
    Ocr(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::FileSystem(err.to_string())
    }
}

impl From<async_openai::error::OpenAIError> for AppError {
    fn from(err: async_openai::error::OpenAIError) -> Self {
        AppError::InternalError(format!("OpenAI API error: {}", err))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Serialization(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Authentication(ref msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::RateLimit { retry_after } => (
                StatusCode::TOO_MANY_REQUESTS,
                format!("Rate limited. Retry after {} seconds", retry_after),
            ),
            AppError::InvalidAmount { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
            AppError::Network(_) => (StatusCode::BAD_GATEWAY, "Network error".to_string()),
            AppError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error".to_string()),
            AppError::CloudflareBlock => (StatusCode::FORBIDDEN, "Access blocked".to_string()),
            AppError::NoAvailableAccounts => (StatusCode::SERVICE_UNAVAILABLE, "No available accounts".to_string()),
            AppError::InvalidReceipt(ref msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::AmountMismatch { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::InvalidInput(ref msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Encryption(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Encryption error".to_string()),
            AppError::SessionExpired => (StatusCode::UNAUTHORIZED, "Session expired".to_string()),
            AppError::OrderNotFound(ref id) => (StatusCode::NOT_FOUND, format!("Order {} not found", id)),
            AppError::WebSocket(ref msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::AIService(_) => (StatusCode::SERVICE_UNAVAILABLE, "AI service error".to_string()),
            AppError::EmailService(_) => (StatusCode::SERVICE_UNAVAILABLE, "Email service error".to_string()),
            AppError::OCRProcessing(_) => (StatusCode::UNPROCESSABLE_ENTITY, "OCR processing error".to_string()),
            AppError::NotImplemented(ref msg) => (StatusCode::NOT_IMPLEMENTED, msg.clone()),
            AppError::Gmail(ref msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::FileSystem(ref msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::Serialization(ref msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Configuration(ref msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::EmailError(ref msg) => (StatusCode::SERVICE_UNAVAILABLE, msg.clone()),
            AppError::OcrError(ref msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::Ocr(ref msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::ValidationError(ref msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::BadRequest(ref msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::NotFound(ref msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::InternalError(ref msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}