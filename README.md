# Yiyin (壹印)

Yiyin is an open-source desktop application for adding configurable photo
frames and camera metadata to images. It is free, contains no product
watermark, and performs image processing locally.

The application uses Tauri 2 and Rust for the desktop runtime, native
integration, configuration, EXIF handling, task scheduling, and rendering.
React is the presentation layer inside the system WebView.

## Origin and attribution

This repository is an independently maintained fork of
[ggchivalrous/yiyin](https://github.com/ggchivalrous/yiyin), originally created
by ggchivalrous and licensed under `GPL-3.0-only`. This fork remains under the
same license; see [NOTICE](NOTICE) for copyright attribution.

## Features

- Import JPEG, PNG, and WebP images through the native picker or drag and drop.
- Render camera metadata, custom fields, templates, fonts, and light/dark logos.
- Preview output before export and process two export tasks concurrently.
- Preserve source dimensions, density, output quality, and established naming
  behavior.
- Keep configuration and registered resources in application-owned local
  storage.

## Install and run

Packages are not currently published as downloadable releases. Build the
application from source instead. Install the toolchains listed in the
Development section, then run:

```bash
pnpm install --frozen-lockfile
pnpm tauri build
```

This produces unsigned macOS or Windows packages under
`src-tauri/target/release/bundle/`. CI package runs also upload temporary
unsigned artifacts that expire with the run.

On macOS, move `壹印.app` to `/Applications`. Because the bundle is unsigned,
macOS may quarantine a locally built or CI-artifact copy. Inspect the
artifact, then remove the quarantine attribute if you trust it:

```bash
xattr -dr com.apple.quarantine /Applications/壹印.app
```

On Windows, SmartScreen may warn about the unsigned NSIS installer. Inspect the
publisher and file source before choosing to continue.

## Usage

Open the application, adjust the rendering options, add one or more images,
and select `生成印框`. Enable `快速输出` to start automatically or `实时预览`
to preview the selected image.

<img src="static/软件界面.jpg" height="300" alt="Yiyin application window" />

<img src="static/输出印框.jpg" height="300" alt="Yiyin output workflow" />

### Example output

Landscape:

<img src="static/最终效果.jpg" height="300" alt="Landscape output" />

Portrait:

<img src="static/最终效果.png" height="300" alt="Portrait output" />

Portrait converted to landscape:

<img src="static/最终效果-竖转横.jpeg" height="300" alt="Portrait-to-landscape output" />

### Custom fonts

Use the font control in the title bar to select a bundled font or register a
local TTF/OTF file. Registered files are copied into application-owned local
storage and are not uploaded.

<img src="static/字体列表.jpg" height="300" alt="Bundled font list" />

<img src="static/添加字体.jpg" height="300" alt="Registering a custom font" />

## Development

Required toolchains:

- Node.js 24
- pnpm 11.13.0
- Rust 1.97.0

Install and validate the project:

```bash
pnpm install --frozen-lockfile
pnpm ci
cargo fmt --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
bash tests/architecture/dependency-boundaries.sh
```

Run the desktop application with `pnpm tauri dev`. Build unsigned macOS or
Windows packages with `pnpm tauri build`. Architecture and adapter details are
documented in [src-tauri/README.md](src-tauri/README.md).

## Security and privacy

The application does not include telemetry, an updater, a network service, or
a Node sidecar. React cannot access arbitrary paths, dialogs, the shell, or the
network. Native operations are exposed through dedicated Rust commands, and
external links use a Rust-owned allowlist.

## Feedback

Report problems or suggestions through the project
[issues](https://github.com/ggchivalrous/yiyin/issues), or contact the author
through [Bilibili](https://space.bilibili.com/94829489).

QQ group: `718615618`.

## License

Yiyin is licensed under `GPL-3.0-only`. See [LICENSE](LICENSE) for the full
text and [NOTICE](NOTICE) for upstream copyright attribution.
