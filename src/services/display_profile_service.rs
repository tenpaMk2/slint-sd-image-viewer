//! ディスプレイICCプロファイル取得サービス。

use std::fmt;

#[cfg(target_os = "windows")]
use std::ffi::OsString;
#[cfg(target_os = "windows")]
use std::hash::{Hash, Hasher};
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStringExt;

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
    #[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
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
    #[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
    pub fn screen_id_from_position(&self, x: i32, y: i32) -> Option<u32> {
        #[cfg(target_os = "macos")]
        {
            self.screen_id_from_position_macos(x, y)
        }

        #[cfg(target_os = "windows")]
        {
            self.screen_id_from_position_windows(x, y)
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
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

        #[cfg(target_os = "windows")]
        {
            self.load_display_icc_profile_windows(screen_id)
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            let _ = screen_id;
            Err(DisplayProfileError::PlatformError(
                "Display ICC profile is not supported on this platform".to_string(),
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

    #[cfg(target_os = "windows")]
    fn screen_id_from_position_windows(&self, x: i32, y: i32) -> Option<u32> {
        use windows::Win32::Foundation::POINT;
        use windows::Win32::Graphics::Gdi::{
            GetMonitorInfoW, MonitorFromPoint, MONITORINFOEXW, MONITOR_DEFAULTTONEAREST,
        };

        let monitor = unsafe { MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST) };
        if monitor.0.is_null() {
            return None;
        }

        let mut monitor_info = MONITORINFOEXW::default();
        monitor_info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

        let success = unsafe {
            GetMonitorInfoW(
                monitor,
                (&mut monitor_info as *mut MONITORINFOEXW).cast(),
            )
        }
        .as_bool();

        if !success {
            return None;
        }

        Some(Self::monitor_id_from_device_name(&monitor_info.szDevice))
    }

    #[cfg(target_os = "windows")]
    fn load_display_icc_profile_windows(
        &self,
        screen_id: Option<u32>,
    ) -> Result<Vec<u8>, DisplayProfileError> {
        use windows::core::{w, BOOL, PCWSTR, PWSTR};
        use windows::Win32::Foundation::{LPARAM, RECT};
        use windows::Win32::Graphics::Gdi::{
            CreateDCW, DeleteDC, EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR,
            MONITORINFOEXW,
        };
        use windows::Win32::UI::ColorSystem::GetICMProfileW;

        unsafe extern "system" fn enum_monitor_proc(
            monitor: HMONITOR,
            _hdc: HDC,
            _rect: *mut RECT,
            lparam: LPARAM,
        ) -> BOOL {
            let monitors = &mut *(lparam.0 as *mut Vec<(u32, Vec<u16>)>);

            let mut monitor_info = MONITORINFOEXW::default();
            monitor_info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

            if GetMonitorInfoW(
                monitor,
                (&mut monitor_info as *mut MONITORINFOEXW).cast(),
            )
            .as_bool()
            {
                let device_name = monitor_info.szDevice.to_vec();
                let id = DisplayProfileService::monitor_id_from_device_name(&monitor_info.szDevice);
                monitors.push((id, device_name));
            }

            true.into()
        }

        let mut monitors: Vec<(u32, Vec<u16>)> = Vec::new();

        let enum_ok = unsafe {
            EnumDisplayMonitors(
                None,
                None,
                Some(enum_monitor_proc),
                LPARAM((&mut monitors as *mut Vec<(u32, Vec<u16>)>) as isize),
            )
        }
        .as_bool();

        if !enum_ok || monitors.is_empty() {
            return Err(DisplayProfileError::PlatformError(
                "No display found".to_string(),
            ));
        }

        let target_device = if let Some(target_id) = screen_id {
            monitors
                .iter()
                .find(|(id, _)| *id == target_id)
                .map(|(_, name)| name.clone())
                .unwrap_or_else(|| monitors[0].1.clone())
        } else {
            monitors[0].1.clone()
        };

        let hdc = unsafe {
            CreateDCW(
                w!("DISPLAY"),
                PCWSTR(target_device.as_ptr()),
                PCWSTR::null(),
                None,
            )
        };

        if hdc.0.is_null() {
            return Err(DisplayProfileError::PlatformError(
                "Failed to create display device context".to_string(),
            ));
        }

        let mut profile_path_len: u32 = 0;
        let _ = unsafe { GetICMProfileW(hdc, &mut profile_path_len, Some(PWSTR::null())) };

        if profile_path_len == 0 {
            unsafe {
                let _ = DeleteDC(hdc);
            }
            return Err(DisplayProfileError::PlatformError(
                "No display ICC profile available".to_string(),
            ));
        }

        let mut profile_path_buf = vec![0u16; profile_path_len as usize];
        let profile_ok = unsafe {
            GetICMProfileW(
                hdc,
                &mut profile_path_len,
                Some(PWSTR(profile_path_buf.as_mut_ptr())),
            )
        }
        .as_bool();

        unsafe {
            let _ = DeleteDC(hdc);
        }

        if !profile_ok {
            return Err(DisplayProfileError::PlatformError(
                "Failed to query display ICC profile path".to_string(),
            ));
        }

        let nul_pos = profile_path_buf
            .iter()
            .position(|c| *c == 0)
            .unwrap_or(profile_path_buf.len());
        let profile_path = OsString::from_wide(&profile_path_buf[..nul_pos]);

        std::fs::read(profile_path).map_err(|e| {
            DisplayProfileError::PlatformError(format!("Failed to read display ICC profile: {}", e))
        })
    }

    #[cfg(target_os = "windows")]
    fn monitor_id_from_device_name(device_name: &[u16]) -> u32 {
        let nul_pos = device_name
            .iter()
            .position(|c| *c == 0)
            .unwrap_or(device_name.len());
        let name = String::from_utf16_lossy(&device_name[..nul_pos]);

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        name.hash(&mut hasher);
        hasher.finish() as u32
    }
}
