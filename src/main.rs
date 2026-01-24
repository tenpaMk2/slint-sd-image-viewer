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

/// Helper function to load an image in a background thread and update UI
fn load_and_display_image(ui: slint::Weak<AppWindow>, path: PathBuf, error_prefix: String) {
    // rayonで別スレッドで画像を読み込む（全ての重い処理を含む）
    rayon::spawn(move || {
        let result = image_loader::load_image_blocking(&path);

        // 読み込み完了後、invoke_from_event_loopでUIスレッドに戻して更新
        // UIスレッドでは軽い処理のみ実行
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui.upgrade() {
                match result {
                    Ok((data, width, height)) => {
                        // 軽い処理のみ：SharedPixelBufferの作成
                        let image = image_loader::create_slint_image(data, width, height);
                        ui.set_dynamic_image(image);
                        ui.set_image_loaded(true);
                        ui.global::<ErrorState>().set_error_message("".into());
                    }
                    Err(e) => {
                        ui.global::<ErrorState>()
                            .set_error_message(format!("{}: {}", error_prefix, e).into());
                    }
                }
            }
        });
    });
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let state = Arc::new(Mutex::new(ImageViewerState::default()));

    ui.global::<Logic>().on_select_image({
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
                        ui.global::<ErrorState>()
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

    ui.global::<Logic>().on_next_image({
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

    ui.global::<Logic>().on_prev_image({
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

    ui.run()?;

    Ok(())
}
