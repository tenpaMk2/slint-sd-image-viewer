# Slint SD Image Viewer

![icon](./bundle/icon.png)

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

Because this app is unsigned, follow this local workaround:

1. Open the `.dmg` and move or copy **Slint SD Image Viewer.app** into `~/Downloads`.
2. Run the following command in Terminal:

```sh
xattr -cr ~/Downloads/Slint\ SD\ Image\ Viewer.app/
```

This command has security implications. Use it only if you understand the risk.
It removes the downloaded-from-Internet quarantine attribute, which bypasses Gatekeeper checks for this app.

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
