//! Custom Error Types for Layer 5 Business Logic
//!
//! Provides strongly-typed errors instead of Box<dyn Error>.
//! This reduces fat pointer overhead on hot paths (16,829+ RPS).

use std::fmt;

/// Result type alias for Layer 5 operations
pub type Layer5Result<T> = Result<T, Layer5Error>;

/// Custom error type for Layer 5 business logic operations
///
/// Uses an enum instead of Box<dyn Error> to avoid heap allocation
/// and fat pointer overhead on every error path.
#[derive(Debug)]
pub enum Layer5Error {
    /// Database operation failed
    Database(String),
    /// Template rendering failed
    TemplateRender(String),
    /// Compression operation failed
    Compression(String),
    /// Cache operation failed
    Cache(String),
    /// Timeout occurred
    Timeout(String),
    /// Invalid input provided
    InvalidInput(String),
    /// Resource not found
    NotFound(String),
    /// Authentication/authorization failed
    Forbidden(String),
    /// Task join error (from `spawn_blocking`)
    TaskJoin(String),
    /// Generic internal error
    Internal(String),
}

impl fmt::Display for Layer5Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(msg) => write!(f, "Database error: {msg}"),
            Self::TemplateRender(msg) => write!(f, "Template render error: {msg}"),
            Self::Compression(msg) => write!(f, "Compression error: {msg}"),
            Self::Cache(msg) => write!(f, "Cache error: {msg}"),
            Self::Timeout(msg) => write!(f, "Timeout: {msg}"),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            Self::NotFound(msg) => write!(f, "Not found: {msg}"),
            Self::Forbidden(msg) => write!(f, "Forbidden: {msg}"),
            Self::TaskJoin(msg) => write!(f, "Task join error: {msg}"),
            Self::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl std::error::Error for Layer5Error {}

// Conversion implementations for common error types

impl From<sqlx::Error> for Layer5Error {
    #[inline]
    fn from(e: sqlx::Error) -> Self {
        Self::Database(e.to_string())
    }
}

impl From<tera::Error> for Layer5Error {
    #[inline]
    fn from(e: tera::Error) -> Self {
        Self::TemplateRender(e.to_string())
    }
}

impl From<std::io::Error> for Layer5Error {
    #[inline]
    fn from(e: std::io::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<tokio::task::JoinError> for Layer5Error {
    #[inline]
    fn from(e: tokio::task::JoinError) -> Self {
        Self::TaskJoin(e.to_string())
    }
}

impl From<tokio::time::error::Elapsed> for Layer5Error {
    #[inline]
    fn from(_: tokio::time::error::Elapsed) -> Self {
        Self::Timeout("Operation timed out".to_string())
    }
}

impl Layer5Error {
    /// Convert to boxed error for backward compatibility with legacy APIs
    #[inline]
    #[must_use] 
    pub fn into_boxed(self) -> Box<dyn std::error::Error + Send + Sync> {
        Box::new(self)
    }

    /// Check if error is a not found error
    #[inline]
    #[must_use] 
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }

    /// Check if error is a timeout
    #[inline]
    #[must_use] 
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout(_))
    }

    /// Convert to HTTP status code
    #[inline]
    #[must_use] 
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::InvalidInput(_) => StatusCode::BAD_REQUEST,
            Self::Timeout(_) => StatusCode::GATEWAY_TIMEOUT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
