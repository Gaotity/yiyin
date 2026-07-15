# Third-Party Dependencies and Assets

## Runtime dependencies

| Component             | Version policy  | License metadata     | Notes                                                                                                                                                                                                                                                                   |
| --------------------- | --------------- | -------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Electron              | `43.1.1`        | MIT                  | Desktop runtime; installation script is explicitly allowed. This exact user-required version has a temporary minimum-release-age exception because it was less than 24 hours old when the lockfile was created; the exception must be removed after the cooling period. |
| Sharp                 | `0.35.3`        | Apache-2.0           | Image decoding and processing; installation script is explicitly allowed.                                                                                                                                                                                               |
| ExifReader            | `4.41.0`        | MPL-2.0              | Reads metadata in-process without bundled executables.                                                                                                                                                                                                                  |
| Zod                   | `4.4.3`         | MIT                  | Runtime validation for IPC and configuration boundaries.                                                                                                                                                                                                                |
| `@ggchivalrous/db-ui` | exactly `1.3.1` | MIT package metadata | Single-maintainer provenance is a maintenance and supply-chain risk. Replacement is deferred to a separate UI project.                                                                                                                                                  |

License metadata must be revalidated when these versions change. Dependabot updates require the same validation and a frozen lockfile.

Electron's standard runtime contains Chromium codec libraries such as `libffmpeg.dylib` or `ffmpeg.dll`. These are part of the pinned Electron distribution, not a callable FFmpeg executable or a separately bundled archive. Package verification rejects standalone `ffmpeg` executables and inherited FFmpeg archives while allowing the required Electron runtime libraries.

## Bundled assets

The application intentionally bundles no third-party fonts, camera-manufacturer logos, sample photographs, donation QR codes, community graphics, legacy screenshots, or inherited application icons. The UI uses system fonts and text-only camera brands.

No production application icon or store artwork is approved. New brand assets require explicit ownership or redistribution records before public distribution can be enabled.

## Build-only components

Electron Builder, Electron Fuses, esbuild, and platform packaging helpers are development dependencies. CI may produce private workflow artifacts for smoke testing, but no workflow publishes a GitHub Release or publicly distributes unsigned packages.
