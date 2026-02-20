//! ディスプレイICCプロファイル取得サービス。

use std::fmt;

/// ディスプレイICCプロファイル取得時のエラー。
#[derive(Debug)]
pub enum DisplayProfileError {
    /// 画面一覧が取得できなかった。
    NoDisplay,
    /// カラースペースが取得できなかった。
    NoColorSpace,
    /// ICCデータが取得できなかった。
    NoIccData,
    /// プラットフォーム依存エラー。
    #[cfg(not(target_os = "macos"))]
    PlatformError(String),
}

impl fmt::Display for DisplayProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDisplay => write!(f, "No display found"),
            Self::NoColorSpace => write!(f, "No display color space available"),
            Self::NoIccData => write!(f, "No display ICC profile available"),
            #[cfg(not(target_os = "macos"))]
            Self::PlatformError(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for DisplayProfileError {}

/// ディスプレイICCプロファイル読み込みサービス。
pub struct DisplayProfileService;

impl DisplayProfileService {
    /// 新しいサービスを作成する。
    pub fn new() -> Self {
        Self
    }

    /// 最初に検出されたディスプレイのICCプロファイルを読み込む。
    pub fn load_first_display_icc_profile(&self) -> Result<Vec<u8>, DisplayProfileError> {
        #[cfg(target_os = "macos")]
        {
            self.load_first_display_icc_profile_macos()
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(DisplayProfileError::PlatformError(
                "Display ICC profile is supported only on macOS".to_string(),
            ))
        }
    }

    #[cfg(target_os = "macos")]
    fn load_first_display_icc_profile_macos(&self) -> Result<Vec<u8>, DisplayProfileError> {
        use objc2::rc::{autoreleasepool, Retained};
        use objc2::runtime::AnyObject;
        use objc2::{msg_send, ClassType};
        use objc2_app_kit::NSScreen;
        use objc2_foundation::{NSArray, NSData};

        autoreleasepool(|_| {
            let screens: Option<Retained<NSArray<NSScreen>>> = unsafe {
                // 安全性: Cocoaのクラスメソッド呼び出し
                msg_send![NSScreen::class(), screens]
            };

            let screens = screens.ok_or(DisplayProfileError::NoDisplay)?;

            let first_screen: Option<&NSScreen> = unsafe {
                // 安全性: firstObjectは所有権を移さない参照を返す
                msg_send![&*screens, firstObject]
            };
            let first_screen = first_screen.ok_or(DisplayProfileError::NoDisplay)?;

            let color_space: Option<Retained<AnyObject>> = unsafe {
                // 安全性: NSScreenのcolorSpace取得
                msg_send![first_screen, colorSpace]
            };
            let color_space = color_space.ok_or(DisplayProfileError::NoColorSpace)?;

            let icc_data: Option<Retained<NSData>> = unsafe {
                // 安全性: NSColorSpaceのICCProfileData取得
                msg_send![&*color_space, ICCProfileData]
            };
            let icc_data = icc_data.ok_or(DisplayProfileError::NoIccData)?;

            let length = icc_data.len();
            if length == 0 {
                return Err(DisplayProfileError::NoIccData);
            }

            Ok(icc_data.to_vec())
        })
    }
}
