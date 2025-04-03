// Custom error types
use thiserror::Error;

/// Custom error enum for all price index service errors
///
/// This enum provides specific error variants for different failure scenarios,
/// making error handling more structured and informative. Each variant
/// includes context information to aid in debugging and error reporting.
#[derive(Error, Debug)]
pub enum PriceIndexError {
    /// Errors related to exchange-specific failures
    #[error("Exchange error: {0}")]
    ExchangeError(String),

    /// Errors specific to WebSocket connections
    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    /// HTTP request errors from the reqwest client
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// JSON parsing or serialization errors
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Errors related to invalid price data from exchanges
    #[error("Invalid price data: {0}")]
    InvalidPriceData(String),
}

/// A type alias for Result that uses our custom error type
///
/// This simplifies function signatures throughout the codebase by providing
/// a consistent Result type that automatically uses PriceIndexError.
pub type Result<T> = std::result::Result<T, PriceIndexError>;
