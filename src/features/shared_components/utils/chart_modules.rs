// src/features/shared_components/utils/chart_modules.rs
//
// Chart modules utility for loading and bundling JavaScript chart components
// This provides a unified way to load all chart modules (gauge, bar, line, doughnut)
// with error handling and caching support.

use std::{env, path::Path};
use tokio::fs::read_dir;

/// Load and bundle all chart modules from shared_assets/js/chart_modules/
/// 
/// This function:
/// 1. Reads all .js files from the chart_modules directory
/// 2. Orders them with priority: gauge.js, bar.js, line.js, doughnut.js
/// 3. Wraps each module in try-catch for error isolation
/// 4. Returns concatenated JavaScript content
/// 
/// # Caching
/// - In production: Uses provided cache for performance
/// - In debug mode (DEBUG=1): Bypasses cache for hot reload
/// 
/// # Arguments
/// * `chart_modules_cache` - RwLock cache for storing bundled content
/// 
/// # Returns
/// * Bundled JavaScript content as String
pub async fn get_chart_modules_content(
    chart_modules_cache: &tokio::sync::RwLock<Option<String>>
) -> String {
    // If not in debug mode, try cache first
    let debug = env::var("DEBUG").unwrap_or_default() == "1";
    if !debug {
        if let Some(cached) = chart_modules_cache.read().await.clone() {
            return cached;
        }
    }

    let source_dir = Path::new("shared_assets").join("js").join("chart_modules");
    let priority_order = vec!["gauge.js", "bar.js", "line.js", "doughnut.js"];

    // Read directory entries
    let mut entries = match read_dir(&source_dir).await {
        Ok(rd) => rd,
        Err(e) => {
            eprintln!("Chart modules directory not found: {}", e);
            return "// No chart modules found".to_string();
        }
    };

    // Collect all .js files
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

    // Order files: priority first, then alphabetically
    let mut ordered = Vec::new();
    for p in &priority_order {
        if let Some(idx) = all_files.iter().position(|f| f == p) {
            ordered.push(all_files.remove(idx));
        }
    }
    all_files.sort();
    ordered.extend(all_files);

    // Parallel file reading with concurrent futures
    let file_futures: Vec<_> = ordered
        .iter()
        .map(|filename| {
            let path = source_dir.join(filename);
            let filename_clone = filename.clone();
            async move {
                match tokio::fs::read_to_string(&path).await {
                    Ok(content) => {
                        // Wrap each module in try-catch for error isolation
                        let wrapped = format!(
                            "// ==================== {name} ====================\ntry {{\n{code}\n}} catch (error) {{\n    console.error('Error loading chart module {name}:', error);\n}}\n// ==================== End {name} ====================",
                            name = filename_clone,
                            code = content
                        );
                        wrapped
                    }
                    Err(e) => {
                        eprintln!("Failed to read chart module {}: {}", filename_clone, e);
                        format!("// Warning: {name} not found", name = filename_clone)
                    }
                }
            }
        })
        .collect();

    // Await all file reads concurrently for performance
    let parts = futures::future::join_all(file_futures).await;

    // Final concatenation in CPU thread pool to avoid blocking async runtime
    let final_content = tokio::task::spawn_blocking(move || {
        parts.join("\n\n")
    })
    .await
    .unwrap_or_else(|e| {
        eprintln!("Chart modules concatenation error: {:#?}", e);
        "// Error loading chart modules".to_string()
    });

    // Cache if not in debug mode
    if !debug {
        let mut w = chart_modules_cache.write().await;
        *w = Some(final_content.clone());
    }

    println!("✅ Loaded {} chart modules", ordered.len());
    final_content
}

/// Chart module loading configuration
pub struct ChartModulesConfig {
    pub source_directory: String,
    pub priority_order: Vec<String>,
    pub enable_cache: bool,
}

impl Default for ChartModulesConfig {
    fn default() -> Self {
        Self {
            source_directory: "shared_assets/js/chart_modules".to_string(),
            priority_order: vec![
                "gauge.js".to_string(),
                "bar.js".to_string(),
                "line.js".to_string(),
                "doughnut.js".to_string(),
            ],
            enable_cache: true,
        }
    }
}

/// Advanced chart modules loader with configurable options
pub async fn load_chart_modules_with_config(
    config: ChartModulesConfig,
    cache: &tokio::sync::RwLock<Option<String>>,
) -> Result<String, anyhow::Error> {
    // Check cache first if enabled
    if config.enable_cache {
        if let Some(cached) = cache.read().await.clone() {
            return Ok(cached);
        }
    }

    let source_dir = Path::new(&config.source_directory);
    
    // Validate source directory exists
    if !source_dir.exists() {
        anyhow::bail!("Chart modules directory does not exist: {}", config.source_directory);
    }

    // Rest of the loading logic would go here...
    // This is a placeholder for future enhancements
    
    get_chart_modules_content(cache).await;
    Ok("// Advanced loader not yet implemented".to_string())
}
