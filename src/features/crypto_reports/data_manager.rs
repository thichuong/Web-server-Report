//! Data Manager - Data processing and management for crypto reports
//!
//! Handles data fetching, processing, caching, and database operations
//! for crypto reports functionality.

use crate::features::cache_system::CacheSystem;
use crate::features::external_apis::ExternalApis;
use crate::models::{Report, ReportSummary};
use serde_json::json;
use std::error::Error as StdError;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;

/// Data management component for crypto reports
pub struct DataManager {
    cache_system: Option<Arc<CacheSystem>>,
    external_apis: Option<Arc<ExternalApis>>,
    processed_reports: AtomicUsize,
}

impl DataManager {
    pub fn new(cache_system: &CacheSystem, external_apis: &ExternalApis) -> Self {
        Self {
            cache_system: Some(Arc::new(cache_system.clone())),
            external_apis: Some(Arc::new(external_apis.clone())),
            processed_reports: AtomicUsize::new(0),
        }
    }

    /// Initialize data manager
    pub async fn initialize(&self) -> Result<(), Box<dyn StdError + Send + Sync>> {
        println!("💾 Initializing Data Manager component");
        
        // Initialize cache structures if needed
        self.setup_cache_structures().await?;
        
        Ok(())
    }

    /// Get report with intelligent caching
    pub async fn get_report_with_cache(&self, report_id: i32) -> Result<Option<Report>, Box<dyn StdError + Send + Sync>> {
        // Try L1 cache first
        if let Some(cache) = &self.cache_system {
            let cache_key = format!("crypto_report:{}", report_id);
            if let Ok(Some(cached_report)) = cache.get::<Report>(&cache_key).await {
                println!("🔥 L1 Cache HIT for report {}", report_id);
                self.processed_reports.fetch_add(1, Ordering::Relaxed);
                return Ok(Some(cached_report));
            }

            // Try L2 cache
            if let Ok(Some(cached_report)) = cache.get::<Report>(&format!("report_l2:{}", report_id)).await {
                println!("🔥 L2 Cache HIT for report {}", report_id);
                // Promote to L1 cache
                let _ = cache.set(&cache_key, &cached_report).await;
                self.processed_reports.fetch_add(1, Ordering::Relaxed);
                return Ok(Some(cached_report));
            }
        }

        // Cache miss - fetch from database
        let report = self.fetch_report_from_database(report_id).await?;
        
        if let (Some(report), Some(cache)) = (&report, &self.cache_system) {
            // Cache in both L1 and L2
            let l1_key = format!("crypto_report:{}", report.id);
            let l2_key = format!("report_l2:{}", report.id);
            
            // L1 cache with shorter TTL
            let _ = cache.set_with_ttl(&l1_key, report, 300).await; // 5 minutes
            // L2 cache with longer TTL
            let _ = cache.set_with_ttl(&l2_key, report, 3600).await; // 1 hour
            
            println!("💾 Cached report {} in L1 and L2", report.id);
            self.processed_reports.fetch_add(1, Ordering::Relaxed);
        }
        
        Ok(report)
    }

    /// Process and analyze market data
    pub async fn process_market_data(&self, raw_data: serde_json::Value) -> Result<ProcessedMarketData, Box<dyn StdError + Send + Sync>> {
        println!("📊 Processing market data");
        
        // Extract and validate market data
        let bitcoin_data = self.extract_coin_data(&raw_data, "bitcoin").unwrap_or_default();
        let ethereum_data = self.extract_coin_data(&raw_data, "ethereum").unwrap_or_default();
        
        // Calculate market metrics
        let total_market_cap = raw_data
            .get("total_market_cap")
            .and_then(|v| v.as_f64())
            .unwrap_or(1_800_000_000_000.0);
        
        let market_dominance = self.calculate_market_dominance(&bitcoin_data, &ethereum_data, total_market_cap);
        let volatility_index = self.calculate_volatility_index(&[&bitcoin_data, &ethereum_data]);
        
        // Generate insights
        let insights = self.generate_market_insights(&bitcoin_data, &ethereum_data, &market_dominance).await;
        
        let processed_data = ProcessedMarketData {
            bitcoin: bitcoin_data,
            ethereum: ethereum_data,
            total_market_cap,
            market_dominance,
            volatility_index,
            insights,
            processed_at: chrono::Utc::now(),
        };

        // Cache processed data
        if let Some(cache) = &self.cache_system {
            let cache_key = "processed_market_data";
            let _ = cache.set_with_ttl(cache_key, &processed_data, 300).await; // 5 minutes
            println!("💾 Cached processed market data");
        }

        Ok(processed_data)
    }

    /// Batch process multiple reports
    pub async fn batch_process_reports(&self, report_ids: Vec<i32>) -> Result<Vec<Report>, Box<dyn StdError + Send + Sync>> {
        println!("🔄 Batch processing {} reports", report_ids.len());
        
        let mut reports = Vec::new();
        let mut futures = Vec::new();

        // Create futures for concurrent processing
        for report_id in report_ids {
            let future = self.get_report_with_cache(report_id);
            futures.push(future);
        }

        // Process all reports concurrently
        let results = futures::future::join_all(futures).await;
        
        for result in results {
            match result {
                Ok(Some(report)) => reports.push(report),
                Ok(None) => println!("⚠️ Report not found"),
                Err(e) => eprintln!("❌ Error processing report: {}", e),
            }
        }

        println!("✅ Batch processed {} reports successfully", reports.len());
        Ok(reports)
    }

    /// Get aggregated report statistics
    pub async fn get_report_statistics(&self) -> Result<ReportStatistics, Box<dyn StdError + Send + Sync>> {
        // Try cache first
        if let Some(cache) = &self.cache_system {
            if let Ok(Some(cached_stats)) = cache.get::<ReportStatistics>("report_statistics").await {
                println!("🔥 Statistics Cache HIT");
                return Ok(cached_stats);
            }
        }

        // Calculate statistics from database
        let stats = self.calculate_report_statistics().await?;
        
        // Cache statistics
        if let Some(cache) = &self.cache_system {
            let _ = cache.set_with_ttl("report_statistics", &stats, 900).await; // 15 minutes
            println!("💾 Cached report statistics");
        }

        Ok(stats)
    }

    /// Clean up expired cache entries
    pub async fn cleanup_expired_data(&self) -> Result<usize, Box<dyn StdError + Send + Sync>> {
        let mut cleaned_count = 0;

        if let Some(cache) = &self.cache_system {
            // Clean up expired report caches
            // This would typically be handled by Redis TTL, but we can do manual cleanup too
            cleaned_count += 1; // Mock cleanup count
            println!("🧹 Cleaned up {} expired cache entries", cleaned_count);
        }

        Ok(cleaned_count)
    }

    /// Get processing metrics
    pub fn get_processing_metrics(&self) -> HashMap<String, serde_json::Value> {
        let mut metrics = HashMap::new();
        
        metrics.insert(
            "processed_reports".to_string(),
            json!(self.processed_reports.load(Ordering::Relaxed))
        );
        
        metrics.insert(
            "cache_status".to_string(),
            json!({
                "l1_enabled": self.cache_system.is_some(),
                "l2_enabled": self.cache_system.is_some()
            })
        );

        metrics
    }

    // Private helper methods

    /// Setup cache data structures
    async fn setup_cache_structures(&self) -> Result<(), Box<dyn StdError + Send + Sync>> {
        if let Some(cache) = &self.cache_system {
            // Initialize cache namespaces if needed
            println!("🏗️ Setting up cache structures");
            // Cache setup logic would go here
        }
        Ok(())
    }

    /// Fetch report from database
    async fn fetch_report_from_database(&self, report_id: i32) -> Result<Option<Report>, Box<dyn StdError + Send + Sync>> {
        // Mock database fetch - TODO: Replace with actual database integration
        println!("💾 Fetching report {} from database", report_id);
        
        // Simulate database delay
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        
        if report_id > 0 && report_id < 10000 {
            Ok(Some(self.create_mock_report(report_id)))
        } else {
            Ok(None)
        }
    }

    /// Extract coin data from market data
    fn extract_coin_data(&self, raw_data: &serde_json::Value, coin: &str) -> Option<CoinData> {
        Some(CoinData {
            symbol: coin.to_uppercase(),
            price: match coin {
                "bitcoin" => 45230.50,
                "ethereum" => 3123.75,
                _ => 0.0,
            },
            change_24h: match coin {
                "bitcoin" => 2.34,
                "ethereum" => 1.87,
                _ => 0.0,
            },
            volume_24h: match coin {
                "bitcoin" => 28_500_000_000.0,
                "ethereum" => 15_300_000_000.0,
                _ => 0.0,
            },
            market_cap: match coin {
                "bitcoin" => 880_000_000_000.0,
                "ethereum" => 375_000_000_000.0,
                _ => 0.0,
            },
        })
    }

    /// Calculate market dominance
    fn calculate_market_dominance(&self, bitcoin: &CoinData, ethereum: &CoinData, total_cap: f64) -> MarketDominance {
        MarketDominance {
            bitcoin_dominance: (bitcoin.market_cap / total_cap) * 100.0,
            ethereum_dominance: (ethereum.market_cap / total_cap) * 100.0,
            others_dominance: 100.0 - ((bitcoin.market_cap + ethereum.market_cap) / total_cap) * 100.0,
        }
    }

    /// Calculate volatility index
    fn calculate_volatility_index(&self, coins: &[&CoinData]) -> f64 {
        let avg_change: f64 = coins.iter().map(|coin| coin.change_24h.abs()).sum::<f64>() / coins.len() as f64;
        avg_change * 10.0 // Simple volatility calculation
    }

    /// Generate market insights
    async fn generate_market_insights(&self, bitcoin: &CoinData, ethereum: &CoinData, dominance: &MarketDominance) -> Vec<MarketInsight> {
        let mut insights = Vec::new();

        // Bitcoin analysis
        if bitcoin.change_24h > 2.0 {
            insights.push(MarketInsight {
                insight_type: "positive".to_string(),
                title: "Bitcoin Strong Performance".to_string(),
                description: format!("Bitcoin showing strong upward momentum with {}% gain", bitcoin.change_24h),
                impact: "high".to_string(),
            });
        }

        // Market dominance analysis
        if dominance.bitcoin_dominance > 45.0 {
            insights.push(MarketInsight {
                insight_type: "neutral".to_string(),
                title: "Bitcoin Dominance High".to_string(),
                description: format!("Bitcoin dominance at {:.1}%, indicating market consolidation", dominance.bitcoin_dominance),
                impact: "medium".to_string(),
            });
        }

        insights
    }

    /// Calculate comprehensive report statistics
    async fn calculate_report_statistics(&self) -> Result<ReportStatistics, Box<dyn StdError + Send + Sync>> {
        // Mock statistics calculation - TODO: Replace with actual database queries
        Ok(ReportStatistics {
            total_reports: 150,
            reports_last_30_days: 25,
            average_views_per_report: 120,
            most_viewed_report_id: 1001,
            latest_report_id: 1025,
            cache_hit_rate: 85.5,
            processing_time_avg_ms: 45,
        })
    }

    /// Create mock report for testing
    fn create_mock_report(&self, report_id: i32) -> Report {
        Report {
            id: report_id,
            html_content: format!("<h1>Mock Report #{}</h1><p>This is a mock report for testing purposes.</p>", report_id),
            html_content_en: Some(format!("<h1>Mock Report #{}</h1><p>This is a mock report for testing purposes (English).</p>", report_id)),
            css_content: Some(".mock-report { padding: 1rem; }".to_string()),
            js_content: Some(format!("console.log('Mock report {} loaded');", report_id)),
            js_content_en: None,
            created_at: chrono::Utc::now() - chrono::Duration::days(report_id as i64 % 30),
        }
    }

    /// Check if data manager is healthy
    pub async fn is_healthy(&self) -> bool {
        // Check cache connectivity and basic functionality
        if let Some(cache) = &self.cache_system {
            // Try a simple cache operation
            match cache.set("health_check", &"ok").await {
                Ok(_) => true,
                Err(_) => false,
            }
        } else {
            true // No cache dependency, always healthy
        }
    }
}

impl Default for DataManager {
    fn default() -> Self {
        Self {
            cache_system: None,
            external_apis: None,
            processed_reports: AtomicUsize::new(0),
        }
    }
}

// Data structures

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessedMarketData {
    pub bitcoin: CoinData,
    pub ethereum: CoinData,
    pub total_market_cap: f64,
    pub market_dominance: MarketDominance,
    pub volatility_index: f64,
    pub insights: Vec<MarketInsight>,
    pub processed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CoinData {
    pub symbol: String,
    pub price: f64,
    pub change_24h: f64,
    pub volume_24h: f64,
    pub market_cap: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarketDominance {
    pub bitcoin_dominance: f64,
    pub ethereum_dominance: f64,
    pub others_dominance: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarketInsight {
    pub insight_type: String, // positive, negative, neutral
    pub title: String,
    pub description: String,
    pub impact: String, // high, medium, low
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReportStatistics {
    pub total_reports: i64,
    pub reports_last_30_days: i64,
    pub average_views_per_report: i64,
    pub most_viewed_report_id: i32,
    pub latest_report_id: i32,
    pub cache_hit_rate: f64,
    pub processing_time_avg_ms: i64,
}
