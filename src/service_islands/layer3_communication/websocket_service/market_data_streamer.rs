//! Market Data Streamer Component
//! 
//! This component streams real-time market data from Layer 2 External APIs
//! to connected WebSocket clients, following Service Islands Architecture.

use anyhow::Result;
use serde_json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::interval;

use crate::service_islands::layer2_external_services::external_apis_island::ExternalApisIsland;
use crate::service_islands::ServiceIslands;

/// Market Data Streamer
/// 
/// Streams real-time market data from External APIs to WebSocket clients.
/// This component bridges Layer 2 (External Services) with Layer 3 (Communication).
pub struct MarketDataStreamer {
    /// Reference to Layer 2 External APIs
    external_apis: Option<Arc<ExternalApisIsland>>,
    /// Reference to Service Islands for Layer 5 access
    service_islands: Option<Arc<ServiceIslands>>,
    /// Stream interval for updates
    update_interval: Duration,
    /// Active streaming flag
    is_streaming: std::sync::atomic::AtomicBool,
}

impl MarketDataStreamer {
    /// Create new Market Data Streamer without External APIs dependency
    pub fn new() -> Self {
        Self {
            external_apis: None,
            service_islands: None,
            update_interval: Duration::from_secs(5), // 5 seconds for faster updates
            is_streaming: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Set Service Islands reference for Layer 5 access (compatibility method)
    pub fn with_service_islands(mut self, service_islands: Arc<ServiceIslands>) -> Self {
        self.service_islands = Some(service_islands);
        self
    }
    
    /// Health check for market data streamer
    /// 
    /// Begins periodic streaming of market data using Layer 5 → Layer 3 → Layer 2 flow
    /// to match the same data source as HTTP API and WebSocket initial messages.
    pub async fn start_streaming(&self, broadcast_tx: broadcast::Sender<String>) -> Result<()> {
        if let Some(service_islands) = &self.service_islands {
            println!("🌊 Starting market data streaming using Layer 5 → Layer 3 → Layer 2 flow...");
            
            self.is_streaming.store(true, std::sync::atomic::Ordering::Relaxed);
            
            let service_islands_clone = service_islands.clone();
            let broadcast_tx_clone = broadcast_tx.clone();
            let update_interval = self.update_interval;
            
            // Spawn background task for streaming
            tokio::spawn(async move {
                let mut interval_timer = interval(update_interval);
                let mut consecutive_failures = 0;
                let max_consecutive_failures = 5;
                
                loop {
                    interval_timer.tick().await;
                    
                    // Check if broadcast channel is still active
                    if broadcast_tx_clone.receiver_count() == 0 {
                        println!("📡 No WebSocket receivers - continuing to stream for future connections");
                    }
                    
                    // 🔧 FIX: Use same Layer 5 function as HTTP API and WebSocket initial message
                    // This ensures all three messages use identical Layer 2 access pattern
                    match service_islands_clone.crypto_reports.fetch_realtime_market_data().await {
                        Ok(dashboard_data) => {
                            // Reset consecutive failures on success
                            consecutive_failures = 0;
                            
                            // 🔍 DEBUG: Log detailed dashboard data values
                            println!("🔍 [DEBUG] Dashboard data fetched successfully:");
                            if let Some(market_cap) = dashboard_data.get("market_cap_usd") {
                                println!("  💰 Market Cap: ${:?}", market_cap);
                            }
                            if let Some(volume) = dashboard_data.get("volume_24h_usd") {
                                println!("  📊 24h Volume: ${:?}", volume);
                            }
                            if let Some(btc_price) = dashboard_data.get("btc_price_usd") {
                                println!("  ₿ BTC Price: ${:?}", btc_price);
                            }
                            if let Some(btc_change) = dashboard_data.get("btc_change_24h") {
                                println!("  📈 BTC 24h Change: {:?}%", btc_change);
                            }
                            if let Some(fng) = dashboard_data.get("fng_value") {
                                println!("  😨 Fear & Greed Index: {:?}", fng);
                            }
                            if let Some(rsi) = dashboard_data.get("rsi_14") {
                                println!("  📈 RSI 14: {:?}", rsi);
                            }
                            if let Some(btc_dom) = dashboard_data.get("btc_market_cap_percentage") {
                                println!("  ₿ BTC Dominance: {:?}%", btc_dom);
                            }
                            if let Some(eth_dom) = dashboard_data.get("eth_market_cap_percentage") {
                                println!("  Ξ ETH Dominance: {:?}%", eth_dom);
                            }
                            
                            // Create WebSocket message
                            let ws_message = serde_json::json!({
                                "type": "dashboard_update",
                                "data": dashboard_data,
                                "timestamp": chrono::Utc::now().to_rfc3339(),
                                "source": "external_apis"
                            });
                            
                            // Broadcast to all WebSocket clients
                            let message_str = ws_message.to_string();
                            match broadcast_tx_clone.send(message_str) {
                                Ok(receiver_count) => {
                                    println!("📊 Dashboard data broadcasted to {} WebSocket clients", receiver_count);
                                }
                                Err(e) => {
                                    // This is expected when no clients are connected
                                    println!("� Broadcast ready - waiting for WebSocket connections ({})", e);
                                }
                            }
                        }
                        Err(e) => {
                            consecutive_failures += 1;
                            println!("⚠️ Failed to fetch dashboard data for streaming (attempt {}/{}): {}", 
                                consecutive_failures, max_consecutive_failures, e);
                            
                            // Only broadcast error after multiple failures to avoid spam
                            if consecutive_failures >= 3 {
                                let error_message = serde_json::json!({
                                    "type": "error",
                                    "message": "Temporary issue with real-time market data",
                                    "error": e.to_string(),
                                    "consecutive_failures": consecutive_failures,
                                    "timestamp": chrono::Utc::now().to_rfc3339()
                                });
                                
                                let _ = broadcast_tx_clone.send(error_message.to_string());
                            }
                            
                            // If too many failures, increase interval temporarily
                            if consecutive_failures >= max_consecutive_failures {
                                println!("⚠️ Too many consecutive failures - taking a break");
                                tokio::time::sleep(Duration::from_secs(60)).await; // 1 minute break
                                consecutive_failures = 0; // Reset counter after break
                            }
                        }
                    }
                }
            });
            
            println!("✅ Market data streaming started successfully");
            Ok(())
        } else {
            println!("⚠️ Service Islands not configured - market data streaming disabled");
            Ok(())
        }
    }
    
    /// Start BTC price streaming (higher frequency) - OPTIMIZED
    /// 
    /// Streams BTC price updates using unified data flow when possible,
    /// with fallback to direct Layer 2 access for backward compatibility.
    #[allow(dead_code)]
    pub async fn start_btc_streaming(&self, broadcast_tx: broadcast::Sender<String>) -> Result<()> {
        if let Some(external_apis) = &self.external_apis {
            println!("₿ Starting BTC price streaming from Layer 2 External APIs...");
            
            let external_apis_clone = external_apis.clone();
            let service_islands_clone = self.service_islands.clone();
            let tx = broadcast_tx;
            
            // BTC updates every 10 seconds for more real-time feeling
            let btc_interval = Duration::from_secs(10);
            
            tokio::spawn(async move {
                let mut interval_timer = interval(btc_interval);
                let mut consecutive_failures = 0;
                let _max_consecutive_failures = 3;
                
                loop {
                    interval_timer.tick().await;
                    
                    // Use Layer 5 → Layer 3 → Layer 2 unified flow for consistency
                    if let Some(service_islands_clone) = &service_islands_clone {
                        match service_islands_clone.websocket_service.fetch_market_data_for_layer5().await {
                            Ok(market_data) => {
                                // Reset failures on success
                                consecutive_failures = 0;
                                
                                // 🔍 DEBUG: Log market data including new fields
                                println!("🔍 [DEBUG] Market data fetched via unified flow:");
                                if let Some(price) = market_data.get("btc_price_usd") {
                                    println!("  ₿ Current BTC Price: ${:?}", price);
                                }
                                if let Some(change) = market_data.get("btc_change_24h") {
                                    println!("  📈 BTC 24h Change: {:?}%", change);
                                }
                                if let Some(market_cap) = market_data.get("market_cap_usd") {
                                    println!("  💰 Market Cap: ${:?}", market_cap);
                                }
                                if let Some(mc_change) = market_data.get("market_cap_change_percentage_24h_usd") {
                                    println!("  📊 Market Cap Change 24h: {:?}%", mc_change);
                                }
                                if let Some(btc_dom) = market_data.get("btc_market_cap_percentage") {
                                    println!("  ₿ BTC Dominance: {:?}%", btc_dom);
                                }
                                if let Some(eth_dom) = market_data.get("eth_market_cap_percentage") {
                                    println!("  Ξ ETH Dominance: {:?}%", eth_dom);
                                }
                                
                                // Broadcast to WebSocket clients
                                let message = serde_json::json!({
                                    "type": "price_update",
                                    "data": market_data
                                });
                                
                                if let Err(e) = tx.send(message.to_string()) {
                                    println!("⚠️ Failed to broadcast price update: {}", e);
                                    break;
                                }
                                
                                println!("✅ Market data broadcasted to WebSocket clients");
                            }
                            Err(e) => {
                                consecutive_failures += 1;
                                println!("❌ Market data fetch failed (attempt {}): {}", consecutive_failures, e);
                                
                                // Exponential backoff on failures
                                if consecutive_failures >= 3 {
                                    println!("⚠️ Too many consecutive failures, stopping streaming");
                                    break;
                                }
                            }
                        }
                    } else {
                        // Fallback: Use dashboard summary instead of deprecated fetch_btc_price()
                        match external_apis_clone.fetch_dashboard_summary_v2().await {
                            Ok(dashboard_data) => {
                                // Reset failures on success
                                consecutive_failures = 0;
                                
                                // Extract BTC data from dashboard summary
                                let btc_price = dashboard_data.get("btc_price_usd").cloned().unwrap_or(serde_json::Value::Null);
                                let btc_change = dashboard_data.get("btc_change_24h").cloned().unwrap_or(serde_json::Value::Null);
                                
                                // 🔍 DEBUG: Log BTC price data
                                println!("🔍 [DEBUG] BTC price data from dashboard summary (fallback):");
                                if let Some(price) = dashboard_data.get("btc_price_usd") {
                                    println!("  ₿ Current BTC Price: ${:?}", price);
                                }
                                if let Some(change) = dashboard_data.get("btc_change_24h") {
                                    println!("  📈 24h Change: {:?}%", change);
                                }
                                
                                // Create focused BTC message from dashboard data
                                let ws_message = serde_json::json!({
                                    "type": "price_update",
                                    "timestamp": chrono::Utc::now().to_rfc3339(),
                                    "data": {
                                        "btc_price_usd": btc_price,
                                        "btc_change_24h": btc_change,
                                        "source": "dashboard_aggregated"
                                    },
                                    "source": "dashboard_summary_fallback"
                                });
                                
                                if let Err(e) = tx.send(ws_message.to_string()) {
                                    println!("⚠️ Failed to broadcast price update: {}", e);
                                    break;
                                }
                                
                                println!("✅ BTC price broadcasted to WebSocket clients (fallback)");
                            }
                            Err(e) => {
                                consecutive_failures += 1;
                                println!("❌ BTC price fetch failed (attempt {}): {}", consecutive_failures, e);
                                
                                if consecutive_failures >= 3 {
                                    println!("⚠️ Too many consecutive failures, stopping streaming");
                                    break;
                                }
                            }
                        }
                    }
                }
            });
            
            println!("✅ BTC price streaming started successfully");
            Ok(())
        } else {
            println!("⚠️ External APIs not configured - BTC price streaming disabled");
            Ok(())
        }
    }
    
    /// Health check for market data streamer
    /// 
    /// Improved health check that's more tolerant of temporary API issues.
    pub async fn health_check(&self) -> bool {
        if let Some(external_apis) = &self.external_apis {
            match external_apis.health_check().await {
                Ok(_) => {
                    println!("  ✅ Market Data Streamer - External APIs healthy");
                    true
                }
                Err(e) => {
                    // Check if this is just a rate limit or circuit breaker issue
                    let error_msg = e.to_string();
                    if error_msg.contains("429") || error_msg.contains("Circuit breaker") || error_msg.contains("rate limit") {
                        println!("  ⚠️ Market Data Streamer - External APIs rate limited (still functional)");
                        true // Consider rate limiting as "healthy" since it's temporary
                    } else {
                        println!("  ❌ Market Data Streamer - External APIs unhealthy: {}", e);
                        false
                    }
                }
            }
        } else {
            println!("  ⚠️ Market Data Streamer - External APIs not configured (test mode)");
            true // Not an error, just not configured
        }
    }
}
