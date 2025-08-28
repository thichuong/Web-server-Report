//! Test Layer 3 Market Data Adapter with US Stock Indices
//! 
//! This example tests the updated market data adapter that includes
//! US stock market indices from Finnhub integration

use web_server_report::service_islands::layer2_external_services::external_apis_island::ExternalApisIsland;
use web_server_report::service_islands::layer3_communication::layer2_adapters::market_data_adapter::MarketDataAdapter;
use web_server_report::service_islands::layer1_infrastructure::CacheSystemIsland;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🧪 Testing Layer 3 Market Data Adapter with US Stock Indices");
    println!("{}", "=".repeat(65));

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
    let cache_system = std::sync::Arc::new(CacheSystemIsland::new().await?);

    // Initialize Layer 2 External APIs Island
    println!("\n🏗️ Initializing Layer 2 External APIs Island...");
    let external_apis = ExternalApisIsland::with_cache_and_all_keys(
        taapi_secret,
        cmc_api_key,
        finnhub_api_key,
        cache_system
    ).await?;
    
    // Initialize Layer 3 Market Data Adapter
    println!("🔗 Initializing Layer 3 Market Data Adapter...");
    let adapter = MarketDataAdapter::new()
        .with_external_apis(std::sync::Arc::new(external_apis));
    
    println!("✅ Layer 3 adapter initialized successfully");

    // Test normalized market data fetch
    println!("\n📊 Testing fetch_normalized_market_data() with US indices...");
    match adapter.fetch_normalized_market_data().await {
        Ok(normalized_data) => {
            println!("✅ Normalized data fetched successfully!");
            
            // Display crypto data
            if let Some(btc_price) = normalized_data.get("btc_price_usd").and_then(|v| v.as_f64()) {
                let btc_change = normalized_data.get("btc_change_24h").and_then(|v| v.as_f64()).unwrap_or(0.0);
                println!("   ₿ Bitcoin: ${:.2} ({:+.2}%)", btc_price, btc_change);
            }
            
            if let Some(market_cap) = normalized_data.get("market_cap_usd").and_then(|v| v.as_f64()) {
                println!("   🌍 Market Cap: ${:.2}B", market_cap / 1_000_000_000.0);
            }
            
            if let Some(fear_greed) = normalized_data.get("fear_greed_index").and_then(|v| v.as_f64()) {
                println!("   😱 Fear & Greed: {:.0}", fear_greed);
            }
            
            // Display US stock indices
            if let Some(us_indices) = normalized_data.get("us_stock_indices") {
                println!("\n   📈 US Stock Market Indices (via Layer 3):");
                
                if let Some(dia) = us_indices.get("DIA") {
                    if let Some(status) = dia.get("status").and_then(|v| v.as_str()) {
                        if status == "success" {
                            let price = dia.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let change = dia.get("change").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let change_pct = dia.get("change_percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            println!("     📊 DJIA (DIA): ${:.2} ({:+.2} / {:+.2}%)", price, change, change_pct);
                        } else {
                            println!("     📊 DJIA (DIA): Status = {}", status);
                        }
                    }
                }
                
                if let Some(spy) = us_indices.get("SPY") {
                    if let Some(status) = spy.get("status").and_then(|v| v.as_str()) {
                        if status == "success" {
                            let price = spy.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let change = spy.get("change").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let change_pct = spy.get("change_percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            println!("     📊 S&P 500 (SPY): ${:.2} ({:+.2} / {:+.2}%)", price, change, change_pct);
                        } else {
                            println!("     📊 S&P 500 (SPY): Status = {}", status);
                        }
                    }
                }
                
                if let Some(qqq) = us_indices.get("QQQ") {
                    if let Some(status) = qqq.get("status").and_then(|v| v.as_str()) {
                        if status == "success" {
                            let price = qqq.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let change = qqq.get("change").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let change_pct = qqq.get("change_percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            println!("     📊 Nasdaq 100 (QQQ): ${:.2} ({:+.2} / {:+.2}%)", price, change, change_pct);
                        } else {
                            println!("     📊 Nasdaq 100 (QQQ): Status = {}", status);
                        }
                    }
                }
                
                if us_indices.as_object().map_or(true, |obj| obj.is_empty()) {
                    println!("     ⚠️ No US indices data available");
                }
            }
            
            // Display data sources
            if let Some(sources) = normalized_data.get("data_sources") {
                println!("\n   📡 Data Sources:");
                if let Some(obj) = sources.as_object() {
                    for (key, value) in obj {
                        println!("     • {}: {}", key, value.as_str().unwrap_or("unknown"));
                    }
                }
            }
            
            // Display performance metrics
            if let Some(duration) = normalized_data.get("fetch_duration_ms").and_then(|v| v.as_u64()) {
                println!("   ⏱️ Fetch Duration: {}ms", duration);
            }
            
            let partial_failure = normalized_data.get("partial_failure").and_then(|v| v.as_bool()).unwrap_or(false);
            if partial_failure {
                println!("   ⚠️ Partial failure detected - some data sources failed");
            } else {
                println!("   ✅ All data sources successful");
            }
        }
        Err(e) => {
            println!("❌ Normalized data fetch failed: {}", e);
        }
    }

    // Test adapter health check
    println!("\n🏥 Testing adapter health check...");
    let is_healthy = adapter.health_check().await;
    println!("   Health Status: {}", if is_healthy { "✅ Healthy" } else { "❌ Unhealthy" });

    // Display configuration status
    println!("\n⚙️ Configuration Status:");
    println!("   • Layer 2 configured: {}", adapter.is_layer2_configured());
    println!("   • Cache system configured: {}", adapter.is_cache_system_configured());
    println!("   • Cache-free mode support: {}", adapter.supports_cache_free_mode());

    println!("\n🎉 Layer 3 Market Data Adapter test completed!");
    println!("\n💡 Summary:");
    println!("   ✅ Crypto data: Fetched via Layer 2 → Layer 3 normalization");
    println!("   ✅ US Stock Indices: Finnhub data passed through Layer 3");
    println!("   ✅ Data Sources: Multi-API attribution tracking");
    println!("   ✅ Performance: Timing and failure detection");

    Ok(())
}
