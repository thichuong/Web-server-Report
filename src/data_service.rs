// src/data_service.rs - Service để fetch dữ liệu từ external APIs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Result, Context};

// API URLs
const BASE_GLOBAL_URL: &str = "https://api.coingecko.com/api/v3/global";
const BASE_BTC_PRICE_URL: &str = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd&include_24hr_change=true";
const BASE_FNG_URL: &str = "https://api.alternative.me/fng/?limit=1";
const BASE_RSI_URL_TEMPLATE: &str = "https://api.taapi.io/rsi?secret={secret}&exchange=binance&symbol=BTC/USDT&interval=1d";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashboardSummary {
    pub market_cap: f64,
    pub volume_24h: f64,
    pub btc_price_usd: f64,
    pub btc_change_24h: f64,
    pub fng_value: u32,
    pub rsi_14: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoGlobal {
    data: CoinGeckoGlobalData,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoGlobalData {
    total_market_cap: HashMap<String, f64>,
    total_volume: HashMap<String, f64>,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoBtcPrice {
    bitcoin: BtcPriceData,
}

#[derive(Debug, Deserialize)]
struct BtcPriceData {
    usd: f64,
    usd_24h_change: f64,
}

#[derive(Debug, Deserialize)]
struct FearGreedResponse {
    data: Vec<FearGreedData>,
}

#[derive(Debug, Deserialize)]
struct FearGreedData {
    value: String,
}

#[derive(Debug, Deserialize)]
struct TaapiRsiResponse {
    value: f64,
}

pub struct DataService {
    client: Client,
    taapi_secret: String,
}

impl DataService {
    pub fn new(taapi_secret: String) -> Self {
        // Sử dụng optimized HTTP client từ performance module
        let client = crate::performance::OPTIMIZED_HTTP_CLIENT.clone();
        
        Self {
            client,
            taapi_secret,
        }
    }

    pub async fn fetch_dashboard_summary(&self) -> Result<DashboardSummary> {
        println!("🔄 Fetching dashboard summary from external APIs...");

        // Fetch all data concurrently
        let (global_result, btc_result, fng_result, rsi_result) = tokio::try_join!(
            self.fetch_global_data(),
            self.fetch_btc_price(),
            self.fetch_fear_greed(),
            self.fetch_rsi()
        )?;

        let summary = DashboardSummary {
            market_cap: global_result.0,
            volume_24h: global_result.1,
            btc_price_usd: btc_result.0,
            btc_change_24h: btc_result.1,
            fng_value: fng_result,
            rsi_14: rsi_result,
            last_updated: chrono::Utc::now(),
        };

        println!("✅ Dashboard summary fetched successfully");
        Ok(summary)
    }

    async fn fetch_global_data(&self) -> Result<(f64, f64)> {
        let response = self.client
            .get(BASE_GLOBAL_URL)
            .send()
            .await
            .context("Failed to fetch global data")?;

        let global_data: CoinGeckoGlobal = response
            .json()
            .await
            .context("Failed to parse global data JSON")?;

        let market_cap = global_data.data.total_market_cap
            .get("usd")
            .copied()
            .unwrap_or(0.0);

        let volume_24h = global_data.data.total_volume
            .get("usd")
            .copied()
            .unwrap_or(0.0);

        println!("📊 Market Cap: ${:.2}, Volume 24h: ${:.2}", market_cap, volume_24h);
        Ok((market_cap, volume_24h))
    }

    async fn fetch_btc_price(&self) -> Result<(f64, f64)> {
        let response = self.client
            .get(BASE_BTC_PRICE_URL)
            .send()
            .await
            .context("Failed to fetch BTC price")?;

        let btc_data: CoinGeckoBtcPrice = response
            .json()
            .await
            .context("Failed to parse BTC price JSON")?;

        let price = btc_data.bitcoin.usd;
        let change_24h = btc_data.bitcoin.usd_24h_change;

        println!("₿ BTC Price: ${:.2}, Change 24h: {:.2}%", price, change_24h);
        Ok((price, change_24h))
    }

    async fn fetch_fear_greed(&self) -> Result<u32> {
        let response = self.client
            .get(BASE_FNG_URL)
            .send()
            .await
            .context("Failed to fetch Fear & Greed index")?;

        let fng_data: FearGreedResponse = response
            .json()
            .await
            .context("Failed to parse Fear & Greed JSON")?;

        let value = fng_data.data
            .first()
            .context("No Fear & Greed data found")?
            .value
            .parse::<u32>()
            .context("Failed to parse Fear & Greed value")?;

        println!("😨 Fear & Greed Index: {}", value);
        Ok(value)
    }

    async fn fetch_rsi(&self) -> Result<f64> {
        let url = BASE_RSI_URL_TEMPLATE.replace("{secret}", &self.taapi_secret);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch RSI data")?;

        let rsi_data: TaapiRsiResponse = response
            .json()
            .await
            .context("Failed to parse RSI JSON")?;

        println!("📈 RSI 14: {:.2}", rsi_data.value);
        Ok(rsi_data.value)
    }
}
