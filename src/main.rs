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

use state::AppState;
use routes::create_router;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let taapi_secret = env::var("TAAPI_SECRET").expect("TAAPI_SECRET must be set in .env");
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    // Optimized connection pool với better error handling
    println!("🔄 Connecting to database...");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10) // Giảm max connections cho Railway free tier
        .min_connections(2)  // Giảm min connections
        .max_lifetime(std::time::Duration::from_secs(1800)) // 30 phút
        .idle_timeout(std::time::Duration::from_secs(300))  // 5 phút idle (giảm từ 10 phút)
        .acquire_timeout(std::time::Duration::from_secs(30)) // Tăng timeout lên 30s
        .test_before_acquire(true) // Bật test connection để đảm bảo connection valid
        .connect(&database_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}. Check DATABASE_URL and ensure the database is accessible.", e))?;
    
    // Test connection với retry
    println!("🔍 Testing database connection...");
    for attempt in 1..=3 {
        match sqlx::query("SELECT 1").fetch_one(&pool).await {
            Ok(_) => {
                println!("✅ Database connection successful!");
                break;
            }
            Err(e) if attempt < 3 => {
                println!("⚠️ Database connection attempt {} failed: {}. Retrying...", attempt, e);
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Database connection failed after 3 attempts: {}. Please check:\n1. DATABASE_URL is correct\n2. Database is running and accessible\n3. Network connection is stable", e));
            }
        }
    }

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
    println!("🗂️  Database pool: max_connections={}, min_connections={}", num_cpus::get() * 4, num_cpus::get());
    println!("🏃 Rayon thread pool: {} worker threads", num_cpus::get());
    println!("💾 Cache: DashMap (lock-free), Atomic counters");
    println!("⚡ Optimizations: LTO=fat, opt-level=3, codegen-units=1");
    
    // Sử dụng axum::Server::bind cho compatibility với axum 0.6
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
