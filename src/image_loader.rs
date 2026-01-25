use crate::error::Result;
use crate::metadata::{self, SdParameters};
use image::ImageFormat;
use log::error;
use slint::{Image, Rgb8Pixel, SharedPixelBuffer};
use std::io::Cursor;
use std::path::Path;

/// Loaded image data with metadata
#[derive(Clone)]
pub struct LoadedImageData {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub rating: Option<u8>,
    pub sd_parameters: Option<SdParameters>,
}

/// Load image and metadata from a file path.
/// Uses image crate for decoding all formats.
/// - PNG: Single file I/O with metadata extracted from the same bytes
/// - Other formats: Image data from memory, metadata from separate file I/O
pub fn load_image_with_metadata(path: &Path) -> Result<LoadedImageData> {
    // Single file read into memory
    let file_bytes = std::fs::read(path).map_err(|e| {
        error!("Failed to read file {:?}: {}", path, e);
        e
    })?;

    // Detect format using image crate
    let reader = image::ImageReader::new(Cursor::new(&file_bytes))
        .with_guessed_format()
        .map_err(|e| {
            error!("Failed to guess image format for {:?}: {}", path, e);
            e
        })?;

    let format = reader.format().ok_or_else(|| {
        error!("Unsupported or unrecognized image format for {:?}", path);
        crate::error::AppError::ImageLoad("Unsupported or unrecognized image format".to_string())
    })?;

    // Decode image using image crate
    let img = reader.decode().map_err(|e| {
        error!("Failed to decode image {:?}: {}", path, e);
        e
    })?;

    let rgb8 = img.to_rgb8();
    let width = rgb8.width();
    let height = rgb8.height();
    let data = rgb8.into_raw();

    // Extract metadata based on format
    let (rating, sd_parameters) = match format {
        ImageFormat::Png => {
            // PNG: Use png crate to extract metadata from same bytes (optimized)
            let decoder = png::Decoder::new(Cursor::new(&file_bytes));
            let reader = decoder.read_info().map_err(|e| {
                error!("Failed to read PNG info for {:?}: {}", path, e);
                e
            })?;

            let info = reader.info().clone();

            // Extract rating from XMP
            let rating = metadata::extract_xmp_rdf_from_info(&info)
                .ok()
                .flatten()
                .and_then(|xmp_rdf| metadata::parse_xmp_rating_from_rdf(&xmp_rdf));

            // Extract SD parameters from tEXt chunk
            let sd_parameters = metadata::extract_sd_parameters_from_info(&info)
                .ok()
                .flatten()
                .and_then(|param_str| SdParameters::parse(&param_str).ok());

            (rating, sd_parameters)
        }
        ImageFormat::Jpeg | ImageFormat::WebP => {
            // JPEG/WebP: Read XMP rating from file (additional I/O)
            let rating = metadata::read_xmp_rating(path).ok().flatten();
            // SD parameters are PNG-specific, skip for other formats
            (rating, None)
        }
        _ => {
            // Other formats: Read XMP rating from file (additional I/O)
            let rating = metadata::read_xmp_rating(path).ok().flatten();
            // SD parameters are PNG-specific, skip for other formats
            (rating, None)
        }
    };

    Ok(LoadedImageData {
        data,
        width,
        height,
        rating,
        sd_parameters,
    })
}

/// Convert RGB8 data to Slint Image (UIスレッドで軽い処理のみ)
pub fn create_slint_image(data: Vec<u8>, width: u32, height: u32) -> Image {
    let buffer = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(&data, width, height);
    Image::from_rgb8(buffer)
}
