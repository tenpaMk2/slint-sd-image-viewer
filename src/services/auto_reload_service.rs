//! Service for handling auto-reload functionality.
//!
//! Provides directory monitoring and change detection for auto-reload feature.

use crate::config::SUPPORTED_IMAGE_EXTENSIONS;
use crate::error::NavigationError;
use crate::file_utils::PathExt;
use crate::services::NavigationService;
use log::{debug, warn};
use notify_debouncer_mini::{new_debouncer_opt, notify::RecursiveMode, Config};
use std::path::PathBuf;
use std::time::Duration;

/// Service for managing auto-reload checks.
pub struct AutoReloadService {
    navigation_service: NavigationService,
}

/// Handles debounced file system events.
fn handle_debounced_events<F>(
    events: Vec<notify_debouncer_mini::DebouncedEvent>,
    navigation_service: &NavigationService,
    on_change: &std::sync::Arc<F>,
) where
    F: Fn(PathBuf) + Send + Sync + 'static,
{
    if events.is_empty() {
        return;
    }

    // Filter out non-image files - we only care about supported image formats
    let file_events: Vec<_> = events
        .into_iter()
        .filter(|event| {
            event
                .path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext_str| {
                    SUPPORTED_IMAGE_EXTENSIONS.contains(&ext_str.to_lowercase().as_str())
                })
                .unwrap_or(false)
        })
        .collect();

    if file_events.is_empty() {
        return;
    }

    debug!("Debounced file system events: {} events", file_events.len());
    for event in &file_events {
        debug!("  - {:?} for {}", event.kind, event.path.format_for_log());
    }

    if let Err(e) = navigation_service.rescan_directory() {
        warn!("Failed to rescan directory: {}", e);
        return;
    }

    let path = match navigation_service.navigate_to_last() {
        Ok(path) => path,
        Err(e) => {
            warn!("Failed to navigate to last image: {}", e);
            return;
        }
    };

    debug!("Navigating to last image: {}", path.format_for_log());
    let on_change_clone = on_change.clone();
    let _ = slint::invoke_from_event_loop(move || {
        on_change_clone(path);
    });
}

impl AutoReloadService {
    /// Creates a new auto-reload service.
    pub fn new(navigation_service: NavigationService) -> Self {
        Self { navigation_service }
    }

    /// Starts watching the directory for changes with debouncing.
    ///
    /// Returns a `Debouncer` that monitors the directory for file changes.
    /// When changes are detected (after a 300ms debounce period), it rescans
    /// the directory and navigates to the last image.
    pub fn start_watching<F>(
        &self,
        state: std::sync::Arc<std::sync::Mutex<crate::state::NavigationState>>,
        on_change: F,
    ) -> Result<crate::state::AutoReloadDebouncer, NavigationError>
    where
        F: Fn(PathBuf) + Send + Sync + 'static,
    {
        // Get the current directory to watch
        let directory = {
            let state_lock = state.lock().map_err(|_| {
                NavigationError::DirectoryScanFailed("Failed to lock state".to_string())
            })?;
            state_lock.get_current_directory().ok_or_else(|| {
                NavigationError::DirectoryScanFailed("No directory selected".to_string())
            })?
        };

        let navigation_service = self.navigation_service.clone();
        let on_change = std::sync::Arc::new(on_change);

        // Create a debounced watcher with 300ms debounce period using PollWatcher backend
        let notify_config = notify_debouncer_mini::notify::Config::default()
            .with_poll_interval(Duration::from_secs(2));
        let debouncer_config = Config::default()
            .with_timeout(Duration::from_millis(500))
            .with_notify_config(notify_config);

        let mut debouncer = new_debouncer_opt::<_, notify_debouncer_mini::notify::PollWatcher>(
            debouncer_config,
            move |res: notify_debouncer_mini::DebounceEventResult| match res {
                Ok(events) => {
                    handle_debounced_events(events, &navigation_service, &on_change);
                }
                Err(error) => {
                    let error_msg = error.to_string();
                    if !error_msg.contains(".tmp") {
                        warn!("File watcher error: {}", error);
                    }
                }
            },
        )
        .map_err(|e| {
            NavigationError::DirectoryScanFailed(format!("Failed to create debouncer: {}", e))
        })?;

        // Start watching the directory (non-recursive) using PollWatcher backend
        debouncer
            .watcher()
            .watch(&directory, RecursiveMode::NonRecursive)
            .map_err(|e| {
                NavigationError::DirectoryScanFailed(format!("Failed to watch directory: {}", e))
            })?;

        Ok(debouncer)
    }

    /// Navigates to the last image without checking for changes.
    pub fn navigate_to_last(&self) -> Result<PathBuf, NavigationError> {
        self.navigation_service.navigate_to_last()
    }
}
