//! Dashboard data response DTOs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Response for dashboard summary endpoints
/// Used by:
/// - GET /api/dashboard/data
/// - GET /api/crypto/dashboard-summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardDataResponse {
    // Bitcoin
    pub btc_price_usd: f64,
    pub btc_change_24h: f64,
    pub btc_market_cap_percentage: f64,
    pub btc_rsi_14: f64,

    // Ethereum
    pub eth_price_usd: f64,
    pub eth_change_24h: f64,
    pub eth_market_cap_percentage: f64,

    // BNB
    pub bnb_price_usd: f64,
    pub bnb_change_24h: f64,

    // Solana
    pub sol_price_usd: f64,
    pub sol_change_24h: f64,

    // XRP
    pub xrp_price_usd: f64,
    pub xrp_change_24h: f64,

    // Cardano
    pub ada_price_usd: f64,
    pub ada_change_24h: f64,

    // Chainlink
    pub link_price_usd: f64,
    pub link_change_24h: f64,

    // Market metrics
    pub market_cap_usd: f64,
    pub market_cap_change_percentage_24h_usd: f64,
    pub volume_24h_usd: f64,

    // Fear & Greed Index
    pub fng_value: i32,

    // US Stock Indices
    pub us_stock_indices: HashMap<String, StockIndexData>,

    // Metadata
    pub fetch_duration_ms: u64,
    pub partial_failure: bool,
    pub last_updated: String,
    pub timestamp: String,

    /// Optional note field (only present in fallback scenarios)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// US Stock Index data structure
/// Symbol is the HashMap key in DashboardDataResponse.us_stock_indices
/// Display name mapping should be handled by the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockIndexData {
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub status: String,
}
