// Custom error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PriceIndexError {
    #[error("Exchange error: {0}")]
    ExchangeError(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid price data: {0}")]
    InvalidPriceData(String),
}

/// A type alias for Result that uses our custom error type
pub type Result<T> = std::result::Result<T, PriceIndexError>;
