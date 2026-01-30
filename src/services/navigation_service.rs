//! Service for handling image navigation operations.
//!
//! Provides high-level navigation methods that coordinate between
//! NavigationState, ImageCache, and file system operations.

use crate::error::NavigationError;
use crate::image_cache::ImageCache;
use crate::state::NavigationState;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Result type for navigation operations.
pub type NavigationResult = Result<PathBuf, NavigationError>;

/// Service for managing image navigation.
pub struct NavigationService {
    navigation: Arc<Mutex<NavigationState>>,
    #[allow(dead_code)]
    cache: Arc<Mutex<ImageCache>>,
}

impl NavigationService {
    /// Creates a new navigation service.
    pub fn new(navigation: Arc<Mutex<NavigationState>>, cache: Arc<Mutex<ImageCache>>) -> Self {
        Self { navigation, cache }
    }

    /// Navigates to the next image and returns its path.
    pub fn next(&self) -> NavigationResult {
        let mut nav_state = self.navigation.lock().unwrap();
        nav_state.navigate_next()?;
        nav_state
            .current_path()
            .ok_or(NavigationError::NoCurrentPath)
    }

    /// Navigates to the previous image and returns its path.
    pub fn previous(&self) -> NavigationResult {
        let mut nav_state = self.navigation.lock().unwrap();
        nav_state.navigate_prev()?;
        nav_state
            .current_path()
            .ok_or(NavigationError::NoCurrentPath)
    }

    /// Selects a specific image file and updates the directory context.
    ///
    /// This scans the parent directory and sets up the file list for navigation.
    pub fn select_image(&self, path: PathBuf) -> Result<PathBuf, NavigationError> {
        let mut nav_state = self.navigation.lock().unwrap();
        nav_state.update_directory(path.clone())?;
        Ok(path)
    }

    /// Navigates to the last image in the current directory.
    pub fn navigate_to_last(&self) -> NavigationResult {
        let mut nav_state = self.navigation.lock().unwrap();
        nav_state.navigate_to_last()?;
        nav_state
            .current_path()
            .ok_or(NavigationError::NoCurrentPath)
    }

    /// Rescans the current directory and returns the new image count.
    pub fn rescan_directory(&self) -> Result<usize, NavigationError> {
        let mut nav_state = self.navigation.lock().unwrap();
        nav_state.rescan_directory()?;
        Ok(nav_state.image_count())
    }

    /// Gets the current image count.
    pub fn image_count(&self) -> usize {
        let nav_state = self.navigation.lock().unwrap();
        nav_state.image_count()
    }

    /// Gets paths for preloading adjacent images.
    #[allow(dead_code)]
    pub fn get_adjacent_paths(&self) -> (Option<PathBuf>, Option<PathBuf>) {
        let nav_state = self.navigation.lock().unwrap();
        (nav_state.peek_next_image(), nav_state.peek_prev_image())
    }
}
