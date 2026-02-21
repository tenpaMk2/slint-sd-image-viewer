use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

const SOURCE_ICON_PATH: &str = "bundle/icon.png";
const GENERATED_ICON_PATH: &str = "target/generated/icon.icns";
const ICON_SIZES: &[(u32, u32, &str)] = &[
    (16, 16, "icon_16x16.png"),
    (32, 32, "icon_16x16@2x.png"),
    (32, 32, "icon_32x32.png"),
    (64, 64, "icon_32x32@2x.png"),
    (128, 128, "icon_128x128.png"),
    (256, 256, "icon_128x128@2x.png"),
    (256, 256, "icon_256x256.png"),
    (512, 512, "icon_256x256@2x.png"),
    (512, 512, "icon_512x512.png"),
];

fn main() {
    println!("cargo:rerun-if-changed={SOURCE_ICON_PATH}");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        generate_macos_icon().expect("Failed to generate macOS app icon");
    }

    slint_build::compile("ui/app-window.slint").expect("Slint build failed");
}

fn generate_macos_icon() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let source_png = manifest_dir.join(SOURCE_ICON_PATH);
    if !source_png.exists() {
        println!(
            "cargo:warning=Skipping icon generation because {SOURCE_ICON_PATH} was not found."
        );
        return Ok(());
    }

    let iconset_dir = prepare_iconset_dir()?;
    for (width, height, file_name) in ICON_SIZES {
        let destination = iconset_dir.join(file_name);
        resize_icon(&source_png, &destination, *width, *height)?;
    }

    fs::copy(&source_png, iconset_dir.join("icon_512x512@2x.png"))?;

    let output_icns = manifest_dir.join(GENERATED_ICON_PATH);
    if let Some(parent) = output_icns.parent() {
        fs::create_dir_all(parent)?;
    }

    create_icns(&iconset_dir, &output_icns)?;

    Ok(())
}

fn prepare_iconset_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let iconset_dir = out_dir.join("icon.iconset");

    if iconset_dir.exists() {
        fs::remove_dir_all(&iconset_dir)?;
    }
    fs::create_dir_all(&iconset_dir)?;

    Ok(iconset_dir)
}

fn resize_icon(
    source_png: &Path,
    destination: &Path,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("sips")
        .arg("-z")
        .arg(width.to_string())
        .arg(height.to_string())
        .arg(source_png)
        .arg("--out")
        .arg(destination)
        .status()?;
    if !status.success() {
        return Err(format!("sips command failed with status {}", status).into());
    }

    Ok(())
}

fn create_icns(iconset_dir: &Path, output_icns: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("iconutil")
        .arg("-c")
        .arg("icns")
        .arg(iconset_dir)
        .arg("-o")
        .arg(output_icns)
        .status()?;
    if !status.success() {
        return Err(format!("iconutil command failed with status {}", status).into());
    }

    Ok(())
}
