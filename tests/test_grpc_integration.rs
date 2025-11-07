// Quick test to verify gRPC integration between monolith and Layer2 service
// Run with: cargo test --test test_grpc_integration

use web_server_report::service_islands::layer3_communication::layer2_grpc_client::Layer2GrpcClient;

#[tokio::test]
async fn test_grpc_health_check() {
    let client = Layer2GrpcClient::new("http://localhost:50051".to_string())
        .expect("Failed to create gRPC client");

    let result = client.health_check().await;

    match result {
        Ok(is_healthy) => {
            println!("✅ gRPC Health Check: {}", if is_healthy { "Healthy" } else { "Unhealthy" });
            assert!(is_healthy, "Service should be healthy");
        }
        Err(e) => {
            panic!("❌ gRPC Health Check failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_grpc_fetch_crypto_prices() {
    let client = Layer2GrpcClient::new("http://localhost:50051".to_string())
        .expect("Failed to create gRPC client");

    let result = client.fetch_crypto_prices().await;

    match result {
        Ok(data) => {
            println!("✅ gRPC Fetch Crypto Prices successful");
            println!("   Data sample: {}", serde_json::to_string_pretty(&data).unwrap_or_default());
            assert!(data.is_object(), "Should return JSON object");
        }
        Err(e) => {
            panic!("❌ gRPC Fetch Crypto Prices failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_grpc_dashboard_summary() {
    let client = Layer2GrpcClient::new("http://localhost:50051".to_string())
        .expect("Failed to create gRPC client");

    let result = client.fetch_dashboard_summary_v2(false).await;

    match result {
        Ok(data) => {
            println!("✅ gRPC Dashboard Summary successful");

            // Verify key fields exist
            assert!(data.get("btc_price_usd").is_some(), "Should have btc_price_usd");
            assert!(data.get("eth_price_usd").is_some(), "Should have eth_price_usd");

            println!("   BTC Price: ${}", data.get("btc_price_usd").and_then(|v| v.as_f64()).unwrap_or(0.0));
            println!("   ETH Price: ${}", data.get("eth_price_usd").and_then(|v| v.as_f64()).unwrap_or(0.0));
        }
        Err(e) => {
            panic!("❌ gRPC Dashboard Summary failed: {}", e);
        }
    }
}
