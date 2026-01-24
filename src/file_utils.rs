use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub fn scan_directory(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut image_files = Vec::new();
    let valid_extensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp"];

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    if valid_extensions.contains(&ext_str.to_lowercase().as_str()) {
                        image_files.push(path);
                    }
                }
            }
        }
    }

    image_files.sort();
    Ok(image_files)
}
