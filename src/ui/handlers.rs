//! Event handlers for UI callbacks.
//!
//! Sets up all Logic callbacks (select_image, next_image, prev_image, etc.)
//! using the appropriate threading model for each operation type.

use crate::ui::image_display::load_and_display_image;
use crate::{metadata, state::AppState};
use log::warn;
use rfd::AsyncFileDialog;
use slint::ComponentHandle;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Checks if a write operation is already in progress for the specified file.
fn is_write_in_progress(current_writing: &Arc<Mutex<Option<PathBuf>>>, path: &PathBuf) -> bool {
    let writing = current_writing.lock().unwrap();
    if let Some(ref writing_path) = *writing {
        if writing_path == path {
            warn!("XMP write already in progress for: {:?}", path);
            return true;
        }
    }
    false
}

/// Marks a file as being written.
fn mark_file_as_writing(current_writing: &Arc<Mutex<Option<PathBuf>>>, path: PathBuf) {
    let mut writing = current_writing.lock().unwrap();
    *writing = Some(path);
}

/// Clears the writing lock for a file.
fn clear_writing_lock(current_writing: &Arc<Mutex<Option<PathBuf>>>) {
    let mut writing = current_writing.lock().unwrap();
    *writing = None;
}

/// Updates the UI after a successful rating write.
fn update_ui_after_rating_success(
    ui: &crate::AppWindow,
    rating: u8,
    state: Arc<Mutex<crate::state::NavigationState>>,
    cache: Arc<Mutex<crate::image_cache::ImageCache>>,
) {
    let current_path = {
        let nav_state = state.lock().ok();
        nav_state.and_then(|s| s.get_current_file_path())
    };

    if let Ok(mut nav_state) = state.lock() {
        nav_state.set_current_rating(Some(rating));
    }
    ui.global::<crate::ViewState>()
        .set_current_rating(rating as i32);
    ui.global::<crate::ViewState>().set_error_message("".into());

    // Update cache if the image is cached
    if let Some(path) = current_path {
        if let Ok(mut cache) = cache.lock() {
            cache.update_rating(&path, Some(rating));
        }
    }
}

/// Updates the UI after a failed rating write.
fn update_ui_after_rating_error(ui: &crate::AppWindow, error: String) {
    ui.global::<crate::ViewState>()
        .set_error_message(error.into());
}

/// Creates a rating handler closure for the specified rating value.
fn create_rating_handler(
    ui_handle: slint::Weak<crate::AppWindow>,
    state: Arc<Mutex<crate::state::NavigationState>>,
    current_writing: Arc<Mutex<Option<PathBuf>>>,
    cache: Arc<Mutex<crate::image_cache::ImageCache>>,
    rating: u8,
) -> impl Fn() {
    move || {
        let current_path = {
            let nav_state = state.lock().unwrap();
            nav_state.get_current_file_path()
        };

        let Some(path) = current_path else {
            if let Some(ui) = ui_handle.upgrade() {
                ui.global::<crate::ViewState>()
                    .set_error_message("No image file selected".into());
            }
            return;
        };

        if is_write_in_progress(&current_writing, &path) {
            return;
        }

        mark_file_as_writing(&current_writing, path.clone());

        if let Some(ui) = ui_handle.upgrade() {
            ui.global::<crate::ViewState>().set_rating_in_progress(true);
        }

        let ui_handle_clone = ui_handle.clone();
        let state_clone = state.clone();
        let current_writing_clone = current_writing.clone();
        let cache_clone = cache.clone();

        rayon::spawn(move || {
            let write_result = metadata::write_xmp_rating(&path, rating);
            clear_writing_lock(&current_writing_clone);

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_handle_clone.upgrade() {
                    ui.global::<crate::ViewState>()
                        .set_rating_in_progress(false);

                    match write_result {
                        Ok(()) => {
                            update_ui_after_rating_success(&ui, rating, state_clone, cache_clone)
                        }
                        Err(e) => update_ui_after_rating_error(&ui, e.to_string()),
                    }
                }
            });
        });
    }
}

/// Sets up the file selection handler.
fn setup_file_selection_handler(ui: &crate::AppWindow, app_state: &AppState) {
    ui.global::<crate::Logic>().on_select_image({
        let ui_handle = ui.as_weak();
        let state = app_state.navigation.clone();
        let cache = app_state.image_cache.clone();
        move || {
            let ui_handle = ui_handle.clone();
            let state = state.clone();
            let cache = cache.clone();
            let _ = slint::spawn_local(async move {
                let Some(file_handle) = AsyncFileDialog::new().pick_file().await else {
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.global::<crate::ViewState>()
                            .set_error_message("No file selected".into());
                    }
                    return;
                };

                let path = file_handle.path().to_path_buf();

                load_and_display_image(
                    ui_handle.clone(),
                    path.clone(),
                    "Failed to load image".to_string(),
                    state.clone(),
                    cache.clone(),
                );

                let state_clone = state.clone();
                let path_clone = path.clone();
                rayon::spawn(move || {
                    let mut state = state_clone.lock().unwrap();
                    state.update_directory(path_clone);
                });
            });
        }
    });
}

/// Sets up the navigation handlers (next and previous image).
fn setup_navigation_handlers(ui: &crate::AppWindow, app_state: &AppState) {
    ui.global::<crate::Logic>().on_next_image({
        let ui_handle = ui.as_weak();
        let state = app_state.navigation.clone();
        let cache = app_state.image_cache.clone();
        move || {
            let next_path = {
                let mut state = state.lock().unwrap();
                state.next_image()
            };

            if let Some(path) = next_path {
                load_and_display_image(
                    ui_handle.clone(),
                    path,
                    "Failed to load next image".to_string(),
                    state.clone(),
                    cache.clone(),
                );
            }
        }
    });

    ui.global::<crate::Logic>().on_prev_image({
        let ui_handle = ui.as_weak();
        let state = app_state.navigation.clone();
        let cache = app_state.image_cache.clone();
        move || {
            let prev_path = {
                let mut state = state.lock().unwrap();
                state.prev_image()
            };

            if let Some(path) = prev_path {
                load_and_display_image(
                    ui_handle.clone(),
                    path,
                    "Failed to load previous image".to_string(),
                    state.clone(),
                    cache.clone(),
                );
            }
        }
    });
}

/// Sets up the rating handlers (rate-0 through rate-5).
fn setup_rating_handlers(ui: &crate::AppWindow, app_state: &AppState) {
    for rating in 0..=5 {
        let handler = create_rating_handler(
            ui.as_weak(),
            app_state.navigation.clone(),
            app_state.current_writing_file.clone(),
            app_state.image_cache.clone(),
            rating,
        );

        match rating {
            0 => ui.global::<crate::Logic>().on_rate_0(handler),
            1 => ui.global::<crate::Logic>().on_rate_1(handler),
            2 => ui.global::<crate::Logic>().on_rate_2(handler),
            3 => ui.global::<crate::Logic>().on_rate_3(handler),
            4 => ui.global::<crate::Logic>().on_rate_4(handler),
            5 => ui.global::<crate::Logic>().on_rate_5(handler),
            _ => unreachable!(),
        }
    }
}

/// Sets up all UI event handlers for the application.
///
/// Takes the UI handle and shared application state, then registers
/// callbacks for image selection, navigation, and other user actions.
pub fn setup_handlers(ui: &crate::AppWindow, app_state: AppState) {
    setup_file_selection_handler(ui, &app_state);
    setup_navigation_handlers(ui, &app_state);
    setup_rating_handlers(ui, &app_state);
}
