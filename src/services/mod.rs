//! Service layer for business logic.
//!
//! Separates business logic from UI handlers for better testability and maintainability.

pub mod auto_reload_service;
pub mod navigation_service;
pub mod rating_service;

pub use auto_reload_service::AutoReloadService;
pub use navigation_service::NavigationService;
pub use rating_service::RatingService;
