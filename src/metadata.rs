//! XMP metadata handling for image files.

use crate::error::{AppError, Result};
use std::path::Path;
use xmp_toolkit::{OpenFileOptions, XmpFile, XmpMeta, XmpValue};

/// Read XMP Rating from an image file.
///
/// Returns `Ok(Some(rating))` if rating exists (0-5),
/// `Ok(None)` if no rating is set,
/// `Err` if reading fails.
pub fn read_xmp_rating(path: &Path) -> Result<Option<u8>> {
    let mut xmp_file = XmpFile::new()
        .map_err(|e| AppError::XmpRead(format!("Failed to create XmpFile: {}", e)))?;

    // Open file for reading only
    xmp_file
        .open_file(
            path.to_str()
                .ok_or_else(|| AppError::XmpRead("Invalid UTF-8 in file path".to_string()))?,
            OpenFileOptions::default().only_xmp().for_read(),
        )
        .map_err(|e| AppError::XmpRead(format!("Failed to open file: {}", e)))?;

    // Get XMP metadata
    let result = match xmp_file.xmp() {
        Some(xmp_meta) => {
            // Try to get xmp:Rating property
            if let Some(rating_property) =
                xmp_meta.property("http://ns.adobe.com/xap/1.0/", "Rating")
            {
                match rating_property.value.parse::<u8>() {
                    Ok(rating) if rating <= 5 => Some(rating),
                    _ => None,
                }
            } else {
                None
            }
        }
        None => None,
    };

    // Close file
    xmp_file.close();

    Ok(result)
}

/// Write XMP Rating to an image file.
///
/// Rating must be in range 0-5.
/// Returns `Err` if writing fails or rating is out of range.
pub fn write_xmp_rating(path: &Path, rating: u8) -> Result<()> {
    // Validate rating range
    if rating > 5 {
        return Err(AppError::XmpWrite(format!(
            "Rating must be 0-5, got {}",
            rating
        )));
    }

    let mut xmp_file = XmpFile::new()
        .map_err(|e| AppError::XmpWrite(format!("Failed to create XmpFile: {}", e)))?;

    // Open file for update
    xmp_file
        .open_file(
            path.to_str()
                .ok_or_else(|| AppError::XmpWrite("Invalid UTF-8 in file path".to_string()))?,
            OpenFileOptions::default().only_xmp().for_update(),
        )
        .map_err(|e| AppError::XmpWrite(format!("Failed to open file for update: {}", e)))?;

    // Get existing XMP or create new one
    let mut xmp_meta = match xmp_file.xmp() {
        Some(xmp) => xmp,
        None => XmpMeta::new()
            .map_err(|e| AppError::XmpWrite(format!("Failed to create new XMP: {}", e)))?,
    };

    // Set xmp:Rating
    let rating_value = XmpValue::new(rating.to_string());
    xmp_meta
        .set_property("http://ns.adobe.com/xap/1.0/", "Rating", &rating_value)
        .map_err(|e| AppError::XmpWrite(format!("Failed to set Rating: {}", e)))?;

    // Put updated XMP back to file
    xmp_file
        .put_xmp(&xmp_meta)
        .map_err(|e| AppError::XmpWrite(format!("Failed to put XMP: {}", e)))?;

    // Close file (this writes the changes)
    xmp_file.close();

    Ok(())
}
