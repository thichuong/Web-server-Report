//! Chart Modules Island - Layer 1 Infrastructure
//!
//! This island provides chart modules loading and management functionality.
//! It handles the reading, concatenation, and serving of JavaScript chart modules
//! from the `shared_assets` directory.

use std::env;
use std::path::Path;
use tracing::{debug, error, info, warn};

/// Chart Modules Island
///
/// Handles the loading and management of chart JavaScript modules.
/// This is a foundational infrastructure service that provides chart modules
/// to higher-level services in the architecture.
#[derive(Clone)]
pub struct ChartModulesIsland {
    /// Base directory for chart modules
    base_dir: String,
    /// Priority order for loading chart modules
    priority_order: Vec<String>,
}

impl ChartModulesIsland {
    /// Create a new `ChartModulesIsland`
    #[must_use]
    pub fn new() -> Self {
        Self {
            base_dir: "shared_assets/js/chart_modules".to_string(),
            priority_order: vec![
                "gauge.js".to_string(),
                "bar.js".to_string(),
                "line.js".to_string(),
                "doughnut.js".to_string(),
            ],
        }
    }

    /// Create a new `ChartModulesIsland` with custom configuration
    #[allow(dead_code)]
    #[must_use]
    pub fn with_config(base_dir: String, priority_order: Vec<String>) -> Self {
        Self {
            base_dir,
            priority_order,
        }
    }

    /// Health check for chart modules island
    #[must_use]
    pub fn health_check(&self) -> bool {
        // Check if the base directory exists and is accessible
        let source_dir = Path::new(&self.base_dir);
        source_dir.exists()
    }

    /// Get chart modules content
    ///
    /// Reads and concatenates all JavaScript chart modules from the configured directory.
    /// Returns a single string containing all chart modules wrapped in error handling.
    ///
    /// Features:
    /// - Priority-based loading order
    /// - Error handling for individual modules
    /// - Debug mode support via environment variable
    pub fn get_chart_modules_content(&self) -> String {
        debug!(
            "📊 ChartModulesIsland: Loading chart modules from {}",
            self.base_dir
        );

        // Check debug mode
        let _debug = env::var("DEBUG").unwrap_or_default() == "1";

        let source_dir = Path::new(&self.base_dir);

        // Read directory entries
        let entries = match std::fs::read_dir(source_dir) {
            Ok(rd) => rd,
            Err(e) => {
                error!(
                    "❌ ChartModulesIsland: Error reading directory {}: {}",
                    self.base_dir, e
                );
                return "// No chart modules found".to_string();
            }
        };

        // Collect all JavaScript files (case-insensitive check)
        let mut all_files = Vec::new();
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_file() {
                    if let Some(name) = entry.file_name().to_str() {
                        // Use Path::extension() for proper case-insensitive extension checking
                        if std::path::Path::new(name)
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("js"))
                        {
                            all_files.push(name.to_string());
                        }
                    }
                }
            }
        }

        if all_files.is_empty() {
            warn!(
                "⚠️ ChartModulesIsland: No JavaScript files found in {}",
                self.base_dir
            );
            return "// No chart modules found".to_string();
        }

        // Order files: priority first, then alphabetically
        let mut ordered = Vec::new();
        for priority_file in &self.priority_order {
            if let Some(idx) = all_files.iter().position(|f| f == priority_file) {
                ordered.push(all_files.remove(idx));
            }
        }
        all_files.sort();
        ordered.extend(all_files);

        debug!(
            "📋 ChartModulesIsland: Loading {} chart modules in order: {:?}",
            ordered.len(),
            ordered
        );

        // Synchronous file reading
        // Use into_iter to take ownership and avoid clones
        let content_parts: Vec<String> = ordered
            .into_iter()
            .map(|filename| {
                let path = source_dir.join(&filename);
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        // Pre-allocate string capacity to avoid reallocations
                        let capacity = 100 + filename.len() * 3 + content.len();
                        let mut wrapped = String::with_capacity(capacity);
                        wrapped.push_str("// ==================== ");
                        wrapped.push_str(&filename);
                        wrapped.push_str(" ====================\ntry {\n");
                        wrapped.push_str(&content);
                        wrapped.push_str(
                            "\n} catch (error) {\n    console.error('Error loading chart module ",
                        );
                        wrapped.push_str(&filename);
                        wrapped.push_str(":', error);\n}\n// ==================== End ");
                        wrapped.push_str(&filename);
                        wrapped.push_str(" ====================");

                        debug!("✅ ChartModulesIsland: Loaded chart module {}", filename);
                        wrapped
                    }
                    Err(e) => {
                        error!("❌ ChartModulesIsland: Error loading {}: {}", filename, e);
                        format!("// Warning: {filename} not found - {e}")
                    }
                }
            })
            .collect();

        let final_content = content_parts.join("\n\n");

        info!("✅ ChartModulesIsland: Successfully loaded and concatenated all chart modules");
        final_content
    }

    /// Get chart modules content with caching support
    ///
    /// This method will integrate with the caching system when implemented.
    /// For now, it delegates to the main loading method.
    #[must_use]
    pub fn get_cached_chart_modules_content(&self) -> String {
        // TODO: Implement caching integration with Layer 1 Cache System Island
        // For now, always load from files
        self.get_chart_modules_content()
    }

    /// Get available chart module names
    ///
    /// Returns a list of available chart module file names without loading content.
    #[allow(dead_code)]
    #[must_use]
    pub fn get_available_modules(&self) -> Vec<String> {
        let source_dir = Path::new(&self.base_dir);

        let Ok(entries) = std::fs::read_dir(source_dir) else {
            return Vec::new();
        };

        let mut modules = Vec::new();
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_file() {
                    if let Some(name) = entry.file_name().to_str() {
                        // Use Path::extension() for proper case-insensitive extension checking
                        if std::path::Path::new(name)
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("js"))
                        {
                            modules.push(name.to_string());
                        }
                    }
                }
            }
        }

        modules.sort();
        modules
    }

    /// Set priority order for chart modules loading
    #[allow(dead_code)]
    pub fn set_priority_order(&mut self, priority_order: Vec<String>) {
        self.priority_order = priority_order;
    }

    /// Get current priority order
    #[allow(dead_code)]
    #[must_use]
    pub fn get_priority_order(&self) -> &Vec<String> {
        &self.priority_order
    }

    /// Set base directory for chart modules
    #[allow(dead_code)]
    pub fn set_base_dir(&mut self, base_dir: String) {
        self.base_dir = base_dir;
    }

    /// Get current base directory
    #[allow(dead_code)]
    #[must_use]
    pub fn get_base_dir(&self) -> &String {
        &self.base_dir
    }
}

impl Default for ChartModulesIsland {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ChartModulesIsland {
    fn drop(&mut self) {
        // Cleanup confirmation for debugging
        debug!(
            "🧹 ChartModulesIsland: Cleanup completed (base_dir: {})",
            self.base_dir
        );
    }
}
