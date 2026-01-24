//! Event handlers for UI callbacks.
//!
//! Sets up all Logic callbacks (select_image, next_image, prev_image, etc.)
//! using the appropriate threading model for each operation type.

use crate::state::NavigationState;
use crate::ui::image_display::load_and_display_image;
use rfd::AsyncFileDialog;
use slint::ComponentHandle;
use std::sync::{Arc, Mutex};

/// Sets up all UI event handlers for the application.
///
/// Takes the UI handle and shared navigation state, then registers
/// callbacks for image selection, navigation, and other user actions.
pub fn setup_handlers(ui: &crate::AppWindow, state: Arc<Mutex<NavigationState>>) {
    // Image selection handler
    // Uses slint::spawn_local because AsyncFileDialog must run on the main thread
    ui.global::<crate::Logic>().on_select_image({
        let ui_handle = ui.as_weak();
        let state = state.clone();
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
        let state = state.clone();
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
                );
            }
        }
    });

    // Previous image handler
    ui.global::<crate::Logic>().on_prev_image({
        let ui_handle = ui.as_weak();
        let state = state.clone();
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
                );
            }
        }
    });
}
