//! Navigation state for managing image file lists and current position.

use crate::error::NavigationError;
use crate::file_utils::{self, PathExt};
use log::{debug, warn};
use std::path::PathBuf;

/// Direction for navigation through images.
#[derive(Debug, Clone, Copy)]
enum Direction {
    Next,
    Previous,
}

/// Manages the current directory, list of image files, and current file path.
#[derive(Default)]
pub struct NavigationState {
    current_directory: Option<PathBuf>,
    image_files: Vec<PathBuf>,
    current_file_path: Option<PathBuf>,
    current_rating: Option<u8>,
}

impl NavigationState {
    /// Creates a new empty navigation state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Navigates to an image in the specified direction.
    fn navigate_to(&mut self, direction: Direction) -> Result<(), NavigationError> {
        if self.image_files.is_empty() {
            warn!("No images available for navigation");
            return Err(NavigationError::NoImages);
        }

        let current_path = self
            .current_file_path
            .as_ref()
            .ok_or(NavigationError::NoCurrentPath)?;
        let current_index = self.find_file_index(current_path);

        let new_index = match direction {
            Direction::Next => {
                if current_index + 1 < self.image_files.len() {
                    current_index + 1
                } else {
                    // Wrap around to the first image
                    debug!("Reached last image, wrapping to first");
                    0
                }
            }
            Direction::Previous => {
                if current_index > 0 {
                    current_index - 1
                } else {
                    // Wrap around to the last image
                    debug!("Reached first image, wrapping to last");
                    self.image_files.len() - 1
                }
            }
        };

        let path = self.image_files[new_index].clone();
        self.current_file_path = Some(path.clone());
        self.current_rating = None;
        debug!("Navigated to: {:?}", path);
        Ok(())
    }

    /// Navigates to the next image in the list.
    pub fn navigate_next(&mut self) -> Result<(), NavigationError> {
        self.navigate_to(Direction::Next)
    }

    /// Navigates to the previous image in the list.
    pub fn navigate_prev(&mut self) -> Result<(), NavigationError> {
        self.navigate_to(Direction::Previous)
    }

    /// Updates the directory context based on a selected file path.
    /// Scans the parent directory and sets the current file path to the selected file.
    pub fn update_directory(&mut self, file_path: PathBuf) -> Result<(), NavigationError> {
        let start = std::time::Instant::now();
        let parent = file_path.parent().ok_or_else(|| {
            NavigationError::DirectoryScanFailed("No parent directory".to_string())
        })?;
        debug!("Starting directory update for: {:?}", parent);

        self.current_directory = Some(parent.to_path_buf());

        let files = file_utils::scan_directory(parent).map_err(|e| {
            NavigationError::DirectoryScanFailed(format!("Failed to scan directory: {}", e))
        })?;

        self.image_files = files;
        self.current_file_path = Some(file_path.clone());
        self.current_rating = None;

        debug!(
            "Completed directory update for: {:?} in {:?}",
            parent,
            start.elapsed()
        );
        Ok(())
    }

    /// Finds the index of a file in the image files list.
    pub fn find_file_index(&self, file_path: &PathBuf) -> usize {
        self.image_files
            .iter()
            .position(|p| p == file_path)
            .unwrap_or(0)
    }

    /// Returns the current file path, if set.
    pub fn current_path(&self) -> Option<PathBuf> {
        self.current_file_path.clone()
    }

    /// Sets the current rating.
    pub fn set_current_rating(&mut self, rating: Option<u8>) {
        self.current_rating = rating;
    }

    /// Returns the path to the next image without changing the current file path.
    pub fn peek_next_image(&self) -> Option<PathBuf> {
        let current_path = self.current_file_path.as_ref()?;
        let current_index = self.find_file_index(current_path);

        if current_index + 1 < self.image_files.len() {
            Some(self.image_files[current_index + 1].clone())
        } else {
            None
        }
    }

    /// Returns the path to the previous image without changing the current file path.
    pub fn peek_prev_image(&self) -> Option<PathBuf> {
        let current_path = self.current_file_path.as_ref()?;
        let current_index = self.find_file_index(current_path);

        if current_index > 0 {
            Some(self.image_files[current_index - 1].clone())
        } else {
            None
        }
    }

    /// Returns the current directory path.
    pub fn get_current_directory(&self) -> Option<PathBuf> {
        self.current_directory.clone()
    }

    /// Navigates to the last image in the list.
    pub fn navigate_to_last(&mut self) -> Result<(), NavigationError> {
        if self.image_files.is_empty() {
            warn!("No images available for navigation to last");
            return Err(NavigationError::NoImages);
        }

        let last_index = self.image_files.len() - 1;
        let path = self.image_files[last_index].clone();
        self.current_file_path = Some(path.clone());
        self.current_rating = None;
        debug!("Navigated to last image: {}", path.format_for_log());
        Ok(())
    }

    /// Rescans the current directory.
    pub fn rescan_directory(&mut self) -> Result<(), NavigationError> {
        let current_dir = self.current_directory.as_ref().ok_or_else(|| {
            NavigationError::DirectoryScanFailed("No current directory to rescan".to_string())
        })?;

        let new_files = file_utils::scan_directory(current_dir).map_err(|e| {
            NavigationError::DirectoryScanFailed(format!("Failed to rescan directory: {}", e))
        })?;

        debug!(
            "Directory rescanned: {} -> {} files",
            self.image_files.len(),
            new_files.len()
        );
        self.image_files = new_files;

        Ok(())
    }

    /// Returns the number of images in the current directory.
    pub fn image_count(&self) -> usize {
        self.image_files.len()
    }
}
