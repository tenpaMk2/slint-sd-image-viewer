//! State management for the image viewer application.

use crate::image_cache::ImageCache;
use notify::poll::PollWatcher;
use std::sync::{Arc, Mutex};

pub mod navigation;

pub use navigation::NavigationState;

/// Application-wide state container.
pub struct AppState {
    pub navigation: Arc<Mutex<NavigationState>>,
    /// LRU cache for decoded images.
    pub image_cache: Arc<Mutex<ImageCache>>,
    /// Watcher for auto-reload functionality.
    pub auto_reload_watcher: Arc<Mutex<Option<PollWatcher>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            navigation: Arc::new(Mutex::new(NavigationState::new())),
            image_cache: Arc::new(Mutex::new(ImageCache::new(10))),
            auto_reload_watcher: Arc::new(Mutex::new(None)),
        }
    }
}
