//! Cache-related response DTOs

use serde::Serialize;
use crate::dto::common::CacheOperationStatus;

/// Response for GET /admin/cache/clear endpoint
#[derive(Debug, Serialize)]
pub struct CacheClearResponse {
    pub message: String,
    pub status: CacheOperationStatus,
}

/// Response for GET /metrics endpoint
#[derive(Debug, Serialize)]
pub struct PerformanceMetricsResponse {
    pub performance: PerformanceInfo,
    pub cache_info: String,
}

/// Performance information for metrics
#[derive(Debug, Serialize)]
pub struct PerformanceInfo {
    pub service_islands_active: u8,
    pub uptime: String,
    pub memory_usage: String,
    pub cache_status: String,
}

/// Response for GET /admin/cache/stats endpoint
/// Uses untagged enum to handle available vs unavailable states
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum CacheStatsResponse {
    Available(CacheStatsAvailable),
    Unavailable(CacheStatsUnavailable),
}

/// Cache stats when system is available
#[derive(Debug, Serialize)]
pub struct CacheStatsAvailable {
    pub cache: CacheSystemInfo,
    pub statistics: CacheStatistics,
    pub configuration: CacheConfiguration,
    pub health: CacheHealth,
}

/// Cache stats when system is unavailable
#[derive(Debug, Serialize)]
pub struct CacheStatsUnavailable {
    pub error: String,
    pub cache: CacheStatusOnly,
}

/// Cache system information
#[derive(Debug, Serialize)]
pub struct CacheSystemInfo {
    pub system: String,
    pub l1_cache: String,
    pub l2_cache: String,
    pub status: String,
}

/// Cache statistics with hit rates and counts
#[derive(Debug, Serialize)]
pub struct CacheStatistics {
    pub total_requests: u64,
    pub l1_hits: u64,
    pub l2_hits: u64,
    pub total_hits: u64,
    pub misses: u64,
    pub promotions: usize,
    pub hit_rate: String,
    pub in_flight_requests: usize,
}

/// Cache configuration details
#[derive(Debug, Serialize)]
pub struct CacheConfiguration {
    pub l1_max_capacity: usize,
    pub l1_ttl: String,
    pub l2_ttl: String,
    pub eviction: String,
    pub stampede_protection: String,
}

/// Cache health status and recommendations
#[derive(Debug, Serialize)]
pub struct CacheHealth {
    pub status: String,
    pub recommendation: String,
}

/// Minimal cache status for unavailable state
#[derive(Debug, Serialize)]
pub struct CacheStatusOnly {
    pub status: String,
}
