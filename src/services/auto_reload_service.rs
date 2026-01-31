//! Service for handling auto-reload functionality.
//!
//! Provides directory monitoring and change detection for auto-reload feature.

use crate::error::NavigationError;
use crate::services::NavigationService;
use log::warn;
use notify::{Config, Event, PollWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

/// Service for managing auto-reload checks.
pub struct AutoReloadService {
    navigation_service: NavigationService,
}

impl AutoReloadService {
    /// Creates a new auto-reload service.
    pub fn new(navigation_service: NavigationService) -> Self {
        Self { navigation_service }
    }

    /// Starts watching the directory for changes.
    ///
    /// Returns a `PollWatcher` that monitors the directory for file changes.
    /// When changes are detected, it rescans the directory and navigates to the last image.
    pub fn start_watching<F>(
        &self,
        state: std::sync::Arc<std::sync::Mutex<crate::state::NavigationState>>,
        on_change: F,
    ) -> Result<PollWatcher, NavigationError>
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

        let (tx, rx) = channel::<notify::Result<Event>>();

        // Create a PollWatcher with 2-second polling interval
        let config = Config::default().with_poll_interval(Duration::from_secs(2));
        let navigation_service = self.navigation_service.clone();

        let mut watcher = PollWatcher::new(
            move |res: notify::Result<Event>| {
                if tx.send(res).is_err() {
                    warn!("Failed to send file system event");
                }
            },
            config,
        )
        .map_err(|e| {
            NavigationError::DirectoryScanFailed(format!("Failed to create watcher: {}", e))
        })?;

        // Start watching the directory (non-recursive)
        watcher
            .watch(&directory, RecursiveMode::NonRecursive)
            .map_err(|e| {
                NavigationError::DirectoryScanFailed(format!("Failed to watch directory: {}", e))
            })?;

        // Spawn a thread to handle events
        let on_change = std::sync::Arc::new(on_change);
        let on_change_for_thread = on_change.clone();
        std::thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(Ok(event)) => {
                        use log::debug;
                        debug!("File system event detected: {:?}", event);

                        match navigation_service.rescan_directory() {
                            Ok(_) => match navigation_service.navigate_to_last() {
                                Ok(path) => {
                                    debug!("Navigating to last image: {:?}", path);
                                    let on_change_clone = on_change_for_thread.clone();
                                    let _ = slint::invoke_from_event_loop(move || {
                                        on_change_clone(path);
                                    });
                                }
                                Err(e) => {
                                    warn!("Failed to navigate to last image: {}", e);
                                }
                            },
                            Err(e) => {
                                warn!("Failed to rescan directory: {}", e);
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        // Silently ignore errors for temporary files
                        let error_msg = e.to_string();
                        if !error_msg.contains(".tmp") {
                            warn!("File watcher error: {}", e);
                        }
                    }
                    Err(_) => {
                        // Channel closed, exit thread
                        break;
                    }
                }
            }
        });

        Ok(watcher)
    }

    /// Navigates to the last image without checking for changes.
    pub fn navigate_to_last(&self) -> Result<PathBuf, NavigationError> {
        self.navigation_service.navigate_to_last()
    }
}
