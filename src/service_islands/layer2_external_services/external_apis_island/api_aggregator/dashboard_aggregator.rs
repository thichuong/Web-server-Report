//! Dashboard Aggregator Component
//!
//! This module contains the dashboard data aggregation logic that orchestrates
//! multiple API calls concurrently and handles error processing.

use anyhow::Result;
use std::sync::atomic::Ordering;
use tokio::time::{timeout, Duration};
use super::aggregator_core::ApiAggregator;

impl ApiAggregator {
    /// Fetch dashboard summary v2 - Main method for Layer 2 dashboard data
    /// Returns a focused summary with essential market data
    pub async fn fetch_dashboard_summary_v2(&self) -> Result<serde_json::Value> {
        let start_time = std::time::Instant::now();
        self.total_aggregations.fetch_add(1, Ordering::Relaxed);

        println!("🔄 Starting dashboard summary v2 aggregation...");

        // Fetch essential data concurrently with shorter timeouts for summary
        let btc_future = timeout(Duration::from_secs(8), self.fetch_btc_with_cache());
        let eth_future = timeout(Duration::from_secs(8), self.fetch_eth_with_cache());
        let sol_future = timeout(Duration::from_secs(8), self.fetch_sol_with_cache());
        let xrp_future = timeout(Duration::from_secs(8), self.fetch_xrp_with_cache());
        let ada_future = timeout(Duration::from_secs(8), self.fetch_ada_with_cache());
        let link_future = timeout(Duration::from_secs(8), self.fetch_link_with_cache());
        let global_future = timeout(Duration::from_secs(8), self.fetch_global_with_cache());
        let fng_future = timeout(Duration::from_secs(8), self.fetch_fng_with_cache());
        let rsi_future = timeout(Duration::from_secs(8), self.fetch_rsi_with_cache());
        let us_indices_future = timeout(Duration::from_secs(8), self.fetch_us_indices_with_cache());

        let (btc_result, eth_result, sol_result, xrp_result, ada_result, link_result, global_result, fng_result, rsi_result, us_indices_result) = tokio::join!(
            btc_future, eth_future, sol_future, xrp_future, ada_future, link_future,
            global_future, fng_future, rsi_future, us_indices_future
        );

        let mut partial_failure = false;

        // Process BTC data
        let (btc_price, btc_change) = match btc_result {
            Ok(Ok(btc_data)) => (
                btc_data["price_usd"].as_f64().unwrap_or(0.0),
                btc_data["change_24h"].as_f64().unwrap_or(0.0)
            ),
            _ => {
                partial_failure = true;
                (0.0, 0.0)
            }
        };

        // Process ETH data
        let (eth_price, eth_change) = match eth_result {
            Ok(Ok(eth_data)) => (
                eth_data["price_usd"].as_f64().unwrap_or(0.0),
                eth_data["change_24h"].as_f64().unwrap_or(0.0)
            ),
            _ => {
                partial_failure = true;
                (0.0, 0.0)
            }
        };

        // Process SOL data
        let (sol_price, sol_change) = match sol_result {
            Ok(Ok(sol_data)) => (
                sol_data["price_usd"].as_f64().unwrap_or(0.0),
                sol_data["change_24h"].as_f64().unwrap_or(0.0)
            ),
            _ => {
                partial_failure = true;
                (0.0, 0.0)
            }
        };

        // Process XRP data
        let (xrp_price, xrp_change) = match xrp_result {
            Ok(Ok(xrp_data)) => (
                xrp_data["price_usd"].as_f64().unwrap_or(0.0),
                xrp_data["change_24h"].as_f64().unwrap_or(0.0)
            ),
            _ => {
                partial_failure = true;
                (0.0, 0.0)
            }
        };

        // Process ADA data
        let (ada_price, ada_change) = match ada_result {
            Ok(Ok(ada_data)) => (
                ada_data["price_usd"].as_f64().unwrap_or(0.0),
                ada_data["change_24h"].as_f64().unwrap_or(0.0)
            ),
            _ => {
                partial_failure = true;
                (0.0, 0.0)
            }
        };

        // Process LINK data
        let (link_price, link_change) = match link_result {
            Ok(Ok(link_data)) => (
                link_data["price_usd"].as_f64().unwrap_or(0.0),
                link_data["change_24h"].as_f64().unwrap_or(0.0)
            ),
            _ => {
                partial_failure = true;
                (0.0, 0.0)
            }
        };

        // Process global data
        let (market_cap, volume_24h, market_cap_change, btc_dominance, eth_dominance) = match global_result {
            Ok(Ok(global_data)) => (
                global_data["market_cap"].as_f64().unwrap_or(0.0),
                global_data["volume_24h"].as_f64().unwrap_or(0.0),
                global_data["market_cap_change_percentage_24h_usd"].as_f64().unwrap_or(0.0),
                global_data["btc_market_cap_percentage"].as_f64().unwrap_or(0.0),
                global_data["eth_market_cap_percentage"].as_f64().unwrap_or(0.0)
            ),
            _ => {
                partial_failure = true;
                (0.0, 0.0, 0.0, 0.0, 0.0)
            }
        };

        // Process FNG data
        let fng_value = match fng_result {
            Ok(Ok(fng_data)) => fng_data["value"].as_u64().unwrap_or(50) as u32,
            _ => {
                partial_failure = true;
                50
            }
        };

        // Process RSI data
        let rsi_value = match rsi_result {
            Ok(Ok(rsi_data)) => rsi_data["value"].as_f64().unwrap_or(50.0),
            _ => {
                partial_failure = true;
                50.0
            }
        };

        // Process US Stock Indices data
        let us_indices = match us_indices_result {
            Ok(Ok(indices_data)) => indices_data["indices"].clone(),
            _ => {
                partial_failure = true;
                serde_json::json!({})
            }
        };

        let duration = start_time.elapsed();

        // Update statistics
        if partial_failure {
            self.partial_failures.fetch_add(1, Ordering::Relaxed);
            println!("⚠️ Dashboard summary v2 aggregated with partial failures in {}ms", duration.as_millis());
        } else {
            self.successful_aggregations.fetch_add(1, Ordering::Relaxed);
            println!("✅ Dashboard summary v2 aggregated successfully in {}ms", duration.as_millis());
        }

        // Return focused summary JSON
        Ok(serde_json::json!({
            "btc_price_usd": btc_price,
            "btc_change_24h": btc_change,
            "eth_price_usd": eth_price,
            "eth_change_24h": eth_change,
            "sol_price_usd": sol_price,
            "sol_change_24h": sol_change,
            "xrp_price_usd": xrp_price,
            "xrp_change_24h": xrp_change,
            "ada_price_usd": ada_price,
            "ada_change_24h": ada_change,
            "link_price_usd": link_price,
            "link_change_24h": link_change,
            "market_cap_usd": market_cap,
            "volume_24h_usd": volume_24h,
            "market_cap_change_percentage_24h_usd": market_cap_change,
            "btc_market_cap_percentage": btc_dominance,
            "eth_market_cap_percentage": eth_dominance,
            "fng_value": fng_value,
            "rsi_14": rsi_value,
            "us_stock_indices": us_indices,
            "fetch_duration_ms": duration.as_millis() as u64,
            "partial_failure": partial_failure,
            "last_updated": chrono::Utc::now().to_rfc3339(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}