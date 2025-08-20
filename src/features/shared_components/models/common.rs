// src/features/shared_components/models/common.rs
//
// Common data structures shared across features

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Generic result wrapper for API responses
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResult<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }
    
    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.to_string()),
            timestamp: Utc::now(),
        }
    }
}

/// Pagination metadata
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaginationMeta {
    pub current_page: i64,
    pub per_page: i64,
    pub total_items: i64,
    pub total_pages: i64,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Paginated response wrapper
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub pagination: PaginationMeta,
    pub timestamp: DateTime<Utc>,
}

/// System status enumeration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum SystemStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Maintenance,
}

impl std::fmt::Display for SystemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemStatus::Healthy => write!(f, "healthy"),
            SystemStatus::Degraded => write!(f, "degraded"),
            SystemStatus::Unhealthy => write!(f, "unhealthy"),
            SystemStatus::Maintenance => write!(f, "maintenance"),
        }
    }
}

/// Language enumeration for multi-language support
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Language {
    Vietnamese,
    English,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::Vietnamese => "vi",
            Language::English => "en",
        }
    }
    
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_lowercase().as_str() {
            "vi" | "vietnamese" => Some(Language::Vietnamese),
            "en" | "english" => Some(Language::English),
            _ => None,
        }
    }
}

/// Cache status information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheStatus {
    pub hit: bool,
    pub source: String, // "l1", "l2", "miss"
    pub ttl_remaining: Option<u64>,
}

/// Performance metrics structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PerformanceMetrics {
    pub response_time_ms: u64,
    pub cache_hit_rate: f64,
    pub requests_per_second: f64,
    pub active_connections: u64,
    pub memory_usage_mb: f64,
}

/// Error categories for better error handling
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ErrorCategory {
    ValidationError,
    DatabaseError,
    ExternalApiError,
    CacheError,
    NetworkError,
    AuthenticationError,
    AuthorizationError,
    RateLimitError,
    InternalError,
}

/// Structured error information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorInfo {
    pub category: ErrorCategory,
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub request_id: Option<String>,
}

impl ErrorInfo {
    pub fn new(category: ErrorCategory, code: &str, message: &str) -> Self {
        Self {
            category,
            code: code.to_string(),
            message: message.to_string(),
            details: None,
            timestamp: Utc::now(),
            request_id: None,
        }
    }
    
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
    
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}
