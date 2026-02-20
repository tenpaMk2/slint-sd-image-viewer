//! ディスプレイID状態管理。

use std::sync::{Arc, RwLock};

/// 現在のディスプレイIDを保持する状態ホルダー。
///
/// UI層でウィンドウ移動イベント時に更新され、画像ローダー（ワーカースレッド）から
/// 非同期読み取りされる。プラットフォーム固有ロジックは持たず、純粋な状態管理のみ担当。
#[derive(Clone)]
pub struct DisplayTracker {
    /// 現在のディスプレイID（初期化前またはサポート外環境では `None`）。
    screen_id: Arc<RwLock<Option<u32>>>,
}

impl DisplayTracker {
    /// 新しいトラッカーを作成する（初期状態は `None`）。
    pub fn new() -> Self {
        Self {
            screen_id: Arc::new(RwLock::new(None)),
        }
    }

    /// 現在のディスプレイIDを取得する。
    ///
    /// 複数のワーカースレッドから並行して読み取り可能。
    pub fn current_display_id(&self) -> Option<u32> {
        self.screen_id
            .read()
            .expect("DisplayTracker RwLock poisoned")
            .clone()
    }

    /// ディスプレイIDを更新する。
    ///
    /// UI層から呼び出される。`None` を渡すと未設定状態に戻る。
    pub fn update_display_id(&self, id: Option<u32>) {
        *self
            .screen_id
            .write()
            .expect("DisplayTracker RwLock poisoned") = id;
    }
}

impl Default for DisplayTracker {
    fn default() -> Self {
        Self::new()
    }
}
