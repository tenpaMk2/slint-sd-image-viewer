use std::env;
use std::fs::File;
use std::fs;
#[cfg(target_os = "macos")]
use std::io::BufWriter;
#[cfg(target_os = "macos")]
use std::path::Path;
use std::path::PathBuf;

#[cfg(target_os = "macos")]
use icns::{IconFamily, Image as IcnsImage, PixelFormat};
#[cfg(target_os = "windows")]
use ico::{IconDir, IconDirEntry, IconImage, ResourceType};
use image::imageops::FilterType;

const SOURCE_ICON_PATH: &str = "bundle/icon.png";
#[cfg(target_os = "macos")]
const GENERATED_ICON_PATH: &str = "target/generated/icon.icns";
#[cfg(target_os = "windows")]
const GENERATED_WINDOWS_ICON_PATH: &str = "target/generated/icon.ico";
#[cfg(target_os = "macos")]
const MACOS_ICON_SIZES: &[u32] = &[16, 32, 64, 128, 256, 512, 1024];
#[cfg(target_os = "windows")]
const WINDOWS_ICON_SIZES: &[u32] = &[16, 20, 24, 32, 40, 48, 64, 128, 256];

fn main() {
    println!("cargo:rerun-if-changed={SOURCE_ICON_PATH}");

    #[cfg(target_os = "macos")]
    generate_macos_icon().expect("Failed to generate macOS app icon");

    #[cfg(target_os = "windows")]
    {
        generate_windows_icon().expect("Failed to generate Windows app icon");
        embed_windows_exe_icon().expect("Failed to embed Windows app icon");
    }

    slint_build::compile("ui/app-window.slint").expect("Slint build failed");
}

#[cfg(target_os = "macos")]
fn generate_macos_icon() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let source_png = manifest_dir.join(SOURCE_ICON_PATH);
    if !source_png.exists() {
        println!(
            "cargo:warning=Skipping icon generation because {SOURCE_ICON_PATH} was not found."
        );
        return Ok(());
    }

    let source_image = image::open(&source_png)?;
    let output_icns = manifest_dir.join(GENERATED_ICON_PATH);
    write_macos_icns(&source_image, &output_icns)?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn write_macos_icns(
    source_image: &image::DynamicImage,
    output_icns: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut icon_family = IconFamily::new();

    for size in MACOS_ICON_SIZES {
        let resized = source_image
            .resize_exact(*size, *size, FilterType::Lanczos3)
            .to_rgba8();
        let icon_image = IcnsImage::from_data(PixelFormat::RGBA, *size, *size, resized.into_raw())?;
        icon_family.add_icon(&icon_image)?;
    }

    if let Some(parent) = output_icns.parent() {
        fs::create_dir_all(parent)?;
    }
    let writer = BufWriter::new(File::create(output_icns)?);
    icon_family.write(writer)?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn generate_windows_icon() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let source_png = manifest_dir.join(SOURCE_ICON_PATH);
    if !source_png.exists() {
        println!(
            "cargo:warning=Skipping icon generation because {SOURCE_ICON_PATH} was not found."
        );
        return Ok(());
    }

    let source_image = image::open(&source_png)?;
    let output_ico = manifest_dir.join(GENERATED_WINDOWS_ICON_PATH);
    let mut icon_dir = IconDir::new(ResourceType::Icon);

    for size in WINDOWS_ICON_SIZES {
        let resized = source_image
            .resize_exact(*size, *size, FilterType::Lanczos3)
            .to_rgba8();
        let icon_image = IconImage::from_rgba_data(*size, *size, resized.into_raw());
        icon_dir.add_entry(IconDirEntry::encode(&icon_image)?);
    }

    if let Some(parent) = output_ico.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = File::create(output_ico)?;
    icon_dir.write(file)?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn embed_windows_exe_icon() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let icon_path = manifest_dir.join(GENERATED_WINDOWS_ICON_PATH);
    if !icon_path.exists() {
        println!(
            "cargo:warning=Skipping Windows executable icon embedding because {GENERATED_WINDOWS_ICON_PATH} was not found."
        );
        return Ok(());
    }

    let mut resource = winresource::WindowsResource::new();
    resource.set_icon(icon_path.to_string_lossy().as_ref());
    resource.compile()?;

    Ok(())
}
