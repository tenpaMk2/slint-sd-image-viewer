//! Unified error types for the image viewer application.

use std::fmt;

/// Application-specific errors.
#[derive(Debug)]
pub enum AppError {
    /// Error loading or decoding an image file
    ImageLoad(String),
    /// Error scanning directory for image files
    DirectoryScan(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ImageLoad(msg) => write!(f, "画像読み込みエラー: {}", msg),
            AppError::DirectoryScan(msg) => write!(f, "ディレクトリスキャンエラー: {}", msg),
        }
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
