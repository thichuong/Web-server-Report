//! Health check response DTOs

use crate::dto::common::HealthStatus;
use serde::Serialize;

/// Response for GET /health endpoint
#[derive(Debug, Serialize)]
pub struct HealthCheckResponse {
    pub status: HealthStatus,
    pub service_islands: ServiceIslandsInfo,
}

/// Service Islands information for health checks
#[derive(Debug, Serialize)]
pub struct ServiceIslandsInfo {
    pub total: u8,
    pub operational: u8,
    pub architecture: String,
    pub timestamp: String,
}

/// Response for GET /api/health endpoint
#[derive(Debug, Serialize)]
pub struct ApiHealthResponse {
    pub api: ApiHealthInfo,
}

/// API health information
#[derive(Debug, Serialize)]
pub struct ApiHealthInfo {
    pub status: HealthStatus,
    pub service_islands: u8,
    pub timestamp: String,
}
