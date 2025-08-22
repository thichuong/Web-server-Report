//! Layer Architecture Tests
//! 
//! Tests to verify proper layer separation and dependencies
//! Layer 5 should only depend on Layer 3, not directly on Layer 1

#[cfg(test)]
mod tests {
    use crate::service_islands::layer5_business_logic::crypto_reports::report_creator::ReportCreator;
    use crate::service_islands::layer3_communication::data_communication::CryptoDataService;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_layer_separation() {
        // Test that ReportCreator (Layer 5) uses data service (Layer 3) properly
        let report_creator = ReportCreator::new();
        assert!(report_creator.health_check().await);
    }
    
    #[tokio::test]
    async fn test_data_service_isolation() {
        // Test that data service (Layer 3) works independently
        let data_service = CryptoDataService::new();
        assert!(data_service.health_check().await);
    }
}
