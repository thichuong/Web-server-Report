//! Dashboard Data Service
//! 
//! Layer 3 data communication service for dashboard functionality.
//! Handles caching and data operations for dashboard pages, isolating business logic
//! from infrastructure concerns.

use std::sync::Arc;

// Import from current state - will be refactored when lower layers are implemented
use crate::state::AppState;

/// Dashboard Data Service
/// 
/// Layer 3 service responsible for all dashboard data operations.
/// Acts as the communication layer between business logic and infrastructure.
#[derive(Clone)]
pub struct DashboardDataService {
    // Service state will be added here when needed
}

impl DashboardDataService {
    /// Create a new DashboardDataService
    pub fn new() -> Self {
        Self {
            // Initialize service
        }
    }

    /// Get cached rendered homepage HTML with compression
    /// 
    /// Checks cache for pre-rendered and compressed homepage HTML
    pub async fn get_rendered_homepage_compressed(
        &self,
        state: &Arc<AppState>,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = "dashboard_homepage_compressed";
        
        // Try cache first if available (L1 then L2 fallback with promotion) - OPTIMIZED
        if let Some(ref cache_system) = state.cache_system {
            match cache_system.cache_manager.get(cache_key).await {
                Ok(Some(cached_data)) => {
                    match serde_json::from_value::<Vec<u8>>(cached_data) {
                        Ok(cached_compressed) => {
                            println!("🔥 DashboardDataService: L1 Cache HIT for compressed homepage");
                            return Ok(Some(cached_compressed));
                        }
                        Err(e) => {
                            println!("⚠️ DashboardDataService: L1 Cache deserialization error: {}", e);
                        }
                    }
                }
                Ok(None) => {
                    println!("🔍 DashboardDataService: L1 Cache MISS for homepage");
                }
                Err(e) => {
                    println!("⚠️ DashboardDataService: L1 Cache access error: {}", e);
                }
            }
        }
        
        Ok(None)
    }

    /// Cache rendered homepage HTML with compression
    /// 
    /// Stores pre-rendered and compressed homepage HTML in cache
    pub async fn cache_rendered_homepage_compressed(
        &self,
        state: &Arc<AppState>,
        compressed_data: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = "dashboard_homepage_compressed";
        
        // Cache the compressed data for 5 minutes in both L1 and L2
        if let Some(ref cache_system) = state.cache_system {
            if let Ok(compressed_json) = serde_json::to_value(&compressed_data) {
                match cache_system.cache_manager.set_with_strategy(
                    cache_key, 
                    compressed_json,
                    crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::ShortTerm // 5 minutes
                ).await {
                    Ok(_) => {
                        let size_kb = compressed_data.len() / 1024;
                        println!("💾 DashboardDataService: Cached compressed homepage ({}KB) for 5 minutes", size_kb);
                    }
                    Err(e) => println!("⚠️ DashboardDataService: L1 Cache set error: {}", e),
                }
            }
        }
        
        Ok(())
    }

    /// Health check for dashboard data service
    pub async fn health_check(&self) -> bool {
        // Verify service is functioning properly
        true // Will implement actual health checks
    }
}
