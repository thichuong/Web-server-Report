//! Test CoinMarketCap Fallback Integration
//! 
//! This example demonstrates how the system automatically falls back to CoinMarketCap
//! when CoinGecko fails.

use web_server_report::service_islands::layer2_external_services::external_apis_island::market_data_api::MarketDataApi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🧪 Testing CoinMarketCap Fallback Integration");
    println!("{}", "=".repeat(60));

    // Test 1: MarketDataApi without CoinMarketCap key (CoinGecko only)
    println!("\n📊 Test 1: CoinGecko Only (no fallback)");
    let api_gecko_only = MarketDataApi::new("test_taapi_secret".to_string()).await?;
    
    match api_gecko_only.fetch_btc_price().await {
        Ok(data) => {
            println!("✅ Bitcoin price fetched successfully:");
            println!("   Source: {}", data.get("source").unwrap_or(&serde_json::json!("unknown")));
            println!("   Price: ${}", data.get("price_usd").unwrap_or(&serde_json::json!(0)));
            println!("   24h Change: {}%", data.get("change_24h").unwrap_or(&serde_json::json!(0)));
        }
        Err(e) => {
            println!("❌ Failed to fetch Bitcoin price: {}", e);
        }
    }

    // Test 2: MarketDataApi with CoinMarketCap key (with fallback)
    println!("\n📊 Test 2: CoinGecko + CoinMarketCap Fallback");
    
    // Try to get CMC API key from environment, use fake one for demo
    let cmc_key = std::env::var("CMC_API_KEY").unwrap_or_else(|_| {
        println!("⚠️ No CMC_API_KEY found in environment, using fake key for demo");
        "fake-cmc-key-for-demo".to_string()
    });
    
    let api_with_fallback = MarketDataApi::with_cmc_key(
        "test_taapi_secret".to_string(),
        Some(cmc_key)
    ).await?;

    // Test Bitcoin price with fallback
    println!("\n🔄 Testing Bitcoin price fetch (with fallback capability):");
    match api_with_fallback.fetch_btc_price().await {
        Ok(data) => {
            println!("✅ Bitcoin price fetched successfully:");
            println!("   Source: {}", data.get("source").unwrap_or(&serde_json::json!("unknown")));
            println!("   Price: ${}", data.get("price_usd").unwrap_or(&serde_json::json!(0)));
            println!("   24h Change: {}%", data.get("change_24h").unwrap_or(&serde_json::json!(0)));
        }
        Err(e) => {
            println!("❌ Failed to fetch Bitcoin price (both APIs failed): {}", e);
        }
    }

    // Test Global market data with fallback
    println!("\n🌍 Testing Global market data fetch (with fallback capability):");
    match api_with_fallback.fetch_global_data().await {
        Ok(data) => {
            println!("✅ Global market data fetched successfully:");
            println!("   Source: {}", data.get("source").unwrap_or(&serde_json::json!("unknown")));
            println!("   Market Cap: ${:.2}B", 
                data.get("market_cap").unwrap_or(&serde_json::json!(0)).as_f64().unwrap_or(0.0) / 1_000_000_000.0
            );
            println!("   24h Volume: ${:.2}B", 
                data.get("volume_24h").unwrap_or(&serde_json::json!(0)).as_f64().unwrap_or(0.0) / 1_000_000_000.0
            );
            println!("   BTC Dominance: {}%", data.get("btc_market_cap_percentage").unwrap_or(&serde_json::json!(0)));
        }
        Err(e) => {
            println!("❌ Failed to fetch global data (both APIs failed): {}", e);
        }
    }

    // Test 3: API Statistics
    println!("\n📈 API Statistics:");
    let stats = api_with_fallback.get_api_stats();
    println!("   Total API calls: {}", stats.get("total_api_calls").unwrap_or(&serde_json::json!(0)));
    println!("   Successful calls: {}", stats.get("successful_calls").unwrap_or(&serde_json::json!(0)));
    println!("   Failed calls: {}", stats.get("failed_calls").unwrap_or(&serde_json::json!(0)));
    println!("   Success rate: {}%", stats.get("success_rate").unwrap_or(&serde_json::json!(0)));
    println!("   Has CoinMarketCap key: {}", stats.get("has_coinmarketcap_key").unwrap_or(&serde_json::json!(false)));

    println!("\n🎉 Test completed!");
    println!("\nNote: If you see 'fake-cmc-key-for-demo', set the CMC_API_KEY environment variable");
    println!("with a real CoinMarketCap API key to test the actual fallback functionality.");

    Ok(())
}
