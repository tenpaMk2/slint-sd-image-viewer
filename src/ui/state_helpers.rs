//! Helper functions to set multiple ViewerState properties in a grouped manner.
//!
//! Instead of calling individual setters like set_current_filename, set_file_size_formatted, etc.,
//! these functions group related properties together for better code organization and maintainability.

use log::error;
use slint::ComponentHandle;

/// Sets all file information properties at once.
///
/// Groups: current-filename, file-size-formatted, image-width, image-height,
/// file-created-date, file-modified-date
pub fn set_file_info(
    ui: &crate::AppWindow,
    filename: &str,
    file_size: &str,
    width: u32,
    height: u32,
    created_date: &str,
    modified_date: &str,
) {
    let viewer_state = ui.global::<crate::ViewerState>();
    viewer_state.set_current_filename(filename.into());
    viewer_state.set_file_size_formatted(file_size.into());
    viewer_state.set_image_width(width as i32);
    viewer_state.set_image_height(height as i32);
    viewer_state.set_file_created_date(created_date.into());
    viewer_state.set_file_modified_date(modified_date.into());
}

/// Sets all prompt-related properties at once.
///
/// Groups: positive-prompt, negative-prompt, sd-parameters
pub fn set_prompts_and_parameters(
    ui: &crate::AppWindow,
    positive: &str,
    negative: &str,
    parameters: Vec<(slint::SharedString, slint::SharedString)>,
) {
    let viewer_state = ui.global::<crate::ViewerState>();
    viewer_state.set_positive_prompt(positive.into());
    viewer_state.set_negative_prompt(negative.into());
    viewer_state.set_sd_parameters(slint::ModelRc::new(slint::VecModel::from(parameters)));
}

/// Clears all prompt-related properties.
///
/// Sets empty strings for prompts and empty array for parameters.
pub fn clear_prompts_and_parameters(ui: &crate::AppWindow) {
    set_prompts_and_parameters(ui, "", "", vec![]);
}

/// Sets an error message in the UI with a prefix.
///
/// Logs the error and updates the ViewerState error-message property.
pub fn set_error_with_prefix(ui: &crate::AppWindow, prefix: &str, error: String) {
    let error_message = format!("{}: {}", prefix, error);
    error!("{}", error_message);
    ui.global::<crate::ViewerState>()
        .set_error_message(error_message.into());
}

/// Sets all rating-related properties at once.
///
/// Groups: current-rating, rating-in-progress
pub fn set_rating_info(ui: &crate::AppWindow, rating: i32, in_progress: bool) {
    let viewer_state = ui.global::<crate::ViewerState>();
    viewer_state.set_current_rating(rating);
    viewer_state.set_rating_in_progress(in_progress);
}

/// Sets all navigation-related properties at once.
///
/// Groups: current-index, total-index, auto-reload-active
pub fn set_navigation_info(
    ui: &crate::AppWindow,
    current_index: i32,
    total_index: i32,
    auto_reload_active: bool,
) {
    let viewer_state = ui.global::<crate::ViewerState>();
    viewer_state.set_current_index(current_index);
    viewer_state.set_total_index(total_index);
    viewer_state.set_auto_reload_active(auto_reload_active);
}
