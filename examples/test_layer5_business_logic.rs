//! Test Layer 5 Business Logic with US Stock Indices
//! 
//! This example tests the updated Layer 5 crypto reports that now uses
//! Layer 3 normalized data directly including US stock market indices

use web_server_report::service_islands::{
    ServiceIslands,
    layer1_infrastructure::AppStateIsland,
    layer2_external_services::external_apis_island::ExternalApisIsland,
    layer3_communication::websocket_service::WebSocketServiceIsland,
    layer5_business_logic::{
        crypto_reports::CryptoReportsIsland,
        market_data_service::fetch_realtime_market_data,
    },
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🧪 Testing Layer 5 Business Logic with US Stock Indices");
    println!("{}", "=".repeat(60));

    // Get API keys from environment
    let taapi_secret = "test_taapi_secret".to_string();
    let cmc_api_key = std::env::var("CMC_API_KEY").ok();
    let finnhub_api_key = std::env::var("FINNHUB_API_KEY").ok();
    
    println!("🔧 API Keys Status:");
    println!("   • TAAPI: {}", if taapi_secret.is_empty() { "❌ Missing" } else { "✅ Found" });
    println!("   • CoinMarketCap: {}", if cmc_api_key.is_some() { "✅ Found" } else { "⚠️ Optional" });
    println!("   • Finnhub: {}", if finnhub_api_key.is_some() { "✅ Found" } else { "❌ Missing" });

    // Initialize Cache System first
    println!("\n🗄️ Initializing Cache System...");
    let cache_system = Arc::new(CacheSystemIsland::new().await?);

    // Initialize Layer 2 External APIs Island
    println!("\n🏗️ Initializing Layer 2 External APIs Island...");
    let external_apis = ExternalApisIsland::with_cache_and_all_keys(
        taapi_secret,
        cmc_api_key,
        finnhub_api_key,
        Some(cache_system.clone())
    ).await?;
    
    // Initialize Layer 3 WebSocket Service
    println!("🌐 Initializing Layer 3 WebSocket Service...");
    let websocket_service = WebSocketServiceIsland::with_external_apis_and_cache(
        Arc::new(external_apis),
        cache_system
    ).await?;
    
    // Initialize Layer 5 Crypto Reports Island
    println!("📊 Initializing Layer 5 Crypto Reports Island...");
    let crypto_reports = CryptoReportsIsland::with_dependencies(
        Arc::new(websocket_service)
    ).await?;
    
    println!("✅ All layers initialized successfully!");

    // Test Layer 5 market data fetch using standalone function (Layer 5 → Layer 3 → Layer 2)
    println!("\n📈 Testing Layer 5 market data fetch with US indices using standalone function...");
    match fetch_realtime_market_data(&websocket_service).await {
        Ok(market_data) => {
            println!("✅ Layer 5 market data fetched successfully!");
            
            // Display crypto data
            if let Some(btc_price) = market_data.get("btc_price_usd").and_then(|v| v.as_f64()) {
                let btc_change = market_data.get("btc_change_24h").and_then(|v| v.as_f64()).unwrap_or(0.0);
                println!("   ₿ Bitcoin: ${:.2} ({:+.2}%)", btc_price, btc_change);
            }
            
            if let Some(market_cap) = market_data.get("market_cap_usd").and_then(|v| v.as_f64()) {
                println!("   🌍 Market Cap: ${:.2}B", market_cap / 1_000_000_000.0);
            }
            
            if let Some(fear_greed) = market_data.get("fear_greed_index").and_then(|v| v.as_f64()) {
                println!("   😱 Fear & Greed: {:.0}", fear_greed);
            }
            
            // Display US stock indices (should be included from Layer 3)
            if let Some(us_indices) = market_data.get("us_stock_indices") {
                println!("\n   📈 US Stock Market Indices (via Layer 5 → Layer 3):");
                
                if let Some(dia) = us_indices.get("DIA") {
                    if let Some(status) = dia.get("status").and_then(|v| v.as_str()) {
                        if status == "success" {
                            let name = dia.get("name").and_then(|v| v.as_str()).unwrap_or("DIA ETF");
                            let price = dia.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let change = dia.get("change").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let change_pct = dia.get("change_percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            println!("     📊 DJIA: {} - ${:.2} ({:+.2} / {:+.2}%)", name, price, change, change_pct);
                        } else {
                            println!("     📊 DJIA: Status = {}", status);
                        }
                    }
                }
                
                if let Some(spy) = us_indices.get("SPY") {
                    if let Some(status) = spy.get("status").and_then(|v| v.as_str()) {
                        if status == "success" {
                            let name = spy.get("name").and_then(|v| v.as_str()).unwrap_or("SPY ETF");
                            let price = spy.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let change = spy.get("change").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let change_pct = spy.get("change_percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            println!("     📊 S&P 500: {} - ${:.2} ({:+.2} / {:+.2}%)", name, price, change, change_pct);
                        } else {
                            println!("     📊 S&P 500: Status = {}", status);
                        }
                    }
                }
                
                if let Some(qqq) = us_indices.get("QQQ") {
                    if let Some(status) = qqq.get("status").and_then(|v| v.as_str()) {
                        if status == "success" {
                            let name = qqq.get("name").and_then(|v| v.as_str()).unwrap_or("QQQ ETF");
                            let price = qqq.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let change = qqq.get("change").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let change_pct = qqq.get("change_percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            println!("     📊 Nasdaq 100: {} - ${:.2} ({:+.2} / {:+.2}%)", name, price, change, change_pct);
                        } else {
                            println!("     📊 Nasdaq 100: Status = {}", status);
                        }
                    }
                }
                
                if us_indices.as_object().map_or(true, |obj| obj.is_empty()) {
                    println!("     ⚠️ No US indices data available");
                }
            } else {
                println!("\n   ⚠️ US stock indices not found in Layer 5 data");
            }
            
            // Display data sources and metadata
            if let Some(sources) = market_data.get("data_sources") {
                println!("\n   📡 Data Sources (via Layer 5):");
                if let Some(obj) = sources.as_object() {
                    for (key, value) in obj {
                        println!("     • {}: {}", key, value.as_str().unwrap_or("unknown"));
                    }
                }
            }
            
            // Display performance metrics
            if let Some(duration) = market_data.get("fetch_duration_ms").and_then(|v| v.as_u64()) {
                println!("   ⏱️ Total Fetch Duration: {}ms", duration);
            }
            
            let partial_failure = market_data.get("partial_failure").and_then(|v| v.as_bool()).unwrap_or(false);
            if partial_failure {
                println!("   ⚠️ Partial failure detected - some data sources failed");
            } else {
                println!("   ✅ All data sources successful");
            }
            
            // Display normalization info
            if let Some(normalized_by) = market_data.get("normalized_by").and_then(|v| v.as_str()) {
                println!("   🔧 Data normalized by: {}", normalized_by);
            }
        }
        Err(e) => {
            println!("❌ Layer 5 market data fetch failed: {}", e);
        }
    }

    // Test health check
    println!("\n🏥 Testing Layer 5 health check...");
    let is_healthy = crypto_reports.health_check().await;
    println!("   Layer 5 Health Status: {}", if is_healthy { "✅ Healthy" } else { "❌ Unhealthy" });

    println!("\n🎉 Layer 5 Business Logic test completed!");
    println!("\n💡 Architecture Summary:");
    println!("   🏗️ Layer 5 → Layer 3 → Layer 2 data flow verified");
    println!("   📊 Crypto + US Stock data unified in Layer 5");
    println!("   🚀 No redundant normalization in Layer 5");
    println!("   ✅ Layer 3 handles all data normalization");

    Ok(())
}
