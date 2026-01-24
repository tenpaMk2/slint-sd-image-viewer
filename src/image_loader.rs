use slint::{Image, Rgb8Pixel, SharedPixelBuffer};
use std::error::Error;
use std::path::Path;

/// Load an image from a file path and convert it to a Slint Image
pub async fn load_image(path: &Path) -> Result<Image, Box<dyn Error>> {
    let image = image::ImageReader::open(path)
        .map_err(|e| format!("Failed to open file: {}", e))?
        .with_guessed_format()
        .map_err(|e| format!("Failed to detect image format: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let buffer = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
        &image.to_rgb8().as_raw(),
        image.width(),
        image.height(),
    );

    Ok(Image::from_rgb8(buffer))
}
