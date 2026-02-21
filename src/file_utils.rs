use crate::config::SUPPORTED_IMAGE_EXTENSIONS;
use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Extension trait for Path to add logging utilities.
pub trait PathExt {
    /// Formats a file path for compact logging.
    /// Returns the first 10 characters, "...", and the last 10 characters of the filename.
    fn format_for_log(&self) -> String;
}

impl PathExt for Path {
    fn format_for_log(&self) -> String {
        let filename = self.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if filename.len() <= 23 {
            filename.to_string()
        } else {
            let chars: Vec<char> = filename.chars().collect();
            let first: String = chars.iter().take(10).collect();
            let last: String = chars.iter().rev().take(10).rev().collect();
            format!("{}...{}", first, last)
        }
    }
}

/// Checks if a file is a supported image based on its extension.
pub fn is_supported_image(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext_str| SUPPORTED_IMAGE_EXTENSIONS.contains(&ext_str.to_lowercase().as_str()))
            .unwrap_or(false)
}

/// Scans a directory and returns a sorted list of supported image files.
pub fn scan_directory(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut image_files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| is_supported_image(path))
        .collect();

    image_files.sort();
    Ok(image_files)
}
