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

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let state = Arc::new(Mutex::new(ImageViewerState::default()));

    ui.global::<Logic>().on_load_image({
        let ui_handle = ui.as_weak();
        let state = state.clone();
        move || {
            let ui_handle = ui_handle.clone();
            let state = state.clone();
            let _ = slint::spawn_local(async_compat::Compat::new(async move {
                let ui = ui_handle.unwrap();

                // Show file dialog
                let file_handle = match AsyncFileDialog::new().pick_file().await {
                    Some(handle) => handle,
                    None => {
                        ui.set_error_message("No file selected".into());
                        return;
                    }
                };

                let path = file_handle.path().to_path_buf();

                // Load image
                match image_loader::load_image(&path).await {
                    Ok(image) => {
                        ui.set_dynamic_image(image);
                        ui.set_image_loaded(true);
                        ui.set_error_message("".into());

                        // Update state with directory info
                        let mut state = state.lock().unwrap();
                        state.update_directory(path.into());
                    }
                    Err(e) => {
                        ui.set_error_message(format!("Failed to load image: {}", e).into());
                    }
                }
            }));
        }
    });

    ui.global::<Logic>().on_next_image({
        let ui_handle = ui.as_weak();
        let state = state.clone();
        move || {
            let ui_handle = ui_handle.clone();
            let state = state.clone();
            let _ = slint::spawn_local(async_compat::Compat::new(async move {
                let ui = ui_handle.unwrap();
                let next_path = {
                    let mut state = state.lock().unwrap();
                    state.next_image()
                };

                if let Some(path) = next_path {
                    match image_loader::load_image(&path).await {
                        Ok(image) => {
                            ui.set_dynamic_image(image);
                            ui.set_image_loaded(true);
                            ui.set_error_message("".into());
                        }
                        Err(e) => {
                            ui.set_error_message(
                                format!("Failed to load next image: {}", e).into(),
                            );
                        }
                    }
                }
            }));
        }
    });

    ui.global::<Logic>().on_prev_image({
        let ui_handle = ui.as_weak();
        let state = state.clone();
        move || {
            let ui_handle = ui_handle.clone();
            let state = state.clone();
            let _ = slint::spawn_local(async_compat::Compat::new(async move {
                let ui = ui_handle.unwrap();
                let prev_path = {
                    let mut state = state.lock().unwrap();
                    state.prev_image()
                };

                if let Some(path) = prev_path {
                    match image_loader::load_image(&path).await {
                        Ok(image) => {
                            ui.set_dynamic_image(image);
                            ui.set_image_loaded(true);
                            ui.set_error_message("".into());
                        }
                        Err(e) => {
                            ui.set_error_message(
                                format!("Failed to load previous image: {}", e).into(),
                            );
                        }
                    }
                }
            }));
        }
    });

    ui.run()?;

    Ok(())
}
