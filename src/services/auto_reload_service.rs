//! Service for handling auto-reload functionality.
//!
//! Provides directory monitoring and change detection for auto-reload feature.

use crate::error::NavigationError;
use crate::services::NavigationService;
use std::path::PathBuf;

/// Result of a reload check operation.
#[derive(Debug)]
pub struct ReloadCheckResult {
    /// Whether new images were detected.
    pub has_changes: bool,
    /// The path to the newly navigated image, if navigation occurred.
    pub new_image_path: Option<PathBuf>,
}

/// Service for managing auto-reload checks.
pub struct AutoReloadService {
    navigation_service: NavigationService,
}

impl AutoReloadService {
    /// Creates a new auto-reload service.
    pub fn new(navigation_service: NavigationService) -> Self {
        Self { navigation_service }
    }

    /// Performs a single reload check.
    ///
    /// Rescans the directory and navigates to the last image if new images are detected.
    pub fn check_for_updates(&self) -> Result<ReloadCheckResult, NavigationError> {
        let old_count = self.navigation_service.image_count();
        let new_count = self.navigation_service.rescan_directory()?;

        if old_count == new_count {
            return Ok(ReloadCheckResult {
                has_changes: false,
                new_image_path: None,
            });
        }

        let path = self.navigation_service.navigate_to_last()?;
        Ok(ReloadCheckResult {
            has_changes: true,
            new_image_path: Some(path),
        })
    }

    /// Navigates to the last image without checking for changes.
    pub fn navigate_to_last(&self) -> Result<PathBuf, NavigationError> {
        self.navigation_service.navigate_to_last()
    }
}
