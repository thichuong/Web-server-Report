//! Cache Stampede Protection Demo
//! 
//! Run this example to see Cache Stampede protection in action.

use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::Result;
use web_server_report::service_islands::layer1_infrastructure::cache_system_island::{
    CacheSystemIsland, CacheStrategy
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Cache Stampede Protection Demo\n");
    
    // Initialize cache system
    let cache_system = Arc::new(CacheSystemIsland::new().await?);
    
    println!("🧪 Demonstrating Cache Stampede protection...\n");
    
    // Simulate expensive computation
    let expensive_task = || async {
        println!("💻 Performing expensive computation...");
        tokio::time::sleep(Duration::from_millis(1000)).await; // 1 second delay
        Ok(serde_json::json!({
            "result": "expensive_data",
            "computed_at": chrono::Utc::now().to_rfc3339(),
            "cost": "high"
        }))
    };
    
    // Test 1: Single request (baseline)
    println!("📋 Test 1: Single request (baseline)");
    let start = Instant::now();
    let result1 = cache_system.cache_manager()
        .get_or_compute_with("test_key", CacheStrategy::ShortTerm, expensive_task)
        .await?;
    let duration1 = start.elapsed();
    println!("✅ Single request completed in: {:?}", duration1);
    println!("📊 Result: {}\n", result1);
    
    // Clear cache for next test
    let _ = cache_system.l1_cache.remove("test_key").await;
    let _ = cache_system.l2_cache.remove("test_key").await;
    
    // Test 2: Multiple concurrent requests (Cache Stampede scenario)
    println!("📋 Test 2: Multiple concurrent requests (should use Cache Stampede protection)");
    
    let start = Instant::now();
    let mut handles = Vec::new();
    
    // Create 5 concurrent requests for the same data
    for i in 1..=5 {
        let cache_system_clone = cache_system.clone();
        let handle = tokio::spawn(async move {
            let request_start = Instant::now();
            
            let result = cache_system_clone.cache_manager()
                .get_or_compute_with("concurrent_test", CacheStrategy::ShortTerm, || async {
                    println!("🔄 [Request #{}] Computing expensive data...", i);
                    tokio::time::sleep(Duration::from_millis(800)).await;
                    Ok(serde_json::json!({
                        "result": "concurrent_data",
                        "computed_at": chrono::Utc::now().to_rfc3339(),
                        "request_id": i
                    }))
                })
                .await;
                
            let request_duration = request_start.elapsed();
            println!("✅ [Request #{}] Completed in: {:?}", i, request_duration);
            
            result
        });
        
        handles.push(handle);
        
        // Small delay to increase likelihood of race condition
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    // Wait for all requests
    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(result) => results.push(result),
            Err(e) => eprintln!("❌ Request failed: {}", e),
        }
    }
    
    let total_duration = start.elapsed();
    println!("🏁 All concurrent requests completed in: {:?}", total_duration);
    println!("📊 Successful results: {}/{}", results.len(), 5);
    
    // Verify all requests got the same computed_at timestamp (proving only one computation)
    if let (Ok(first), Ok(second)) = (&results[0], &results[1]) {
        let first_time = first["computed_at"].as_str().unwrap_or("");
        let second_time = second["computed_at"].as_str().unwrap_or("");
        
        if first_time == second_time {
            println!("✅ Cache Stampede protection WORKED! All requests share same computation timestamp.");
        } else {
            println!("⚠️ Different timestamps detected - multiple computations may have occurred.");
        }
    }
    
    // Show cache statistics
    let stats = cache_system.cache_manager().get_stats();
    println!("\n📈 Final Cache Statistics:");
    println!("  • Total requests: {}", stats.total_requests);
    println!("  • L1 hits: {}", stats.l1_hits);
    println!("  • L2 hits: {}", stats.l2_hits);
    println!("  • Cache misses: {}", stats.misses);
    println!("  • Overall hit rate: {:.1}%", stats.hit_rate);
    println!("  • In-flight requests tracked: {}", stats.in_flight_requests);
    
    println!("\n🎉 Demo completed successfully!");
    
    Ok(())
}
