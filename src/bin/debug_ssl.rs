// Debug SSL connectivity issues
use reqwest::Client;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing SSL connectivity with different client configs...");

    // Test 1: Basic client (similar to current setup)
    println!("\n📡 Test 1: Optimized client (current setup)");
    let client1 = Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .pool_max_idle_per_host(20)
        .pool_idle_timeout(Duration::from_secs(30))
        .tcp_keepalive(Duration::from_secs(60))
        .danger_accept_invalid_certs(false)
        .tls_built_in_root_certs(true)
        .https_only(false)
        .http2_prior_knowledge()
        .user_agent("Mozilla/5.0 (compatible; RustWebServer/1.0)")
        .tcp_nodelay(true)
        .build()?;

    test_api_call(&client1, "CoinGecko Global", "https://api.coingecko.com/api/v3/global").await;
    test_api_call(&client1, "CoinGecko Ping", "https://api.coingecko.com/api/v3/ping").await;

    // Test 2: Simple client
    println!("\n📡 Test 2: Simple client");
    let client2 = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    test_api_call(&client2, "CoinGecko Simple", "https://api.coingecko.com/api/v3/ping").await;

    // Test 3: No HTTP/2 prior knowledge
    println!("\n📡 Test 3: No HTTP/2 prior knowledge");
    let client3 = Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (compatible; RustWebServer/1.0)")
        .build()?;

    test_api_call(&client3, "CoinGecko No H2", "https://api.coingecko.com/api/v3/ping").await;

    println!("\n✅ SSL connectivity tests completed!");
    Ok(())
}

async fn test_api_call(client: &Client, name: &str, url: &str) {
    match client.get(url).send().await {
        Ok(response) => {
            println!("✅ {}: {} - {}", name, response.status(), url);
            if let Ok(text) = response.text().await {
                if text.len() > 100 {
                    println!("   Response: {}...", &text[..100]);
                } else {
                    println!("   Response: {}", text);
                }
            }
        }
        Err(e) => {
            println!("❌ {}: Error - {}", name, e);
            println!("   URL: {}", url);
            
            // Check error type
            if e.is_timeout() {
                println!("   Error type: Timeout");
            } else if e.is_connect() {
                println!("   Error type: Connection");
            } else if e.is_request() {
                println!("   Error type: Request");
            } else if e.is_body() {
                println!("   Error type: Body");
            } else if e.is_decode() {
                println!("   Error type: Decode");
            } else {
                println!("   Error type: Other - {}", e);
            }
        }
    }
}
