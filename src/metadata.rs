//! XMP metadata handling for image files.

use crate::error::{AppError, Result};
use std::path::Path;
use xmp_toolkit::{OpenFileOptions, XmpFile, XmpMeta, XmpValue};

const XMP_NAMESPACE: &str = "http://ns.adobe.com/xap/1.0/";
const RATING_PROPERTY: &str = "Rating";
const MAX_RATING: u8 = 5;

/// Converts a path to a string, returning an error if the path is not valid UTF-8.
fn path_to_str(path: &Path) -> Result<&str> {
    path.to_str()
        .ok_or_else(|| AppError::XmpRead("Invalid UTF-8 in file path".to_string()))
}

/// Opens an XMP file for reading.
fn open_xmp_for_read(path: &Path) -> Result<XmpFile> {
    let mut xmp_file = XmpFile::new()
        .map_err(|e| AppError::XmpRead(format!("Failed to create XmpFile: {}", e)))?;

    xmp_file
        .open_file(
            path_to_str(path)?,
            OpenFileOptions::default().only_xmp().for_read(),
        )
        .map_err(|e| AppError::XmpRead(format!("Failed to open file: {}", e)))?;

    Ok(xmp_file)
}

/// Opens an XMP file for update.
fn open_xmp_for_update(path: &Path) -> Result<XmpFile> {
    let mut xmp_file = XmpFile::new()
        .map_err(|e| AppError::XmpWrite(format!("Failed to create XmpFile: {}", e)))?;

    xmp_file
        .open_file(
            path_to_str(path).map_err(|e| match e {
                AppError::XmpRead(msg) => AppError::XmpWrite(msg),
                other => other,
            })?,
            OpenFileOptions::default().only_xmp().for_update(),
        )
        .map_err(|e| AppError::XmpWrite(format!("Failed to open file for update: {}", e)))?;

    Ok(xmp_file)
}

/// Extracts and validates the rating value from XMP metadata.
fn extract_rating_from_xmp(xmp_meta: XmpMeta) -> Option<u8> {
    let rating_property = xmp_meta.property(XMP_NAMESPACE, RATING_PROPERTY)?;
    let rating = rating_property.value.parse::<u8>().ok()?;

    if rating <= MAX_RATING {
        Some(rating)
    } else {
        None
    }
}

/// Read XMP Rating from an image file.
///
/// Returns `Ok(Some(rating))` if rating exists (0-5),
/// `Ok(None)` if no rating is set,
/// `Err` if reading fails.
pub fn read_xmp_rating(path: &Path) -> Result<Option<u8>> {
    let mut xmp_file = open_xmp_for_read(path)?;
    let rating = xmp_file.xmp().and_then(extract_rating_from_xmp);
    xmp_file.close();
    Ok(rating)
}

/// Validates the rating value.
fn validate_rating(rating: u8) -> Result<()> {
    if rating > MAX_RATING {
        Err(AppError::XmpWrite(format!(
            "Rating must be 0-{}, got {}",
            MAX_RATING, rating
        )))
    } else {
        Ok(())
    }
}

/// Gets or creates XMP metadata from an XMP file.
fn get_or_create_xmp_meta(xmp_file: &mut XmpFile) -> Result<XmpMeta> {
    match xmp_file.xmp() {
        Some(xmp) => Ok(xmp),
        None => XmpMeta::new()
            .map_err(|e| AppError::XmpWrite(format!("Failed to create new XMP: {}", e))),
    }
}

/// Sets the rating property in XMP metadata.
fn set_rating_property(xmp_meta: &mut XmpMeta, rating: u8) -> Result<()> {
    let rating_value = XmpValue::new(rating.to_string());
    xmp_meta
        .set_property(XMP_NAMESPACE, RATING_PROPERTY, &rating_value)
        .map_err(|e| AppError::XmpWrite(format!("Failed to set Rating: {}", e)))
}

/// Writes XMP metadata back to the file.
fn write_xmp_to_file(xmp_file: &mut XmpFile, xmp_meta: &XmpMeta) -> Result<()> {
    xmp_file
        .put_xmp(xmp_meta)
        .map_err(|e| AppError::XmpWrite(format!("Failed to put XMP: {}", e)))
}

/// Write XMP Rating to an image file.
///
/// Rating must be in range 0-5.
/// Returns `Err` if writing fails or rating is out of range.
pub fn write_xmp_rating(path: &Path, rating: u8) -> Result<()> {
    validate_rating(rating)?;

    let mut xmp_file = open_xmp_for_update(path)?;
    let mut xmp_meta = get_or_create_xmp_meta(&mut xmp_file)?;
    set_rating_property(&mut xmp_meta, rating)?;
    write_xmp_to_file(&mut xmp_file, &xmp_meta)?;
    xmp_file.close();

    Ok(())
}
