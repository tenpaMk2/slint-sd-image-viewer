// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

mod config;
mod error;
mod file_utils;
mod image_loader;
mod metadata;
mod state;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let app = AppWindow::new()?;
    let app_state = state::AppState::new();

    // Setup all UI event handlers
    ui::setup_handlers(&app, app_state);

    app.run()?;

    Ok(())
}
