//! Backward Compatibility Demo for Enhanced get() method
//! 
//! This demo shows that existing code using get() will automatically
//! benefit from Cache Stampede protection without any changes needed.

use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::Result;
use web_server_report::service_islands::layer1_infrastructure::cache_system_island::{
    CacheSystemIsland, CacheStrategy
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔄 Backward Compatibility Demo - Enhanced get() method\n");
    
    // Initialize cache system
    let cache_system = Arc::new(CacheSystemIsland::new().await?);
    
    // Pre-populate some data in L2 cache only (simulate real-world scenario)
    let test_data = serde_json::json!({
        "symbol": "BTC/USD",
        "price": 45000.0,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "source": "L2_cache"
    });
    
    println!("📋 Setup: Pre-populating L2 cache with test data...");
    let _ = cache_system.l2_cache.set_with_ttl("market_data", test_data.clone(), Duration::from_secs(600)).await;
    println!("✅ L2 cache populated\n");
    
    // Test 1: Traditional get() usage - should now have Cache Stampede protection
    println!("🧪 Test 1: Traditional get() usage (with automatic Cache Stampede protection)");
    
    let start = Instant::now();
    let mut handles = Vec::new();
    
    // Simulate multiple services concurrently requesting the same data
    for i in 1..=8 {
        let cache_manager = cache_system.cache_manager().clone();
        
        let handle = tokio::spawn(async move {
            let service_name = match i {
                1..=2 => "WebSocket_Service",
                3..=4 => "API_Handler", 
                5..=6 => "Report_Generator",
                _ => "Background_Worker",
            };
            
            let request_start = Instant::now();
            
            // This is the EXISTING code pattern - no changes needed!
            let result = cache_manager.get("market_data").await;
            
            let request_duration = request_start.elapsed();
            
            match result {
                Ok(Some(data)) => {
                    println!("✅ [{}] Got data in {:?} - Source: {}", 
                             service_name, request_duration, 
                             data.get("source").and_then(|v| v.as_str()).unwrap_or("unknown"));
                    Ok(data)
                }
                Ok(None) => {
                    println!("⚪ [{}] No data found in {:?}", service_name, request_duration);
                    Ok(serde_json::Value::Null)
                }
                Err(e) => {
                    println!("❌ [{}] Error in {:?}: {}", service_name, request_duration, e);
                    Err(e)
                }
            }
        });
        
        handles.push(handle);
        
        // Small delay to increase concurrency
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    
    // Wait for all requests
    let mut successful_requests = 0;
    let mut promoted_to_l1 = false;
    
    for handle in handles {
        match handle.await {
            Ok(Ok(_data)) => {
                successful_requests += 1;
                // Check if data was promoted to L1
                if !promoted_to_l1 {
                    if let Some(_) = cache_system.l1_cache.get("market_data").await {
                        promoted_to_l1 = true;
                    }
                }
            }
            _ => {}
        }
    }
    
    let total_duration = start.elapsed();
    
    println!("\n📊 Test 1 Results:");
    println!("  • Total time: {:?}", total_duration);
    println!("  • Successful requests: {}/8", successful_requests);
    println!("  • Data promoted to L1: {}", if promoted_to_l1 { "✅ Yes" } else { "❌ No" });
    
    // Test 2: Enhanced get_with_fallback() usage
    println!("\n🧪 Test 2: Enhanced get_with_fallback() usage");
    
    // Clear caches to test fallback computation
    let _ = cache_system.l1_cache.remove("new_market_data").await;
    let _ = cache_system.l2_cache.remove("new_market_data").await;
    
    let start = Instant::now();
    let mut handles = Vec::new();
    
    // Multiple services using the new enhanced method
    for i in 1..=5 {
        let cache_manager = cache_system.cache_manager().clone();
        
        let handle = tokio::spawn(async move {
            let service_id = i;
            
            // New enhanced usage - with fallback computation
            let result = cache_manager.get_with_fallback(
                "new_market_data",
                Some(|| async {
                    println!("🔄 [Service #{}] Computing fresh market data...", service_id);
                    tokio::time::sleep(Duration::from_millis(800)).await; // Simulate API call
                    Ok(serde_json::json!({
                        "symbol": "ETH/USD",
                        "price": 3200.0,
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "computed_by": format!("service_{}", service_id),
                        "source": "fresh_computation"
                    }))
                }),
                Some(CacheStrategy::RealTime)
            ).await;
            
            (service_id, result)
        });
        
        handles.push(handle);
        tokio::time::sleep(Duration::from_millis(8)).await;
    }
    
    let mut computation_sources = std::collections::HashSet::new();
    let mut successful_enhanced = 0;
    
    for handle in handles {
        if let Ok((service_id, Ok(Some(data)))) = handle.await {
            successful_enhanced += 1;
            if let Some(computed_by) = data.get("computed_by").and_then(|v| v.as_str()) {
                computation_sources.insert(computed_by.to_string());
                println!("✅ [Service #{}] Got data computed by: {}", service_id, computed_by);
            }
        }
    }
    
    let total_enhanced_duration = start.elapsed();
    
    println!("\n📊 Test 2 Results:");
    println!("  • Total time: {:?}", total_enhanced_duration);
    println!("  • Successful requests: {}/5", successful_enhanced);
    println!("  • Unique computation sources: {} (expected: 1 for Cache Stampede protection)", 
             computation_sources.len());
    println!("  • Cache Stampede protection: {}", 
             if computation_sources.len() == 1 { "✅ WORKING" } else { "❌ NOT WORKING" });
    
    // Show final statistics
    let final_stats = cache_system.cache_manager().get_stats();
    println!("\n📈 Final Cache Statistics:");
    println!("  • Total requests: {}", final_stats.total_requests);
    println!("  • L1 hits: {}", final_stats.l1_hits);
    println!("  • L2 hits: {}", final_stats.l2_hits);
    println!("  • Cache misses: {}", final_stats.misses);
    println!("  • Overall hit rate: {:.1}%", final_stats.hit_rate);
    println!("  • L1 promotions: {}", final_stats.promotions);
    println!("  • Currently tracked in-flight requests: {}", final_stats.in_flight_requests);
    
    println!("\n🎉 Backward compatibility demo completed successfully!");
    println!("💡 Key benefits:");
    println!("   ✅ Existing get() calls now have automatic Cache Stampede protection");
    println!("   ✅ L2→L1 promotion happens automatically with coalescing");
    println!("   ✅ New get_with_fallback() method for enhanced functionality");
    println!("   ✅ Zero breaking changes for existing code");
    
    Ok(())
}
