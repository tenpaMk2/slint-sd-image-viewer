// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

slint::include_modules!();

mod config;
mod error;
mod file_utils;
mod image_loader;
mod state;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = AppWindow::new()?;
    let navigation_state = Arc::new(Mutex::new(state::NavigationState::new()));

    // Setup all UI event handlers
    ui::setup_handlers(&app, navigation_state);

    app.run()?;

    Ok(())
}
