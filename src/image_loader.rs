use crate::error::{AppError, Result};
use crate::metadata::{self, SdParameters};
use crate::services::default_color_management_service;
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
    let file_bytes = read_file_bytes(path)?;
    let reader = create_image_reader(&file_bytes, path)?;
    let format = detect_format(&reader, path)?;

    let (img, image_icc_profile) = decode_image_and_icc(reader, path)?;
    let (mut data, width, height) = convert_to_rgb8(img);
    apply_color_management(path, &mut data, image_icc_profile.as_deref());

    let (rating, sd_parameters) = extract_metadata(path, &file_bytes, format)?;
    let (file_name, file_size_formatted, created_date, modified_date) =
        build_file_info(path, &file_bytes);

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

/// 画像ファイルをメモリへ読み込む。
fn read_file_bytes(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|e| {
        error!("Failed to read file {:?}: {}", path, e);
        e.into()
    })
}

/// フォーマット推定済みのImageReaderを生成する。
fn create_image_reader<'a>(
    file_bytes: &'a [u8],
    path: &Path,
) -> Result<image::ImageReader<Cursor<&'a [u8]>>> {
    image::ImageReader::new(Cursor::new(file_bytes))
        .with_guessed_format()
        .map_err(|e| {
            error!("Failed to guess image format for {:?}: {}", path, e);
            e.into()
        })
}

/// 画像フォーマットを検出する。
fn detect_format(reader: &image::ImageReader<Cursor<&[u8]>>, path: &Path) -> Result<ImageFormat> {
    reader.format().ok_or_else(|| {
        error!("Unsupported or unrecognized image format for {:?}", path);
        AppError::ImageLoad("Unsupported or unrecognized image format".to_string())
    })
}

/// 画像をデコードし、取得可能ならICCプロファイルも返す。
fn decode_image_and_icc(
    reader: image::ImageReader<Cursor<&[u8]>>,
    path: &Path,
) -> Result<(image::DynamicImage, Option<Vec<u8>>)> {
    use image::ImageDecoder;

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

    Ok((img, image_icc_profile))
}

/// DynamicImageをRGB8生配列へ変換する。
fn convert_to_rgb8(img: image::DynamicImage) -> (Vec<u8>, u32, u32) {
    let rgb8 = img.to_rgb8();
    let width = rgb8.width();
    let height = rgb8.height();
    (rgb8.into_raw(), width, height)
}

/// 色管理サービスを適用する。
fn apply_color_management(path: &Path, rgb_data: &mut [u8], image_icc_profile: Option<&[u8]>) {
    if let Err(err) = default_color_management_service().apply_to_rgb8(rgb_data, image_icc_profile)
    {
        error!(
            "Color management failed for {:?}, fallback to uncorrected pixels: {}",
            path, err
        );
    }
}

/// 画像メタデータを抽出する。
fn extract_metadata(
    path: &Path,
    file_bytes: &[u8],
    format: ImageFormat,
) -> Result<(Option<u8>, Option<SdParameters>)> {
    match format {
        ImageFormat::Png => {
            let decoder = png::Decoder::new(Cursor::new(file_bytes));
            let reader = decoder.read_info().map_err(|e| {
                error!("Failed to read PNG info for {:?}: {}", path, e);
                e
            })?;

            let info = reader.info().clone();

            let rating = metadata::extract_xmp_rdf_from_info(&info)
                .ok()
                .flatten()
                .and_then(|xmp_rdf| metadata::parse_xmp_rating_from_rdf(&xmp_rdf));

            let sd_parameters = metadata::extract_sd_parameters_from_info(&info)
                .ok()
                .flatten()
                .and_then(|param_str| SdParameters::parse(&param_str).ok());

            Ok((rating, sd_parameters))
        }
        _ => {
            let rating = metadata::read_xmp_rating(path).ok().flatten();
            Ok((rating, None))
        }
    }
}

/// 表示用のファイル情報を組み立てる。
fn build_file_info(path: &Path, file_bytes: &[u8]) -> (String, String, String, String) {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let file_size_bytes = file_bytes.len() as u64;
    let file_size_formatted = format_file_size(file_size_bytes);

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

    (file_name, file_size_formatted, created_date, modified_date)
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
