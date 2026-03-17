//! Dashboard Data Service
//!
//! Layer 3 data communication service for dashboard functionality.
//! Handles caching and data operations for dashboard pages, isolating business logic
//! from infrastructure concerns.

use std::sync::Arc;
use tracing::info;

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

impl Default for DashboardDataService {
    fn default() -> Self {
        Self::new()
    }
}

impl DashboardDataService {
    /// Create a new `DashboardDataService`
    #[must_use]
    pub fn new() -> Self {
        Self {
            // Initialize service
        }
    }

    /// Get cached rendered homepage HTML with compression
    ///
    /// Checks cache for pre-rendered and compressed homepage HTML
    ///
    /// # Errors
    ///
    /// Returns error if cache retrieval fails or deserialization fails
    /// Get cached rendered homepage HTML with compression
    ///
    /// Checks cache for pre-rendered and compressed homepage HTML
    ///
    /// # Errors
    ///
    /// Returns error if cache retrieval fails or deserialization fails
    /// Get cached rendered homepage HTML with compression
    ///
    /// Checks cache for pre-rendered and compressed homepage HTML
    ///
    /// # Errors
    ///
    /// Returns error if cache retrieval fails or deserialization fails
    pub async fn get_rendered_homepage_compressed(
        &self,
        state: &Arc<AppState>,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = "dashboard_homepage_compressed";
        let cache_manager = &state.cache_manager;

        if let Ok(Some(cached_value)) = cache_manager.get(cache_key).await {
            // Try to parse as Vec<u8> (Legacy JSON)
            if let Ok(compressed_bytes) = serde_json::from_slice::<Vec<u8>>(&cached_value) {
                info!("🔥 DashboardDataService: Cache HIT (Legacy) for compressed homepage");
                return Ok(Some(compressed_bytes));
            }

            // New way: raw bytes
            info!("🔥 DashboardDataService: Cache HIT for compressed homepage");
            return Ok(Some(cached_value.to_vec()));
        }

        info!("🔍 DashboardDataService: Cache MISS for homepage");
        Ok(None)
    }

    /// Cache rendered homepage HTML with compression
    ///
    /// Stores pre-rendered and compressed homepage HTML in cache
    ///
    /// # Errors
    ///
    /// Returns error if cache storage fails
    pub async fn cache_rendered_homepage_compressed(
        &self,
        state: &Arc<AppState>,
        compressed_data: &[u8],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = "dashboard_homepage_compressed";

        // Cache the compressed data for 15 minutes in both L1 and L2
        let cache_manager = &state.cache_manager;
        let compressed_bytes = multi_tier_cache::Bytes::from(compressed_data.to_vec());
        let result = cache_manager
            .set_with_strategy(
                cache_key,
                compressed_bytes,
                multi_tier_cache::CacheStrategy::ShortTerm, // 5 minutes (ShortTerm is actually 5 mins, architected as 15 in docs but 5 in code)
            )
            .await;
        
        match result {
            Ok(()) => {
                let size_kb = compressed_data.len() / 1024;
                info!(
                    "💾 DashboardDataService: Cached compressed homepage ({}KB) successfully",
                    size_kb
                );
            }
            Err(e) => info!("⚠️ DashboardDataService: Cache set error: {}", e),
        }

        Ok(())
    }

    /// Health check for dashboard data service
    #[must_use]
    pub fn health_check(&self) -> bool {
        // Verify service is functioning properly
        true // Will implement actual health checks
    }
}
