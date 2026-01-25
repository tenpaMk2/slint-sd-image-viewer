//! Image cache for fast navigation.
//!
//! Caches decoded RGB8 image data with metadata using an LRU policy.
//! This allows instant display of recently viewed images.

use crate::image_loader::LoadedImageData;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::PathBuf;

/// LRU cache for storing decoded images.
pub struct ImageCache {
    cache: LruCache<PathBuf, LoadedImageData>,
}

impl ImageCache {
    /// Creates a new image cache with the specified capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(capacity).expect("Capacity must be non-zero")),
        }
    }

    /// Retrieves an image from the cache if it exists.
    pub fn get(&mut self, path: &PathBuf) -> Option<LoadedImageData> {
        let result = self.cache.get(path).cloned();
        if result.is_some() {
            log::info!("Cache HIT: {}", path.display());
        } else {
            log::info!("Cache MISS: {}", path.display());
        }
        result
    }

    /// Stores an image in the cache.
    pub fn put(&mut self, path: PathBuf, image_data: LoadedImageData) {
        log::info!(
            "Cache PUT: {} ({}x{})",
            path.display(),
            image_data.width,
            image_data.height
        );
        self.cache.put(path, image_data);
    }

    /// Updates the rating of a cached image without changing its position in the LRU.
    pub fn update_rating(&mut self, path: &PathBuf, rating: Option<u8>) {
        if let Some(cached) = self.cache.peek_mut(path) {
            cached.rating = rating;
        }
    }

    /// Checks if an image is in the cache.
    pub fn contains(&mut self, path: &PathBuf) -> bool {
        self.cache.contains(path)
    }
}
