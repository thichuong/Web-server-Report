//! HTTP Endpoint Benchmark for /crypto_report
//! 
//! This benchmark tests the maximum load capacity of the /crypto_report endpoint
//! to determine how many requests it can handle per second with Cache Stampede protection.

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::time::sleep;
use reqwest::Client;
use anyhow::Result;

// Test parameters
const BENCHMARK_DURATION: Duration = Duration::from_secs(1);
const SERVER_URL: &str = "http://127.0.0.1:8050";
const ENDPOINT: &str = "/crypto_report";
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

// Metrics
static TOTAL_REQUESTS: AtomicU64 = AtomicU64::new(0);
static SUCCESSFUL_REQUESTS: AtomicU64 = AtomicU64::new(0);
static FAILED_REQUESTS: AtomicU64 = AtomicU64::new(0);
static TOTAL_RESPONSE_TIME_MS: AtomicU64 = AtomicU64::new(0);

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 HTTP Endpoint Load Test - /crypto_report");
    println!("📋 Test Parameters:");
    println!("  • Target URL: {}{}", SERVER_URL, ENDPOINT);
    println!("  • Test Duration: {:?}", BENCHMARK_DURATION);
    println!("  • Client Timeout: {:?}", CLIENT_TIMEOUT);
    println!();

    // Check if server is running
    println!("🔍 Checking server availability...");
    let client = Client::new();
    match client.get(&format!("{}/health", SERVER_URL))
        .timeout(Duration::from_secs(5))
        .send()
        .await 
    {
        Ok(response) => {
            if response.status().is_success() {
                println!("✅ Server is running and responsive");
            } else {
                println!("⚠️ Server responded but with status: {}", response.status());
            }
        }
        Err(_) => {
            println!("❌ Server not responding! Please start the server first:");
            println!("   cargo run --release");
            println!();
            return Ok(());
        }
    }
    
    println!();

    // Run different load scenarios
    println!("📊 Running load test scenarios...\n");
    
    // Scenario 1: Gradual ramp-up
    run_gradual_ramp_up_test().await?;
    
    // Reset metrics
    reset_metrics();
    sleep(Duration::from_millis(500)).await;
    
    // Scenario 2: Sustained high load
    run_sustained_high_load_test().await?;
    
    // Reset metrics  
    reset_metrics();
    sleep(Duration::from_millis(500)).await;
    
    // Scenario 3: Burst load test
    run_burst_load_test().await?;
    
    println!("\n🎉 HTTP endpoint benchmark completed!");
    
    Ok(())
}

async fn run_gradual_ramp_up_test() -> Result<()> {
    println!("📋 Scenario 1: Gradual Ramp-up Test (10-100 concurrent clients)");
    
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    // Ramp up from 10 to 100 concurrent clients over 1 second
    for batch in 1..=10 {
        let batch_size = batch * 10; // 10, 20, 30, ..., 100
        
        for _ in 0..10 { // 10 clients per batch
            let handle = tokio::spawn(async move {
                run_client_requests(Duration::from_millis(100)).await
            });
            handles.push(handle);
        }
        
        // Small delay between batches for ramp-up effect
        sleep(Duration::from_millis(100)).await;
        
        if start_time.elapsed() >= BENCHMARK_DURATION {
            break;
        }
    }
    
    // Wait for all clients to complete
    for handle in handles {
        let _ = handle.await;
    }
    
    let total_time = start_time.elapsed();
    print_results("Gradual Ramp-up", total_time).await;
    
    Ok(())
}

async fn run_sustained_high_load_test() -> Result<()> {
    println!("📋 Scenario 2: Sustained High Load (200 concurrent clients for 1 second)");
    
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    // Launch 200 concurrent clients immediately
    for client_id in 1..=200 {
        let handle = tokio::spawn(async move {
            // Each client makes requests for the full duration
            run_client_requests(BENCHMARK_DURATION).await
        });
        handles.push(handle);
        
        // Very small stagger to avoid overwhelming the spawn system
        if client_id % 50 == 0 {
            sleep(Duration::from_millis(1)).await;
        }
    }
    
    // Wait for all clients or timeout
    let timeout_duration = BENCHMARK_DURATION + Duration::from_secs(2);
    let results = tokio::time::timeout(timeout_duration, async {
        for handle in handles {
            let _ = handle.await;
        }
    }).await;
    
    match results {
        Ok(_) => println!("✅ All clients completed within timeout"),
        Err(_) => println!("⚠️ Some clients timed out"),
    }
    
    let total_time = start_time.elapsed();
    print_results("Sustained High Load", total_time).await;
    
    Ok(())
}

async fn run_burst_load_test() -> Result<()> {
    println!("📋 Scenario 3: Burst Load Test (500+ concurrent requests at once)");
    
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    // Launch 500 clients in quick bursts
    for burst in 0..5 {
        for _ in 0..100 { // 100 clients per burst
            let handle = tokio::spawn(async move {
                // Each client makes just a few requests quickly
                run_client_requests(Duration::from_millis(200)).await
            });
            handles.push(handle);
        }
        
        // Very small delay between bursts
        sleep(Duration::from_millis(10)).await;
        
        println!("  💥 Burst #{} launched (100 concurrent clients)", burst + 1);
    }
    
    // Wait for all requests to complete
    for handle in handles {
        let _ = handle.await;
    }
    
    let total_time = start_time.elapsed();
    print_results("Burst Load", total_time).await;
    
    Ok(())
}

async fn run_client_requests(duration: Duration) {
    let client = Client::builder()
        .timeout(CLIENT_TIMEOUT)
        .build()
        .unwrap_or_else(|_| Client::new());
        
    let start_time = Instant::now();
    
    while start_time.elapsed() < duration {
        let request_start = Instant::now();
        TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
        
        match client
            .get(&format!("{}{}", SERVER_URL, ENDPOINT))
            .send()
            .await 
        {
            Ok(response) => {
                let response_time_ms = request_start.elapsed().as_millis() as u64;
                TOTAL_RESPONSE_TIME_MS.fetch_add(response_time_ms, Ordering::Relaxed);
                
                if response.status().is_success() {
                    SUCCESSFUL_REQUESTS.fetch_add(1, Ordering::Relaxed);
                } else {
                    FAILED_REQUESTS.fetch_add(1, Ordering::Relaxed);
                    eprintln!("⚠️ HTTP error: {}", response.status());
                }
            }
            Err(e) => {
                FAILED_REQUESTS.fetch_add(1, Ordering::Relaxed);
                if e.is_timeout() {
                    eprintln!("⏱️ Request timeout");
                } else if e.is_connect() {
                    eprintln!("🔌 Connection error");
                } else {
                    eprintln!("❌ Request error: {}", e);
                }
            }
        }
        
        // Small delay to avoid overwhelming the client
        sleep(Duration::from_millis(1)).await;
    }
}

async fn print_results(scenario_name: &str, total_time: Duration) {
    let total_requests = TOTAL_REQUESTS.load(Ordering::Relaxed);
    let successful_requests = SUCCESSFUL_REQUESTS.load(Ordering::Relaxed);
    let failed_requests = FAILED_REQUESTS.load(Ordering::Relaxed);
    let total_response_time = TOTAL_RESPONSE_TIME_MS.load(Ordering::Relaxed);
    
    let requests_per_second = if total_time.as_secs_f64() > 0.0 {
        total_requests as f64 / total_time.as_secs_f64()
    } else {
        0.0
    };
    
    let avg_response_time_ms = if successful_requests > 0 {
        total_response_time as f64 / successful_requests as f64
    } else {
        0.0
    };
    
    let success_rate = if total_requests > 0 {
        (successful_requests as f64 / total_requests as f64) * 100.0
    } else {
        0.0
    };
    
    println!("  📊 {} Results:", scenario_name);
    println!("    • Test duration: {:?}", total_time);
    println!("    • Total requests: {}", total_requests);
    println!("    • Successful requests: {}", successful_requests);
    println!("    • Failed requests: {}", failed_requests);
    println!("    • Success rate: {:.1}%", success_rate);
    println!("    • Requests per second: {:.1} req/s", requests_per_second);
    println!("    • Average response time: {:.1} ms", avg_response_time_ms);
    
    // Performance rating
    let performance_rating = match requests_per_second {
        rps if rps >= 1000.0 => "🚀 EXCELLENT (1000+ req/s)",
        rps if rps >= 500.0 => "🔥 VERY GOOD (500+ req/s)", 
        rps if rps >= 200.0 => "✅ GOOD (200+ req/s)",
        rps if rps >= 100.0 => "⚡ DECENT (100+ req/s)",
        rps if rps >= 50.0 => "⚠️ MODERATE (50+ req/s)",
        _ => "❌ POOR (<50 req/s)",
    };
    
    println!("    • Performance rating: {}", performance_rating);
    
    // Cache effectiveness (based on response times)
    let cache_effectiveness = match avg_response_time_ms {
        rt if rt <= 10.0 => "🎯 EXCELLENT (Cache working perfectly)",
        rt if rt <= 50.0 => "✅ GOOD (Likely cache hits)",
        rt if rt <= 200.0 => "⚡ MODERATE (Mixed cache performance)", 
        rt if rt <= 500.0 => "⚠️ SLOW (Cache misses or compute load)",
        _ => "❌ VERY SLOW (Potential cache stampede or overload)",
    };
    
    println!("    • Cache effectiveness: {}", cache_effectiveness);
    println!();
}

fn reset_metrics() {
    TOTAL_REQUESTS.store(0, Ordering::Relaxed);
    SUCCESSFUL_REQUESTS.store(0, Ordering::Relaxed);
    FAILED_REQUESTS.store(0, Ordering::Relaxed);
    TOTAL_RESPONSE_TIME_MS.store(0, Ordering::Relaxed);
}
