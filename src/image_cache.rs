//! Image cache for fast navigation.
//!
//! Caches decoded RGB8 image data with metadata using an LRU policy.
//! This allows instant display of recently viewed images.

use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::PathBuf;

/// Cached image data including RGB8 pixel data and metadata.
#[derive(Clone)]
pub struct CachedImage {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub rating: Option<u8>,
}

impl CachedImage {
    /// Creates a CachedImage from raw image data.
    pub fn new(data: Vec<u8>, width: u32, height: u32, rating: Option<u8>) -> Self {
        Self {
            data,
            width,
            height,
            rating,
        }
    }
}

/// LRU cache for storing decoded images.
pub struct ImageCache {
    cache: LruCache<PathBuf, CachedImage>,
}

impl ImageCache {
    /// Creates a new image cache with the specified capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(capacity).expect("Capacity must be non-zero")),
        }
    }

    /// Retrieves an image from the cache if it exists.
    pub fn get(&mut self, path: &PathBuf) -> Option<CachedImage> {
        let result = self.cache.get(path).cloned();
        if result.is_some() {
            log::info!("Cache HIT: {}", path.display());
        } else {
            log::info!("Cache MISS: {}", path.display());
        }
        result
    }

    /// Stores an image in the cache.
    pub fn put(&mut self, path: PathBuf, cached_image: CachedImage) {
        log::info!(
            "Cache PUT: {} ({}x{})",
            path.display(),
            cached_image.width,
            cached_image.height
        );
        self.cache.put(path, cached_image);
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
