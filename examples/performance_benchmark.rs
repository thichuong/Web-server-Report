//! Performance Benchmark for Cache Stampede Protection
//! 
//! This demo tests performance under various load scenarios to demonstrate
//! the effectiveness of Cache Stampede protection in real-world conditions.

use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::Result;
use tokio::time::sleep;
use web_server_report::service_islands::layer1_infrastructure::cache_system_island::{
    CacheSystemIsland, CacheStrategy
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Cache Stampede Protection - Performance Benchmark\n");
    
    // Initialize cache system
    let cache_system = Arc::new(CacheSystemIsland::new().await?);
    
    println!("📋 Running multiple performance scenarios...\n");
    
    // Scenario 1: Low concurrency (2-4 requests)
    await_scenario_low_concurrency(cache_system.clone()).await?;
    
    // Clear cache
    clear_cache(&cache_system).await;
    
    // Scenario 2: Medium concurrency (10-20 requests) 
    await_scenario_medium_concurrency(cache_system.clone()).await?;
    
    // Clear cache
    clear_cache(&cache_system).await;
    
    // Scenario 3: High concurrency (50-100 requests)
    await_scenario_high_concurrency(cache_system.clone()).await?;
    
    // Clear cache  
    clear_cache(&cache_system).await;
    
    // Scenario 4: Extreme concurrency (200+ requests)
    await_scenario_extreme_concurrency(cache_system.clone()).await?;
    
    // Final statistics
    println!("\n📊 Final System Statistics:");
    let final_stats = cache_system.cache_manager().get_stats();
    println!("  • Total requests processed: {}", final_stats.total_requests);
    println!("  • Overall hit rate: {:.1}%", final_stats.hit_rate);
    println!("  • L1 hits: {}", final_stats.l1_hits);
    println!("  • L2 hits: {}", final_stats.l2_hits);
    println!("  • Cache misses: {}", final_stats.misses);
    println!("  • L1 promotions: {}", final_stats.promotions);
    
    println!("\n🎉 Performance benchmark completed!");
    
    Ok(())
}

async fn clear_cache(cache_system: &Arc<CacheSystemIsland>) {
    let _ = cache_system.l1_cache.remove("perf_test_data").await;
    let _ = cache_system.l2_cache.remove("perf_test_data").await;
    sleep(Duration::from_millis(100)).await; // Allow cleanup
}

async fn await_scenario_low_concurrency(cache_system: Arc<CacheSystemIsland>) -> Result<()> {
    println!("📋 Scenario 1: Low Concurrency (4 concurrent requests)");
    
    let start = Instant::now();
    let mut handles = Vec::new();
    
    for i in 1..=4 {
        let cache_system = cache_system.clone();
        let handle = tokio::spawn(async move {
            let request_start = Instant::now();
            
            let result = cache_system.cache_manager().get_or_compute_with(
                "perf_test_data",
                CacheStrategy::ShortTerm,
                || async {
                    sleep(Duration::from_millis(300)).await; // 300ms computation
                    Ok(serde_json::json!({
                        "scenario": "low_concurrency",
                        "computed_by": format!("request_{}", i),
                        "value": format!("data_value_{}", i),
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }))
                }
            ).await;
            
            let duration = request_start.elapsed();
            (i, result, duration)
        });
        
        handles.push(handle);
        sleep(Duration::from_millis(5)).await; // Small stagger
    }
    
    let mut successful = 0;
    let mut total_duration = Duration::ZERO;
    let mut computation_sources = std::collections::HashSet::new();
    
    for handle in handles {
        if let Ok((req_id, Ok(data), duration)) = handle.await {
            successful += 1;
            total_duration += duration;
            
            if let Some(computed_by) = data.get("computed_by").and_then(|v| v.as_str()) {
                computation_sources.insert(computed_by.to_string());
            }
            
            println!("  ✅ Request #{} completed in {:?}", req_id, duration);
        }
    }
    
    let scenario_duration = start.elapsed();
    
    println!("  📊 Results:");
    println!("    • Scenario duration: {:?}", scenario_duration);
    println!("    • Successful requests: {}/4", successful);
    println!("    • Unique computations: {} (expected: 1)", computation_sources.len());
    println!("    • Cache Stampede protection: {}", 
             if computation_sources.len() == 1 { "✅ WORKING" } else { "❌ FAILED" });
    println!();
    
    Ok(())
}

async fn await_scenario_medium_concurrency(cache_system: Arc<CacheSystemIsland>) -> Result<()> {
    println!("📋 Scenario 2: Medium Concurrency (15 concurrent requests)");
    
    let start = Instant::now();
    let mut handles = Vec::new();
    
    for i in 1..=15 {
        let cache_system = cache_system.clone();
        let handle = tokio::spawn(async move {
            let request_start = Instant::now();
            
            let result = cache_system.cache_manager().get_or_compute_with(
                "perf_test_data",
                CacheStrategy::ShortTerm,
                || async {
                    sleep(Duration::from_millis(500)).await; // 500ms computation
                    Ok(serde_json::json!({
                        "scenario": "medium_concurrency", 
                        "computed_by": format!("request_{}", i),
                        "value": format!("medium_data_{}", i),
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }))
                }
            ).await;
            
            let duration = request_start.elapsed();
            (i, result, duration)
        });
        
        handles.push(handle);
        sleep(Duration::from_millis(2)).await; // Very small stagger
    }
    
    let mut successful = 0;
    let mut computation_sources = std::collections::HashSet::new();
    
    for handle in handles {
        if let Ok((_req_id, Ok(data), _duration)) = handle.await {
            successful += 1;
            
            if let Some(computed_by) = data.get("computed_by").and_then(|v| v.as_str()) {
                computation_sources.insert(computed_by.to_string());
            }
        }
    }
    
    let scenario_duration = start.elapsed();
    
    println!("  📊 Results:");
    println!("    • Scenario duration: {:?}", scenario_duration);
    println!("    • Successful requests: {}/15", successful);
    println!("    • Unique computations: {} (expected: 1)", computation_sources.len());
    println!("    • Cache Stampede protection: {}", 
             if computation_sources.len() == 1 { "✅ WORKING" } else { "❌ FAILED" });
    println!();
    
    Ok(())
}

async fn await_scenario_high_concurrency(cache_system: Arc<CacheSystemIsland>) -> Result<()> {
    println!("📋 Scenario 3: High Concurrency (75 concurrent requests)");
    
    let start = Instant::now();
    let mut handles = Vec::new();
    
    for i in 1..=75 {
        let cache_system = cache_system.clone();
        let handle = tokio::spawn(async move {
            let result = cache_system.cache_manager().get_or_compute_with(
                "perf_test_data",
                CacheStrategy::ShortTerm,
                || async {
                    sleep(Duration::from_millis(800)).await; // 800ms computation
                    Ok(serde_json::json!({
                        "scenario": "high_concurrency",
                        "computed_by": format!("request_{}", i),
                        "value": format!("high_data_{}", i),
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }))
                }
            ).await;
            
            (i, result)
        });
        
        handles.push(handle);
        // No stagger - true concurrency test
    }
    
    let mut successful = 0;
    let mut computation_sources = std::collections::HashSet::new();
    
    for handle in handles {
        if let Ok((req_id, Ok(data))) = handle.await {
            successful += 1;
            
            if let Some(computed_by) = data.get("computed_by").and_then(|v| v.as_str()) {
                computation_sources.insert(computed_by.to_string());
            }
            
            // Print only every 10th result to avoid spam
            if req_id % 10 == 0 {
                println!("  ✅ Request #{} completed", req_id);
            }
        }
    }
    
    let scenario_duration = start.elapsed();
    
    println!("  📊 Results:");
    println!("    • Scenario duration: {:?}", scenario_duration);
    println!("    • Successful requests: {}/75", successful);
    println!("    • Unique computations: {} (expected: 1)", computation_sources.len());
    println!("    • Cache Stampede protection: {}", 
             if computation_sources.len() == 1 { "✅ WORKING" } else { "❌ FAILED" });
    println!();
    
    Ok(())
}

async fn await_scenario_extreme_concurrency(cache_system: Arc<CacheSystemIsland>) -> Result<()> {
    println!("📋 Scenario 4: Extreme Concurrency (250 concurrent requests)");
    
    let start = Instant::now();
    let mut handles = Vec::new();
    
    // Split into batches to manage memory
    for batch in 0..5 {
        let mut batch_handles = Vec::new();
        
        for i in 1..=50 {
            let req_id = batch * 50 + i;
            let cache_system = cache_system.clone();
            let handle = tokio::spawn(async move {
                let result = cache_system.cache_manager().get_or_compute_with(
                    "perf_test_data",
                    CacheStrategy::ShortTerm,
                    || async {
                        sleep(Duration::from_millis(1000)).await; // 1000ms computation
                        Ok(serde_json::json!({
                            "scenario": "extreme_concurrency",
                            "computed_by": format!("request_{}", req_id),
                            "value": format!("extreme_data_{}", req_id),
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        }))
                    }
                ).await;
                
                (req_id, result)
            });
            
            batch_handles.push(handle);
        }
        
        handles.extend(batch_handles);
        
        // Small delay between batches
        sleep(Duration::from_millis(10)).await;
    }
    
    let mut successful = 0;
    let mut computation_sources = std::collections::HashSet::new();
    
    for handle in handles {
        if let Ok((req_id, Ok(data))) = handle.await {
            successful += 1;
            
            if let Some(computed_by) = data.get("computed_by").and_then(|v| v.as_str()) {
                computation_sources.insert(computed_by.to_string());
            }
            
            // Print only every 25th result to avoid spam
            if req_id % 25 == 0 {
                println!("  ✅ Request #{} completed", req_id);
            }
        }
    }
    
    let scenario_duration = start.elapsed();
    
    println!("  📊 Results:");
    println!("    • Scenario duration: {:?}", scenario_duration);
    println!("    • Successful requests: {}/250", successful);
    println!("    • Unique computations: {} (expected: 1)", computation_sources.len());
    println!("    • Cache Stampede protection: {}", 
             if computation_sources.len() == 1 { "✅ WORKING" } else { "❌ FAILED" });
    println!("    • Estimated time without protection: {:?}", 
             scenario_duration * computation_sources.len() as u32);
    println!();
    
    Ok(())
}
