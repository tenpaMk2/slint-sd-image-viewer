// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rfd::AsyncFileDialog;
use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

slint::include_modules!();

mod file_utils;
mod image_loader;

#[derive(Default)]
struct ImageViewerState {
    current_directory: Option<PathBuf>,
    image_files: Vec<PathBuf>,
    current_index: usize,
}

impl ImageViewerState {
    fn next_image(&mut self) -> Option<PathBuf> {
        if !self.image_files.is_empty() && self.current_index + 1 < self.image_files.len() {
            self.current_index += 1;
            Some(self.image_files[self.current_index].clone())
        } else {
            None
        }
    }

    fn prev_image(&mut self) -> Option<PathBuf> {
        if !self.image_files.is_empty() && self.current_index > 0 {
            self.current_index -= 1;
            Some(self.image_files[self.current_index].clone())
        } else {
            None
        }
    }

    fn update_directory(&mut self, file_path: PathBuf) {
        if let Some(parent) = file_path.parent() {
            self.current_directory = Some(parent.to_path_buf());

            // Scan directory for image files
            if let Ok(files) = file_utils::scan_directory(parent) {
                self.image_files = files;
                // Find current file index
                self.current_index = self
                    .image_files
                    .iter()
                    .position(|p| p == &file_path)
                    .unwrap_or(0);
            }
        }
    }
}

/// Helper function to spawn an async task with proper wrapping
fn spawn_image_task<F>(future: F)
where
    F: std::future::Future<Output = ()> + 'static,
{
    let _ = slint::spawn_local(async_compat::Compat::new(future));
}

/// Helper function to load an image and update the UI
async fn load_and_display_image(ui: slint::Weak<AppWindow>, path: PathBuf, error_prefix: &str) {
    let ui = ui.unwrap();

    match image_loader::load_image(&path).await {
        Ok(image) => {
            ui.set_dynamic_image(image);
            ui.set_image_loaded(true);
            ui.set_error_message("".into());
        }
        Err(e) => {
            ui.set_error_message(format!("{}: {}", error_prefix, e).into());
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let state = Arc::new(Mutex::new(ImageViewerState::default()));

    ui.global::<Logic>().on_load_image({
        let ui_handle = ui.as_weak();
        let state = state.clone();
        move || {
            let ui_handle = ui_handle.clone();
            let state = state.clone();
            spawn_image_task(async move {
                let ui = ui_handle.clone();

                // Show file dialog
                let file_handle = match AsyncFileDialog::new().pick_file().await {
                    Some(handle) => handle,
                    None => {
                        ui.unwrap().set_error_message("No file selected".into());
                        return;
                    }
                };

                let path = file_handle.path().to_path_buf();

                // Load image and update UI
                load_and_display_image(ui.clone(), path.clone(), "Failed to load image").await;

                // Update state with directory info
                let mut state = state.lock().unwrap();
                state.update_directory(path);
            });
        }
    });

    ui.global::<Logic>().on_next_image({
        let ui_handle = ui.as_weak();
        let state = state.clone();
        move || {
            let ui_handle = ui_handle.clone();
            let state = state.clone();
            spawn_image_task(async move {
                let next_path = {
                    let mut state = state.lock().unwrap();
                    state.next_image()
                };

                if let Some(path) = next_path {
                    load_and_display_image(ui_handle, path, "Failed to load next image").await;
                }
            });
        }
    });

    ui.global::<Logic>().on_prev_image({
        let ui_handle = ui.as_weak();
        let state = state.clone();
        move || {
            let ui_handle = ui_handle.clone();
            let state = state.clone();
            spawn_image_task(async move {
                let prev_path = {
                    let mut state = state.lock().unwrap();
                    state.prev_image()
                };

                if let Some(path) = prev_path {
                    load_and_display_image(ui_handle, path, "Failed to load previous image").await;
                }
            });
        }
    });

    ui.run()?;

    Ok(())
}
