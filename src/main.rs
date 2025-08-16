use dotenvy::dotenv;
use std::{env, net::SocketAddr, sync::Arc};

mod data_service;
mod websocket_service;
mod models;
mod state;
mod handlers;
mod routes;
mod utils;

use state::AppState;
use routes::create_router;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let taapi_secret = env::var("TAAPI_SECRET").expect("TAAPI_SECRET must be set in .env");
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    // Tối ưu connection pool cho đa luồng
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(32) // Tăng từ default 10 lên 32 cho 16 cores
        .min_connections(8)  // Duy trì ít nhất 8 connections
        .max_lifetime(std::time::Duration::from_secs(30 * 60)) // 30 phút
        .idle_timeout(std::time::Duration::from_secs(10 * 60)) // 10 phút idle
        .acquire_timeout(std::time::Duration::from_secs(30)) // Timeout nếu không lấy được connection
        .connect(&database_url).await?;

    // Initialize AppState
    let state = AppState::new(pool, taapi_secret, &redis_url).await?;
    let shared_state = Arc::new(state);

    // Prime the cache
    {
        let s = Arc::clone(&shared_state);
        tokio::spawn(async move {
            s.prime_cache().await;
        });
    }

    // Create router with all routes
    let app = create_router(shared_state);

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8000);
    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();

    println!("🚀 Starting high-performance Rust server");
    println!("📍 Address: http://{}", addr);
    println!("🖥️  Available CPUs: {}", num_cpus::get());
    println!("🗂️  Database pool: max_connections=32, min_connections=8");
    println!("🏃 Rayon thread pool: {} worker threads", num_cpus::get());
    println!("💾 Cache: DashMap (lock-free), Atomic counters");
    
    // Sử dụng axum::Server::bind cho compatibility với axum 0.6
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
