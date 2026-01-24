use slint::{Image, Rgb8Pixel, SharedPixelBuffer};
use std::error::Error;
use std::path::Path;

/// Load an image from a file path and return RGB8 data ready for Slint
/// 別スレッドで実行する同期関数（重い処理を全て含む）
pub fn load_image_blocking(
    path: &Path,
) -> Result<(Vec<u8>, u32, u32), Box<dyn Error + Send + Sync>> {
    let img = image::ImageReader::open(path)
        .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?
        .with_guessed_format()
        .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?
        .decode()
        .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?;

    // 重いRGB8変換も別スレッドで実行
    let rgb8 = img.to_rgb8();
    let width = rgb8.width();
    let height = rgb8.height();
    let data = rgb8.into_raw();

    Ok((data, width, height))
}

/// Convert RGB8 data to Slint Image (UIスレッドで軽い処理のみ)
pub fn create_slint_image(data: Vec<u8>, width: u32, height: u32) -> Image {
    let buffer = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(&data, width, height);
    Image::from_rgb8(buffer)
}
