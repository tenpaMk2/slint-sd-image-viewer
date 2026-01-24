//! State management for the image viewer application.

use crate::image_cache::ImageCache;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub mod navigation;

pub use navigation::NavigationState;

/// Application-wide state container.
pub struct AppState {
    pub navigation: Arc<Mutex<NavigationState>>,
    /// Tracks the file currently being written to prevent concurrent XMP writes.
    pub current_writing_file: Arc<Mutex<Option<PathBuf>>>,
    /// LRU cache for decoded images.
    pub image_cache: Arc<Mutex<ImageCache>>,
    /// Timer for auto-reload functionality.
    pub auto_reload_timer: Arc<Mutex<Option<slint::Timer>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            navigation: Arc::new(Mutex::new(NavigationState::new())),
            current_writing_file: Arc::new(Mutex::new(None)),
            image_cache: Arc::new(Mutex::new(ImageCache::new(10))),
            auto_reload_timer: Arc::new(Mutex::new(None)),
        }
    }
}
