//! Navigation state for managing image file lists and current position.

use crate::file_utils;
use log::{debug, warn};
use std::path::PathBuf;

/// Manages the current directory, list of image files, and current index.
#[derive(Default)]
pub struct NavigationState {
    current_directory: Option<PathBuf>,
    image_files: Vec<PathBuf>,
    current_index: usize,
    current_file_path: Option<PathBuf>,
    current_rating: Option<u8>,
}

impl NavigationState {
    /// Creates a new empty navigation state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the path to the next image in the list, if available.
    pub fn next_image(&mut self) -> Option<PathBuf> {
        if !self.image_files.is_empty() && self.current_index + 1 < self.image_files.len() {
            self.current_index += 1;
            let path = self.image_files[self.current_index].clone();
            self.current_file_path = Some(path.clone());
            self.current_rating = None; // Reset rating until loaded
            Some(path)
        } else {
            warn!("No next image available");
            None
        }
    }

    /// Returns the path to the previous image in the list, if available.
    pub fn prev_image(&mut self) -> Option<PathBuf> {
        if !self.image_files.is_empty() && self.current_index > 0 {
            self.current_index -= 1;
            let path = self.image_files[self.current_index].clone();
            self.current_file_path = Some(path.clone());
            self.current_rating = None; // Reset rating until loaded
            Some(path)
        } else {
            warn!("No previous image available");
            None
        }
    }

    /// Updates the directory context based on a selected file path.
    /// Scans the parent directory and sets the current index to the selected file.
    pub fn update_directory(&mut self, file_path: PathBuf) {
        let start = std::time::Instant::now();
        debug!("Starting directory update for: {:?}", file_path);

        if let Some(parent) = file_path.parent() {
            self.current_directory = Some(parent.to_path_buf());

            // Scan directory for image files
            if let Ok(files) = file_utils::scan_directory(parent) {
                self.image_files = files;
                // Find current file index
                self.current_index = self
                    .image_files
                    .iter()
                    .position(|p| p == &file_path)
                    .unwrap_or(0);

                self.current_file_path = Some(file_path.clone());
                self.current_rating = None; // Reset rating until loaded
            }
        }

        let elapsed = start.elapsed();
        debug!(
            "Completed directory update for {:?} in {:?}",
            file_path, elapsed
        );
    }

    /// Returns the current file path.
    pub fn get_current_file_path(&self) -> Option<PathBuf> {
        self.current_file_path.clone()
    }

    /// Sets the current rating.
    pub fn set_current_rating(&mut self, rating: Option<u8>) {
        self.current_rating = rating;
    }
}
