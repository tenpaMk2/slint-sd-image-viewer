use crate::config::SUPPORTED_IMAGE_EXTENSIONS;
use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};

pub fn scan_directory(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut image_files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext_str| {
                        SUPPORTED_IMAGE_EXTENSIONS.contains(&ext_str.to_lowercase().as_str())
                    })
                    .unwrap_or(false)
        })
        .collect();

    image_files.sort();
    Ok(image_files)
}
