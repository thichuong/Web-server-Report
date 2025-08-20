//! Metrics Aggregator - Aggregates metrics from all system components
//!
//! Collects and combines metrics from cache system, external APIs, WebSocket service,
//! and performance collector to provide unified system-wide metrics.

use crate::features::shared_components::SystemStatus;
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::time::Instant;

/// Aggregates metrics from all system components
pub struct MetricsAggregator {
    collection_start: Instant,
    custom_metrics: HashMap<String, Value>,
}

impl MetricsAggregator {
    pub fn new() -> Self {
        Self {
            collection_start: Instant::now(),
            custom_metrics: HashMap::new(),
        }
    }

    /// Get all metrics aggregated from system components
    pub async fn get_all_metrics(&self) -> Value {
        let collection_duration = self.collection_start.elapsed().as_secs();
        
        json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "collection_duration_seconds": collection_duration,
            "system_overview": self.get_system_overview().await,
            "layer_metrics": self.get_layer_metrics().await,
            "custom_metrics": self.custom_metrics,
            "aggregation_info": {
                "collection_start": chrono::Utc::now().to_rfc3339(),
                "total_components": 4, // health_monitor, performance_collector, connectivity_tester, metrics_aggregator
                "metrics_version": "1.0.0"
            }
        })
    }

    /// Get high-level system overview
    async fn get_system_overview(&self) -> Value {
        json!({
            "service": "AI Investment Report Server",
            "architecture": "Service Islands",
            "layer_count": 5,
            "features_active": 5, // shared_components, cache_system, external_apis, websocket_service, health_system
            "monitoring_status": "active",
            "health_status": "healthy" // TODO: Integrate actual health status
        })
    }

    /// Get metrics organized by architectural layers
    async fn get_layer_metrics(&self) -> Value {
        json!({
            "layer_1_infrastructure": {
                "shared_components": {
                    "status": "active",
                    "components": ["ApiResult", "SystemStatus", "PerformanceMetrics", "FeatureContext"]
                },
                "cache_system": {
                    "status": "active",
                    "l1_cache_active": true,
                    "l2_cache_active": false, // TODO: Check actual Redis status
                    "components": ["CacheManager", "MultiTierCache", "CacheStats", "CacheKeys"]
                }
            },
            "layer_2_external_services": {
                "external_apis": {
                    "status": "active",
                    "market_data_provider_active": true,
                    "rate_limiting_active": true,
                    "components": ["MarketDataProvider", "ApiClient", "RateLimiter", "models"]
                }
            },
            "layer_3_communication": {
                "websocket_service": {
                    "status": "active",
                    "connection_manager_active": true,
                    "broadcast_service_active": true,
                    "components": ["ConnectionManager", "BroadcastService", "MessageHandler", "HeartbeatManager"]
                }
            },
            "layer_4_observability": {
                "health_system": {
                    "status": "active",
                    "health_monitoring_active": true,
                    "performance_collection_active": true,
                    "connectivity_testing_active": true,
                    "components": ["HealthMonitor", "PerformanceCollector", "ConnectivityTester", "MetricsAggregator"]
                }
            },
            "layer_5_business_logic": {
                "dashboard": {
                    "status": "pending_extraction",
                    "components": []
                },
                "crypto_reports": {
                    "status": "pending_extraction",
                    "components": []
                }
            }
        })
    }

    /// Add a custom metric
    pub fn add_custom_metric(&mut self, key: String, value: Value) {
        self.custom_metrics.insert(key, value);
    }

    /// Remove a custom metric
    pub fn remove_custom_metric(&mut self, key: &str) -> Option<Value> {
        self.custom_metrics.remove(key)
    }

    /// Get cache system metrics (Layer 1)
    pub async fn get_cache_metrics(&self) -> Value {
        // TODO: Integrate with actual cache system
        json!({
            "l1_cache": {
                "type": "moka",
                "max_capacity": 2000,
                "ttl_minutes": 5,
                "entries": 0,
                "hits": 0,
                "misses": 0,
                "hit_rate": 0.0
            },
            "l2_cache": {
                "type": "redis",
                "connected": false,
                "ttl_hours": 1,
                "hits": 0,
                "misses": 0
            },
            "unified_stats": {
                "total_hits": 0,
                "total_misses": 0,
                "overall_hit_rate": 0.0
            }
        })
    }

    /// Get external APIs metrics (Layer 2)
    pub async fn get_external_api_metrics(&self) -> Value {
        // TODO: Integrate with actual external API service
        json!({
            "rate_limiting": {
                "requests_per_minute": 60,
                "current_usage": 0,
                "circuit_breaker_status": "closed"
            },
            "market_data_provider": {
                "btc_optimized_cache": true,
                "cache_hit_rate": 0.0,
                "api_calls_made": 0
            },
            "api_client": {
                "retry_logic_active": true,
                "timeout_seconds": 10,
                "failed_requests": 0
            }
        })
    }

    /// Get WebSocket service metrics (Layer 3)
    pub async fn get_websocket_metrics(&self) -> Value {
        // TODO: Integrate with actual WebSocket service
        json!({
            "connection_manager": {
                "active_connections": 0,
                "total_connections": 0,
                "connection_errors": 0
            },
            "broadcast_service": {
                "messages_sent": 0,
                "broadcast_failures": 0,
                "scheduled_broadcasts": 0
            },
            "heartbeat_manager": {
                "active_heartbeats": 0,
                "timeout_count": 0,
                "ping_interval_seconds": 30
            }
        })
    }

    /// Get health system metrics (Layer 4)
    pub async fn get_health_metrics(&self) -> Value {
        json!({
            "health_monitor": {
                "last_check": chrono::Utc::now().to_rfc3339(),
                "status": "healthy",
                "uptime_seconds": self.collection_start.elapsed().as_secs()
            },
            "performance_collector": {
                "metrics_collected": true,
                "request_tracking": true,
                "response_time_tracking": true
            },
            "connectivity_tester": {
                "external_services_tested": 3,
                "ssl_validation_active": true,
                "test_interval_seconds": 300
            }
        })
    }

    /// Get business logic metrics (Layer 5)
    pub async fn get_business_metrics(&self) -> Value {
        // TODO: Implement when business logic layers are extracted
        json!({
            "dashboard": {
                "status": "not_extracted",
                "pending": true
            },
            "crypto_reports": {
                "status": "not_extracted",
                "pending": true
            }
        })
    }

    /// Reset all custom metrics
    pub fn reset_custom_metrics(&mut self) {
        self.custom_metrics.clear();
    }

    /// Get metrics summary for quick status check
    pub async fn get_metrics_summary(&self) -> Value {
        json!({
            "system_healthy": true,
            "active_layers": 4,
            "pending_layers": 1,
            "total_features": 5,
            "monitoring_active": true,
            "uptime_seconds": self.collection_start.elapsed().as_secs(),
            "custom_metrics_count": self.custom_metrics.len()
        })
    }
}

impl Default for MetricsAggregator {
    fn default() -> Self {
        Self::new()
    }
}
