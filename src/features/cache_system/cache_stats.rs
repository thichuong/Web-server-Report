// src/features/cache_system/cache_stats.rs
//
// Cache statistics and health monitoring structures

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CacheStats {
    pub l1_entry_count: u64,
    pub l1_hit_count: u64,
    pub l1_miss_count: u64,
    pub l1_hit_rate: f64, // Percentage
}

#[derive(Debug, Clone, Serialize)]
pub struct CacheHealthCheck {
    pub l1_healthy: bool,
    pub l2_healthy: bool,
    pub overall_healthy: bool,
}

#[derive(Debug)]
pub enum CacheError {
    SerializationError(String),
    ConnectionError(String),
    TimeoutError,
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CacheError::SerializationError(msg) => write!(f, "Cache serialization error: {}", msg),
            CacheError::ConnectionError(msg) => write!(f, "Cache connection error: {}", msg),
            CacheError::TimeoutError => write!(f, "Cache operation timeout"),
        }
    }
}

impl std::error::Error for CacheError {}
