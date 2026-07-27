# 16 — Remove the Legacy Runtime, Update Documentation, and Run Full Acceptance

**What to build:** The cutover finishes: every trace of the old runtime is deleted — Electron app code, the Svelte web app, shared legacy modules, old build/release scripts and configs, bundled FFmpeg/ExifTool binaries, generated logger code, old CI workflows, and (after confirming the captured artifacts are committed) the temporary legacy capture tooling and Electron fixture mode. Icons, screenshots, logos, donation images, bundled fonts, README product content, and fixture inputs/outputs are migrated to their new homes first. A TDD-first final-tree test asserts no tracked runtime file or dependency references electron, svelte, sharp, ffmpeg, exiftool, `db-ui`, Node sidecars, legacy render callbacks, or legacy IPC routes. The LICENSE becomes standard GPL-3.0-only text; READMEs document the new commands and unsigned bundles; the Tauri adapter gets its own README (crate direction, command tracing, ownership/async boundaries, DTO generation, fmt/clippy/test commands). The full local verification matrix runs, an unsigned macOS bundle is built and verified, and manual acceptance covers drag-drop, both drawers, preview, export, output directory, and second-launch focus. Delivery means pushing the branch and opening the assigned PR — never merging, never publishing.

**Blocked by:** All of tickets 01–15

**Status:** completed

- [x] Full verification matrix passes: frozen install, format, lint, typecheck, Vitest, Playwright, audit, release-age, build, cargo fmt/clippy/test, cargo deny, boundary scripts, final-tree test — with no tracked changes produced
- [x] App launches to a single 900×730「壹印」window and exits cleanly; unsigned macOS bundle builds and passes bundle verification
- [x] Manual acceptance passes: drag-drop, settings drawer, template drawer, preview, export, output directory, second-launch focus
- [x] Final tree contains only the Tauri/Rust/React runtime plus compatibility fixtures; branch pushed and PR #3 opened — not merged, not published
