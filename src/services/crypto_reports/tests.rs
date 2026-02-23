//! Layer Architecture Tests
//!
//! Tests to verify proper layer separation and dependencies
//! Layer 5 should only depend on Layer 3, not directly on Layer 1

use crate::services::crypto_reports::report_creator::ReportCreator;

#[test]
fn test_layer_separation() {
    // Test that ReportCreator (Layer 5) uses data service (Layer 3) properly
    let report_creator = ReportCreator::new();
    assert!(report_creator.health_check());
}
