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
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CacheOperationStatus {
    Queued,
    Completed,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_serialization() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            serde_json::to_string(&HealthStatus::Healthy)?,
            "\"healthy\""
        );
        assert_eq!(
            serde_json::to_string(&HealthStatus::Unhealthy)?,
            "\"unhealthy\""
        );
        Ok(())
    }

    #[test]
    fn test_cache_operation_status_serialization() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            serde_json::to_string(&CacheOperationStatus::Queued)?,
            "\"queued\""
        );
        assert_eq!(
            serde_json::to_string(&CacheOperationStatus::Completed)?,
            "\"completed\""
        );
        assert_eq!(
            serde_json::to_string(&CacheOperationStatus::Failed)?,
            "\"failed\""
        );
        Ok(())
    }
}
