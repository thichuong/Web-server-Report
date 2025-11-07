//! Layer2 gRPC Client - gRPC bridge to Layer2 Data Service
//!
//! This module replaces HTTP calls with high-performance gRPC calls.

use anyhow::{Context, Result};
use serde_json::Value;

// Include generated protobuf code
pub mod market_data {
    tonic::include_proto!("market_data");
}

use market_data::market_data_service_client::MarketDataServiceClient;
use market_data::*;

/// gRPC client for communicating with Layer2 Data Service
pub struct Layer2GrpcClient {
    endpoint: String,
}

impl Layer2GrpcClient {
    /// Create a new Layer2 gRPC client
    pub fn new(endpoint: String) -> Result<Self> {
        Ok(Self { endpoint })
    }

    /// Fetch dashboard summary from Layer2 service via gRPC
    ///
    /// This replaces the HTTP call with a high-performance gRPC call
    pub async fn fetch_dashboard_summary_v2(&self, force_realtime_refresh: bool) -> Result<Value> {
        let mut client = MarketDataServiceClient::connect(self.endpoint.clone())
            .await
            .context("Failed to connect to Layer2 gRPC service")?;

        let request = tonic::Request::new(DashboardSummaryRequest {
            force_realtime_refresh,
        });

        let response = client
            .get_dashboard_summary(request)
            .await
            .context("Failed to fetch dashboard summary via gRPC")?;

        let data = response.into_inner();

        // Convert protobuf response to JSON for compatibility
        let json = serde_json::json!({
            "btc_price_usd": data.btc_price_usd,
            "btc_change_24h": data.btc_change_24h,
            "eth_price_usd": data.eth_price_usd,
            "eth_change_24h": data.eth_change_24h,
            "sol_price_usd": data.sol_price_usd,
            "sol_change_24h": data.sol_change_24h,
            "xrp_price_usd": data.xrp_price_usd,
            "xrp_change_24h": data.xrp_change_24h,
            "ada_price_usd": data.ada_price_usd,
            "ada_change_24h": data.ada_change_24h,
            "link_price_usd": data.link_price_usd,
            "link_change_24h": data.link_change_24h,
            "bnb_price_usd": data.bnb_price_usd,
            "bnb_change_24h": data.bnb_change_24h,
            "market_cap_usd": data.market_cap_usd,
            "volume_24h_usd": data.volume_24h_usd,
            "market_cap_change_percentage_24h_usd": data.market_cap_change_percentage_24h_usd,
            "btc_market_cap_percentage": data.btc_market_cap_percentage,
            "eth_market_cap_percentage": data.eth_market_cap_percentage,
            "fng_value": data.fng_value,
            "btc_rsi_14": data.btc_rsi_14,
            "us_stock_indices": Self::convert_us_indices(data.us_stock_indices),
            "partial_failure": data.partial_failure,
            "fetch_duration_ms": data.fetch_duration_ms,
            "timestamp": data.timestamp,
        });

        Ok(json)
    }

    /// Health check for Layer2 gRPC service
    pub async fn health_check(&self) -> Result<bool> {
        let mut client = MarketDataServiceClient::connect(self.endpoint.clone())
            .await
            .context("Failed to connect to Layer2 gRPC service")?;

        let request = tonic::Request::new(HealthCheckRequest {});

        let response = client.health_check(request).await;

        match response {
            Ok(resp) => {
                let health = resp.into_inner();
                Ok(health.status == "healthy" && health.external_apis_healthy)
            }
            Err(_) => Ok(false),
        }
    }

    /// Fetch crypto prices only
    pub async fn fetch_crypto_prices(&self) -> Result<Value> {
        let mut client = MarketDataServiceClient::connect(self.endpoint.clone())
            .await
            .context("Failed to connect to Layer2 gRPC service")?;

        let request = tonic::Request::new(CryptoPricesRequest {
            force_refresh: false,
        });

        let response = client
            .get_crypto_prices(request)
            .await
            .context("Failed to fetch crypto prices via gRPC")?;

        let data = response.into_inner();

        // Convert to JSON
        let prices: serde_json::Map<String, Value> = data
            .prices
            .into_iter()
            .map(|(symbol, price)| {
                (
                    symbol,
                    serde_json::json!({
                        "symbol": price.symbol,
                        "last_price": price.last_price,
                        "price_change_percent": price.price_change_percent,
                    }),
                )
            })
            .collect();

        Ok(Value::Object(prices))
    }

    /// Fetch global market data
    pub async fn fetch_global_market_data(&self) -> Result<Value> {
        let mut client = MarketDataServiceClient::connect(self.endpoint.clone())
            .await
            .context("Failed to connect to Layer2 gRPC service")?;

        let request = tonic::Request::new(GlobalMarketDataRequest {});

        let response = client
            .get_global_market_data(request)
            .await
            .context("Failed to fetch global market data via gRPC")?;

        let data = response.into_inner();

        Ok(serde_json::json!({
            "total_market_cap": {
                "usd": data.total_market_cap_usd
            },
            "total_volume": {
                "usd": data.total_volume_usd
            },
            "market_cap_change_percentage_24h_usd": data.market_cap_change_percentage_24h_usd,
            "market_cap_percentage": {
                "btc": data.btc_market_cap_percentage,
                "eth": data.eth_market_cap_percentage
            }
        }))
    }

    fn convert_us_indices(indices: Option<UsStockIndices>) -> Value {
        if let Some(idx) = indices {
            serde_json::json!({
                "SPX": Self::convert_stock_index(idx.spx),
                "DJI": Self::convert_stock_index(idx.dji),
                "IXIC": Self::convert_stock_index(idx.ixic),
            })
        } else {
            serde_json::json!({})
        }
    }

    fn convert_stock_index(index: Option<StockIndex>) -> Value {
        if let Some(idx) = index {
            serde_json::json!({
                "c": idx.current_price,
                "d": idx.daily_change,
                "dp": idx.daily_change_percentage,
                "t": idx.timestamp,
            })
        } else {
            serde_json::json!({})
        }
    }
}
