//! Navigation state for managing image file lists and current position.

use crate::file_utils;
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
    fn navigate_to(&mut self, direction: Direction) -> Option<PathBuf> {
        if self.image_files.is_empty() {
            warn!("No images available for navigation");
            return None;
        }

        let current_path = self.current_file_path.as_ref()?;
        let current_index = self.find_file_index(current_path);

        let new_index = match direction {
            Direction::Next => {
                if current_index + 1 < self.image_files.len() {
                    current_index + 1
                } else {
                    warn!("No next image available");
                    return None;
                }
            }
            Direction::Previous => {
                if current_index > 0 {
                    current_index - 1
                } else {
                    warn!("No previous image available");
                    return None;
                }
            }
        };

        let path = self.image_files[new_index].clone();
        self.current_file_path = Some(path.clone());
        self.current_rating = None;
        Some(path)
    }

    /// Returns the path to the next image in the list, if available.
    pub fn next_image(&mut self) -> Option<PathBuf> {
        self.navigate_to(Direction::Next)
    }

    /// Returns the path to the previous image in the list, if available.
    pub fn prev_image(&mut self) -> Option<PathBuf> {
        self.navigate_to(Direction::Previous)
    }

    /// Updates the directory context based on a selected file path.
    /// Scans the parent directory and sets the current file path to the selected file.
    pub fn update_directory(&mut self, file_path: PathBuf) {
        let start = std::time::Instant::now();
        debug!("Starting directory update for: {:?}", file_path);

        if let Some(parent) = file_path.parent() {
            self.current_directory = Some(parent.to_path_buf());

            if let Ok(files) = file_utils::scan_directory(parent) {
                self.image_files = files;
                self.current_file_path = Some(file_path.clone());
                self.current_rating = None;
            }
        }

        debug!(
            "Completed directory update for {:?} in {:?}",
            file_path,
            start.elapsed()
        );
    }

    /// Finds the index of a file in the image files list.
    fn find_file_index(&self, file_path: &PathBuf) -> usize {
        self.image_files
            .iter()
            .position(|p| p == file_path)
            .unwrap_or(0)
    }

    /// Returns the current file path.
    pub fn get_current_file_path(&self) -> Option<PathBuf> {
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
}
