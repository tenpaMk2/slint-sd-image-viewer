use crate::config::SUPPORTED_IMAGE_EXTENSIONS;
use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};

pub fn scan_directory(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut image_files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    if SUPPORTED_IMAGE_EXTENSIONS.contains(&ext_str.to_lowercase().as_str()) {
                        image_files.push(path);
                    }
                }
            }
        }
    }

    image_files.sort();
    Ok(image_files)
}
