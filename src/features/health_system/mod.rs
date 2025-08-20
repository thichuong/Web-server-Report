// src/features/health_system/mod.rs - Health System Feature
//
// This feature provides system health monitoring, metrics collection,
// and performance tracking. It observes other systems without modifying them,
// making it completely independent and safe to remove.

pub mod handlers;
pub mod models;

use crate::features::{Feature, FeatureHealthStatus};
use crate::features::cache_system::CacheSystemFeature;
use axum::{Router, routing::get};
use std::sync::Arc;

///! Health System - Layer 4: Observability
//! 
//! Provides comprehensive system monitoring, health checks, performance metrics,
//! and SSL connectivity testing for production monitoring and debugging.
//!
//! ## Dependencies (Layer 4 - Observability)
//! - Layer 1: shared_components (ApiResult, error handling)
//! - Layer 1: cache_system (cache statistics and health monitoring)
//! - Layer 2: external_apis (SSL connectivity testing for external services)
//! - Layer 3: websocket_service (connection health monitoring)
//!
//! ## Service Island Architecture
//! This module provides observability infrastructure for the entire system,
//! serving as the monitoring layer that tracks performance, health, and connectivity.

pub mod health_monitor;
pub mod performance_collector;
pub mod connectivity_tester;
pub mod metrics_aggregator;
pub mod handlers;

pub use health_monitor::HealthMonitor;
pub use performance_collector::PerformanceCollector;
pub use connectivity_tester::ConnectivityTester;
pub use metrics_aggregator::MetricsAggregator;
pub use handlers::health_system_routes;

/// Health System Service Island - Layer 4 (Observability)
/// 
/// Provides centralized monitoring and health checking capabilities across all system layers
pub struct HealthSystem {
    pub health_monitor: HealthMonitor,
    pub performance_collector: PerformanceCollector,
    pub connectivity_tester: ConnectivityTester,
    pub metrics_aggregator: MetricsAggregator,
}

impl HealthSystem {
    pub fn new() -> Self {
        Self {
            health_monitor: HealthMonitor::new(),
            performance_collector: PerformanceCollector::new(),
            connectivity_tester: ConnectivityTester::new(),
            metrics_aggregator: MetricsAggregator::new(),
        }
    }

    /// Get comprehensive system health status
    pub async fn get_system_health(&self) -> serde_json::Value {
        self.health_monitor.get_comprehensive_health().await
    }

    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> serde_json::Value {
        self.performance_collector.get_metrics().await
    }

    /// Test external service connectivity
    pub async fn test_connectivity(&self) -> serde_json::Value {
        self.connectivity_tester.test_all_services().await
    }

    /// Get aggregated metrics across all system components
    pub async fn get_aggregated_metrics(&self) -> serde_json::Value {
        self.metrics_aggregator.get_all_metrics().await
    }

    /// Get Axum routes for health system endpoints
    pub fn routes() -> axum::Router<std::sync::Arc<crate::state::AppState>> {
        handlers::health_system_routes()
    }
}

impl Default for HealthSystem {
    fn default() -> Self {
        Self::new()
    }
}
pub struct HealthSystemFeature {
    cache_system: Arc<CacheSystemFeature>,
}

impl HealthSystemFeature {
    pub async fn new(cache_system: Arc<CacheSystemFeature>) -> Result<Self, anyhow::Error> {
        Ok(Self {
            cache_system,
        })
    }
    
    /// Get HTTP routes for health endpoints
    pub fn routes(&self) -> Router<Arc<crate::state::AppState>> {
        Router::new()
            .route("/health", get(crate::handlers::health))
            .route("/metrics", get(crate::handlers::performance_metrics))
            .route("/admin/cache/stats", get(crate::handlers::cache_stats))
            .route("/admin/cache/clear", get(crate::handlers::clear_cache))
    }
}

#[async_trait::async_trait]
impl Feature for HealthSystemFeature {
    type Config = Arc<CacheSystemFeature>;
    type Error = anyhow::Error;
    
    async fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Self::new(config).await
    }
    
    fn name(&self) -> &'static str {
        "health_system"
    }
    
    async fn health_check(&self) -> FeatureHealthStatus {
        // Health system is always healthy as it's just an observer
        FeatureHealthStatus::healthy("health_system")
            .with_details(serde_json::json!({
                "monitoring": "active",
                "endpoints_active": 4
            }))
    }
}
