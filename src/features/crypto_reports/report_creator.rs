//! Report Creator - Report creation and business logic management
//!
//! Handles new report generation, data processing,
//! and business logic for crypto report creation.

use crate::features::external_apis::ExternalApis;
use crate::features::cache_system::CacheSystem;
use crate::models::Report;
use serde_json::json;
use std::error::Error as StdError;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

/// Report creation component
pub struct ReportCreator {
    external_apis: Option<Arc<ExternalApis>>,
    cache_system: Option<Arc<CacheSystem>>,
    next_report_id: AtomicI32,
}

impl ReportCreator {
    pub fn new(external_apis: &ExternalApis, cache_system: &CacheSystem) -> Self {
        Self {
            external_apis: Some(Arc::new(external_apis.clone())),
            cache_system: Some(Arc::new(cache_system.clone())),
            next_report_id: AtomicI32::new(1000), // Start from 1000 for mock reports
        }
    }

    /// Initialize report creator
    pub async fn initialize(&self) -> Result<(), Box<dyn StdError + Send + Sync>> {
        println!("📊 Initializing Report Creator component");
        
        // Initialize next report ID from database (mock for now)
        let latest_id = self.get_latest_report_id().await.unwrap_or(1000);
        self.next_report_id.store(latest_id + 1, Ordering::Relaxed);
        
        Ok(())
    }

    /// Create new crypto report
    pub async fn create_new_report(&self, input_data: serde_json::Value) -> Result<Report, Box<dyn StdError + Send + Sync>> {
        println!("🏗️ Creating new crypto report");
        
        // Get next report ID
        let report_id = self.next_report_id.fetch_add(1, Ordering::Relaxed);
        
        // Fetch market data if external APIs are available
        let market_data = self.fetch_market_data().await?;
        
        // Generate report content
        let report_content = self.generate_report_content(report_id, &market_data, &input_data).await?;
        
        // Create report object
        let report = Report {
            id: report_id,
            html_content: report_content.html_content,
            html_content_en: report_content.html_content_en,
            css_content: Some(report_content.css_content),
            js_content: Some(report_content.js_content),
            js_content_en: None,
            created_at: chrono::Utc::now(),
        };

        // Save to database (mock for now)
        self.save_report_to_database(&report).await?;
        
        // Cache the new report
        if let Some(cache) = &self.cache_system {
            let cache_key = format!("crypto_report:{}", report.id);
            if let Err(e) = cache.set(&cache_key, &report).await {
                eprintln!("⚠️ Failed to cache new report: {}", e);
            } else {
                println!("💾 Cached new report {} in cache", report.id);
            }
        }

        println!("✅ Successfully created crypto report #{}", report.id);
        Ok(report)
    }

    /// Generate comprehensive report content
    async fn generate_report_content(
        &self, 
        report_id: i32, 
        market_data: &serde_json::Value,
        input_data: &serde_json::Value
    ) -> Result<ReportContent, Box<dyn StdError + Send + Sync>> {
        
        // Extract market metrics
        let bitcoin_price = market_data.get("bitcoin_price").and_then(|v| v.as_f64()).unwrap_or(45000.0);
        let ethereum_price = market_data.get("ethereum_price").and_then(|v| v.as_f64()).unwrap_or(3000.0);
        let market_cap = market_data.get("total_market_cap").and_then(|v| v.as_f64()).unwrap_or(1_800_000_000_000.0);
        
        // Generate Vietnamese content
        let html_content_vi = format!(r#"
            <section class="report-overview">
                <h2>Tổng Quan Thị Trường</h2>
                <div class="market-summary">
                    <p>Báo cáo thị trường tiền mã hóa ngày {} cho thấy những diễn biến đáng chú ý trong thị trường tài sản số.</p>
                    <div class="highlight-stats">
                        <div class="stat-card">
                            <h3>Tổng Giá Trị Thị Trường</h3>
                            <p class="stat-value">${:.2} Tỷ USD</p>
                        </div>
                        <div class="stat-card">
                            <h3>Số Lượng Đồng Coin Hoạt Động</h3>
                            <p class="stat-value">10,000+</p>
                        </div>
                    </div>
                </div>
            </section>

            <section class="top-cryptocurrencies">
                <h2>Top Tiền Mã Hóa Hàng Đầu</h2>
                <div class="crypto-grid">
                    <div class="crypto-card featured">
                        <div class="crypto-header">
                            <img src="/assets/icons/bitcoin.png" alt="Bitcoin" class="crypto-icon">
                            <div class="crypto-info">
                                <h3>Bitcoin (BTC)</h3>
                                <p class="crypto-rank">#1</p>
                            </div>
                        </div>
                        <div class="crypto-metrics">
                            <div class="price-info">
                                <p class="current-price">${:.2}</p>
                                <p class="price-change positive">+2.4%</p>
                            </div>
                            <div class="volume-info">
                                <p class="label">Khối lượng 24h</p>
                                <p class="value">$28.5B</p>
                            </div>
                        </div>
                    </div>

                    <div class="crypto-card">
                        <div class="crypto-header">
                            <img src="/assets/icons/ethereum.png" alt="Ethereum" class="crypto-icon">
                            <div class="crypto-info">
                                <h3>Ethereum (ETH)</h3>
                                <p class="crypto-rank">#2</p>
                            </div>
                        </div>
                        <div class="crypto-metrics">
                            <div class="price-info">
                                <p class="current-price">${:.2}</p>
                                <p class="price-change positive">+1.8%</p>
                            </div>
                            <div class="volume-info">
                                <p class="label">Khối lượng 24h</p>
                                <p class="value">$15.3B</p>
                            </div>
                        </div>
                    </div>
                </div>
            </section>

            <section class="market-analysis">
                <h2>Phân Tích Thị Trường</h2>
                <div class="analysis-content">
                    <div class="trend-analysis">
                        <h3>Xu Hướng Tăng Trưởng</h3>
                        <p>Thị trường đang cho thấy sự phục hồi mạnh mẽ sau giai đoạn điều chỉnh. Bitcoin và Ethereum dẫn đầu đà tăng trưởng với thanh khoản cao.</p>
                        
                        <div class="key-insights">
                            <div class="insight-item">
                                <i class="fas fa-chart-line insight-icon positive"></i>
                                <div>
                                    <h4>Tích Cực</h4>
                                    <p>Khối lượng giao dịch tăng 15% so với tuần trước</p>
                                </div>
                            </div>
                            <div class="insight-item">
                                <i class="fas fa-users insight-icon positive"></i>
                                <div>
                                    <h4>Sự Quan Tâm Tăng Cao</h4>
                                    <p>Số lượng ví mới tạo tăng 8% trong 24h qua</p>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="risk-assessment">
                        <h3>Đánh Giá Rủi Ro</h3>
                        <div class="risk-indicators">
                            <div class="risk-item low">
                                <span class="risk-level">Thấp</span>
                                <span class="risk-desc">Thanh khoản ổn định</span>
                            </div>
                            <div class="risk-item medium">
                                <span class="risk-level">Trung Bình</span>
                                <span class="risk-desc">Biến động giá</span>
                            </div>
                        </div>
                    </div>
                </div>
            </section>

            <section class="investment-recommendations">
                <h2>Khuyến Nghị Đầu Tư</h2>
                <div class="recommendations-grid">
                    <div class="recommendation-card buy">
                        <h3>Khuyến Nghị MUA</h3>
                        <div class="recommended-coins">
                            <div class="coin-recommendation">
                                <strong>Bitcoin (BTC)</strong>
                                <p>Mục tiêu giá: $50,000 (+10.5%)</p>
                                <p class="timeframe">Thời gian: 1-2 tháng</p>
                            </div>
                        </div>
                    </div>
                    
                    <div class="recommendation-card hold">
                        <h3>Khuyến Nghị GIỮ</h3>
                        <div class="recommended-coins">
                            <div class="coin-recommendation">
                                <strong>Ethereum (ETH)</strong>
                                <p>Theo dõi mức hỗ trợ $2,900</p>
                                <p class="timeframe">Đánh giá lại sau 1 tuần</p>
                            </div>
                        </div>
                    </div>
                </div>
            </section>

            <section class="market-outlook">
                <h2>Triển Vọng Thị Trường</h2>
                <div class="outlook-content">
                    <div class="short-term">
                        <h3>Ngắn Hạn (1-4 tuần)</h3>
                        <p>Thị trường có thể tiếp tục xu hướng tích cực với sự hỗ trợ từ các yếu tố cơ bản mạnh mẽ.</p>
                    </div>
                    <div class="long-term">
                        <h3>Dài Hạn (3-6 tháng)</h3>
                        <p>Triển vọng dài hạn vẫn lạc quan với sự phát triển của công nghệ blockchain và việc áp dụng rộng rãi hơn.</p>
                    </div>
                </div>
            </section>

            <footer class="report-footer">
                <div class="disclaimer">
                    <h3>Lưu Ý Quan Trọng</h3>
                    <p><strong>Khuyến cáo rủi ro:</strong> Đầu tư tiền mã hóa mang tính rủi ro cao. Các thông tin trong báo cáo này chỉ mang tính chất tham khảo và không phải là lời khuyên đầu tư cụ thể. Nhà đầu tư nên tự nghiên cứu và đánh giá kỹ trước khi đưa ra quyết định đầu tư.</p>
                </div>
                <div class="report-meta">
                    <p>Báo cáo được tạo bởi AI Investment Analysis System</p>
                    <p>Thời gian tạo: {}</p>
                    <p>ID Báo cáo: #{}</p>
                </div>
            </footer>
        "#, 
            chrono::Utc::now().format("%d/%m/%Y"),
            market_cap / 1_000_000_000_000.0,  // Convert to trillions
            bitcoin_price,
            ethereum_price,
            chrono::Utc::now().format("%d/%m/%Y %H:%M UTC"),
            report_id
        );

        // Generate English content
        let html_content_en = format!(r#"
            <section class="report-overview">
                <h2>Market Overview</h2>
                <div class="market-summary">
                    <p>Cryptocurrency market report for {} shows significant developments in the digital asset market.</p>
                    <div class="highlight-stats">
                        <div class="stat-card">
                            <h3>Total Market Cap</h3>
                            <p class="stat-value">${:.2} Trillion USD</p>
                        </div>
                        <div class="stat-card">
                            <h3>Active Cryptocurrencies</h3>
                            <p class="stat-value">10,000+</p>
                        </div>
                    </div>
                </div>
            </section>

            <section class="top-cryptocurrencies">
                <h2>Top Cryptocurrencies</h2>
                <div class="crypto-grid">
                    <div class="crypto-card featured">
                        <div class="crypto-header">
                            <img src="/assets/icons/bitcoin.png" alt="Bitcoin" class="crypto-icon">
                            <div class="crypto-info">
                                <h3>Bitcoin (BTC)</h3>
                                <p class="crypto-rank">#1</p>
                            </div>
                        </div>
                        <div class="crypto-metrics">
                            <div class="price-info">
                                <p class="current-price">${:.2}</p>
                                <p class="price-change positive">+2.4%</p>
                            </div>
                            <div class="volume-info">
                                <p class="label">24h Volume</p>
                                <p class="value">$28.5B</p>
                            </div>
                        </div>
                    </div>

                    <div class="crypto-card">
                        <div class="crypto-header">
                            <img src="/assets/icons/ethereum.png" alt="Ethereum" class="crypto-icon">
                            <div class="crypto-info">
                                <h3>Ethereum (ETH)</h3>
                                <p class="crypto-rank">#2</p>
                            </div>
                        </div>
                        <div class="crypto-metrics">
                            <div class="price-info">
                                <p class="current-price">${:.2}</p>
                                <p class="price-change positive">+1.8%</p>
                            </div>
                            <div class="volume-info">
                                <p class="label">24h Volume</p>
                                <p class="value">$15.3B</p>
                            </div>
                        </div>
                    </div>
                </div>
            </section>

            <section class="investment-recommendations">
                <h2>Investment Recommendations</h2>
                <div class="recommendations-grid">
                    <div class="recommendation-card buy">
                        <h3>BUY Recommendation</h3>
                        <div class="recommended-coins">
                            <div class="coin-recommendation">
                                <strong>Bitcoin (BTC)</strong>
                                <p>Price target: $50,000 (+10.5%)</p>
                                <p class="timeframe">Timeframe: 1-2 months</p>
                            </div>
                        </div>
                    </div>
                    
                    <div class="recommendation-card hold">
                        <h3>HOLD Recommendation</h3>
                        <div class="recommended-coins">
                            <div class="coin-recommendation">
                                <strong>Ethereum (ETH)</strong>
                                <p>Monitor support level at $2,900</p>
                                <p class="timeframe">Re-evaluate in 1 week</p>
                            </div>
                        </div>
                    </div>
                </div>
            </section>

            <footer class="report-footer">
                <div class="disclaimer">
                    <h3>Important Notice</h3>
                    <p><strong>Risk Warning:</strong> Cryptocurrency investment carries high risk. Information in this report is for reference only and does not constitute specific investment advice. Investors should conduct their own research and evaluation before making investment decisions.</p>
                </div>
                <div class="report-meta">
                    <p>Report generated by AI Investment Analysis System</p>
                    <p>Generated: {}</p>
                    <p>Report ID: #{}</p>
                </div>
            </footer>
        "#,
            chrono::Utc::now().format("%m/%d/%Y"),
            market_cap / 1_000_000_000_000.0,
            bitcoin_price,
            ethereum_price,
            chrono::Utc::now().format("%m/%d/%Y %H:%M UTC"),
            report_id
        );

        // Generate CSS content
        let css_content = r#"
            .report-overview { margin-bottom: 2rem; padding: 1.5rem; background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 100%); border-radius: 12px; }
            .market-summary p { font-size: 1.1rem; line-height: 1.6; color: #495057; margin-bottom: 1.5rem; }
            .highlight-stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1rem; }
            .stat-card { background: white; padding: 1.5rem; border-radius: 8px; text-align: center; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
            .stat-card h3 { font-size: 0.9rem; color: #6c757d; margin-bottom: 0.5rem; text-transform: uppercase; }
            .stat-value { font-size: 1.8rem; font-weight: bold; color: #007bff; }

            .crypto-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1.5rem; margin-top: 1rem; }
            .crypto-card { background: white; padding: 1.5rem; border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.1); transition: transform 0.3s ease; }
            .crypto-card:hover { transform: translateY(-5px); }
            .crypto-card.featured { border: 2px solid #ffc107; background: linear-gradient(135deg, #fff9c4 0%, #ffffff 100%); }
            .crypto-header { display: flex; align-items: center; margin-bottom: 1rem; }
            .crypto-icon { width: 40px; height: 40px; margin-right: 1rem; }
            .crypto-info h3 { margin: 0; font-size: 1.3rem; color: #333; }
            .crypto-rank { color: #6c757d; font-size: 0.9rem; margin: 0; }
            .crypto-metrics { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }
            .current-price { font-size: 1.5rem; font-weight: bold; color: #333; margin: 0; }
            .price-change { margin: 0; font-weight: 600; }
            .price-change.positive { color: #28a745; }
            .price-change.negative { color: #dc3545; }
            .volume-info .label { font-size: 0.8rem; color: #6c757d; margin: 0; }
            .volume-info .value { font-weight: bold; color: #333; margin: 0; }

            .analysis-content { display: grid; grid-template-columns: 2fr 1fr; gap: 2rem; margin-top: 1rem; }
            .trend-analysis { background: #f8f9fa; padding: 1.5rem; border-radius: 8px; }
            .key-insights { margin-top: 1rem; }
            .insight-item { display: flex; align-items: center; margin-bottom: 1rem; }
            .insight-icon { width: 30px; height: 30px; border-radius: 50%; display: flex; align-items: center; justify-content: center; margin-right: 1rem; }
            .insight-icon.positive { background: #28a745; color: white; }
            .insight-item h4 { margin: 0 0 0.25rem 0; font-size: 1rem; }
            .insight-item p { margin: 0; font-size: 0.9rem; color: #6c757d; }

            .risk-assessment { background: #fff3cd; padding: 1.5rem; border-radius: 8px; }
            .risk-indicators { margin-top: 1rem; }
            .risk-item { display: flex; justify-content: space-between; padding: 0.5rem 0; border-bottom: 1px solid #ffeaa7; }
            .risk-item:last-child { border-bottom: none; }
            .risk-level { font-weight: bold; }
            .risk-item.low .risk-level { color: #28a745; }
            .risk-item.medium .risk-level { color: #ffc107; }

            .recommendations-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1.5rem; margin-top: 1rem; }
            .recommendation-card { padding: 1.5rem; border-radius: 12px; }
            .recommendation-card.buy { background: linear-gradient(135deg, #d4edda 0%, #c3e6cb 100%); border: 2px solid #28a745; }
            .recommendation-card.hold { background: linear-gradient(135deg, #fff3cd 0%, #ffeaa7 100%); border: 2px solid #ffc107; }
            .recommendation-card h3 { color: #333; margin-bottom: 1rem; }
            .coin-recommendation { background: white; padding: 1rem; border-radius: 8px; margin-bottom: 1rem; }
            .coin-recommendation strong { display: block; margin-bottom: 0.5rem; }
            .timeframe { font-style: italic; color: #6c757d; margin-top: 0.5rem; }

            .outlook-content { display: grid; grid-template-columns: 1fr 1fr; gap: 2rem; margin-top: 1rem; }
            .short-term, .long-term { background: #f8f9fa; padding: 1.5rem; border-radius: 8px; }

            .report-footer { margin-top: 3rem; padding-top: 2rem; border-top: 2px solid #dee2e6; }
            .disclaimer { background: #f8d7da; padding: 1.5rem; border-radius: 8px; margin-bottom: 1rem; }
            .disclaimer h3 { color: #721c24; margin-bottom: 1rem; }
            .disclaimer p { color: #721c24; line-height: 1.6; }
            .report-meta { text-align: center; color: #6c757d; font-size: 0.9rem; }
            .report-meta p { margin: 0.25rem 0; }

            @media (max-width: 768px) {
                .analysis-content, .outlook-content { grid-template-columns: 1fr; }
                .crypto-metrics { grid-template-columns: 1fr; text-align: center; }
                .recommendations-grid { grid-template-columns: 1fr; }
            }
        "#.to_string();

        // Generate JavaScript content
        let js_content = format!(r#"
            console.log('Crypto Report #{} JavaScript loaded');
            
            document.addEventListener('DOMContentLoaded', function() {{
                console.log('Initializing crypto report interactions');
                
                // Add interactive hover effects
                const cryptoCards = document.querySelectorAll('.crypto-card');
                cryptoCards.forEach(card => {{
                    card.addEventListener('mouseenter', function() {{
                        this.style.transform = 'translateY(-5px) scale(1.02)';
                        this.style.boxShadow = '0 8px 25px rgba(0,0,0,0.15)';
                    }});
                    
                    card.addEventListener('mouseleave', function() {{
                        this.style.transform = 'translateY(0) scale(1)';
                        this.style.boxShadow = '0 4px 15px rgba(0,0,0,0.1)';
                    }});
                }});
                
                // Add click handlers for recommendation cards
                const recommendationCards = document.querySelectorAll('.recommendation-card');
                recommendationCards.forEach(card => {{
                    card.addEventListener('click', function() {{
                        const cardType = this.classList.contains('buy') ? 'BUY' : 'HOLD';
                        console.log(`Clicked on ${{cardType}} recommendation`);
                        // Could add more interaction logic here
                    }});
                }});
                
                // Initialize any charts or visualizations
                if (typeof initializeReportCharts === 'function') {{
                    initializeReportCharts();
                }}
            }});
            
            // Report-specific utility functions
            function highlightSection(sectionId) {{
                const section = document.getElementById(sectionId);
                if (section) {{
                    section.scrollIntoView({{ behavior: 'smooth' }});
                    section.style.background = '#fff3cd';
                    setTimeout(() => {{
                        section.style.background = '';
                    }}, 2000);
                }}
            }}
            
            // Export report data for external use
            window.reportData = {{
                reportId: {},
                generated: '{}',
                marketData: {{
                    bitcoinPrice: {},
                    ethereumPrice: {},
                    totalMarketCap: {}
                }}
            }};
        "#, 
            report_id,
            report_id,
            chrono::Utc::now().to_rfc3339(),
            bitcoin_price,
            ethereum_price,
            market_cap
        );

        Ok(ReportContent {
            html_content: html_content_vi,
            html_content_en: Some(html_content_en),
            css_content,
            js_content,
        })
    }

    /// Fetch market data from external APIs
    async fn fetch_market_data(&self) -> Result<serde_json::Value, Box<dyn StdError + Send + Sync>> {
        // Mock market data - TODO: Replace with actual API calls
        Ok(json!({
            "bitcoin_price": 45230.50,
            "ethereum_price": 3123.75,
            "total_market_cap": 1_800_000_000_000.0,
            "market_dominance": {
                "bitcoin": 42.5,
                "ethereum": 18.2
            },
            "24h_volume": 85_000_000_000.0,
            "fear_greed_index": 68
        }))
    }

    /// Save report to database (mock implementation)
    async fn save_report_to_database(&self, report: &Report) -> Result<(), Box<dyn StdError + Send + Sync>> {
        // Mock save - TODO: Replace with actual database integration
        println!("📄 Saved report #{} to database (mock)", report.id);
        
        // Simulate database save delay
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        Ok(())
    }

    /// Get latest report ID from database
    async fn get_latest_report_id(&self) -> Result<i32, Box<dyn StdError + Send + Sync>> {
        // Mock implementation - TODO: Replace with actual database query
        Ok(1000)
    }

    /// Check if report creator is healthy
    pub async fn is_healthy(&self) -> bool {
        // Simple health check - could check external API connectivity
        true
    }
}

impl Default for ReportCreator {
    fn default() -> Self {
        Self {
            external_apis: None,
            cache_system: None,
            next_report_id: AtomicI32::new(1000),
        }
    }
}

/// Report content structure
struct ReportContent {
    pub html_content: String,
    pub html_content_en: Option<String>,
    pub css_content: String,
    pub js_content: String,
}
