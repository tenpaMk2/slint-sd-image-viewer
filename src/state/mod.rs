//! State management for the image viewer application.

use crate::image_cache::ImageCache;
use notify_debouncer_mini::{notify::PollWatcher, Debouncer};
use std::sync::{Arc, Mutex};

pub mod navigation;

pub use navigation::NavigationState;

/// Type alias for the auto-reload debouncer.
pub type AutoReloadDebouncer = Debouncer<PollWatcher>;

/// Application-wide state container.
pub struct AppState {
    pub navigation: Arc<Mutex<NavigationState>>,
    /// LRU cache for decoded images.
    pub image_cache: Arc<Mutex<ImageCache>>,
    /// Debouncer for auto-reload functionality.
    pub auto_reload_watcher: Arc<Mutex<Option<AutoReloadDebouncer>>>,
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
