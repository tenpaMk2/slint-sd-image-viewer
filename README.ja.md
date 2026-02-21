# Slint SD Image Viewer

_[English🌐](README.md) | [日本語🇯🇵](README.ja.md)_

Slint + Rust で構築されたデスクトップ画像ビューアーアプリケーションです。
Stable Diffusion 画像のメタデータ表示と、XMP レーティング管理に対応しています。

🚧 W.I.P. 🚧

## ダウンロード

[Releases ページからどうぞ。](https://github.com/tenpaMk2/slint-sd-image-viewer/releases)

## 機能

- **画像表示**: JPG / JPEG / PNG / GIF / BMP / WebP をサポート
- **Stable Diffusion メタデータ表示**: PNG の `parameters` テキストから SD パラメータを抽出・表示
- **XMP レーティング**: `0`〜`5` キーでレーティングを設定（XMP `xap:Rating`）
- **キーボードナビゲーション**: `←` / `→` で前後画像に移動
- **自動リロード**: `L` でディレクトリ監視の ON/OFF を切り替え
- **画像ファイルコピー**: `Ctrl+C` で現在画像ファイルをクリップボードにコピー
- **クロスプラットフォーム**: macOS / Windows / Linux をサポート

## 技術スタック

- **アプリケーション**: Rust 2021
- **UI**: [Slint](https://slint.dev/) 1.x
- **主要ライブラリ**: `image`, `png`, `xmp_toolkit`, `notify-debouncer-mini`

## 開発

### 前提条件

- [Rust](https://rustup.rs/)
- （デスクトップ配布をする場合）`cargo-packager`

`cargo-packager` 未導入の場合:

```bash
cargo install cargo-packager --locked
```

### はじめに

1. リポジトリをクローン:

```bash
git clone https://github.com/tenpaMk2/slint-sd-image-viewer.git
cd slint-sd-image-viewer
```

2. 開発実行:

```bash
cargo run
```

### ビルドコマンド

- `cargo run` - 開発実行
- `cargo build --release` - リリースビルド
- `cargo packager --release --formats app` - macOS 向け `.app` バンドル作成（署名なし）
- `cargo packager --release --formats nsis` - Windows 向けインストーラー（`.exe`）作成

## macOS ローカル配布（Developer ID 署名なし）

Apple Developer ID 署名なしでローカル配布したい場合は、次の手順を利用します。

```bash
cargo packager --release --formats app
```

- 出力先: `target/release/packager/**/Slint SD Image Viewer.app`
- 生成されたアプリは未署名です

### macOS 初回起動

本アプリはApple税を払ってないので未署名です！
Gatekeeper により初回起動がブロックされるのでApple 公式手順に従って許可してください。

1. 一度アプリを開こうとする
2. **システム設定** > **プライバシーとセキュリティ** を開く
3. **セキュリティ** 欄で対象アプリに対して **このまま開く** を選ぶ
4. **開く** を押して認証する

参考: https://support.apple.com/ja-jp/guide/mac-help/mh40616/mac

## 画像形式対応

| 形式 | 表示 | SDパラメータ抽出 | XMPレーティング |
| --- | --- | --- | --- |
| PNG | ✅ | ✅ (`parameters` テキスト) | ✅ |
| JPG / JPEG | ✅ | 🚧 | ✅ |
| WebP | ✅ | 🚧 | ✅ |
| GIF | ✅ | - | 🚧 |
| BMP | ✅ | - | 🚧 |

## ライセンス

MIT License - 詳細は [LICENSE](LICENSE) を参照してください。

## 貢献

Issue / Pull Request を歓迎します。

- [GitHub Issues](https://github.com/tenpaMk2/slint-sd-image-viewer/issues)
