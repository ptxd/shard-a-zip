use std::path::Path;

/// Generates output ZIP filename with index
/// e.g., "archive.zip" -> "archive1.zip", "archive2.zip"
pub fn get_output_filename(parent_path: &Path, index: usize) -> std::path::PathBuf {
    let parent_dir = parent_path.parent().unwrap_or(Path::new("."));
    let stem = parent_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    
    parent_dir.join(format!("{}{}.zip", stem, index))
}

/// Formats bytes into human-readable string
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Maximum size for each output ZIP (25 MB)
pub const MAX_ZIP_SIZE: u64 = 25 * 1024 * 1024;

/// Buffer size for streaming (64 KB)
pub const BUFFER_SIZE: usize = 64 * 1024;
