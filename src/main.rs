use dotenvy::dotenv;
use std::{env, net::SocketAddr, sync::Arc};

mod service_islands;
mod performance;
mod routes;

use service_islands::ServiceIslands;
use routes::create_service_islands_router;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();

    println!("🚀 Starting Web Server with Service Islands Architecture...");
    
    // Initialize Service Islands Architecture
    println!("🏝️ Initializing Service Islands Architecture...");
    let service_islands = Arc::new(ServiceIslands::initialize().await?);

    // Note: WebSocket and streaming functionality is now handled by separate websocket service

    // Perform initial health check
    println!("🔍 Performing initial health check...");
    if service_islands.health_check().await {
        println!("✅ Service Islands Architecture is healthy!");
    } else {
        println!("⚠️ Some Service Islands may have issues - continuing with startup...");
    }

    // Create comprehensive router using Service Islands
    let app = create_service_islands_router(service_islands);

    // Start server
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse()
        .expect("PORT must be a valid number");
    
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("HOST and PORT must form a valid address");
    println!("🌐 Server listening on http://{}", addr);
    
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    
    Ok(())
}
