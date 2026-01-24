//! Image loading and display logic.
//!
//! Uses `rayon::spawn` for CPU-intensive image decoding operations,
//! then `slint::invoke_from_event_loop` to update UI from the background thread.

use crate::{image_loader, metadata, state::NavigationState};
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
) {
    let image = image_loader::create_slint_image(image_data.data, image_data.width, image_data.height);
    
    ui.global::<crate::ViewState>()
        .set_dynamic_image(image);
    ui.global::<crate::ViewState>().set_image_loaded(true);
    ui.global::<crate::ViewState>().set_error_message("".into());

    let rating_i32 = image_data.rating.map(|r| r as i32).unwrap_or(-1);
    ui.global::<crate::ViewState>()
        .set_current_rating(rating_i32);

    if let Ok(mut nav_state) = state.lock() {
        nav_state.set_current_rating(image_data.rating);
    }
}

/// Updates the UI with an error message.
fn update_ui_with_error(ui: &crate::AppWindow, error_prefix: &str, error: String) {
    let error_message = format!("{}: {}", error_prefix, error);
    error!("{}", error_message);
    ui.global::<crate::ViewState>()
        .set_error_message(error_message.into());
}

/// Helper function to load an image in a background thread and update UI.
///
/// This function:
/// 1. Spawns a rayon thread to decode the image (CPU-intensive)
/// 2. Uses invoke_from_event_loop to return to the UI thread
/// 3. Updates ViewState with the loaded image or error message
pub fn load_and_display_image(
    ui: slint::Weak<crate::AppWindow>,
    path: PathBuf,
    error_prefix: String,
    state: Arc<Mutex<NavigationState>>,
) {
    rayon::spawn(move || {
        let result = load_image_with_metadata(&path);

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui.upgrade() {
                match result {
                    Ok(image_data) => update_ui_with_image(&ui, image_data, state),
                    Err(error) => update_ui_with_error(&ui, &error_prefix, error),
                }
            }
        });
    });
}
