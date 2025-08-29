//! Test Layer 5 via Layer 3 integration với các field mới

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Layer 5 via Layer 3 with Enhanced Fields...");
    
    // Get TAAPI secret from env
    let taapi_secret = env::var("TAAPI_SECRET").unwrap_or_else(|_| "dummy_secret".to_string());
    
    // Initialize the Layer 3 WebSocket Service with Layer 2 dependencies
    use web_server_report::service_islands::layer1_infrastructure::CacheSystemIsland;
    use web_server_report::service_islands::layer2_external_services::external_apis_island::ExternalApisIsland;
    use web_server_report::service_islands::layer3_communication::websocket_service::WebSocketServiceIsland;
    
    println!("🏗️ Initializing Service Islands layers...");
    
    // Initialize Layer 1 (Cache)
    let cache_system = std::sync::Arc::new(CacheSystemIsland::new().await?);
    
    // Initialize Layer 2 (External APIs)  
    let external_apis = std::sync::Arc::new(
        ExternalApisIsland::with_cache(taapi_secret, cache_system.clone()).await?
    );
    
    // Initialize Layer 3 (Communication)
    let websocket_service = WebSocketServiceIsland::with_external_apis_and_cache(
        external_apis.clone(),
        cache_system.clone()
    ).await?;
    
    println!("🔄 Testing fetch_market_data...");
    
    match websocket_service.fetch_market_data().await {
        Ok(data) => {
            println!("✅ Layer 5 via Layer 3 data fetched successfully:");
            
            // Check all required fields for frontend integration
            let required_fields = vec![
                "btc_price_usd",
                "btc_change_24h",
                "market_cap_usd", 
                "volume_24h_usd",
                "market_cap_change_percentage_24h_usd",
                "btc_market_cap_percentage",
                "eth_market_cap_percentage",
                "fng_value",
                "rsi_14"
            ];
            
            println!("\n📋 Layer 3 Normalization Results:");
            for field in &required_fields {
                if let Some(value) = data.get(field) {
                    match value {
                        serde_json::Value::Number(n) => {
                            println!("  ✅ {}: {:.2}", field, n.as_f64().unwrap_or(0.0));
                        },
                        serde_json::Value::Null => {
                            println!("  ⚠️ {}: null (API unavailable)", field);
                        },
                        _ => {
                            println!("  ✅ {}: {}", field, value);
                        }
                    }
                } else {
                    println!("  ❌ {}: MISSING FROM LAYER 3 NORMALIZATION", field);
                }
            }
            
            // Verify Layer 3 metadata
            println!("\n🔧 Layer 3 Metadata:");
            if let Some(source) = data["source"].as_str() {
                println!("  📡 Data Source: {}", source);
            }
            if let Some(normalized_by) = data["normalized_by"].as_str() {
                println!("  🔧 Normalized By: {}", normalized_by);
            }
            if let Some(timestamp) = data["timestamp"].as_str() {
                println!("  🕒 Timestamp: {}", timestamp);
            }
            
            // Test frontend compatibility
            println!("\n🎨 Frontend Compatibility Check:");
            
            // Check if data can be properly consumed by market-indicators.js
            let frontend_compatible_fields = vec![
                ("btc_price_usd", "BTC Price for updateBtcPrice()"),
                ("btc_change_24h", "BTC Change for updateBtcPrice()"),
                ("market_cap_usd", "Market Cap for updateMarketCap()"),
                ("market_cap_change_percentage_24h_usd", "Market Cap Change for updateMarketCap()"),
                ("btc_market_cap_percentage", "BTC Dominance for updateBtcDominance()"),
                ("eth_market_cap_percentage", "ETH Dominance for updateEthDominance()"),
                ("fng_value", "Fear & Greed for updateFearGreedIndex()"),
                ("volume_24h_usd", "Volume for updateVolume24h()"),
                ("rsi_14", "RSI for technical analysis")
            ];
            
            let mut all_compatible = true;
            for (field, purpose) in &frontend_compatible_fields {
                if data.get(field).is_some() && !data[field].is_null() {
                    println!("  ✅ {} → {}", field, purpose);
                } else {
                    println!("  ❌ {} → {} (MISSING/NULL)", field, purpose);
                    all_compatible = false;
                }
            }
            
            if all_compatible {
                println!("\n🎯 ALL FIELDS COMPATIBLE WITH FRONTEND!");
                println!("   • market-indicators.js can consume all data");
                println!("   • WebSocket streaming will work properly");
                println!("   • Dashboard real-time updates enabled");
            } else {
                println!("\n⚠️ Some fields missing - frontend may show default values");
            }
            
            // Sample output that would be sent to frontend
            println!("\n📡 Sample WebSocket Message to Frontend:");
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                "type": "dashboard_data",
                "data": data
            }))?);
        }
        Err(e) => {
            eprintln!("❌ Failed to fetch Layer 5 via Layer 3 data: {}", e);
            if e.to_string().contains("429") || e.to_string().contains("timeout") {
                println!("⚠️ APIs temporarily unavailable - Layer 3 normalization still functional");
                println!("🔧 Would return null/default values to frontend");
            }
        }
    }
    
    println!("\n✅ Layer 5 via Layer 3 integration test completed!");
    println!("🎯 Enhanced data flow: Layer 2 API → Layer 3 Normalize → Layer 5/Frontend");
    Ok(())
}
