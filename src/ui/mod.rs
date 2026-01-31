//! UI module for handling user interactions and UI updates.
//!
//! Threading model:
//! - `slint::spawn_local`: UI非同期処理（ファイルダイアログなど、メインスレッドで実行する必要がある処理）
//! - `rayon::spawn`: CPU集約的処理（画像デコード、ディレクトリスキャンなど、別スレッドで実行可能な重い処理）
//! - `slint::invoke_from_event_loop`: rayonからUIスレッドへの結果返却時に使用

pub mod handlers;
pub mod image_display;
mod state_helpers;

pub use handlers::setup_handlers;
pub use state_helpers::*;
