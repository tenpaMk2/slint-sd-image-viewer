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

    // Setup all UI event handlers
    ui::setup_handlers(&app, app_state);

    app.run()?;

    Ok(())
}
