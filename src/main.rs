use dotenvy::dotenv;
use std::{env, net::SocketAddr, sync::Arc};

mod data_service;
mod websocket_service;
mod models;
mod state;
mod handlers;
mod routes;
mod utils;
mod performance;
mod cache;

use state::AppState;
use routes::create_router;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let taapi_secret = env::var("TAAPI_SECRET").expect("TAAPI_SECRET must be set in .env");
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    println!("🚀 Starting Web Server with Multi-Tier Cache System...");
    println!("   Database: {}", if database_url.contains("localhost") { "Local" } else { "Remote" });
    println!("   Redis: {}", if redis_url.contains("localhost") { "Local" } else { "Remote" });

    // Initialize application state với multi-tier cache
    println!("🔄 Initializing application state...");
    let state = Arc::new(AppState::new(&database_url, &redis_url, taapi_secret).await?);

    // Prime the cache at startup
    state.prime_cache().await;

    // Create router with all routes
    let app = create_router(state.clone());

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8000);
    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();

    println!("🚀 Starting high-performance Rust server with Multi-Tier Cache");
    println!("📍 Address: http://{}", addr);
    println!("🖥️  Available CPUs: {}", num_cpus::get());
    println!("💾 Cache System: L1 (In-Memory) + L2 (Redis)");
    println!("⚡ Optimizations: LTO=fat, opt-level=3, concurrent APIs");
    
    // Sử dụng axum::Server::bind cho compatibility với axum 0.6
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
