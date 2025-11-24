//! Health System Island - Layer 4: Observability
//!
//! This island handles all system monitoring and health checking operations including:
//! - Comprehensive health monitoring
//! - SSL certificate validation
//! - Performance metrics tracking
//! - Connectivity testing for external services
//! - Alerting and reporting

use tracing::info;

pub mod connectivity_tester;
pub mod health_checker;
pub mod performance_monitor;
pub mod ssl_tester;

/// Health System Island
///
/// The main health system service island that coordinates all monitoring-related
/// functionality. This island is responsible for system health monitoring,
/// performance tracking, and external service connectivity testing.
pub struct HealthSystemIsland {
    pub health_checker: health_checker::HealthChecker,
    pub ssl_tester: ssl_tester::SslTester,
    pub performance_monitor: performance_monitor::PerformanceMonitor,
    pub connectivity_tester: connectivity_tester::ConnectivityTester,
}

impl HealthSystemIsland {
    /// Initialize the Health System Island
    ///
    /// Creates a new Health System Island with all its components properly initialized.
    pub async fn new() -> Result<Self, anyhow::Error> {
        info!("🔍 Initializing Health System Island...");

        let health_checker = health_checker::HealthChecker::new();
        let ssl_tester = ssl_tester::SslTester::new();
        let performance_monitor = performance_monitor::PerformanceMonitor::new();
        let connectivity_tester = connectivity_tester::ConnectivityTester::new();

        info!("✅ Health System Island initialized successfully!");

        Ok(Self {
            health_checker,
            ssl_tester,
            performance_monitor,
            connectivity_tester,
        })
    }

    /// Health check for Health System Island
    ///
    /// Verifies that all components of the Health System Island are functioning properly.
    pub async fn health_check(&self) -> bool {
        // Check all components
        let checker_ok = self.health_checker.health_check().await;
        let ssl_ok = self.ssl_tester.health_check().await;
        let monitor_ok = self.performance_monitor.health_check().await;
        let connectivity_ok = self.connectivity_tester.health_check().await;

        checker_ok && ssl_ok && monitor_ok && connectivity_ok
    }
}
