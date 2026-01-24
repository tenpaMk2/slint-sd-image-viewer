//! State management for the image viewer application.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub mod navigation;

pub use navigation::NavigationState;

/// Application-wide state container.
pub struct AppState {
    pub navigation: Arc<Mutex<NavigationState>>,
    /// Tracks the file currently being written to prevent concurrent XMP writes.
    pub current_writing_file: Arc<Mutex<Option<PathBuf>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            navigation: Arc::new(Mutex::new(NavigationState::new())),
            current_writing_file: Arc::new(Mutex::new(None)),
        }
    }
}
