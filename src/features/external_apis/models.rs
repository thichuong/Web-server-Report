// src/features/external_apis/models.rs
//
// Data structures for external API responses and requests

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Dashboard summary with market data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashboardSummary {
    pub market_cap: f64,
    pub volume_24h: f64,
    pub btc_price_usd: f64,
    pub btc_change_24h: f64,
    pub fng_value: u32,
    pub rsi_14: f64,
    pub last_updated: DateTime<Utc>,
}

/// Market data for specific symbols
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketData {
    pub symbol: String,
    pub price: f64,
    pub volume_24h: f64,
    pub change_24h: f64,
    pub last_updated: DateTime<Utc>,
}

/// Technical indicator data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TechnicalIndicator {
    pub symbol: String,
    pub indicator: String,
    pub period: String,
    pub value: f64,
    pub last_updated: DateTime<Utc>,
}

/// Rate limiting status for monitoring
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RateLimitStatus {
    pub btc_api_circuit_breaker_open: bool,
    pub seconds_since_last_btc_fetch: u64,
    pub can_fetch_btc_now: bool,
}

// CoinGecko API response structures
#[derive(Debug, Deserialize)]
pub(crate) struct CoinGeckoGlobal {
    pub data: CoinGeckoGlobalData,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CoinGeckoGlobalData {
    pub total_market_cap: HashMap<String, f64>,
    pub total_volume: HashMap<String, f64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CoinGeckoBtcPrice {
    pub bitcoin: BtcPriceData,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BtcPriceData {
    pub usd: f64,
    pub usd_24h_change: f64,
}

// Fear & Greed Index API response structures
#[derive(Debug, Deserialize)]
pub(crate) struct FearGreedResponse {
    pub data: Vec<FearGreedData>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FearGreedData {
    pub value: String,
}

// TAAPI.io response structures
#[derive(Debug, Deserialize)]
pub(crate) struct TaapiRsiResponse {
    pub value: f64,
}

/// API endpoint configuration
#[derive(Debug, Clone)]
pub struct ApiEndpoint {
    pub url: String,
    pub name: String,
    pub timeout_seconds: u64,
    pub rate_limit_interval_seconds: u64,
}

impl ApiEndpoint {
    pub fn coingecko_global() -> Self {
        Self {
            url: "https://api.coingecko.com/api/v3/global".to_string(),
            name: "coingecko_global".to_string(),
            timeout_seconds: 10,
            rate_limit_interval_seconds: 1,
        }
    }

    pub fn coingecko_btc_price() -> Self {
        Self {
            url: "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd&include_24hr_change=true".to_string(),
            name: "coingecko_btc_price".to_string(),
            timeout_seconds: 5,
            rate_limit_interval_seconds: 3, // 3 second minimum interval
        }
    }

    pub fn fear_greed_index() -> Self {
        Self {
            url: "https://api.alternative.me/fng/?limit=1".to_string(),
            name: "fear_greed_index".to_string(),
            timeout_seconds: 10,
            rate_limit_interval_seconds: 60, // 1 minute interval
        }
    }

    pub fn taapi_rsi(secret: &str) -> Self {
        let url = format!(
            "https://api.taapi.io/rsi?secret={}&exchange=binance&symbol=BTC/USDT&interval=1d",
            secret
        );
        Self {
            url,
            name: "taapi_rsi".to_string(),
            timeout_seconds: 15,
            rate_limit_interval_seconds: 60, // 1 minute interval
        }
    }
}

/// API request context with retry and error handling info
#[derive(Debug, Clone)]
pub struct ApiRequestContext {
    pub endpoint: String,
    pub attempt: u32,
    pub max_retries: u32,
    pub backoff_seconds: u64,
}

/// API response envelope
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub response_time_ms: u64,
    pub from_cache: bool,
    pub timestamp: DateTime<Utc>,
}
