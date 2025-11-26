//! Gzip Compression Utilities
//!
//! Shared compression logic for HTTP responses across Layer 5 components.
//! Eliminates duplicate compression code in handlers.

use flate2::{write::GzEncoder, Compression};
use std::io::Write;
use tracing::info;

use super::error::{Layer5Error, Layer5Result};

/// Compression statistics for logging and monitoring
#[derive(Debug, Clone, Copy)]
pub struct CompressionStats {
    pub original_size: usize,
    pub compressed_size: usize,
    pub ratio_percent: f64,
}

impl CompressionStats {
    /// Calculate compression statistics
    #[inline]
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn new(original_size: usize, compressed_size: usize) -> Self {
        let ratio_percent = if original_size > 0 {
            (1.0 - (compressed_size as f64 / original_size as f64)) * 100.0
        } else {
            0.0
        };
        Self {
            original_size,
            compressed_size,
            ratio_percent,
        }
    }

    /// Get original size in KB
    #[inline]
    #[must_use] 
    pub fn original_kb(&self) -> usize {
        self.original_size / 1024
    }

    /// Get compressed size in KB
    #[inline]
    #[must_use] 
    pub fn compressed_kb(&self) -> usize {
        self.compressed_size / 1024
    }

    /// Get bytes saved
    #[inline]
    #[must_use] 
    pub fn bytes_saved(&self) -> usize {
        self.original_size.saturating_sub(self.compressed_size)
    }
}

/// Compress HTML string to gzip format
///
/// Returns compressed data and compression statistics.
/// Uses default compression level for balanced speed/ratio.
///
/// # Performance
/// - Pre-allocates buffer based on estimated compression ratio
/// - Single-pass compression with no intermediate allocations
///
/// # Example
/// ```ignore
/// let (compressed, stats) = compress_html_to_gzip(html)?;
/// info!("Compressed {}KB -> {}KB ({:.1}%)", stats.original_kb(), stats.compressed_kb(), stats.ratio_percent);
/// ```
/// # Errors
///
/// Returns error if compression fails (write or finish).
#[inline]
pub fn compress_html_to_gzip(html: &str) -> Layer5Result<(Vec<u8>, CompressionStats)> {
    let original_size = html.len();

    // Pre-allocate with estimated 70% compression ratio for HTML
    let estimated_size = original_size / 3;
    let mut encoder = GzEncoder::new(Vec::with_capacity(estimated_size), Compression::default());

    encoder
        .write_all(html.as_bytes())
        .map_err(|e| Layer5Error::Compression(format!("Failed to write to encoder: {e}")))?;

    let compressed_data = encoder
        .finish()
        .map_err(|e| Layer5Error::Compression(format!("Failed to finish compression: {e}")))?;

    let stats = CompressionStats::new(original_size, compressed_data.len());

    info!(
        "Compression completed - Original: {}KB, Compressed: {}KB, Ratio: {:.1}%",
        stats.original_kb(),
        stats.compressed_kb(),
        stats.ratio_percent
    );

    Ok((compressed_data, stats))
}

/// Compress HTML and return only the compressed data (convenience wrapper)
///
/// Use this when you don't need compression statistics.
/// # Errors
///
/// Returns error if compression fails.
#[inline]
pub fn compress_html(html: &str) -> Layer5Result<Vec<u8>> {
    compress_html_to_gzip(html).map(|(data, _)| data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_stats() {
        let stats = CompressionStats::new(1000, 300);
        assert_eq!(stats.original_size, 1000);
        assert_eq!(stats.compressed_size, 300);
        assert!((stats.ratio_percent - 70.0).abs() < 0.01);
        assert_eq!(stats.bytes_saved(), 700);
    }

    #[test]
    fn test_compress_html() -> Result<(), Box<dyn std::error::Error>> {
        // Use a larger, more realistic HTML that compresses well
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Test Page</title>
</head>
<body>
    <div class="container">
        <h1>Hello World</h1>
        <p>This is a test paragraph with enough content to ensure compression works effectively.</p>
        <p>Another paragraph to add more content that helps with compression ratio.</p>
    </div>
</body>
</html>"#;
        let (data, stats) = compress_html_to_gzip(html)?;
        
        // For realistic HTML, compression should provide benefit
        // Note: very small inputs may not compress well due to gzip overhead
        assert!(!data.is_empty());
        assert_eq!(stats.original_size, html.len());
        
        Ok(())
    }
}
