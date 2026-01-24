//! Event handlers for UI callbacks.
//!
//! Sets up all Logic callbacks (select_image, next_image, prev_image, etc.)
//! using the appropriate threading model for each operation type.

use crate::ui::image_display::load_and_display_image;
use crate::{metadata, state::AppState};
use log::warn;
use rfd::AsyncFileDialog;
use slint::ComponentHandle;

/// Sets up all UI event handlers for the application.
///
/// Takes the UI handle and shared application state, then registers
/// callbacks for image selection, navigation, and other user actions.
pub fn setup_handlers(ui: &crate::AppWindow, app_state: AppState) {
    // Image selection handler
    // Uses slint::spawn_local because AsyncFileDialog must run on the main thread
    ui.global::<crate::Logic>().on_select_image({
        let ui_handle = ui.as_weak();
        let state = app_state.navigation.clone();
        move || {
            let ui_handle = ui_handle.clone();
            let state = state.clone();
            let _ = slint::spawn_local(async move {
                // Show file dialog
                // AsyncFileDialogはメインスレッドで実行する必要があるのでrayon禁止。
                let Some(file_handle) = AsyncFileDialog::new().pick_file().await else {
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.global::<crate::ViewState>()
                            .set_error_message("No file selected".into());
                    }
                    return;
                };

                let path = file_handle.path().to_path_buf();

                // Load image and update UI (rayonで別スレッド実行)
                load_and_display_image(
                    ui_handle.clone(),
                    path.clone(),
                    "Failed to load image".to_string(),
                    state.clone(),
                );

                // Update state with directory info (rayonで別スレッド実行)
                let state_clone = state.clone();
                let path_clone = path.clone();
                rayon::spawn(move || {
                    let mut state = state_clone.lock().unwrap();
                    state.update_directory(path_clone);
                });
            });
        }
    });

    // Next image handler
    ui.global::<crate::Logic>().on_next_image({
        let ui_handle = ui.as_weak();
        let state = app_state.navigation.clone();
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
                );
            }
        }
    });

    // Previous image handler
    ui.global::<crate::Logic>().on_prev_image({
        let ui_handle = ui.as_weak();
        let state = app_state.navigation.clone();
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
                );
            }
        }
    });

    // Rating handlers (rate-0 through rate-5)
    macro_rules! setup_rating_handler {
        ($rating:expr) => {{
            let ui_handle = ui.as_weak();
            let state = app_state.navigation.clone();
            let current_writing = app_state.current_writing_file.clone();
            move || {
                // Get current file path
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

                // Check if another write operation is in progress for this file
                {
                    let mut writing = current_writing.lock().unwrap();
                    if let Some(ref writing_path) = *writing {
                        if writing_path == &path {
                            warn!("XMP write already in progress for: {:?}", path);
                            return;
                        }
                    }
                    // Mark this file as being written
                    *writing = Some(path.clone());
                }

                // Immediately disable rating UI
                if let Some(ui) = ui_handle.upgrade() {
                    ui.global::<crate::ViewState>().set_rating_in_progress(true);
                }

                let ui_handle_clone = ui_handle.clone();
                let state_clone = state.clone();
                let current_writing_clone = current_writing.clone();

                rayon::spawn(move || {
                    // Write XMP rating
                    let write_result = metadata::write_xmp_rating(&path, $rating);

                    // Clear the writing lock
                    {
                        let mut writing = current_writing_clone.lock().unwrap();
                        *writing = None;
                    }

                    // Update UI and state
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_handle_clone.upgrade() {
                            ui.global::<crate::ViewState>()
                                .set_rating_in_progress(false);

                            match write_result {
                                Ok(()) => {
                                    // Update rating in state and UI atomically
                                    if let Ok(mut nav_state) = state_clone.lock() {
                                        nav_state.set_current_rating(Some($rating));
                                    }
                                    ui.global::<crate::ViewState>()
                                        .set_current_rating($rating as i32);
                                    ui.global::<crate::ViewState>().set_error_message("".into());
                                }
                                Err(e) => {
                                    // Show error message
                                    ui.global::<crate::ViewState>()
                                        .set_error_message(e.to_string().into());
                                }
                            }
                        }
                    });
                });
            }
        }};
    }

    ui.global::<crate::Logic>()
        .on_rate_0(setup_rating_handler!(0));
    ui.global::<crate::Logic>()
        .on_rate_1(setup_rating_handler!(1));
    ui.global::<crate::Logic>()
        .on_rate_2(setup_rating_handler!(2));
    ui.global::<crate::Logic>()
        .on_rate_3(setup_rating_handler!(3));
    ui.global::<crate::Logic>()
        .on_rate_4(setup_rating_handler!(4));
    ui.global::<crate::Logic>()
        .on_rate_5(setup_rating_handler!(5));
}
