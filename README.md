# Slint SD Image Viewer

_[English🌐](README.md) | [日本語🇯🇵](README.ja.md)_

Desktop image viewer built with Slint + Rust.
Focused on viewing Stable Diffusion metadata and managing XMP ratings.

🚧 W.I.P. 🚧

## Download

[Get it from the Releases page.](https://github.com/tenpaMk2/slint-sd-image-viewer/releases)

## Features

- **Image viewing**: Supports JPG / JPEG / PNG / GIF / BMP / WebP
- **Stable Diffusion metadata**: Extracts and displays SD parameters from PNG `parameters` text
- **XMP rating**: Set rating with `0`-`5` keys (XMP `xap:Rating`)
- **Keyboard navigation**: Move between images with `←` / `→`
- **Auto reload**: Toggle directory watching with `L`
- **Copy image file**: Copy the current image file to clipboard with `Ctrl+C`
- **Cross-platform**: Supports macOS / Windows / Linux

## Tech Stack

- **Application**: Rust 2021
- **UI**: [Slint](https://slint.dev/) 1.x
- **Main crates**: `image`, `png`, `xmp_toolkit`, `notify-debouncer-mini`

## Development

### Prerequisites

- [Rust](https://rustup.rs/)
- (`cargo-packager` for desktop packaging)

If `cargo-packager` is not installed:

```bash
cargo install cargo-packager --locked
```

### Getting Started

1. Clone the repository:

```bash
git clone https://github.com/tenpaMk2/slint-sd-image-viewer.git
cd slint-sd-image-viewer
```

2. Run in development:

```bash
cargo run
```

### Build Commands

- `cargo run` - Run in development
- `cargo build --release` - Release build
- `cargo packager --release --formats app` - Build macOS `.app` bundle (unsigned)
- `cargo packager --release --formats nsis` - Build Windows installer (`.exe`)

## macOS Local Distribution (without Developer ID)

Use this flow when you only need local distribution and do not want Apple Developer ID signing.

```bash
cargo packager --release --formats app
```

- Output app: `target/release/packager/**/Slint SD Image Viewer.app`
- The app is unsigned

### First Launch on macOS

For unsigned apps, first launch may be blocked by Gatekeeper.
Use Apple's official flow:

1. Try to open the app once.
2. Open **System Settings** > **Privacy & Security**.
3. In **Security**, click **Open Anyway** for this app.
4. Click **Open** and authenticate.

Reference: https://support.apple.com/ja-jp/guide/mac-help/mh40616/mac

## Supported Formats

| Format | View | SD Parameter Extraction | XMP Rating |
| --- | --- | --- | --- |
| PNG | ✅ | ✅ (`parameters` text) | ✅ |
| JPG / JPEG | ✅ | 🚧 | ✅ |
| WebP | ✅ | 🚧 | ✅ |
| GIF | ✅ | - | 🚧 |
| BMP | ✅ | - | 🚧 |

## License

MIT License - See [LICENSE](LICENSE) for details.

## Contributing

Issues and Pull Requests are welcome.

- [GitHub Issues](https://github.com/tenpaMk2/slint-sd-image-viewer/issues)
