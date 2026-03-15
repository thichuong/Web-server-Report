use anyhow::Result;
use std::path::Path;
use tracing::{debug, info, warn};

/// Pre-load and concatenate all chart modules JavaScript files
///
/// # Errors
/// Returns an error if the directory cannot be read or files cannot be accessed.
pub fn load_chart_modules() -> Result<String> {
    debug!("📦 Loading chart modules...");

    // Default priority order for chart modules
    let priority_order = vec![
        "gauge.js".to_string(),
        "bar.js".to_string(),
        "line.js".to_string(),
        "pie.js".to_string(),
        "doughnut.js".to_string(),
    ];

    let source_dir = Path::new("shared_assets").join("js").join("chart_modules");

    if !source_dir.exists() {
        warn!("⚠️ Chart modules directory not found: {:?}", source_dir);
        return Ok(String::new());
    }

    let mut final_content = String::new();
    final_content.push_str("// Chart Modules - Pre-loaded during server startup\n\n");

    // Read all files in the directory
    let dir_entries = std::fs::read_dir(&source_dir)?;
    let mut all_files = Vec::new();

    for entry in dir_entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && path.extension().is_some_and(|ext| ext == "js")
            && let Some(file_name) = path.file_name().and_then(|n| n.to_str())
        {
            all_files.push(file_name.to_string());
        }
    }

    // Sort files according to priority order
    let mut sorted_files = Vec::new();

    // First, add files in priority order
    for priority_file in &priority_order {
        if all_files.contains(priority_file) {
            sorted_files.push(priority_file.clone());
        }
    }

    // Then add remaining files
    for file in &all_files {
        if !sorted_files.contains(file) {
            sorted_files.push(file.clone());
        }
    }

    debug!("📦 Loading chart modules in order: {:?}", sorted_files);

    // Read and concatenate files
    for file_name in &sorted_files {
        let file_path = source_dir.join(file_name);
        match std::fs::read_to_string(&file_path) {
            Ok(content) => {
                use std::fmt::Write;
                writeln!(final_content, "// === {file_name} ===").ok();
                final_content.push_str(&content);
                final_content.push_str("\n\n");
                debug!("  ✅ Loaded: {} ({} bytes)", file_name, content.len());
            }
            Err(e) => {
                warn!("  ⚠️ Failed to load {}: {}", file_name, e);
            }
        }
    }

    info!("✅ Chart modules loaded. ({} bytes)", final_content.len());
    Ok(final_content)
}
