//! Clipboard service for copying file paths to OS clipboard.
//!
//! Supports macOS, Windows, and Linux with platform-specific implementations
//! for copying file lists in native formats.

use log::info;
use std::fmt;
use std::path::PathBuf;

/// Errors that can occur during clipboard operations.
#[derive(Debug)]
pub enum ClipboardError {
    /// No files were provided to copy.
    EmptyPaths,
    /// No valid file paths could be processed.
    InvalidPaths,
    /// One or more files do not exist.
    FileNotFound(PathBuf),
    /// Platform-specific error occurred.
    PlatformError(String),
}

impl fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPaths => write!(f, "No files to copy"),
            Self::InvalidPaths => write!(f, "No valid file paths"),
            Self::FileNotFound(path) => write!(f, "File not found: {:?}", path),
            Self::PlatformError(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ClipboardError {}

#[cfg(target_os = "macos")]
use {
    objc2::rc::{autoreleasepool, Retained},
    objc2::runtime::ProtocolObject,
    objc2::{msg_send, ClassType},
    objc2_app_kit::{NSPasteboard, NSPasteboardWriting},
    objc2_foundation::{NSArray, NSString, NSURL},
};

#[cfg(target_os = "windows")]
use {
    std::os::windows::ffi::OsStrExt,
    windows::{
        Win32::Foundation::{HANDLE, HWND},
        Win32::System::Com::{CoInitialize, CoUninitialize},
        Win32::System::DataExchange::{
            CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
        },
        Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
        Win32::UI::Shell::DROPFILES,
    },
};

#[cfg(target_os = "linux")]
use arboard::Clipboard;

/// Service for managing clipboard operations.
pub struct ClipboardService;

impl ClipboardService {
    /// Creates a new clipboard service.
    pub fn new() -> Self {
        Self
    }

    /// Copies the specified file paths to the clipboard.
    pub fn copy_files(&self, paths: Vec<PathBuf>) -> Result<(), ClipboardError> {
        // Validate paths
        Self::validate_paths(&paths)?;

        info!("Copying {} file(s) to clipboard", paths.len());

        #[cfg(target_os = "macos")]
        {
            self.copy_files_macos(paths)
        }

        #[cfg(target_os = "windows")]
        {
            self.copy_files_windows(paths)
        }

        #[cfg(target_os = "linux")]
        {
            self.copy_files_linux(paths)
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            Err(ClipboardError::PlatformError(
                "Clipboard operation not supported on this platform".to_string(),
            ))
        }
    }

    /// Validates that paths are not empty and all files exist.
    fn validate_paths(paths: &[PathBuf]) -> Result<(), ClipboardError> {
        if paths.is_empty() {
            return Err(ClipboardError::EmptyPaths);
        }

        // Check all files exist
        for path in paths {
            if !path.exists() {
                return Err(ClipboardError::FileNotFound(path.clone()));
            }
        }

        Ok(())
    }

    /// Converts paths to string slices, filtering out invalid UTF-8 paths.
    fn paths_to_strings(paths: &[PathBuf]) -> Result<Vec<String>, ClipboardError> {
        let strings: Vec<String> = paths
            .iter()
            .filter_map(|p| p.to_str().map(String::from))
            .collect();

        if strings.is_empty() {
            return Err(ClipboardError::InvalidPaths);
        }

        Ok(strings)
    }

    /// macOS implementation: Copy files using NSPasteboard
    #[cfg(target_os = "macos")]
    fn copy_files_macos(&self, paths: Vec<PathBuf>) -> Result<(), ClipboardError> {
        autoreleasepool(|_| {
            // Get general pasteboard
            let pasteboard: Option<Retained<NSPasteboard>> =
                unsafe { msg_send![NSPasteboard::class(), generalPasteboard] };

            let pasteboard = pasteboard.ok_or_else(|| {
                ClipboardError::PlatformError("Failed to get pasteboard".to_string())
            })?;

            // Clear existing contents
            pasteboard.clearContents();

            // Convert paths to valid strings first
            let path_strings = Self::paths_to_strings(&paths)?;

            // Convert to NSURL array
            let ns_urls: Vec<Retained<NSURL>> = path_strings
                .iter()
                .map(|path_str| {
                    let ns_string = NSString::from_str(path_str);
                    NSURL::fileURLWithPath(&ns_string)
                })
                .collect();

            // Write URLs to clipboard
            let success = unsafe {
                let ns_urls_slice: Vec<&NSURL> = ns_urls.iter().map(|url| url.as_ref()).collect();
                let url_array = NSArray::from_slice(&ns_urls_slice);

                #[allow(clippy::as_conversions)]
                let writing_array = &*(url_array.as_ref() as *const NSArray<NSURL>
                    as *const NSArray<ProtocolObject<dyn NSPasteboardWriting>>);

                pasteboard.writeObjects(writing_array)
            };

            if success {
                info!("Successfully copied files to clipboard");
                Ok(())
            } else {
                Err(ClipboardError::PlatformError(
                    "Failed to write to clipboard".to_string(),
                ))
            }
        })
    }

    /// Windows implementation: Copy files using CF_HDROP format
    #[cfg(target_os = "windows")]
    fn copy_files_windows(&self, paths: Vec<PathBuf>) -> Result<(), ClipboardError> {
        // RAII guard for clipboard - automatically closes on drop
        struct ClipboardGuard;
        impl Drop for ClipboardGuard {
            fn drop(&mut self) {
                unsafe {
                    let _ = CloseClipboard();
                }
            }
        }

        // COM initialization
        unsafe {
            CoInitialize(None).ok().map_err(|_| {
                ClipboardError::PlatformError("Failed to initialize COM".to_string())
            })?;
        }

        let result = (|| -> Result<(), ClipboardError> {
            unsafe {
                // Open clipboard
                OpenClipboard(Some(HWND::default())).map_err(|_| {
                    ClipboardError::PlatformError("Failed to open clipboard".to_string())
                })?;

                // Guard ensures clipboard is closed even on early return
                let _guard = ClipboardGuard;

                EmptyClipboard().map_err(|_| {
                    ClipboardError::PlatformError("Failed to clear clipboard".to_string())
                })?;

                // CF_HDROP format
                let cf_hdrop = 15u32;

                // Prepare path strings (each path is null-terminated, final double-null)
                let mut buffer = Vec::new();
                let dropfiles_size = std::mem::size_of::<DROPFILES>();

                // Reserve space for DROPFILES structure
                buffer.resize(dropfiles_size, 0u8);

                for path in &paths {
                    let wide_path: Vec<u16> = std::ffi::OsStr::new(path)
                        .encode_wide()
                        .chain(std::iter::once(0))
                        .collect();

                    let byte_slice = std::slice::from_raw_parts(
                        wide_path.as_ptr() as *const u8,
                        wide_path.len() * 2,
                    );
                    buffer.extend_from_slice(byte_slice);
                }

                // Add final null terminator
                buffer.extend_from_slice(&[0u8, 0u8]);

                // Set up DROPFILES structure
                let dropfiles = buffer.as_mut_ptr() as *mut DROPFILES;
                (*dropfiles).pFiles = dropfiles_size as u32;
                (*dropfiles).pt.x = 0;
                (*dropfiles).pt.y = 0;
                (*dropfiles).fNC = false.into();
                (*dropfiles).fWide = true.into(); // Use Unicode

                // Copy to global memory
                let hmem = GlobalAlloc(GMEM_MOVEABLE, buffer.len()).map_err(|_| {
                    ClipboardError::PlatformError("Failed to allocate global memory".to_string())
                })?;

                if hmem.is_invalid() {
                    return Err(ClipboardError::PlatformError(
                        "Failed to allocate global memory".to_string(),
                    ));
                }

                let ptr = GlobalLock(hmem);
                if ptr.is_null() {
                    return Err(ClipboardError::PlatformError(
                        "Failed to lock global memory".to_string(),
                    ));
                }

                std::ptr::copy_nonoverlapping(buffer.as_ptr(), ptr as *mut u8, buffer.len());
                GlobalUnlock(hmem).ok();

                // Set clipboard data
                SetClipboardData(cf_hdrop, Some(HANDLE(hmem.0))).map_err(|_| {
                    ClipboardError::PlatformError("Failed to set clipboard data".to_string())
                })?;

                info!("Successfully copied files to clipboard");
                Ok(())
            }
        })();

        // COM cleanup
        unsafe {
            CoUninitialize();
        }

        result
    }

    /// Linux implementation: Copy files using arboard with file URI list
    #[cfg(target_os = "linux")]
    fn copy_files_linux(&self, paths: Vec<PathBuf>) -> Result<(), ClipboardError> {
        let mut clipboard = Clipboard::new().map_err(|e| {
            ClipboardError::PlatformError(format!("Failed to access clipboard: {}", e))
        })?;

        // Convert paths to valid strings first
        let path_strings = Self::paths_to_strings(&paths)?;

        // Join paths with newlines
        // Note: arboard expects text, so we use simple text format
        // For proper file list, we would need text/uri-list MIME type
        let text = path_strings.join("\n");

        clipboard.set_text(text).map_err(|e| {
            ClipboardError::PlatformError(format!("Failed to set clipboard: {}", e))
        })?;

        info!("Successfully copied files to clipboard");
        Ok(())
    }
}
