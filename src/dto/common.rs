//! Common types shared across multiple DTO response structs

use serde::Serialize;

/// Health status for services and APIs
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
}

/// Cache operation status for tracking async operations
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CacheOperationStatus {
    Queued,
    Completed,
    Failed,
}
