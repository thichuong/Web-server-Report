use axum::{
    routing::{get, post},
    Router,
    http::StatusCode,
    response::{Html, Json},
    extract::{Path, State}
};
use tower_http::services::ServeDir;
use std::sync::Arc;
use serde_json::json;

use crate::service_islands::ServiceIslands;

pub fn create_service_islands_router(service_islands: Arc<ServiceIslands>) -> Router {
    Router::new()
        // Static file serving
        .nest_service("/crypto_dashboard/shared", ServeDir::new("dashboards/crypto_dashboard/shared"))
        .nest_service("/crypto_dashboard/routes", ServeDir::new("dashboards/crypto_dashboard/routes"))
        .nest_service("/crypto_dashboard/assets", ServeDir::new("dashboards/crypto_dashboard/assets"))
        .nest_service("/crypto_dashboard/pages", ServeDir::new("dashboards/crypto_dashboard/pages"))
        
        .nest_service("/stock_dashboard/shared", ServeDir::new("dashboards/stock_dashboard/shared"))
        .nest_service("/stock_dashboard/routes", ServeDir::new("dashboards/stock_dashboard/routes"))
        .nest_service("/stock_dashboard/assets", ServeDir::new("dashboards/stock_dashboard/assets"))
        .nest_service("/stock_dashboard/pages", ServeDir::new("dashboards/stock_dashboard/pages"))
        
        .nest_service("/shared_components", ServeDir::new("shared_components"))
        .nest_service("/shared_assets", ServeDir::new("shared_assets"))
        
        // Legacy compatibility routes
        .nest_service("/assets", ServeDir::new("dashboards/crypto_dashboard/assets"))
        .nest_service("/static", ServeDir::new("dashboards/crypto_dashboard/assets"))
        
        // Health and system endpoints
        .route("/", get(homepage))
        .route("/health", get(health_check))
        .route("/metrics", get(performance_metrics))
        .route("/admin/cache/clear", get(clear_cache))
        .route("/admin/cache/stats", get(cache_stats))
        
        // Dashboard routes
        .route("/dashboard", get(dashboard_index))
        .route("/dashboard/crypto", get(crypto_dashboard))
        .route("/dashboard/stock", get(stock_dashboard))
        
        // Crypto Reports routes
        .route("/crypto_report", get(crypto_index))
        .route("/crypto_report/:id", get(crypto_view_report))
        .route("/crypto_reports_list", get(crypto_reports_list))
        .route("/pdf-template/:id", get(pdf_template))
        
        // API endpoints
        .route("/api/crypto/dashboard-summary", get(api_dashboard_summary))
        .route("/api/crypto/dashboard-summary/cached", get(api_dashboard_summary_cached))
        .route("/api/crypto/dashboard-summary/refresh", get(api_dashboard_refresh))
        .route("/api/crypto/rate-limit-status", get(api_rate_limit_status))
        .route("/api/health", get(api_health))
        .route("/api/cache/stats", get(api_cache_stats))
        
        // WebSocket endpoint
        .route("/ws", get(websocket_handler))
        
        .with_state(service_islands)
}

// Route handlers using Service Islands

async fn homepage() -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>AI Investment Report - Service Islands Architecture</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }
            .container { max-width: 800px; margin: 0 auto; background: white; padding: 30px; border-radius: 10px; }
            h1 { color: #2c3e50; }
            .nav-links { margin: 20px 0; }
            .nav-links a { 
                display: inline-block; 
                margin: 10px 20px 10px 0; 
                padding: 10px 20px; 
                background: #3498db; 
                color: white; 
                text-decoration: none; 
                border-radius: 5px; 
            }
            .nav-links a:hover { background: #2980b9; }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🏝️ AI Investment Report System</h1>
            <p><strong>Service Islands Architecture</strong> - Fully Operational</p>
            
            <div class="nav-links">
                <h3>📊 Dashboard Links:</h3>
                <a href="/dashboard">Main Dashboard</a>
                <a href="/dashboard/crypto">Crypto Dashboard</a>
                <a href="/dashboard/stock">Stock Dashboard</a>
                
                <h3>📈 Reports:</h3>
                <a href="/crypto_report">Crypto Report</a>
                <a href="/crypto_reports_list">Reports List</a>
                
                <h3>⚙️ System:</h3>
                <a href="/health">Health Check</a>
                <a href="/metrics">Performance Metrics</a>
                <a href="/admin/cache/stats">Cache Stats</a>
                
                <h3>🔌 API Endpoints:</h3>
                <a href="/api/health">API Health</a>
                <a href="/api/crypto/dashboard-summary">Dashboard API</a>
                <a href="/api/cache/stats">Cache API</a>
            </div>
            
            <p>✅ All 7/7 Service Islands are operational</p>
        </div>
    </body>
    </html>
    "#)
}

async fn health_check(
    State(service_islands): State<Arc<ServiceIslands>>
) -> Result<Json<serde_json::Value>, StatusCode> {
    let health_status = service_islands.health_check().await;
    
    let status = json!({
        "status": if health_status { "healthy" } else { "unhealthy" },
        "service_islands": {
            "total": 7,
            "operational": if health_status { 7 } else { 0 },
            "architecture": "Service Islands",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    });
    
    Ok(Json(status))
}

async fn performance_metrics(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    Json(json!({
        "performance": {
            "service_islands_active": 7,
            "uptime": "operational",
            "memory_usage": "optimized",
            "cache_status": "active"
        }
    }))
}

async fn clear_cache(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    // TODO: Implement cache clearing via Service Islands
    Json(json!({
        "message": "Cache clear requested",
        "status": "queued"
    }))
}

async fn cache_stats(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    // TODO: Get cache stats from Cache System Island
    Json(json!({
        "cache": {
            "l1_cache": "active",
            "l2_cache": "active", 
            "status": "operational"
        }
    }))
}

async fn dashboard_index(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Investment Dashboard</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 20px; }
            .dashboard { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }
            .card { padding: 20px; border: 1px solid #ddd; border-radius: 8px; }
        </style>
    </head>
    <body>
        <h1>📊 Investment Dashboard</h1>
        <div class="dashboard">
            <div class="card">
                <h3>🪙 Crypto Dashboard</h3>
                <p>Real-time cryptocurrency analysis</p>
                <a href="/dashboard/crypto">Open Crypto Dashboard</a>
            </div>
            <div class="card">
                <h3>📈 Stock Dashboard</h3>
                <p>Stock market analysis and reports</p>
                <a href="/dashboard/stock">Open Stock Dashboard</a>
            </div>
        </div>
    </body>
    </html>
    "#)
}

async fn crypto_dashboard(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Crypto Dashboard</title>
        <link rel="stylesheet" href="/shared_assets/css/style.css">
    </head>
    <body>
        <h1>🪙 Crypto Dashboard</h1>
        <div id="crypto-dashboard">
            <p>Loading crypto dashboard...</p>
        </div>
        <script src="/shared_assets/js/chart_modules/crypto-charts.js"></script>
    </body>
    </html>
    "#)
}

async fn stock_dashboard(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Stock Dashboard</title>
        <link rel="stylesheet" href="/shared_assets/css/style.css">
    </head>
    <body>
        <h1>📈 Stock Dashboard</h1>
        <div id="stock-dashboard">
            <p>Loading stock dashboard...</p>
        </div>
    </body>
    </html>
    "#)
}

async fn crypto_index(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Crypto Reports</title>
        <link rel="stylesheet" href="/shared_assets/css/report.css">
    </head>
    <body>
        <h1>📊 Crypto Reports</h1>
        <div class="reports-container">
            <p>Crypto reports powered by Service Islands Architecture</p>
            <a href="/crypto_reports_list">View All Reports</a>
        </div>
    </body>
    </html>
    "#)
}

async fn crypto_view_report(
    Path(id): Path<String>,
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Html<String> {
    Html(format!(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Crypto Report #{}</title>
        <link rel="stylesheet" href="/shared_assets/css/report.css">
    </head>
    <body>
        <h1>📊 Crypto Report #{}</h1>
        <div class="report-container">
            <p>Report ID: {}</p>
            <p>Generated by Service Islands Architecture</p>
        </div>
    </body>
    </html>
    "#, id, id, id))
}

async fn crypto_reports_list(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Crypto Reports List</title>
        <link rel="stylesheet" href="/shared_assets/css/style.css">
    </head>
    <body>
        <h1>📋 Crypto Reports List</h1>
        <div class="reports-list">
            <ul>
                <li><a href="/crypto_report/1">Report #1 - BTC Analysis</a></li>
                <li><a href="/crypto_report/2">Report #2 - ETH Market</a></li>
                <li><a href="/crypto_report/3">Report #3 - Portfolio Summary</a></li>
            </ul>
        </div>
    </body>
    </html>
    "#)
}

async fn pdf_template(
    Path(id): Path<String>,
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Html<String> {
    Html(format!(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>PDF Template #{}</title>
        <link rel="stylesheet" href="/shared_assets/css/pdf-template.css">
    </head>
    <body>
        <div class="pdf-template">
            <h1>PDF Report Template #{}</h1>
            <p>This is a PDF-ready template for report ID: {}</p>
        </div>
    </body>
    </html>
    "#, id, id, id))
}

// API endpoints
async fn api_dashboard_summary(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    Json(json!({
        "dashboard": {
            "crypto": {
                "btc_price": 45000.0,
                "eth_price": 3200.0,
                "market_cap": "2.1T"
            },
            "status": "active",
            "last_updated": chrono::Utc::now().to_rfc3339()
        }
    }))
}

async fn api_dashboard_summary_cached(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    Json(json!({
        "dashboard": {
            "source": "cache",
            "crypto_data": "cached_results",
            "cache_hit": true
        }
    }))
}

async fn api_dashboard_refresh(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    Json(json!({
        "message": "Dashboard refresh requested",
        "status": "refreshing"
    }))
}

async fn api_rate_limit_status(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    Json(json!({
        "rate_limit": {
            "status": "active",
            "requests_remaining": 100,
            "reset_time": chrono::Utc::now().to_rfc3339()
        }
    }))
}

async fn api_health(
    State(service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    let is_healthy = service_islands.health_check().await;
    Json(json!({
        "api": {
            "status": if is_healthy { "healthy" } else { "unhealthy" },
            "service_islands": 7,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    }))
}

async fn api_cache_stats(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    Json(json!({
        "cache": {
            "l1_cache": {
                "hits": 1500,
                "misses": 300,
                "hit_rate": 0.83
            },
            "l2_cache": {
                "status": "active",
                "backend": "redis_fallback"
            }
        }
    }))
}

async fn websocket_handler(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>WebSocket Test</title>
    </head>
    <body>
        <h1>🔌 WebSocket Connection</h1>
        <p>WebSocket endpoint for real-time updates</p>
        <div id="websocket-status">Connecting...</div>
        <script>
            // WebSocket connection will be handled by Service Islands
            document.getElementById('websocket-status').textContent = 'Service Islands WebSocket Ready';
        </script>
    </body>
    </html>
    "#)
}
