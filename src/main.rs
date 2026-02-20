// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

mod config;
mod error;
mod file_utils;
mod image_cache;
mod image_loader;
mod metadata;
mod services;
mod state;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .format(|buf, record| {
            use std::io::Write;

            let pkg_name = env!("CARGO_PKG_NAME").replace("-", "_");
            let prefix = format!("{}::", pkg_name);
            let target = record
                .target()
                .strip_prefix(&prefix)
                .unwrap_or(record.target());

            let level_style = buf.default_level_style(record.level());
            let level = level_style.render();
            let reset = level_style.render_reset();

            // JSTタイムスタンプ（時刻のみ）
            let timestamp = chrono::Local::now().format("%H:%M:%S");

            writeln!(
                buf,
                "[{} {}{}{} {}] {}",
                timestamp,
                level,
                record.level(),
                reset,
                target,
                record.args()
            )
        })
        .init();

    let app = AppWindow::new()?;
    let app_state = state::AppState::new();

    // Create display tracker for color management
    let display_tracker = ui::DisplayTracker::new();

    // Setup window event hook for display tracking (macOS only)
    #[cfg(target_os = "macos")]
    {
        use i_slint_backend_winit::WinitWindowAccessor;
        use services::DisplayProfileService;

        let display_tracker_clone = display_tracker.clone();
        let window = app.window();

        // Initialize with current window position
        let initial_pos = window.position();
        let screen_id =
            DisplayProfileService::new().screen_id_from_position(initial_pos.x, initial_pos.y);
        log::info!("Initial display screen ID: {:?}", screen_id);
        display_tracker.update_display_id(screen_id);

        // Register window event handler for position changes
        window.on_winit_window_event(move |_window, event| {
            use i_slint_backend_winit::winit::event::WindowEvent;
            use i_slint_backend_winit::EventResult;

            if let WindowEvent::Moved(pos) = event {
                let prev_id = display_tracker_clone.current_display_id();
                let screen_id = DisplayProfileService::new().screen_id_from_position(pos.x, pos.y);

                if screen_id != prev_id {
                    log::info!("Display changed: {:?} -> {:?}", prev_id, screen_id);
                }

                display_tracker_clone.update_display_id(screen_id);
            }

            EventResult::Propagate
        });
    }

    // Setup all UI event handlers
    ui::setup_handlers(&app, app_state, display_tracker);

    app.run()?;

    Ok(())
}
