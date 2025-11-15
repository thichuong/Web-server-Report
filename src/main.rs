use dotenvy::dotenv;
use std::{env, net::SocketAddr, sync::Arc};
use tracing::{info, warn, error};
use tracing_subscriber::EnvFilter;

use web_server_report::{ServiceIslands, routes::create_service_islands_router};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();

    // Initialize tracing subscriber with env_filter support
    // Use RUST_LOG environment variable (defaults to "info" if not set)
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    info!("🚀 Starting Web Server with Service Islands Architecture...");

    // Initialize Service Islands Architecture
    info!("🏝️ Initializing Service Islands Architecture...");
    let service_islands = Arc::new(ServiceIslands::initialize().await?);

    // Note: WebSocket and streaming functionality is now handled by separate websocket service

    // Perform initial health check
    info!("🔍 Performing initial health check...");
    if service_islands.health_check().await {
        info!("✅ Service Islands Architecture is healthy!");
    } else {
        warn!("⚠️ Some Service Islands may have issues - continuing with startup...");
    }

    // Create comprehensive router using Service Islands
    let app = create_service_islands_router(Arc::clone(&service_islands));

    // Start server
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse()
        .unwrap_or_else(|e| {
            warn!("⚠️ Invalid PORT value, using default 8000: {}", e);
            8000
        });

    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .unwrap_or_else(|e| {
            warn!("⚠️ Invalid HOST/PORT combination, using 0.0.0.0:8000: {}", e);
            "0.0.0.0:8000".parse().unwrap()  // This is guaranteed valid
        });
    info!("🌐 Server listening on http://{}", addr);

    // Setup graceful shutdown signal handler
    let shutdown_signal = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            warn!("⚠️ Failed to install CTRL+C signal handler: {}", e);
            // Continue anyway - shutdown will still work via other signals
        }
        info!("\n🛑 Received shutdown signal (Ctrl+C)");
    };

    // Start server with graceful shutdown support
    info!("✅ Server started - Press Ctrl+C to shutdown gracefully");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    // Perform graceful shutdown cleanup
    info!("🧹 Starting graceful shutdown of Service Islands...");
    if let Err(e) = service_islands.shutdown().await {
        error!("⚠️  Shutdown error: {}", e);
    }

    info!("👋 Server shutdown complete - All resources cleaned up");
    Ok(())
}
