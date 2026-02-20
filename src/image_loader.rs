use crate::error::Result;
use crate::metadata::{self, SdParameters};
#[cfg(target_os = "macos")]
use image::ImageDecoder;
use image::ImageFormat;
use log::error;
use slint::{Image, Rgb8Pixel, SharedPixelBuffer};
use std::io::Cursor;
use std::path::Path;

#[cfg(target_os = "macos")]
use crate::services::DisplayProfileService;
#[cfg(target_os = "macos")]
use lcms2::{Flags, Intent, PixelFormat, Profile, Transform};

/// Loaded image data with metadata
#[derive(Clone)]
pub struct LoadedImageData {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub rating: Option<u8>,
    pub sd_parameters: Option<SdParameters>,
    pub file_name: String,
    pub file_size_formatted: String,
    pub created_date: String,
    pub modified_date: String,
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
    #[cfg(target_os = "macos")]
    let (img, image_icc_profile) = {
        let mut decoder = reader.into_decoder().map_err(|e| {
            error!("Failed to create decoder for {:?}: {}", path, e);
            e
        })?;

        let image_icc_profile = decoder.icc_profile().map_err(|e| {
            error!("Failed to read ICC profile for {:?}: {}", path, e);
            e
        })?;

        let img = image::DynamicImage::from_decoder(decoder).map_err(|e| {
            error!("Failed to decode image {:?}: {}", path, e);
            e
        })?;

        (img, image_icc_profile)
    };

    #[cfg(not(target_os = "macos"))]
    let img = reader.decode().map_err(|e| {
        error!("Failed to decode image {:?}: {}", path, e);
        e
    })?;

    let rgb8 = img.to_rgb8();
    let width = rgb8.width();
    let height = rgb8.height();
    let mut data = rgb8.into_raw();

    #[cfg(target_os = "macos")]
    {
        if let Err(err) = apply_display_color_management(&mut data, image_icc_profile.as_deref()) {
            error!(
                "Color management failed for {:?}, fallback to uncorrected pixels: {}",
                path, err
            );
        }
    }

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

    // Extract file name
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    // Get file size and format with commas
    let file_size_bytes = file_bytes.len() as u64;
    let file_size_formatted = format_file_size(file_size_bytes);

    // Get file timestamps
    let (created_date, modified_date) = if let Ok(metadata) = std::fs::metadata(path) {
        let created = metadata
            .created()
            .ok()
            .map(|time| {
                let datetime: chrono::DateTime<chrono::Local> = time.into();
                datetime.format("%Y-%m-%d %H:%M:%S").to_string()
            })
            .unwrap_or_else(|| "N/A".to_string());

        let modified = metadata
            .modified()
            .ok()
            .map(|time| {
                let datetime: chrono::DateTime<chrono::Local> = time.into();
                datetime.format("%Y-%m-%d %H:%M:%S").to_string()
            })
            .unwrap_or_else(|| "N/A".to_string());

        (created, modified)
    } else {
        ("N/A".to_string(), "N/A".to_string())
    };

    Ok(LoadedImageData {
        data,
        width,
        height,
        rating,
        sd_parameters,
        file_name,
        file_size_formatted,
        created_date,
        modified_date,
    })
}

#[cfg(target_os = "macos")]
fn apply_display_color_management(
    rgb_data: &mut [u8],
    image_icc_profile: Option<&[u8]>,
) -> std::result::Result<(), String> {
    let display_icc_profile = DisplayProfileService::new()
        .load_first_display_icc_profile()
        .map_err(|e| format!("Failed to load display ICC profile: {}", e))?;

    let src_profile = match image_icc_profile {
        Some(icc) if !icc.is_empty() => Profile::new_icc(icc)
            .map_err(|e| format!("Failed to parse image ICC profile: {}", e))?,
        _ => Profile::new_srgb(),
    };

    let dst_profile = Profile::new_icc(&display_icc_profile)
        .map_err(|e| format!("Failed to parse display ICC profile: {}", e))?;

    let transform = Transform::new_flags(
        &src_profile,
        PixelFormat::RGB_8,
        &dst_profile,
        PixelFormat::RGB_8,
        Intent::RelativeColorimetric,
        Flags::BLACKPOINT_COMPENSATION,
    )
    .map_err(|e| format!("Failed to create ICC transform: {}", e))?;

    let input = rgb_data.to_vec();
    transform.transform_pixels(&input, rgb_data);

    Ok(())
}

/// Format file size with thousand separators
fn format_file_size(size: u64) -> String {
    let size_str = size.to_string();
    let mut result = String::new();
    let chars: Vec<char> = size_str.chars().collect();

    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }

    format!("{} bytes", result)
}

/// Convert RGB8 data to Slint Image (UIスレッドで軽い処理のみ)
pub fn create_slint_image(data: &[u8], width: u32, height: u32) -> Image {
    let buffer = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(data, width, height);
    Image::from_rgb8(buffer)
}
