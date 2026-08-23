//! Dashboard data response DTOs

use serde::{Deserialize, Serialize};

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

    // Metadata
    pub fetch_duration_ms: u64,
    pub partial_failure: bool,
    pub last_updated: String,
    pub timestamp: String,

    /// Optional note field (only present in fallback scenarios)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_dashboard_data_serialization() {
        let response = DashboardDataResponse {
            btc_price_usd: 60000.0,
            btc_change_24h: 1.5,
            btc_market_cap_percentage: 52.0,
            btc_rsi_14: 65.0,
            eth_price_usd: 3500.0,
            eth_change_24h: 2.0,
            eth_market_cap_percentage: 17.0,
            bnb_price_usd: 600.0,
            bnb_change_24h: 0.5,
            sol_price_usd: 150.0,
            sol_change_24h: 3.0,
            xrp_price_usd: 0.6,
            xrp_change_24h: -1.0,
            ada_price_usd: 0.45,
            ada_change_24h: -0.5,
            link_price_usd: 18.0,
            link_change_24h: 1.2,
            market_cap_usd: 2_500_000_000_000.0,
            market_cap_change_percentage_24h_usd: 1.0,
            volume_24h_usd: 100_000_000_000.0,
            fng_value: 75,
            fetch_duration_ms: 150,
            partial_failure: false,
            last_updated: "2024-03-20T10:00:00Z".to_string(),
            timestamp: "2024-03-20T10:00:00Z".to_string(),
            note: None,
        };

        let serialized = serde_json::to_string(&response).expect("Failed to serialize");
        let deserialized: DashboardDataResponse =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert!((deserialized.btc_price_usd - 60000.0).abs() < f64::EPSILON);
        assert_eq!(deserialized.fng_value, 75);
        assert!(deserialized.note.is_none());
    }

    #[test]
    fn test_dashboard_data_with_note_serialization() {
        let json_data = json!({
            "btc_price_usd": 60000.0,
            "btc_change_24h": 1.5,
            "btc_market_cap_percentage": 52.0,
            "btc_rsi_14": 65.0,
            "eth_price_usd": 3500.0,
            "eth_change_24h": 2.0,
            "eth_market_cap_percentage": 17.0,
            "bnb_price_usd": 600.0,
            "bnb_change_24h": 0.5,
            "sol_price_usd": 150.0,
            "sol_change_24h": 3.0,
            "xrp_price_usd": 0.6,
            "xrp_change_24h": -1.0,
            "ada_price_usd": 0.45,
            "ada_change_24h": -0.5,
            "link_price_usd": 18.0,
            "link_change_24h": 1.2,
            "market_cap_usd": 2_500_000_000_000.0,
            "market_cap_change_percentage_24h_usd": 1.0,
            "volume_24h_usd": 1_000_000_000_000.0,
            "fng_value": 75,
            "fetch_duration_ms": 150,
            "partial_failure": true,
            "last_updated": "2024-03-20T10:00:00Z",
            "timestamp": "2024-03-20T10:00:00Z",
            "note": "Fallback active"
        });

        let deserialized: DashboardDataResponse =
            serde_json::from_value(json_data).expect("Failed to deserialize");
        assert_eq!(deserialized.note, Some("Fallback active".to_string()));
    }
}
