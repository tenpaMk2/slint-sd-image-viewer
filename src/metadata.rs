//! XMP metadata handling for image files.

use crate::error::{AppError, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use xmp_toolkit::{OpenFileOptions, XmpFile, XmpMeta, XmpValue};

const XMP_NAMESPACE: &str = "http://ns.adobe.com/xap/1.0/";
const RATING_PROPERTY: &str = "Rating";
const MAX_RATING: u8 = 5;

// 正規表現を一度だけコンパイル（起動時エラーで早期発見）
static TAG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\(([^:]+):([0-9]+(?:\.[0-9]+)?)\)").expect("Invalid regex pattern for SD tags")
});

static FIELD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(Steps|Sampler|Schedule type|CFG scale|Seed|Size|Model|Denoising strength|Clip skip):\s*([^,]+)")
        .expect("Invalid regex pattern for SD fields")
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdTag {
    pub name: String,
    pub weight: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdParameters {
    pub positive_sd_tags: Vec<SdTag>,
    pub negative_sd_tags: Vec<SdTag>,
    pub steps: Option<String>,
    pub sampler: Option<String>,
    pub schedule_type: Option<String>,
    pub cfg_scale: Option<String>,
    pub seed: Option<String>,
    pub size: Option<String>,
    pub model: Option<String>,
    pub denoising_strength: Option<String>,
    pub clip_skip: Option<String>,
    pub raw: String,
}

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

impl SdParameters {
    /// SDタグ文字列をパースする
    fn parse_sd_tags(s: &str) -> Vec<SdTag> {
        s.split(',')
            .map(|piece| piece.trim())
            .filter_map(|raw_tag| {
                if raw_tag.is_empty() {
                    return None; // 空文字列はスキップ
                }

                // 正規表現マッチング（安全）
                if let Some(caps) = TAG_REGEX.captures(raw_tag) {
                    // キャプチャグループの安全な取得
                    let name = caps.get(1)?.as_str().trim();
                    let weight_str = caps.get(2)?.as_str().trim();

                    if name.is_empty() {
                        return None; // 空のタグ名はスキップ
                    }

                    let weight = weight_str.parse::<f32>().ok();

                    Some(SdTag {
                        name: name.to_string(),
                        weight,
                    })
                } else {
                    // 通常タグ
                    Some(SdTag {
                        name: raw_tag.to_string(),
                        weight: None,
                    })
                }
            })
            .collect()
    }

    /// 全フィールドの値を一括抽出
    fn extract_all_fields(
        text: &str,
    ) -> (
        Option<String>, // steps
        Option<String>, // sampler
        Option<String>, // schedule_type
        Option<String>, // cfg_scale
        Option<String>, // seed
        Option<String>, // size
        Option<String>, // model
        Option<String>, // denoising_strength
        Option<String>, // clip_skip
    ) {
        let mut steps = None;
        let mut sampler = None;
        let mut schedule_type = None;
        let mut cfg_scale = None;
        let mut seed = None;
        let mut size = None;
        let mut model = None;
        let mut denoising_strength = None;
        let mut clip_skip = None;

        // 1回のスキャンで全フィールドを取得
        for cap in FIELD_REGEX.captures_iter(text) {
            if let (Some(key_match), Some(value_match)) = (cap.get(1), cap.get(2)) {
                let key = key_match.as_str();
                let value = value_match.as_str().trim();

                if value.is_empty() {
                    continue;
                }

                match key {
                    "Steps" => steps = Some(value.to_string()),
                    "Sampler" => sampler = Some(value.to_string()),
                    "Schedule type" => schedule_type = Some(value.to_string()),
                    "CFG scale" => cfg_scale = Some(value.to_string()),
                    "Seed" => seed = Some(value.to_string()),
                    "Size" => size = Some(value.to_string()),
                    "Model" => model = Some(value.to_string()),
                    "Denoising strength" => denoising_strength = Some(value.to_string()),
                    "Clip skip" => clip_skip = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        (
            steps,
            sampler,
            schedule_type,
            cfg_scale,
            seed,
            size,
            model,
            denoising_strength,
            clip_skip,
        )
    }

    /// SD Parameters文字列をパースする
    pub fn parse(parameter: &str) -> Result<SdParameters> {
        if parameter.trim().is_empty() {
            return Err(AppError::MetadataRead("Empty parameter string".to_string()));
        }

        // "Negative prompt:" で分割
        let pp_separated: Vec<&str> = parameter.splitn(2, "\nNegative prompt:").collect();
        if pp_separated.len() != 2 {
            return Err(AppError::MetadataRead(
                "\"Negative prompt:\" section not found".to_string(),
            ));
        }

        // "Steps:" で分割
        let np_separated: Vec<&str> = pp_separated[1].splitn(2, "\nSteps:").collect();
        if np_separated.len() != 2 {
            return Err(AppError::MetadataRead(
                "\"Steps:\" section not found".to_string(),
            ));
        }

        let positive_sd_tags = Self::parse_sd_tags(pp_separated[0]);
        let negative_sd_tags = Self::parse_sd_tags(np_separated[0]);

        // フィールド部分から必要な値を一括抽出
        let fields_section = &format!("Steps:{}", np_separated[1]);
        let (
            steps,
            sampler,
            schedule_type,
            cfg_scale,
            seed,
            size,
            model,
            denoising_strength,
            clip_skip,
        ) = Self::extract_all_fields(fields_section);

        Ok(SdParameters {
            positive_sd_tags,
            negative_sd_tags,
            steps,
            sampler,
            schedule_type,
            cfg_scale,
            seed,
            size,
            model,
            denoising_strength,
            clip_skip,
            raw: parameter.to_string(),
        })
    }
}

/// Parses XMP RDF string and extracts rating.
///
/// Returns `Some(rating)` if rating exists and is valid (0-5),
/// `None` if rating doesn't exist or is invalid.
pub fn parse_xmp_rating_from_rdf(xmp_rdf: &str) -> Option<u8> {
    XmpMeta::from_str_with_options(xmp_rdf, Default::default())
        .ok()
        .and_then(extract_rating_from_xmp)
}

/// Extracts XMP RDF string from PNG Info's iTXt chunks.
///
/// Searches for "XML:com.adobe.xmp" or "xmp" keyword in iTXt chunks.
/// Decompresses if necessary.
pub fn extract_xmp_rdf_from_info(info: &png::Info) -> Result<Option<String>> {
    for itxt_chunk in &info.utf8_text {
        if itxt_chunk.keyword == "XML:com.adobe.xmp" || itxt_chunk.keyword == "xmp" {
            let mut chunk = itxt_chunk.clone();
            if chunk.compressed {
                chunk.decompress_text().map_err(|e| {
                    AppError::MetadataRead(format!("Failed to decompress iTXt: {}", e))
                })?;
            }
            let xmp_rdf = chunk
                .get_text()
                .map_err(|e| AppError::MetadataRead(format!("Failed to get iTXt text: {}", e)))?;
            return Ok(Some(xmp_rdf));
        }
    }
    Ok(None)
}

/// Extracts SD parameters string from PNG Info's tEXt chunks.
///
/// Searches for "parameters" keyword in tEXt chunks.
pub fn extract_sd_parameters_from_info(info: &png::Info) -> Result<Option<String>> {
    for chunk in &info.uncompressed_latin1_text {
        if chunk.keyword == "parameters" {
            return Ok(Some(chunk.text.clone()));
        }
    }
    Ok(None)
}
