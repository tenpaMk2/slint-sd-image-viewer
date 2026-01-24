//! Image loading and display logic.
//!
//! Uses `rayon::spawn` for CPU-intensive image decoding operations,
//! then `slint::invoke_from_event_loop` to update UI from the background thread.

use crate::image_loader;
use log::error;
use slint::ComponentHandle;
use std::path::PathBuf;

/// Helper function to load an image in a background thread and update UI.
///
/// This function:
/// 1. Spawns a rayon thread to decode the image (CPU-intensive)
/// 2. Uses invoke_from_event_loop to return to the UI thread
/// 3. Updates ViewState with the loaded image or error message
pub fn load_and_display_image(
    ui: slint::Weak<crate::AppWindow>,
    path: PathBuf,
    error_prefix: String,
) {
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
                        ui.global::<crate::ViewState>().set_dynamic_image(image);
                        ui.global::<crate::ViewState>().set_image_loaded(true);
                        ui.global::<crate::ViewState>().set_error_message("".into());
                    }
                    Err(e) => {
                        error!("{}: {}", error_prefix, e);
                        ui.global::<crate::ViewState>()
                            .set_error_message(format!("{}: {}", error_prefix, e).into());
                    }
                }
            }
        });
    });
}
