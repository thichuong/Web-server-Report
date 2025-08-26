//! Chart Modules Island - Layer 1 Infrastructure
//! 
//! This island provides chart modules loading and management functionality.
//! It handles the reading, concatenation, and serving of JavaScript chart modules
//! from the shared_assets directory.

use std::path::Path;
use std::env;
use tokio::fs::read_dir;

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
    /// Create a new ChartModulesIsland
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

    /// Create a new ChartModulesIsland with custom configuration
    #[allow(dead_code)]
    pub fn with_config(base_dir: String, priority_order: Vec<String>) -> Self {
        Self {
            base_dir,
            priority_order,
        }
    }

    /// Health check for chart modules island
    pub async fn health_check(&self) -> bool {
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
    /// - Concurrent file reading for performance
    /// - Error handling for individual modules
    /// - Debug mode support via environment variable
    pub async fn get_chart_modules_content(&self) -> String {
        println!("📊 ChartModulesIsland: Loading chart modules from {}", self.base_dir);
        
        // Check debug mode
        let _debug = env::var("DEBUG").unwrap_or_default() == "1";
        
        let source_dir = Path::new(&self.base_dir);

        // Read directory entries
        let mut entries = match read_dir(&source_dir).await {
            Ok(rd) => rd,
            Err(e) => {
                eprintln!("❌ ChartModulesIsland: Error reading directory {}: {}", self.base_dir, e);
                return "// No chart modules found".to_string();
            }
        };

        // Collect all JavaScript files
        let mut all_files = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(ft) = entry.file_type().await {
                if ft.is_file() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".js") {
                            all_files.push(name.to_string());
                        }
                    }
                }
            }
        }

        if all_files.is_empty() {
            println!("⚠️ ChartModulesIsland: No JavaScript files found in {}", self.base_dir);
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

        println!("📋 ChartModulesIsland: Loading {} chart modules in order: {:?}", ordered.len(), ordered);

        // Parallel file reading with concurrent futures
        let file_futures: Vec<_> = ordered
            .iter()
            .map(|filename| {
                let path = source_dir.join(filename);
                let filename_clone = filename.clone();
                async move {
                    match tokio::fs::read_to_string(&path).await {
                        Ok(content) => {
                            let wrapped = format!(
                                "// ==================== {name} ====================\ntry {{\n{code}\n}} catch (error) {{\n    console.error('Error loading chart module {name}:', error);\n}}\n// ==================== End {name} ====================",
                                name = filename_clone,
                                code = content
                            );
                            println!("✅ ChartModulesIsland: Loaded chart module {}", filename_clone);
                            wrapped
                        }
                        Err(e) => {
                            eprintln!("❌ ChartModulesIsland: Error loading {}: {}", filename_clone, e);
                            format!("// Warning: {name} not found - {error}", name = filename_clone, error = e)
                        }
                    }
                }
            })
            .collect();

        // Await all file reads concurrently
        let parts = futures::future::join_all(file_futures).await;

        // Final concatenation in CPU thread pool to avoid blocking async runtime
        let final_content = tokio::task::spawn_blocking(move || {
            parts.join("\n\n")
        }).await.unwrap_or_else(|e| {
            eprintln!("❌ ChartModulesIsland: Chart modules concatenation error: {:#?}", e);
            "// Error loading chart modules".to_string()
        });

        println!("✅ ChartModulesIsland: Successfully loaded and concatenated all chart modules");
        final_content
    }

    /// Get chart modules content with caching support
    /// 
    /// This method will integrate with the caching system when implemented.
    /// For now, it delegates to the main loading method.
    pub async fn get_cached_chart_modules_content(&self) -> String {
        // TODO: Implement caching integration with Layer 1 Cache System Island
        // For now, always load from files
        self.get_chart_modules_content().await
    }

    /// Get available chart module names
    /// 
    /// Returns a list of available chart module file names without loading content.
    #[allow(dead_code)]
    pub async fn get_available_modules(&self) -> Vec<String> {
        let source_dir = Path::new(&self.base_dir);

        let mut entries = match read_dir(&source_dir).await {
            Ok(rd) => rd,
            Err(_) => return Vec::new(),
        };

        let mut modules = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(ft) = entry.file_type().await {
                if ft.is_file() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".js") {
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
    pub fn get_base_dir(&self) -> &String {
        &self.base_dir
    }
}

impl Default for ChartModulesIsland {
    fn default() -> Self {
        Self::new()
    }
}
