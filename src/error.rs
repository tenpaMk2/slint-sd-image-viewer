//! Unified error types for the image viewer application.

use std::fmt;

/// Application-specific errors.
#[derive(Debug)]
pub enum AppError {
    /// Error loading or decoding an image file
    ImageLoad(String),
    /// Error scanning directory for image files
    DirectoryScan(String),
    /// Error reading XMP metadata
    XmpRead(String),
    /// Error writing XMP metadata
    XmpWrite(String),
    /// Error reading metadata (including SD parameters)
    MetadataRead(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ImageLoad(msg) => write!(f, "画像読み込みエラー: {}", msg),
            AppError::DirectoryScan(msg) => write!(f, "ディレクトリスキャンエラー: {}", msg),
            AppError::XmpRead(msg) => write!(f, "XMP読み取りエラー: {}", msg),
            AppError::XmpWrite(msg) => write!(f, "XMP書き込みエラー: {}", msg),
            AppError::MetadataRead(msg) => write!(f, "メタデータ読み取りエラー: {}", msg),
        }
    }
}

impl From<png::DecodingError> for AppError {
    fn from(err: png::DecodingError) -> Self {
        AppError::ImageLoad(format!("PNG decoding error: {}", err))
    }
}

impl std::error::Error for AppError {}

impl From<image::ImageError> for AppError {
    fn from(err: image::ImageError) -> Self {
        AppError::ImageLoad(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::DirectoryScan(err.to_string())
    }
}

/// Type alias for Results in this application.
pub type Result<T> = std::result::Result<T, AppError>;
