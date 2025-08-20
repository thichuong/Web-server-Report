// src/features/cache_system/cache_keys.rs
//
// Cache key generators for consistent key naming

/// Cache key generators for consistent key naming
pub struct CacheKeys;

impl CacheKeys {
    pub fn dashboard_summary() -> String {
        "dashboard:summary".to_string()
    }
    
    pub fn dashboard_summary_non_btc() -> String {
        "dashboard:summary:non_btc".to_string()
    }
    
    pub fn crypto_report(symbol: &str, timeframe: &str) -> String {
        format!("crypto:report:{}:{}", symbol.to_lowercase(), timeframe)
    }
    
    pub fn market_data(symbol: &str) -> String {
        format!("market:{}", symbol.to_lowercase())
    }
    
    pub fn user_report(user_id: u32, report_id: u32) -> String {
        format!("user:{}:report:{}", user_id, report_id)
    }

    /// API data cache keys
    pub fn api_data(provider: &str, endpoint: &str) -> String {
        format!("api:{}:{}", provider.to_lowercase(), endpoint)
    }

    /// Technical indicator cache keys
    pub fn technical_indicator(symbol: &str, indicator: &str, period: &str) -> String {
        format!("tech:{}:{}:{}", symbol.to_lowercase(), indicator, period)
    }

    /// Price data cache keys
    pub fn price_data(symbol: &str, interval: &str) -> String {
        format!("price:{}:{}", symbol.to_lowercase(), interval)
    }

    /// Chart modules cache key
    pub fn chart_modules() -> String {
        "ui:chart_modules".to_string()
    }

    /// Template cache keys
    pub fn template(template_name: &str) -> String {
        format!("template:{}", template_name)
    }

    /// Session data cache keys
    pub fn session_data(session_id: &str, data_type: &str) -> String {
        format!("session:{}:{}", session_id, data_type)
    }

    /// Performance metrics cache key
    pub fn performance_metrics() -> String {
        "system:performance_metrics".to_string()
    }

    /// Health check cache key  
    pub fn health_status() -> String {
        "system:health_status".to_string()
    }
}
