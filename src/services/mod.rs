//! Service layer for business logic.
//!
//! Separates business logic from UI handlers for better testability and maintainability.

pub mod auto_reload_service;
pub mod clipboard_service;
pub mod color_management_service;
pub mod display_profile_service;
pub mod navigation_service;
pub mod rating_service;

pub use auto_reload_service::AutoReloadService;
pub use clipboard_service::ClipboardService;
pub use color_management_service::default_color_management_service;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use display_profile_service::DisplayProfileService;
pub use navigation_service::NavigationService;
pub use rating_service::RatingService;
