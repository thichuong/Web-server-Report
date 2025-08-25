//! Cache Stampede Protection Test
//! 
//! This module demonstrates and tests the Cache Stampede protection
//! capabilities of our cache system.

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};
use anyhow::Result;
use serde_json;
use tokio::time::sleep;
use super::{CacheSystemIsland, CacheStrategy};

/// Counter to track how many times the expensive computation is called
static COMPUTATION_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Simulate an expensive computation (e.g., database query, API call)
async fn expensive_computation(key: &str, delay_ms: u64) -> Result<serde_json::Value> {
    let call_number = COMPUTATION_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    
    println!("🔄 [Call #{}] Starting expensive computation for key: '{}' ({}ms delay)", 
             call_number, key, delay_ms);
    
    // Simulate expensive work
    sleep(Duration::from_millis(delay_ms)).await;
    
    let result = serde_json::json!({
        "key": key,
        "computed_at": chrono::Utc::now().to_rfc3339(),
        "computation_number": call_number,
        "delay_ms": delay_ms
    });
    
    println!("✅ [Call #{}] Completed expensive computation for key: '{}'", 
             call_number, key);
    
    Ok(result)
}

/// Test Cache Stampede protection with concurrent requests
pub async fn test_cache_stampede_protection() -> Result<()> {
    println!("\n🧪 Testing Cache Stampede Protection...\n");
    
    // Reset computation counter
    COMPUTATION_COUNTER.store(0, Ordering::Relaxed);
    
    // Initialize cache system
    let cache_system = Arc::new(CacheSystemIsland::new().await?);
    
    // Test parameters
    let key = "test_expensive_data";
    let num_concurrent_requests = 10;
    let computation_delay = 1000; // 1 second
    
    println!("📋 Test Parameters:");
    println!("  • Key: '{}'", key);
    println!("  • Concurrent requests: {}", num_concurrent_requests);
    println!("  • Computation delay: {}ms", computation_delay);
    println!("  • Expected computations: 1 (Cache Stampede protection working)");
    println!();
    
    let start_time = Instant::now();
    
    // Create multiple concurrent requests for the same key
    let mut handles = Vec::new();
    
    for i in 0..num_concurrent_requests {
        let cache_system_clone = cache_system.clone();
        let key_clone = key.to_string();
        
        let handle = tokio::spawn(async move {
            let request_id = i + 1;
            println!("🚀 [Request #{}] Starting...", request_id);
            
            let start = Instant::now();
            
            // Use the Cache Stampede protected method
            let result = cache_system_clone.cache_manager().get_or_compute_with(
                &key_clone,
                CacheStrategy::ShortTerm,
                || async {
                    expensive_computation(&key_clone, computation_delay).await
                }
            ).await;
            
            let duration = start.elapsed();
            
            match result {
                Ok(value) => {
                    let computation_number = value["computation_number"].as_u64().unwrap_or(0);
                    println!("✅ [Request #{}] Completed in {:?} - got data from computation #{}", 
                             request_id, duration, computation_number);
                    (request_id, Ok(value), duration)
                }
                Err(ref e) => {
                    println!("❌ [Request #{}] Failed in {:?}: {}", 
                             request_id, duration, e);
                    (request_id, Err(anyhow::anyhow!("Request failed: {}", e)), duration)
                }
            }
        });
        
        handles.push(handle);
        
        // Small stagger to make the race condition more likely
        sleep(Duration::from_millis(10)).await;
    }
    
    // Wait for all requests to complete
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    
    let total_duration = start_time.elapsed();
    let total_computations = COMPUTATION_COUNTER.load(Ordering::Relaxed);
    
    // Analyze results
    println!("\n📊 Test Results:");
    println!("  • Total time: {:?}", total_duration);
    println!("  • Total computations: {} (expected: 1)", total_computations);
    println!("  • Successful requests: {}/{}", 
             results.iter().filter(|(_, r, _)| r.is_ok()).count(), 
             num_concurrent_requests);
    
    // Check if Cache Stampede protection worked
    if total_computations == 1 {
        println!("  ✅ Cache Stampede protection PASSED! Only 1 computation executed.");
    } else {
        println!("  ❌ Cache Stampede protection FAILED! {} computations executed.", total_computations);
    }
    
    // Verify all requests got the same data
    let successful_results: Vec<_> = results.iter()
        .filter_map(|(_, r, _)| r.as_ref().ok())
        .collect();
    
    if !successful_results.is_empty() {
        let first_computation_number = successful_results[0]["computation_number"].as_u64().unwrap_or(0);
        let all_same = successful_results.iter()
            .all(|r| r["computation_number"].as_u64().unwrap_or(0) == first_computation_number);
        
        if all_same {
            println!("  ✅ Data consistency PASSED! All requests got data from computation #{}.", first_computation_number);
        } else {
            println!("  ❌ Data consistency FAILED! Requests got data from different computations.");
        }
    }
    
    // Show cache statistics
    let stats = cache_system.cache_manager().get_stats();
    println!("\n📈 Cache Statistics:");
    println!("  • Total requests: {}", stats.total_requests);
    println!("  • L1 hits: {}", stats.l1_hits);
    println!("  • L2 hits: {}", stats.l2_hits);
    println!("  • Cache misses: {}", stats.misses);
    println!("  • Hit rate: {:.1}%", stats.hit_rate);
    println!("  • Promotions: {}", stats.promotions);
    println!("  • In-flight requests: {}", stats.in_flight_requests);
    
    println!("\n🎉 Cache Stampede protection test completed!\n");
    
    Ok(())
}

/// Test L1 Cache Stampede protection using Moka's get_with
pub async fn test_l1_cache_stampede() -> Result<()> {
    println!("\n🧪 Testing L1 Cache Stampede Protection (Moka get_with)...\n");
    
    // Reset computation counter
    COMPUTATION_COUNTER.store(0, Ordering::Relaxed);
    
    // Initialize cache system
    let cache_system = Arc::new(CacheSystemIsland::new().await?);
    
    let key = "l1_test_data";
    let num_concurrent_requests = 5;
    let computation_delay = 500; // 500ms
    
    println!("📋 L1 Test Parameters:");
    println!("  • Key: '{}'", key);
    println!("  • Concurrent requests: {}", num_concurrent_requests);
    println!("  • Computation delay: {}ms", computation_delay);
    println!();
    
    let start_time = Instant::now();
    
    // Create concurrent requests
    let mut handles = Vec::new();
    
    for i in 0..num_concurrent_requests {
        let cache_system_clone = cache_system.clone();
        let key_clone = key.to_string();
        
        let handle = tokio::spawn(async move {
            let request_id = i + 1;
            
            // Use L1 cache's get_or_compute_with method directly
            let result = cache_system_clone.l1_cache.get_or_compute_with(
                &key_clone,
                || async {
                    expensive_computation(&key_clone, computation_delay).await
                }
            ).await;
            
            (request_id, result)
        });
        
        handles.push(handle);
        sleep(Duration::from_millis(5)).await; // Small stagger
    }
    
    // Collect results
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    
    let total_duration = start_time.elapsed();
    let total_computations = COMPUTATION_COUNTER.load(Ordering::Relaxed);
    
    println!("📊 L1 Test Results:");
    println!("  • Total time: {:?}", total_duration);
    println!("  • Total computations: {} (expected: 1)", total_computations);
    println!("  • L1 Cache Stampede protection: {}", 
             if total_computations == 1 { "✅ PASSED" } else { "❌ FAILED" });
    
    // Show L1 cache stats
    let l1_stats = cache_system.l1_cache.get_stats();
    println!("\n📈 L1 Cache Statistics:");
    println!("  • Hits: {}", l1_stats.hits);
    println!("  • Misses: {}", l1_stats.misses);
    println!("  • Sets: {}", l1_stats.sets);
    println!("  • Cache size: {}", l1_stats.size);
    
    Ok(())
}
