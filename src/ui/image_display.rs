//! Image loading and display logic.
//!
//! Uses `rayon::spawn` for CPU-intensive image decoding operations,
//! then `slint::invoke_from_event_loop` to update UI from the background thread.

use crate::{
    image_cache::{CachedImage, ImageCache},
    image_loader, metadata,
    state::NavigationState,
};
use log::error;
use slint::ComponentHandle;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Result of loading image and metadata.
struct LoadedImageData {
    data: Vec<u8>,
    width: u32,
    height: u32,
    rating: Option<u8>,
}

/// Loads image and metadata from the specified path.
fn load_image_with_metadata(path: &PathBuf) -> Result<LoadedImageData, String> {
    let (data, width, height) = image_loader::load_image_blocking(path)
        .map_err(|e| format!("Failed to load image: {}", e))?;

    let rating = metadata::read_xmp_rating(path).ok().flatten();

    Ok(LoadedImageData {
        data,
        width,
        height,
        rating,
    })
}

/// Updates the UI with successfully loaded image data.
fn update_ui_with_image(
    ui: &crate::AppWindow,
    image_data: LoadedImageData,
    state: Arc<Mutex<NavigationState>>,
    cache: Arc<Mutex<ImageCache>>,
    path: PathBuf,
) {
    // Store RGB8 data in cache before converting to slint::Image
    if let Ok(mut cache) = cache.lock() {
        cache.put(
            path.clone(),
            CachedImage::new(
                image_data.data.clone(),
                image_data.width,
                image_data.height,
                image_data.rating,
            ),
        );
    }

    let image =
        image_loader::create_slint_image(image_data.data, image_data.width, image_data.height);

    update_ui_state(ui, image, image_data.rating, &state);

    // Trigger preload of adjacent images
    preload_adjacent_images(state, cache);
}

/// Updates the UI with an error message.
fn update_ui_with_error(ui: &crate::AppWindow, error_prefix: &str, error: String) {
    let error_message = format!("{}: {}", error_prefix, error);
    error!("{}", error_message);
    ui.global::<crate::ViewState>()
        .set_error_message(error_message.into());
}

/// Updates the UI state with image and rating information.
fn update_ui_state(
    ui: &crate::AppWindow,
    image: slint::Image,
    rating: Option<u8>,
    state: &Arc<Mutex<NavigationState>>,
) {
    ui.global::<crate::ViewState>().set_dynamic_image(image);
    ui.global::<crate::ViewState>().set_image_loaded(true);
    ui.global::<crate::ViewState>().set_error_message("".into());

    let rating_i32 = rating.map(|r| r as i32).unwrap_or(-1);
    ui.global::<crate::ViewState>()
        .set_current_rating(rating_i32);

    if let Ok(mut nav_state) = state.lock() {
        nav_state.set_current_rating(rating);
    }
}

/// Helper function to load an image in a background thread and update UI.
///
/// This function:
/// 1. Checks the cache first for instant display
/// 2. If cache miss, spawns a rayon thread to decode the image (CPU-intensive)
/// 3. Uses invoke_from_event_loop to return to the UI thread
/// 4. Updates ViewState with the loaded image or error message
pub fn load_and_display_image(
    ui: slint::Weak<crate::AppWindow>,
    path: PathBuf,
    error_prefix: String,
    state: Arc<Mutex<NavigationState>>,
    cache: Arc<Mutex<ImageCache>>,
) {
    // Check cache first
    let cached = cache.lock().ok().and_then(|mut c| c.get(&path));

    if let Some(cached_image) = cached {
        // Cache hit - display immediately
        if let Some(ui) = ui.upgrade() {
            let image = image_loader::create_slint_image(
                cached_image.data,
                cached_image.width,
                cached_image.height,
            );

            update_ui_state(&ui, image, cached_image.rating, &state);

            // Trigger preload even on cache hit
            preload_adjacent_images(state, cache);
        }
        return;
    }

    // Cache miss - load from disk
    let cache_clone = cache.clone();
    rayon::spawn(move || {
        let result = load_image_with_metadata(&path);

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui.upgrade() {
                match result {
                    Ok(image_data) => {
                        update_ui_with_image(&ui, image_data, state, cache_clone, path)
                    }
                    Err(error) => update_ui_with_error(&ui, &error_prefix, error),
                }
            }
        });
    });
}

/// Preloads adjacent images (next and previous) in the background.
fn preload_adjacent_images(state: Arc<Mutex<NavigationState>>, cache: Arc<Mutex<ImageCache>>) {
    let (next_path, prev_path) = {
        if let Ok(nav_state) = state.lock() {
            (nav_state.peek_next_image(), nav_state.peek_prev_image())
        } else {
            return;
        }
    };

    // Preload next image if not in cache
    if let Some(path) = next_path {
        let should_load = cache
            .lock()
            .ok()
            .map(|mut c| !c.contains(&path))
            .unwrap_or(false);

        if should_load {
            let cache_clone = cache.clone();
            rayon::spawn(move || {
                // Silently ignore errors during preload
                if let Ok(image_data) = load_image_with_metadata(&path) {
                    if let Ok(mut cache) = cache_clone.lock() {
                        cache.put(
                            path,
                            CachedImage::new(
                                image_data.data,
                                image_data.width,
                                image_data.height,
                                image_data.rating,
                            ),
                        );
                    }
                }
            });
        }
    }

    // Preload previous image if not in cache
    if let Some(path) = prev_path {
        let should_load = cache
            .lock()
            .ok()
            .map(|mut c| !c.contains(&path))
            .unwrap_or(false);

        if should_load {
            let cache_clone = cache.clone();
            rayon::spawn(move || {
                // Silently ignore errors during preload
                if let Ok(image_data) = load_image_with_metadata(&path) {
                    if let Ok(mut cache) = cache_clone.lock() {
                        cache.put(
                            path,
                            CachedImage::new(
                                image_data.data,
                                image_data.width,
                                image_data.height,
                                image_data.rating,
                            ),
                        );
                    }
                }
            });
        }
    }
}
