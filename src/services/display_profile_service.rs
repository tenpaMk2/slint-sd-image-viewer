//! ディスプレイICCプロファイル取得サービス。

use std::fmt;

/// ディスプレイICCプロファイル取得時のエラー。
#[derive(Debug)]
pub enum DisplayProfileError {
    /// 画面一覧が取得できなかった。
    #[cfg(target_os = "macos")]
    NoDisplay,
    /// カラースペースが取得できなかった。
    #[cfg(target_os = "macos")]
    NoColorSpace,
    /// ICCデータが取得できなかった。
    #[cfg(target_os = "macos")]
    NoIccData,
    /// プラットフォーム依存エラー。
    #[cfg(not(target_os = "macos"))]
    PlatformError(String),
}

impl fmt::Display for DisplayProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(target_os = "macos")]
            Self::NoDisplay => write!(f, "No display found"),
            #[cfg(target_os = "macos")]
            Self::NoColorSpace => write!(f, "No display color space available"),
            #[cfg(target_os = "macos")]
            Self::NoIccData => write!(f, "No display ICC profile available"),
            #[cfg(not(target_os = "macos"))]
            Self::PlatformError(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for DisplayProfileError {}

/// ディスプレイ情報取得・ICCプロファイル読み込みサービス。
pub struct DisplayProfileService;

impl DisplayProfileService {
    /// 新しいサービスを作成する。
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    pub fn new() -> Self {
        Self
    }

    /// 指定座標が含まれるディスプレイのスクリーンIDを取得する。
    ///
    /// # Arguments
    ///
    /// * `x` - スクリーン座標系のX座標（物理ピクセル）
    /// * `y` - スクリーン座標系のY座標（物理ピクセル）
    ///
    /// # Returns
    ///
    /// スクリーンIDが見つかった場合は `Some(id)`、見つからない場合またはプラットフォーム非対応の場合は `None`。
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    pub fn screen_id_from_position(&self, x: i32, y: i32) -> Option<u32> {
        #[cfg(target_os = "macos")]
        {
            self.screen_id_from_position_macos(x, y)
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = (x, y);
            None
        }
    }

    /// 指定スクリーンIDのディスプレイICCプロファイルを読み込む。
    ///
    /// # Arguments
    ///
    /// * `screen_id` - 対象スクリーンID。`None`の場合は先頭ディスプレイを使用。
    ///
    /// # Returns
    ///
    /// ICCプロファイルのバイト列。指定IDが見つからない場合は先頭ディスプレイへフォールバック。
    pub fn load_display_icc_profile(
        &self,
        screen_id: Option<u32>,
    ) -> Result<Vec<u8>, DisplayProfileError> {
        #[cfg(target_os = "macos")]
        {
            self.load_display_icc_profile_macos(screen_id)
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = screen_id;
            Err(DisplayProfileError::PlatformError(
                "Display ICC profile is supported only on macOS".to_string(),
            ))
        }
    }

    /// 最初に検出されたディスプレイのICCプロファイルを読み込む（後方互換用）。
    #[allow(dead_code)]
    pub fn load_first_display_icc_profile(&self) -> Result<Vec<u8>, DisplayProfileError> {
        self.load_display_icc_profile(None)
    }

    #[cfg(target_os = "macos")]
    fn screen_id_from_position_macos(&self, x: i32, y: i32) -> Option<u32> {
        use objc2::rc::{autoreleasepool, Retained};
        use objc2::runtime::AnyObject;
        use objc2::{msg_send, ClassType};
        use objc2_app_kit::NSScreen;
        use objc2_foundation::{NSArray, NSDictionary, NSNumber, NSString};

        autoreleasepool(|_| {
            let screens: Option<Retained<NSArray<NSScreen>>> = unsafe {
                // 安全性: Cocoaのクラスメソッド呼び出し
                msg_send![NSScreen::class(), screens]
            };

            let screens = screens?;
            let count: usize = unsafe { msg_send![&*screens, count] };

            for i in 0..count {
                let screen: Option<&NSScreen> = unsafe { msg_send![&*screens, objectAtIndex: i] };
                let screen = screen?;

                // NSScreen の frame を取得（物理ピクセル座標系）
                let frame: objc2_foundation::NSRect = unsafe { msg_send![screen, frame] };

                // 座標がこのスクリーンの範囲内か判定
                let screen_x = frame.origin.x as i32;
                let screen_y = frame.origin.y as i32;
                let screen_width = frame.size.width as i32;
                let screen_height = frame.size.height as i32;

                if x >= screen_x
                    && x < screen_x + screen_width
                    && y >= screen_y
                    && y < screen_y + screen_height
                {
                    // deviceDescription から NSScreenNumber を取得
                    let device_description: Option<Retained<NSDictionary<NSString, AnyObject>>> =
                        unsafe { msg_send![screen, deviceDescription] };
                    let device_description = device_description?;

                    let key = NSString::from_str("NSScreenNumber");
                    let screen_number: Option<Retained<NSNumber>> =
                        unsafe { msg_send![&*device_description, objectForKey: &*key] };
                    let screen_number = screen_number?;

                    let id: u32 = unsafe { msg_send![&*screen_number, unsignedIntValue] };
                    return Some(id);
                }
            }

            None
        })
    }

    #[cfg(target_os = "macos")]
    fn load_display_icc_profile_macos(
        &self,
        screen_id: Option<u32>,
    ) -> Result<Vec<u8>, DisplayProfileError> {
        use objc2::rc::{autoreleasepool, Retained};
        use objc2::runtime::AnyObject;
        use objc2::{msg_send, ClassType};
        use objc2_app_kit::NSScreen;
        use objc2_foundation::{NSArray, NSData, NSDictionary, NSNumber, NSString};

        autoreleasepool(|_| {
            let screens: Option<Retained<NSArray<NSScreen>>> = unsafe {
                // 安全性: Cocoaのクラスメソッド呼び出し
                msg_send![NSScreen::class(), screens]
            };

            let screens = screens.ok_or(DisplayProfileError::NoDisplay)?;

            let target_screen: Option<&NSScreen> = if let Some(target_id) = screen_id {
                // 指定IDのスクリーンを検索
                let count: usize = unsafe { msg_send![&*screens, count] };
                let mut found_screen: Option<&NSScreen> = None;

                for i in 0..count {
                    let screen: Option<&NSScreen> =
                        unsafe { msg_send![&*screens, objectAtIndex: i] };
                    if let Some(screen) = screen {
                        let device_description: Option<
                            Retained<NSDictionary<NSString, AnyObject>>,
                        > = unsafe { msg_send![screen, deviceDescription] };

                        if let Some(device_description) = device_description {
                            let key = NSString::from_str("NSScreenNumber");
                            let screen_number: Option<Retained<NSNumber>> =
                                unsafe { msg_send![&*device_description, objectForKey: &*key] };

                            if let Some(screen_number) = screen_number {
                                let id: u32 =
                                    unsafe { msg_send![&*screen_number, unsignedIntValue] };
                                if id == target_id {
                                    found_screen = Some(screen);
                                    break;
                                }
                            }
                        }
                    }
                }

                // 見つからない場合は先頭スクリーンへフォールバック
                if found_screen.is_none() {
                    unsafe { msg_send![&*screens, firstObject] }
                } else {
                    found_screen
                }
            } else {
                // screen_id が None の場合は先頭スクリーン
                unsafe { msg_send![&*screens, firstObject] }
            };

            let target_screen = target_screen.ok_or(DisplayProfileError::NoDisplay)?;

            let color_space: Option<Retained<AnyObject>> = unsafe {
                // 安全性: NSScreenのcolorSpace取得
                msg_send![target_screen, colorSpace]
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
