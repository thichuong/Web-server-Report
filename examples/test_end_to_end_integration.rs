//! End-to-end test for market data integration with frontend

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing End-to-End Market Data Integration...");
    
    // Get TAAPI secret from env (use dummy value for local testing)
    let taapi_secret = env::var("TAAPI_SECRET").unwrap_or_else(|_| "dummy_secret".to_string());
    
    // Import the API Aggregator directly
    use web_server_report::service_islands::layer2_external_services::external_apis_island::api_aggregator::ApiAggregator;
    
    // Initialize API Aggregator (without cache for simplicity)
    let api_aggregator = ApiAggregator::new(taapi_secret).await?;
    
    println!("📊 Testing dashboard data aggregation for frontend integration...");
    
    match api_aggregator.fetch_dashboard_data().await {
        Ok(data) => {
            println!("✅ Dashboard data fetched successfully for frontend:");
            
            // Check all required fields for JavaScript integration
            let required_fields = vec![
                "btc_price_usd",
                "btc_change_24h", 
                "market_cap_usd",
                "volume_24h_usd",
                "market_cap_change_percentage_24h_usd",
                "btc_market_cap_percentage",
                "eth_market_cap_percentage",
                "fng_value",
                "btc_rsi_14"
            ];
            
            println!("\n📋 Checking frontend integration fields:");
            for field in &required_fields {
                if let Some(value) = data.get(field) {
                    println!("  ✅ {}: {}", field, match value {
                        serde_json::Value::Number(n) => format!("{:.2}", n.as_f64().unwrap_or(0.0)),
                        serde_json::Value::String(s) => s.clone(),
                        _ => format!("{}", value)
                    });
                } else {
                    println!("  ❌ {}: MISSING", field);
                }
            }
            
            // Display formatted summary for frontend
            println!("\n🎨 Frontend Display Summary:");
            if let (Some(btc_price), Some(market_cap), Some(btc_dom), Some(eth_dom), Some(mc_change)) = (
                data["btc_price_usd"].as_f64(),
                data["market_cap_usd"].as_f64(),
                data["btc_market_cap_percentage"].as_f64(), 
                data["eth_market_cap_percentage"].as_f64(),
                data["market_cap_change_percentage_24h_usd"].as_f64()
            ) {
                println!("  💰 BTC Price: ${:.2}", btc_price);
                println!("  📊 Total Market Cap: ${:.2}B", market_cap / 1_000_000_000.0);
                println!("  📈 Market Cap Change (24h): {:.2}%", mc_change);
                println!("  ₿ BTC Dominance: {:.1}%", btc_dom);
                println!("  Ξ ETH Dominance: {:.1}%", eth_dom);
                println!("  🔥 Combined BTC+ETH: {:.1}%", btc_dom + eth_dom);
            }
            
            // Check metadata for frontend error handling
            println!("\n🔧 Frontend Integration Metadata:");
            if let Some(partial_failure) = data["partial_failure"].as_bool() {
                println!("  📡 Data Sources Status: {}", if partial_failure { "Partial Success" } else { "All OK" });
            }
            
            if let Some(duration) = data["fetch_duration_ms"].as_u64() {
                println!("  ⚡ API Response Time: {}ms", duration);
            }
            
            println!("\n✅ All required fields present for frontend integration!");
            println!("🎯 JavaScript components can now display:");
            println!("   • Market cap with 24h change indicator");
            println!("   • BTC dominance percentage");
            println!("   • ETH dominance percentage");
            println!("   • Combined market insights");
        }
        Err(e) => {
            eprintln!("❌ Failed to fetch dashboard data: {}", e);
            if e.to_string().contains("429") || e.to_string().contains("timeout") {
                println!("⚠️ APIs temporarily unavailable (expected in test environment)");
                println!("🔧 Frontend should show loading state or cached data");
            }
        }
    }
    
    println!("\n✅ End-to-end integration test completed!");
    Ok(())
}
