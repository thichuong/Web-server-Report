//! Test Finnhub Stock Indices Integration
//! 
//! This example demonstrates how to fetch US stock market indices using Finnhub API

use web_server_report::service_islands::layer2_external_services::external_apis_island::api_aggregator::ApiAggregator;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("📈 Testing Finnhub US Stock Indices Integration");
    println!("{}", "=".repeat(60));

    // Get API keys from environment
    let taapi_secret = "test_taapi_secret".to_string();
    let cmc_api_key = std::env::var("CMC_API_KEY").ok();
    let finnhub_api_key = std::env::var("FINNHUB_API_KEY").ok();
    
    if let Some(ref key) = finnhub_api_key {
        if key.is_empty() || key == "your_finnhub_key" {
            println!("⚠️ Invalid Finnhub API key found");
        } else {
            println!("🔑 Finnhub API key found - testing stock indices");
        }
    } else {
        println!("⚠️ No Finnhub API key found");
        println!("   Set FINNHUB_API_KEY environment variable");
        println!("   Get your free API key at: https://finnhub.io/");
    }

    // Create API aggregator with all keys
    println!("\n📊 Initializing API Aggregator with all keys...");
    let aggregator = ApiAggregator::with_all_keys(
        taapi_secret,
        cmc_api_key,
        finnhub_api_key
    ).await?;

    println!("\n🔄 Testing full dashboard data aggregation (including US indices):");
    match aggregator.fetch_dashboard_data().await {
        Ok(dashboard_data) => {
            println!("   ✅ Dashboard data fetched successfully!");
            
            // Display crypto data
            if let Some(btc_price) = dashboard_data.get("btc_price_usd").and_then(|v| v.as_f64()) {
                let btc_change = dashboard_data.get("btc_change_24h").and_then(|v| v.as_f64()).unwrap_or(0.0);
                println!("   ₿ Bitcoin: ${:.2} ({:+.2}%)", btc_price, btc_change);
            }
            
            if let Some(market_cap) = dashboard_data.get("market_cap_usd").and_then(|v| v.as_f64()) {
                println!("   🌍 Total Market Cap: ${:.2}B", market_cap / 1_000_000_000.0);
            }
            
            // Display US stock indices
            if let Some(us_indices) = dashboard_data.get("us_stock_indices") {
                println!("\n   📈 US Stock Market Indices:");
                
                if let Some(dia) = us_indices.get("DIA") {
                    display_index(&dia, "   📊 DJIA (DIA ETF)");
                }
                
                if let Some(spy) = us_indices.get("SPY") {
                    display_index(&spy, "   📊 S&P 500 (SPY ETF)");
                }
                
                if let Some(qqq) = us_indices.get("QQQ") {
                    display_index(&qqq, "   📊 Nasdaq 100 (QQQ ETF)");
                }
                
                if us_indices.as_object().map_or(true, |obj| obj.is_empty()) {
                    println!("   ⚠️ No US indices data available (check Finnhub API key)");
                }
            }
            
            // Display data sources
            if let Some(sources) = dashboard_data.get("data_sources") {
                println!("\n   📡 Data Sources:");
                if let Some(obj) = sources.as_object() {
                    for (key, value) in obj {
                        println!("     • {}: {}", key, value.as_str().unwrap_or("unknown"));
                    }
                }
            }
            
            // Display timing info
            if let Some(duration) = dashboard_data.get("fetch_duration_ms").and_then(|v| v.as_u64()) {
                println!("   ⏱️ Total fetch time: {}ms", duration);
            }
            
            let partial_failure = dashboard_data.get("partial_failure").and_then(|v| v.as_bool()).unwrap_or(false);
            if partial_failure {
                println!("   ⚠️ Some data sources failed - check API keys and connectivity");
            } else {
                println!("   ✅ All data sources successful!");
            }
        }
        Err(e) => {
            println!("   ❌ Dashboard data fetch failed: {}", e);
        }
    }

    println!("\n🎯 Integration Summary:");
    println!("   ✅ Crypto data: CoinGecko + CoinMarketCap fallback");
    println!("   ✅ Technical indicators: TAAPI.io");
    println!("   ✅ Fear & Greed: Alternative.me");
    println!("   ✅ US Stock Indices: Finnhub");
    
    println!("\n💡 API Key Requirements:");
    println!("   • TAAPI_SECRET (required for RSI)");
    println!("   • CMC_API_KEY (optional for crypto fallback)");
    println!("   • FINNHUB_API_KEY (required for US stock indices)");

    println!("\n🎉 Finnhub integration test completed!");

    Ok(())
}

fn display_index(index_data: &serde_json::Value, label: &str) {
    let name = index_data.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
    let price = index_data.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let change = index_data.get("change").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let change_percent = index_data.get("change_percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let status = index_data.get("status").and_then(|v| v.as_str()).unwrap_or("unknown");
    
    if status == "success" && price > 0.0 {
        let change_symbol = if change >= 0.0 { "+" } else { "" };
        println!("{}: {} - {:.2} ({}${:.2} / {}{:.2}%)", 
                 label, name, price, change_symbol, change, change_symbol, change_percent);
    } else {
        println!("{}: {} - Status: {}", label, name, status);
    }
}
