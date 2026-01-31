//! Event handlers for UI callbacks.
//!
//! Sets up all Logic callbacks (select_image, next_image, prev_image, etc.)
//! using the appropriate threading model for each operation type.

use crate::services::{AutoReloadService, NavigationService, RatingService};
use crate::state::AppState;
use crate::ui::image_display::load_and_display_image;
use rfd::AsyncFileDialog;
use slint::ComponentHandle;
use std::sync::{Arc, Mutex};

/// Sets an error message in the UI.
fn set_error_message(ui_handle: &slint::Weak<crate::AppWindow>, message: String) {
    if let Some(ui) = ui_handle.upgrade() {
        ui.global::<crate::ViewerState>()
            .set_error_message(message.into());
    }
}

/// Creates a rating handler closure for the specified rating value.
fn create_rating_handler(
    ui_handle: slint::Weak<crate::AppWindow>,
    rating_service: Arc<RatingService>,
    rating: u8,
) -> impl Fn() {
    move || {
        if let Some(ui) = ui_handle.upgrade() {
            ui.global::<crate::ViewerState>()
                .set_rating_in_progress(true);
        }

        let ui_handle_clone = ui_handle.clone();
        let rating_service_clone = rating_service.clone();

        rayon::spawn(move || {
            let result = rating_service_clone.set_rating(rating);

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_handle_clone.upgrade() {
                    ui.global::<crate::ViewerState>()
                        .set_rating_in_progress(false);

                    match result {
                        Ok(success) => {
                            ui.global::<crate::ViewerState>()
                                .set_current_rating(success.rating as i32);
                            ui.global::<crate::ViewerState>()
                                .set_error_message("".into());
                        }
                        Err(e) => {
                            ui.global::<crate::ViewerState>()
                                .set_error_message(e.to_string().into());
                        }
                    }
                }
            });
        });
    }
}

/// Sets up the file selection handler.
fn setup_file_selection_handler(ui: &crate::AppWindow, app_state: &AppState) {
    let navigation_service = Arc::new(NavigationService::new(app_state.navigation.clone()));

    ui.global::<crate::Logic>().on_select_image({
        let ui_handle = ui.as_weak();
        let state = app_state.navigation.clone();
        let cache = app_state.image_cache.clone();
        let nav_service = navigation_service.clone();
        move || {
            let ui_handle = ui_handle.clone();
            let state = state.clone();
            let cache = cache.clone();
            let nav_service = nav_service.clone();
            let _ = slint::spawn_local(async move {
                let Some(file_handle) = AsyncFileDialog::new().pick_file().await else {
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.global::<crate::ViewerState>()
                            .set_error_message("No file selected".into());
                    }
                    return;
                };

                let path = file_handle.path().to_path_buf();

                // Load and display the selected image immediately
                load_and_display_image(
                    ui_handle.clone(),
                    path.clone(),
                    "Failed to load image".to_string(),
                    state.clone(),
                    cache.clone(),
                );

                // Update directory in background
                let ui_handle_clone = ui_handle.clone();
                rayon::spawn(move || {
                    let result = nav_service.select_image(path);

                    if let Err(e) = result {
                        let _ = slint::invoke_from_event_loop(move || {
                            set_error_message(
                                &ui_handle_clone,
                                format!("Failed to update directory: {}", e),
                            );
                        });
                    }
                });
            });
        }
    });
}

/// Sets up the navigation handlers (next and previous image).
fn setup_navigation_handlers(ui: &crate::AppWindow, app_state: &AppState) {
    let navigation_service = Arc::new(NavigationService::new(app_state.navigation.clone()));

    ui.global::<crate::Logic>().on_next_image({
        let ui_handle = ui.as_weak();
        let state = app_state.navigation.clone();
        let cache = app_state.image_cache.clone();
        let watcher_ref = app_state.auto_reload_watcher.clone();
        let nav_service = navigation_service.clone();
        move || {
            // Stop auto-reload on manual navigation
            stop_auto_reload_internal(&ui_handle, &watcher_ref);

            let result = nav_service.next();

            match result {
                Ok(path) => {
                    load_and_display_image(
                        ui_handle.clone(),
                        path,
                        "Failed to load next image".to_string(),
                        state.clone(),
                        cache.clone(),
                    );
                }
                Err(e) => {
                    set_error_message(&ui_handle, format!("Navigation failed: {}", e));
                }
            }
        }
    });

    ui.global::<crate::Logic>().on_prev_image({
        let ui_handle = ui.as_weak();
        let state = app_state.navigation.clone();
        let cache = app_state.image_cache.clone();
        let watcher_ref = app_state.auto_reload_watcher.clone();
        let nav_service = navigation_service.clone();
        move || {
            // Stop auto-reload on manual navigation
            stop_auto_reload_internal(&ui_handle, &watcher_ref);

            let result = nav_service.previous();

            match result {
                Ok(path) => {
                    load_and_display_image(
                        ui_handle.clone(),
                        path,
                        "Failed to load previous image".to_string(),
                        state.clone(),
                        cache.clone(),
                    );
                }
                Err(e) => {
                    set_error_message(&ui_handle, format!("Navigation failed: {}", e));
                }
            }
        }
    });
}

/// Internal helper to stop the auto-reload watcher.
fn stop_auto_reload_internal(
    ui_handle: &slint::Weak<crate::AppWindow>,
    watcher_ref: &Arc<Mutex<Option<notify::poll::PollWatcher>>>,
) {
    if let Ok(mut watcher_lock) = watcher_ref.lock() {
        if watcher_lock.take().is_some() {
            if let Some(ui) = ui_handle.upgrade() {
                ui.global::<crate::ViewerState>()
                    .set_auto_reload_active(false);
            }
        }
    }
}

/// Sets up the auto-reload handlers.
fn setup_auto_reload_handlers(ui: &crate::AppWindow, app_state: &AppState) {
    let navigation_service = NavigationService::new(app_state.navigation.clone());
    let reload_service = Arc::new(AutoReloadService::new(navigation_service));

    ui.global::<crate::Logic>().on_start_auto_reload({
        let ui_handle = ui.as_weak();
        let state = app_state.navigation.clone();
        let cache = app_state.image_cache.clone();
        let watcher_ref = app_state.auto_reload_watcher.clone();
        let reload_service = reload_service.clone();

        move || {
            // First, navigate to the last image immediately
            let result = reload_service.navigate_to_last();

            match result {
                Ok(path) => {
                    load_and_display_image(
                        ui_handle.clone(),
                        path,
                        "Failed to load last image".to_string(),
                        state.clone(),
                        cache.clone(),
                    );
                }
                Err(e) => {
                    set_error_message(
                        &ui_handle,
                        format!("Failed to navigate to last image: {}", e),
                    );
                    return;
                }
            }

            // Start watching for changes
            let ui_weak = ui_handle.clone();
            let state_clone = state.clone();
            let cache_clone = cache.clone();

            let watcher_result = reload_service.start_watching(state_clone.clone(), move |path| {
                load_and_display_image(
                    ui_weak.clone(),
                    path,
                    "Auto-reload failed".to_string(),
                    state_clone.clone(),
                    cache_clone.clone(),
                );
            });

            match watcher_result {
                Ok(watcher) => {
                    if let Ok(mut watcher_lock) = watcher_ref.lock() {
                        *watcher_lock = Some(watcher);
                    }

                    if let Some(ui) = ui_handle.upgrade() {
                        ui.global::<crate::ViewerState>()
                            .set_auto_reload_active(true);
                    }
                }
                Err(e) => {
                    set_error_message(&ui_handle, format!("Failed to start auto-reload: {}", e));
                }
            }
        }
    });

    ui.global::<crate::Logic>().on_stop_auto_reload({
        let ui_handle = ui.as_weak();
        let watcher_ref = app_state.auto_reload_watcher.clone();

        move || {
            stop_auto_reload_internal(&ui_handle, &watcher_ref);
        }
    });
}

/// Sets up the rating handlers (rate-0 through rate-5).
fn setup_rating_handlers(ui: &crate::AppWindow, app_state: &AppState) {
    let rating_service = Arc::new(RatingService::new(
        app_state.navigation.clone(),
        app_state.image_cache.clone(),
    ));

    for rating in 0..=5 {
        let handler = create_rating_handler(ui.as_weak(), rating_service.clone(), rating);

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
    setup_auto_reload_handlers(ui, &app_state);
    setup_rating_handlers(ui, &app_state);
}
