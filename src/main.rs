#![warn(clippy::pedantic)]
use dotenvy::dotenv;
use std::{env, net::SocketAddr, sync::Arc};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use web_server_report::{routes::create_router, state::AppState};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();

    // Initialize tracing subscriber with env_filter support
    // Use RUST_LOG environment variable (defaults to "info" if not set)
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("🚀 Starting Web Server with Refactored Architecture...");

    // Initialize Application State
    info!("🏗️ Initializing AppState...");
    let state = Arc::new(AppState::new().await?);

    // Note: WebSocket and streaming functionality is now handled by separate websocket service

    // Create comprehensive router using AppState
    let app = create_router(Arc::clone(&state));

    // Start server
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse()
        .unwrap_or_else(|e| {
            warn!("⚠️ Invalid PORT value, using default 8000: {}", e);
            8000
        });

    let addr: SocketAddr = format!("{host}:{port}").parse().unwrap_or_else(|e| {
        warn!(
            "⚠️ Invalid HOST/PORT combination, using 0.0.0.0:8000: {}",
            e
        );
        SocketAddr::from(([0, 0, 0, 0], 8000))
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

    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("✅ Server started - Press Ctrl+C to shutdown gracefully");

    // Start server with graceful shutdown support
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    // Note: Add any app state cleanup here if necessary when gracefully shutting down.

    info!("👋 Server shutdown complete - All resources cleaned up");
    Ok(())
}
