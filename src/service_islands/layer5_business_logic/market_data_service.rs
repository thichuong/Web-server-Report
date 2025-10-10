//! Market Data Service - Layer 5 Business Logic
//! 
//! This module provides common market data functionality for all Layer 5 business logic islands.
//! It serves as a shared service layer for market data operations across different business domains.

use std::sync::Arc;
use crate::service_islands::layer3_communication::websocket_service::WebSocketServiceIsland;

/// Market Data Service for Layer 5
/// 
/// Provides common market data operations that can be used by any Layer 5 business logic island.
/// Maintains proper Service Islands Architecture by going through Layer 3 to access Layer 2.
pub struct MarketDataService {
    /// Layer 3 dependency: WebSocket Service for proper architecture flow
    websocket_service: Arc<WebSocketServiceIsland>,
}

impl MarketDataService {
    /// Create new Market Data Service with Layer 3 dependency
    pub fn new(websocket_service: Arc<WebSocketServiceIsland>) -> Self {
        Self {
            websocket_service,
        }
    }
    
    /// Health check for Market Data Service
    pub async fn health_check(&self) -> bool {
        // Simple health check - try to connect to Layer 3
        println!("🏥 Checking Layer 5 Market Data Service health...");
        
        // We can check if websocket service is available
        match self.websocket_service.health_check().await {
            Ok(_) => {
                println!("  ✅ Layer 5 Market Data Service - Layer 3 dependency healthy");
                true
            }
            Err(e) => {
                println!("  ❌ Layer 5 Market Data Service - Layer 3 dependency unhealthy: {}", e);
                false
            }
        }
    }
}

/// Standalone function for fetching market data when you have a WebSocket Service reference
/// 
/// This function can be used by any Layer 5 component that has access to a WebSocket Service.
/// It's a convenience function that doesn't require creating a MarketDataService instance.
pub async fn fetch_realtime_market_data(
    websocket_service: &WebSocketServiceIsland
) -> Result<serde_json::Value, anyhow::Error> {
    println!("🔄 Layer 5 standalone function requesting market data via Layer 3 (strict architecture)...");
    
    match websocket_service.fetch_market_data(false).await {
        Ok(market_data) => {
            println!("✅ Layer 5 standalone function received market data via Layer 3 → Layer 2 successfully");
            
            // // 🔍 DEBUG: Log market data received by Layer 5
            // if let Some(btc_price) = market_data.get("btc_price_usd") {
            //     println!("  🔍 [Layer 5 standalone] BTC Price received: ${:?}", btc_price);
            // }
            // if let Some(market_cap) = market_data.get("market_cap_usd") {
            //     println!("  🔍 [Layer 5 standalone] Market Cap received: ${:?}", market_cap);
            // }
            // if let Some(mc_change) = market_data.get("market_cap_change_percentage_24h_usd") {
            //     println!("  🔍 [Layer 5 standalone] Market Cap Change 24h received: {:?}%", mc_change);
            // }
            // if let Some(btc_dom) = market_data.get("btc_market_cap_percentage") {
            //     println!("  🔍 [Layer 5 standalone] BTC Dominance received: {:?}%", btc_dom);
            // }
            // if let Some(eth_dom) = market_data.get("eth_market_cap_percentage") {
            //     println!("  🔍 [Layer 5 standalone] ETH Dominance received: {:?}%", eth_dom);
            // }
            // if let Some(fng) = market_data.get("fng_value") {
            //     println!("  🔍 [Layer 5 standalone] Fear & Greed received: {:?}", fng);
            // }
            // if let Some(us_indices) = market_data.get("us_stock_indices") {
            //     println!("  🔍 [Layer 5 standalone] US Stock Indices received: {:?} symbols", 
            //         us_indices.as_object().map_or(0, |obj| obj.len()));
            // }
            
            // ✅ DIRECT USE: Return Layer 3 normalized data directly (no redundant normalization)
            println!("🔧 [Layer 5 standalone] Using Layer 3 normalized data directly for better architecture");
            Ok(market_data)
        }
        Err(e) => {
            println!("❌ Layer 5 standalone function → Layer 3 → Layer 2 flow failed: {}", e);
            Err(anyhow::anyhow!("Standalone market data fetch failed: {}", e))
        }
    }
}
