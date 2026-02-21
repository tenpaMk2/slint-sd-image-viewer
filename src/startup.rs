use slint::ComponentHandle;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::image_cache::ImageCache;
use crate::services::NavigationService;
use crate::state::{AppState, NavigationState};
use crate::ui::DisplayTracker;

fn open_image_path(
    ui: slint::Weak<crate::AppWindow>,
    path: PathBuf,
    navigation: Arc<Mutex<NavigationState>>,
    cache: Arc<Mutex<ImageCache>>,
    display_tracker: DisplayTracker,
    error_prefix: &'static str,
) {
    crate::ui::image_display::load_and_display_image(
        ui.clone(),
        path.clone(),
        error_prefix.to_string(),
        navigation.clone(),
        cache,
        display_tracker,
    );

    rayon::spawn(move || {
        let nav_service = NavigationService::new(navigation);
        if let Err(e) = nav_service.select_image(path) {
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui.upgrade() {
                    crate::ui::set_error_with_prefix(
                        &ui,
                        "Failed to update directory",
                        e.to_string(),
                    );
                }
            });
        }
    });
}

fn startup_image_from_args() -> Option<PathBuf> {
    std::env::args_os()
        .skip(1)
        .filter_map(|arg| {
            let arg_str = arg.to_string_lossy();
            if arg_str.starts_with('-') {
                None
            } else {
                Some(PathBuf::from(arg))
            }
        })
        .find(|path| crate::file_utils::is_supported_image(path))
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn setup_platform_window_hooks(
    app: &crate::AppWindow,
    app_state: &AppState,
    display_tracker: &DisplayTracker,
) {
    use i_slint_backend_winit::WinitWindowAccessor;
    use i_slint_backend_winit::{winit::event::WindowEvent, EventResult};

    let display_tracker_clone = display_tracker.clone();
    let ui_handle = app.as_weak();
    let navigation = app_state.navigation.clone();
    let cache = app_state.image_cache.clone();
    let window = app.window();

    let initial_pos = window.position();
    let screen_id = crate::services::DisplayProfileService::new()
        .screen_id_from_position(initial_pos.x, initial_pos.y);
    log::info!("Initial display screen ID: {:?}", screen_id);
    display_tracker.update_display_id(screen_id);

    window.on_winit_window_event(move |_window, event| {
        match event {
            WindowEvent::Moved(pos) => {
                let prev_id = display_tracker_clone.current_display_id();
                let screen_id = crate::services::DisplayProfileService::new()
                    .screen_id_from_position(pos.x, pos.y);

                if screen_id != prev_id {
                    log::info!("Display changed: {:?} -> {:?}", prev_id, screen_id);
                }

                display_tracker_clone.update_display_id(screen_id);
            }
            WindowEvent::DroppedFile(path) => {
                if crate::file_utils::is_supported_image(path) {
                    open_image_path(
                        ui_handle.clone(),
                        path.clone(),
                        navigation.clone(),
                        cache.clone(),
                        display_tracker_clone.clone(),
                        "Failed to load opened image",
                    );
                }
            }
            _ => {}
        }

        EventResult::Propagate
    });
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn setup_platform_window_hooks(
    _app: &crate::AppWindow,
    _app_state: &AppState,
    display_tracker: &DisplayTracker,
) {
    display_tracker.update_display_id(None);
}

pub fn configure_startup_opening(
    app: &crate::AppWindow,
    app_state: &AppState,
    display_tracker: &DisplayTracker,
) {
    setup_platform_window_hooks(app, app_state, display_tracker);

    if let Some(path) = startup_image_from_args() {
        open_image_path(
            app.as_weak(),
            path,
            app_state.navigation.clone(),
            app_state.image_cache.clone(),
            display_tracker.clone(),
            "Failed to load startup image",
        );
    }
}
