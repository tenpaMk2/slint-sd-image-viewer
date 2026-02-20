//! 色管理サービス。

use std::cell::RefCell;
use std::fmt;

#[cfg(target_os = "macos")]
use lcms2::{Flags, Intent, PixelFormat, Profile, Transform};
use once_cell::sync::Lazy;

#[cfg(target_os = "macos")]
use crate::services::DisplayProfileService;

/// 色管理処理で発生するエラー。
#[derive(Debug)]
pub enum ColorManagementError {
    /// ディスプレイICCプロファイルの取得失敗。
    DisplayProfileLoad(String),
    /// 画像ICCプロファイルの解析失敗。
    SourceProfileParse(String),
    /// ディスプレイICCプロファイルの解析失敗。
    DestinationProfileParse(String),
    /// ICC変換器の作成失敗。
    TransformCreate(String),
}

impl fmt::Display for ColorManagementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DisplayProfileLoad(msg) => {
                write!(f, "Failed to load display ICC profile: {}", msg)
            }
            Self::SourceProfileParse(msg) => {
                write!(f, "Failed to parse image ICC profile: {}", msg)
            }
            Self::DestinationProfileParse(msg) => {
                write!(f, "Failed to parse display ICC profile: {}", msg)
            }
            Self::TransformCreate(msg) => write!(f, "Failed to create ICC transform: {}", msg),
        }
    }
}

impl std::error::Error for ColorManagementError {}

/// RGB8画像データに色管理を適用するサービス。
pub trait ColorManagementService: Send + Sync {
    /// 画像ICCとディスプレイICCを使って色変換を適用する。
    ///
    /// # Arguments
    ///
    /// * `rgb_data` - 変換対象のRGB8データ（in-place変換）
    /// * `image_icc_profile` - 画像に埋め込まれたICCプロファイル（あれば）
    /// * `screen_id` - 対象ディスプレイのスクリーンID（`None`の場合は先頭ディスプレイ）
    fn apply_to_rgb8(
        &self,
        rgb_data: &mut [u8],
        image_icc_profile: Option<&[u8]>,
        screen_id: Option<u32>,
    ) -> Result<(), ColorManagementError>;
}

/// 色管理を適用しないダミー実装。
#[cfg(not(target_os = "macos"))]
pub struct NoopColorManagementService;

#[cfg(not(target_os = "macos"))]
impl ColorManagementService for NoopColorManagementService {
    fn apply_to_rgb8(
        &self,
        _rgb_data: &mut [u8],
        _image_icc_profile: Option<&[u8]>,
        _screen_id: Option<u32>,
    ) -> Result<(), ColorManagementError> {
        Ok(())
    }
}

#[cfg(target_os = "macos")]
thread_local! {
    static COLOR_TRANSFORM_BUFFER: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// macOS向け色管理サービス。
#[cfg(target_os = "macos")]
pub struct MacOsColorManagementService {
    display_profile_service: DisplayProfileService,
}

#[cfg(target_os = "macos")]
impl MacOsColorManagementService {
    /// 新しいサービスを作成する。
    pub fn new() -> Self {
        Self {
            display_profile_service: DisplayProfileService::new(),
        }
    }
}

#[cfg(target_os = "macos")]
impl ColorManagementService for MacOsColorManagementService {
    fn apply_to_rgb8(
        &self,
        rgb_data: &mut [u8],
        image_icc_profile: Option<&[u8]>,
        screen_id: Option<u32>,
    ) -> Result<(), ColorManagementError> {
        let display_icc_profile = self
            .display_profile_service
            .load_display_icc_profile(screen_id)
            .map_err(|e| ColorManagementError::DisplayProfileLoad(e.to_string()))?;

        let src_profile = match image_icc_profile {
            Some(icc) if !icc.is_empty() => Profile::new_icc(icc)
                .map_err(|e| ColorManagementError::SourceProfileParse(e.to_string()))?,
            _ => Profile::new_srgb(),
        };

        let dst_profile = Profile::new_icc(&display_icc_profile)
            .map_err(|e| ColorManagementError::DestinationProfileParse(e.to_string()))?;

        let transform = Transform::new_flags(
            &src_profile,
            PixelFormat::RGB_8,
            &dst_profile,
            PixelFormat::RGB_8,
            Intent::RelativeColorimetric,
            Flags::BLACKPOINT_COMPENSATION,
        )
        .map_err(|e| ColorManagementError::TransformCreate(e.to_string()))?;

        COLOR_TRANSFORM_BUFFER.with(|buffer| {
            let mut buffer = buffer.borrow_mut();
            buffer.resize(rgb_data.len(), 0);
            buffer.copy_from_slice(rgb_data);
            transform.transform_pixels(&buffer[..], rgb_data);
        });

        Ok(())
    }
}

#[cfg(target_os = "macos")]
static DEFAULT_COLOR_MANAGEMENT_SERVICE: Lazy<MacOsColorManagementService> =
    Lazy::new(MacOsColorManagementService::new);

#[cfg(not(target_os = "macos"))]
static DEFAULT_COLOR_MANAGEMENT_SERVICE: Lazy<NoopColorManagementService> =
    Lazy::new(|| NoopColorManagementService);

/// デフォルトの色管理サービスを返す。
pub fn default_color_management_service() -> &'static dyn ColorManagementService {
    &*DEFAULT_COLOR_MANAGEMENT_SERVICE
}
