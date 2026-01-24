use slint::{Image, Rgb8Pixel, SharedPixelBuffer};
use std::error::Error;
use std::path::Path;

/// Load an image from a file path and convert it to a Slint Image
/// ブロッキングI/Oを別スレッドで実行してSlintイベントループをブロックしない
pub async fn load_image(path: &Path) -> Result<Image, Box<dyn Error>> {
    let path = path.to_path_buf();

    // spawn_local側でasync_compatでラップされているため、ここでは不要
    let image = tokio::task::spawn_blocking(
        move || -> Result<image::DynamicImage, Box<dyn Error + Send + Sync>> {
            let img = image::ImageReader::open(&path)
                .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?
                .with_guessed_format()
                .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?
                .decode()
                .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?;
            Ok(img)
        },
    )
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Image load error: {}", e))?;

    let buffer = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
        &image.to_rgb8().as_raw(),
        image.width(),
        image.height(),
    );

    Ok(Image::from_rgb8(buffer))
}
