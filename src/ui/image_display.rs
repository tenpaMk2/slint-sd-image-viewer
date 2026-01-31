//! Image loading and display logic.
//!
//! Uses `rayon::spawn` for CPU-intensive image decoding operations,
//! then `slint::invoke_from_event_loop` to update UI from the background thread.

use crate::{
    image_cache::ImageCache,
    image_loader,
    metadata::{SdParameters, SdTag},
    state::NavigationState,
};
use slint::ComponentHandle;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Updates the UI with successfully loaded image data.
fn update_ui_with_image(
    ui: &crate::AppWindow,
    loaded: &image_loader::LoadedImageData,
    state: &Arc<Mutex<NavigationState>>,
) {
    let image = image_loader::create_slint_image(&loaded.data, loaded.width, loaded.height);
    update_ui_state(ui, image, loaded, state);
}

/// Updates the UI with an error message.
fn update_ui_with_error(ui: &crate::AppWindow, error_prefix: &str, error: String) {
    crate::ui::set_error_with_prefix(ui, error_prefix, error);
}

/// Updates the UI state with image and rating information.
fn update_ui_state(
    ui: &crate::AppWindow,
    image: slint::Image,
    loaded: &image_loader::LoadedImageData,
    state: &Arc<Mutex<NavigationState>>,
) {
    ui.global::<crate::ViewerState>().set_dynamic_image(image);
    ui.global::<crate::ViewerState>().set_image_loaded(true);
    ui.global::<crate::ViewerState>()
        .set_error_message("".into());

    let rating_i32 = loaded.rating.map(|r| r as i32).unwrap_or(-1);
    crate::ui::set_rating_info(ui, rating_i32, false);

    // Set navigation information
    if let Ok(nav_state) = state.lock() {
        let total = nav_state.image_count() as i32;
        let current = if let Some(path) = nav_state.current_path() {
            (nav_state.find_file_index(&path) + 1) as i32 // 1-based index
        } else {
            -1
        };
        let auto_reload = ui.global::<crate::ViewerState>().get_auto_reload_active();
        crate::ui::set_navigation_info(ui, current, total, auto_reload);
    }

    // Set basic file information
    crate::ui::set_file_info(
        ui,
        &loaded.file_name,
        &loaded.file_size_formatted,
        loaded.width,
        loaded.height,
        &loaded.created_date,
        &loaded.modified_date,
    );

    // Update SD parameters
    if let Some(params) = &loaded.sd_parameters {
        // Format positive tags
        let positive_prompt = format_tags(&params.positive_sd_tags);

        // Format negative tags
        let negative_prompt = format_tags(&params.negative_sd_tags);

        // Format other parameters as key-value pairs
        let sd_params = format_sd_parameters(params);

        crate::ui::set_prompts_and_parameters(ui, &positive_prompt, &negative_prompt, sd_params);
    } else {
        // Clear SD parameters
        crate::ui::clear_prompts_and_parameters(ui);
    }

    if let Ok(mut nav_state) = state.lock() {
        nav_state.set_current_rating(loaded.rating);
    }
}

/// Formats SD tags into a comma-separated string with weights.
fn format_tags(tags: &[SdTag]) -> String {
    tags.iter()
        .map(|tag| {
            if let Some(weight) = tag.weight {
                format!("({}:{})", tag.name, weight)
            } else {
                tag.name.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Formats SD parameters into key-value pairs for the table.
fn format_sd_parameters(params: &SdParameters) -> Vec<(slint::SharedString, slint::SharedString)> {
    let mut result = Vec::new();

    if let Some(ref steps) = params.steps {
        result.push(("Steps".into(), steps.clone().into()));
    }
    if let Some(ref sampler) = params.sampler {
        result.push(("Sampler".into(), sampler.clone().into()));
    }
    if let Some(ref schedule_type) = params.schedule_type {
        result.push(("Schedule type".into(), schedule_type.clone().into()));
    }
    if let Some(ref cfg_scale) = params.cfg_scale {
        result.push(("CFG scale".into(), cfg_scale.clone().into()));
    }
    if let Some(ref seed) = params.seed {
        result.push(("Seed".into(), seed.clone().into()));
    }
    if let Some(ref size) = params.size {
        result.push(("Size".into(), size.clone().into()));
    }
    if let Some(ref model) = params.model {
        result.push(("Model".into(), model.clone().into()));
    }
    if let Some(ref denoising_strength) = params.denoising_strength {
        result.push((
            "Denoising strength".into(),
            denoising_strength.clone().into(),
        ));
    }
    if let Some(ref clip_skip) = params.clip_skip {
        result.push(("Clip skip".into(), clip_skip.clone().into()));
    }

    result
}

/// Helper function to load an image in a background thread and update UI.
///
/// This function:
/// 1. Checks the cache first for instant display
/// 2. If cache miss, spawns a rayon thread to decode the image (CPU-intensive)
/// 3. Uses invoke_from_event_loop to return to the UI thread
/// 4. Updates ViewerState with the loaded image or error message
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
                &cached_image.data,
                cached_image.width,
                cached_image.height,
            );

            update_ui_state(&ui, image, &cached_image, &state);

            // Trigger preload even on cache hit
            preload_adjacent_images(state, cache);
        }
        return;
    }

    // Cache miss - load from disk
    let cache_clone = cache.clone();
    let state_clone = state.clone();
    rayon::spawn(move || {
        let result = image_loader::load_image_with_metadata(&path)
            .map_err(|e| format!("Failed to load image: {}", e));

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui.upgrade() {
                match result {
                    Ok(loaded) => {
                        // Store in cache and get reference
                        let cached_ref = if let Ok(mut cache) = cache_clone.lock() {
                            cache.put(path.clone(), loaded);
                            cache.get(&path)
                        } else {
                            None
                        };

                        if let Some(cached) = cached_ref {
                            update_ui_with_image(&ui, &cached, &state_clone);
                        }

                        // Trigger preload after successful display
                        preload_adjacent_images(state_clone, cache_clone);
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
                if let Ok(loaded) = image_loader::load_image_with_metadata(&path) {
                    if let Ok(mut cache) = cache_clone.lock() {
                        cache.put(path, loaded);
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
                if let Ok(loaded) = image_loader::load_image_with_metadata(&path) {
                    if let Ok(mut cache) = cache_clone.lock() {
                        cache.put(path, loaded);
                    }
                }
            });
        }
    }
}
