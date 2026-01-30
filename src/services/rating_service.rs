//! Service for handling image rating operations.
//!
//! Manages XMP rating writes with duplicate write prevention and cache updates.

use crate::error::AppError;
use crate::image_cache::ImageCache;
use crate::metadata;
use crate::state::NavigationState;
use log::warn;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Result type for operations that can notify UI callbacks.
pub type RatingResult = Result<RatingSuccess, AppError>;

/// Success information for rating operations.
#[derive(Debug)]
pub struct RatingSuccess {
    pub rating: u8,
}

/// Service for managing image rating operations.
pub struct RatingService {
    current_writing: Arc<Mutex<Option<PathBuf>>>,
    navigation: Arc<Mutex<NavigationState>>,
    cache: Arc<Mutex<ImageCache>>,
}

impl RatingService {
    /// Creates a new rating service.
    pub fn new(navigation: Arc<Mutex<NavigationState>>, cache: Arc<Mutex<ImageCache>>) -> Self {
        Self {
            current_writing: Arc::new(Mutex::new(None)),
            navigation,
            cache,
        }
    }

    /// Sets the rating for the current image.
    ///
    /// Returns an error if:
    /// - No image is currently selected
    /// - A write is already in progress for this file
    /// - XMP write fails
    pub fn set_rating(&self, rating: u8) -> RatingResult {
        let path = {
            let nav_state = self.navigation.lock().unwrap();
            nav_state.current_path()
        };

        let path = path.ok_or_else(|| AppError::XmpWrite("No image file selected".to_string()))?;

        // Check if write is already in progress
        if self.is_write_in_progress(&path) {
            return Err(AppError::XmpWrite(
                "Write already in progress for this file".to_string(),
            ));
        }

        // Mark as writing
        self.mark_file_as_writing(path.clone());

        // Perform the write
        let write_result = metadata::write_xmp_rating(&path, rating);

        // Clear writing lock
        self.clear_writing_lock();

        // Handle result
        match write_result {
            Ok(()) => {
                // Update navigation state
                if let Ok(mut nav_state) = self.navigation.lock() {
                    nav_state.set_current_rating(Some(rating));
                }

                // Update cache
                if let Ok(mut cache) = self.cache.lock() {
                    cache.update_rating(&path, Some(rating));
                }

                Ok(RatingSuccess { rating })
            }
            Err(e) => Err(AppError::XmpWrite(e.to_string())),
        }
    }

    /// Checks if a write operation is already in progress for the specified file.
    fn is_write_in_progress(&self, path: &PathBuf) -> bool {
        let writing = self.current_writing.lock().unwrap();
        if let Some(ref writing_path) = *writing {
            if writing_path == path {
                warn!("XMP write already in progress for: {:?}", path);
                return true;
            }
        }
        false
    }

    /// Marks a file as being written.
    fn mark_file_as_writing(&self, path: PathBuf) {
        let mut writing = self.current_writing.lock().unwrap();
        *writing = Some(path);
    }

    /// Clears the writing lock.
    fn clear_writing_lock(&self) {
        let mut writing = self.current_writing.lock().unwrap();
        *writing = None;
    }
}
