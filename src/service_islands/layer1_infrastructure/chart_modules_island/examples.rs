//! Chart Modules Island Usage Examples
//! 
//! This file demonstrates how to use the ChartModulesIsland in various scenarios.

use crate::service_islands::layer1_infrastructure::ChartModulesIsland;

/// Example of basic chart modules usage
pub async fn example_basic_usage() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Chart Modules Usage Example ===");
    
    // Create a new chart modules island
    let chart_modules = ChartModulesIsland::new();
    
    // Health check
    let is_healthy = chart_modules.health_check().await;
    println!("Chart modules island health: {}", if is_healthy { "✅ Healthy" } else { "❌ Unhealthy" });
    
    // Get available modules
    let available = chart_modules.get_available_modules().await;
    println!("Available chart modules: {:?}", available);
    
    // Load all chart modules content
    let content = chart_modules.get_chart_modules_content().await;
    println!("Chart modules content loaded: {} characters", content.len());
    
    Ok(())
}

/// Example of custom configuration usage
pub async fn example_custom_config() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom Configuration Example ===");
    
    // Create chart modules with custom configuration
    let custom_priority = vec![
        "line.js".to_string(),
        "bar.js".to_string(),
        "gauge.js".to_string(),
        "doughnut.js".to_string(),
    ];
    
    let chart_modules = ChartModulesIsland::with_config(
        "shared_assets/js/chart_modules".to_string(),
        custom_priority,
    );
    
    // Load content with custom priority
    let content = chart_modules.get_chart_modules_content().await;
    println!("Custom prioritized content loaded: {} characters", content.len());
    
    // Show priority order
    let priority = chart_modules.get_priority_order();
    println!("Current priority order: {:?}", priority);
    
    Ok(())
}

/// Example of integration with business logic
pub async fn example_business_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Business Logic Integration Example ===");
    
    use crate::service_islands::layer5_business_logic::crypto_reports::report_creator::ReportCreator;
    
    // Create report creator (which includes chart modules island)
    let report_creator = ReportCreator::new();
    
    // Health check through business logic
    let is_healthy = report_creator.health_check().await;
    println!("Report creator health (includes chart modules): {}", if is_healthy { "✅ Healthy" } else { "❌ Unhealthy" });
    
    // Get chart modules through business logic layer
    let content = report_creator.get_chart_modules_content().await;
    println!("Chart modules via business logic: {} characters", content.len());
    
    // Get available modules through business logic
    let available = report_creator.get_available_chart_modules().await;
    println!("Available modules via business logic: {:?}", available);
    
    Ok(())
}

/// Run all examples
pub async fn run_all_examples() -> Result<(), Box<dyn std::error::Error>> {
    example_basic_usage().await?;
    println!();
    
    example_custom_config().await?;
    println!();
    
    example_business_integration().await?;
    
    Ok(())
}
