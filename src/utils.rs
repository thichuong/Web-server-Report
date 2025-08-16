use std::{env, path::Path};
use tokio::fs::read_dir;

pub async fn get_chart_modules_content(chart_modules_cache: &tokio::sync::RwLock<Option<String>>) -> String {
    // If not in debug mode, try cache first
    let debug = env::var("DEBUG").unwrap_or_default() == "1";
    if !debug {
        if let Some(cached) = chart_modules_cache.read().await.clone() {
            return cached;
        }
    }

    let source_dir = Path::new("shared_assets").join("js").join("chart_modules");
    let priority_order = vec!["gauge.js", "bar.js", "line.js", "doughnut.js"];

    let mut entries = match read_dir(&source_dir).await {
        Ok(rd) => rd,
        Err(_) => return "// No chart modules found".to_string(),
    };

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

    // Parallel file reading với concurrent futures
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
                        wrapped
                    }
                    Err(_) => {
                        format!("// Warning: {name} not found", name = filename_clone)
                    }
                }
            }
        })
        .collect();

    // Await all file reads concurrently
    let parts = futures::future::join_all(file_futures).await;

    // Final concatenation trong CPU thread pool để avoid blocking async runtime
    let final_content = tokio::task::spawn_blocking(move || {
        parts.join("\n\n")
    }).await.unwrap_or_else(|e| {
        eprintln!("Chart modules concatenation error: {:#?}", e);
        "// Error loading chart modules".to_string()
    });

    // Cache if not debug
    if !debug {
        let mut w = chart_modules_cache.write().await;
        *w = Some(final_content.clone());
    }

    final_content
}
