# Spec: Tauri and Rust Rewrite

**Status:** Implemented — see PR #3 (`codex/tauri-rewrite`). Tickets: `docs/tickets/tauri-rust-rewrite/`.

## Problem Statement

壹印 (Yiyin) is a desktop app that adds EXIF-based watermark frames to photos. It was built on Electron + Svelte, which meant shipping an entire Chromium and Node.js runtime to the user's machine, plus native sidecar binaries (Sharp, FFmpeg, ExifTool), just to compose images and render text. That stack is heavy to install, slow to start, wide in supply-chain surface, and it forces business logic (image rendering, EXIF normalization, persistence) to live in JavaScript where it is entangled with the UI.

The user — a photographer on macOS or Windows — just wants a small, fast, native-feeling app that behaves exactly like the one they already know: same window, same 简体中文 copy, same workflows, same configuration, same output files.

## Solution

Replace the entire runtime in one clean cutover: **Tauri 2 + Rust + React**.

- **Rust owns the product.** Everything that reads or writes files, parses or persists configuration, manages resources, extracts metadata, schedules tasks, or produces images lives in Rust, structured as a compiler-enforced clean architecture (domain / application / infrastructure / Tauri adapter).
- **React is display-only.** It renders Rust-owned state and submits user intent through a narrow, typed, capability-based IPC boundary. It never sees a filesystem path, a raw `File` object, or an EXIF tag.
- **Behavioral compatibility is exact**, with one documented exception: the new Rust renderer may differ *perceptually* (not pixel-identically) from the old Sharp/Canvas renderer, because text metrics, blur algorithms, and JPEG encoders differ. That exception is made measurable with frozen legacy fixtures and golden comparisons with documented tolerances.
- The cutover is a single PR from clean `origin/main`: no dual-runtime period, no Node sidecar, no compatibility shim, no product redesign. Existing users' Electron configuration, fonts, and resources are imported once, idempotently, without modifying the legacy sources.

## User Stories

1. As a photographer, I want to add multiple images at once through a native file dialog, so that I can batch-process a shoot.
2. As a photographer, I want to drag and drop images onto the window, so that I don't have to browse through dialogs.
3. As a photographer, I want the app to reject files that aren't really JPEG/PNG/WebP (verified by content, not extension), so that I get a clear failure instead of a corrupt output.
4. As a photographer, I want to see my imported images in a list and select the one I'm working on, so that I can review each frame.
5. As a photographer, I want to see whether EXIF data is available for each image, so that I know which templates will render.
6. As a photographer, I want to read the normalized EXIF details of the selected image and copy them to the clipboard, so that I can reuse them elsewhere.
7. As a photographer, I want to press `生成印框` to start exporting, so that exports only happen when I explicitly ask.
8. As a photographer, I want `快速输出` to start exports automatically as images arrive, so that repetitive batches take zero clicks.
9. As a photographer, I want a debounced `实时预览` that cancels the stale preview when I change settings or selection, so that I always see the current frame, never an outdated one.
10. As a photographer, I want to see per-task progress, success, failure, and cancellation states, so that I know exactly what happened to each image.
11. As a photographer, I want to clear the queue and visible tasks without deleting already-exported images, so that I can start fresh without losing work.
12. As a photographer, I want to choose and open the output directory, so that I can find my framed photos (default `Pictures/watermark`).
13. As a photographer, I want to edit global rendering options — corner radius, shadow, solid vs. blurred background, background ratio, landscape conversion, image width ratio, text margin, quality, blur — with immediate saving, so that the frame looks the way I want.
14. As a photographer, I want to add, edit, show/hide, override, and delete custom parameter fields, so that I control exactly which EXIF values appear.
15. As a photographer, I want to add, edit, enable/disable, reorder, and delete custom text templates with `{FieldKey}` placeholders, while system templates can't be deleted, so that my layout is flexible but never broken.
16. As a photographer, I want hidden or empty fields to collapse out of the render (no dangling placeholders or empty lines), so that the frame stays clean.
17. As a photographer, I want to use bundled fonts and register my own custom fonts (and remove them later), so that the typography matches my style.
18. As a photographer, I want to reset all configuration to defaults, so that I can recover from experiments.
19. As an existing user upgrading from the Electron version, I want my previous configuration, custom fields, templates, fonts, and image resources imported automatically on first launch — exactly once, without touching the old files — so that nothing I set up is lost.
20. As a photographer, I want outputs named after the source file (`photo.jpg`, `photo-1.jpg` on conflicts), so that outputs are recognizable and never silently overwritten.
21. As a photographer, I want exports written atomically at the source image's density and my chosen quality, so that a crash or cancellation never leaves a half-written file or damages a previous success.
22. As a photographer, I want the same 900×730 frameless window, custom title bar, footer, help/feedback/donation links, and 简体中文 copy I already know, so that the app feels unchanged.
23. As a photographer, I want a second launch of the app to focus the existing window instead of opening a duplicate, so that I don't get confused.
24. As a photographer, I want portrait photos optionally rotated into landscape frames, and explicit background ratios, so that the composition fits my intent.
25. As a security-conscious user, I want the app to have no updater, no telemetry, no analytics, and no arbitrary network access, and to only open whitelisted external links, so that the tool stays local and trustworthy.

## Implementation Decisions

### Workspace and module boundaries (compiler-enforced)

- Cargo workspace with crates `yiyin-domain`, `yiyin-application`, `yiyin-infrastructure`, plus the `src-tauri` adapter (package/lib name `yiyin-desktop`) and the React presentation layer.
- Allowed dependency direction, enforced by the Cargo manifests and a boundary test: `src-tauri → application + infrastructure`, `infrastructure → application + domain`, `application → domain`, `domain → nothing`.
- All first-party Rust crates use `#![forbid(unsafe_code)]`; recoverable production paths ban `unwrap()`/`expect()`.

### `yiyin-domain` — pure rules

- Pure entities, value objects, invariants, state transitions, and renderer-agnostic render plans. Production code has no dependency on Tauri, Tokio, serde, filesystem, image codecs, font engines, UUIDs, or clocks. Never serializes itself, never emits events. Dev-only test dependencies may parse the frozen legacy fixtures.
- Types: `Config`, `RenderOptions`, `Template`, `TemplateField`, `FontSpec`, normalized `Metadata`; `ResourceId`, `TaskId`, `OutputDirectory`, `ImageDimensions`, `ImageDensity`, `Quality` and validated newtypes (`TryFrom`, private fields); `TaskState` with only legal transitions; `RenderRequest`, `RenderPlan`, `RenderStage`; `OutputNameResolver`. Invalid ranges and illegal task states are unrepresentable after construction.
- Preserves v1.6 defaults exactly: quality `100`, radius `2.1`, shadow `6`, main-image width `90`, font `PingFang SC`, etc. `origin_wh_output` stays in the schema for compatibility but has no geometric effect.
- Exact geometry contract: explicit-ratio → portrait-to-landscape swap → main-image width expansion → minimum/shadow top margin → three-quarter text spacing → 2.7% bottom text offset → vertical centering → radius/shadow percentage rules → shadow intermediate plane capped at `10240`. Every `ceil/floor/round` is a named helper; integer rounding is part of the contract.
- Output naming: source stem + `.jpg`; conflicts resolve to `<stem>-<N>.jpg` with N = max numeric suffix + 1, skipping gaps and handling non-numeric suffixes.
- Progress milestones encoded as `RenderStage::percent()`: `1, 10, 20, 30, 50, 60, 70, 90, 100`.

### `yiyin-application` — use cases and outbound ports

- Inbound use cases: `Bootstrap`, `UpdateConfig`, `ResetConfig`, `RegisterImages`, `RegisterFont`, `RemoveFont`, `RegisterOverlay`, `ReadTaskExif`, `StartTasks`, `PreviewTask`, `CancelTask`, `ClearTasks`.
- Outbound ports (object-safe, native Rust traits, synchronous where local; **no `async-trait`**): `ConfigRepository`, `ResourceRepository`, `MetadataReader`, `ImageRenderer`, `TaskQueue`, `TaskEventSink`, `OutputDirectoryGateway`, `Clock`, `IdGenerator`.
- Closed error enum with nine stable codes: `CANCELLED`, `CONFIG_INVALID`, `FILE_INVALID`, `FILE_NOT_FOUND`, `FORBIDDEN`, `INTERNAL`, `INVALID_REQUEST`, `RESOURCE_NOT_FOUND`, `TASK_NOT_FOUND`; `safe_message()` never leaks internals. Paths exist only inside ports; public snapshots are path-free.
- Orchestration semantics: starting a task freezes a copy of the full `RenderRequest`; quick-output enqueues on registration, explicit mode waits; preview B cancels preview A (dedicated replace-latest slot, never publishes output); clear removes registered/queued snapshots only; conflict-safe output names are reserved before enqueue; `Bootstrap` runs legacy import before load and returns migration warnings separately from fatal errors.

### `yiyin-infrastructure` — outbound adapters

- Adapters: `JsonConfigRepository`, `FileResourceRepository`, `ExifMetadataReader`, `CosmicTextImageRenderer`, `TokioTaskQueue`, `UuidGenerator`. First layer allowed to depend on serde, `image`, `kamadak-exif`, `cosmic-text`, Tokio, UUID, concrete filesystem APIs. Adapter errors are translated into application errors before crossing inward.
- Durable config writes: temp file in the same directory → flush + sync → previous file preserved as `.bak` → atomic rename → directory sync. Corrupt existing files become `.invalid.bak` before defaults are restored. Versioned envelope (`{version, config}`), explicit mapping, unknown future-schema fields never silently reinterpreted, missing built-in items appended without deleting user items, runtime private paths never serialized.
- One-time Electron import: only when no committed Tauri config exists; probes the legacy user-data locations per platform in order; staged in a temp directory, validated, atomically published, then a migration marker is written; legacy source hashes verified unchanged; interruption-safe and idempotent on rerun; a single unsupported resource degrades to a warning.
- Resource registry: UUID v4 opaque IDs, records carrying kind/canonical path/MIME/allowed root; owned resources copied in atomically; **containment re-verified on every resolve** (symlink escape → `FORBIDDEN`); JPEG/PNG/WebP magic-byte validation; extension mismatch rejected; source file references never cross the React boundary.
- EXIF adapter: reads only needed tags; reproduces the normalization contract — vendor `CORPORATION` stripping then Title Case, Nikon `Z 7_2` → `ℤ 7 Ⅱ`, Sony `ILCE-` → lowercase `α`, fractional shutter → `1/N`, focal rounding, program/datetime/white-balance/metering strings, EXIF orientation applied before geometry, all-empty → no metadata object. Malformed tags blank the field, never panic.
- Renderer: shared pipeline for preview and export — validate, decode, apply orientation, normalize metadata, build the domain `RenderPlan`, solid or stretched-blur background (image-crate only, no `imageproc`), `cosmic-text` shaping with bundled + registered fonts (deterministic repo fonts for golden tests), logo slots aligned by baseline or center, shadow alpha mask drawn directly, rounded-corner masking, composite in the established order, JPEG encode at source density and chosen quality, atomic publish. Preview differs only in target, cancellation policy, and quality `70`; never publishes an output file and returns a registered resource URL (no data URLs).
- Task queue: Tokio semaphore = 2 (preserves current concurrency), CPU-heavy work on `spawn_blocking`, per-task cancellation token checked at stage boundaries, typed domain statuses via `TaskEventSink`, join errors map to `INTERNAL` without panicking; shutdown rejects new tasks, cancels active ones, waits a bounded grace (5s). No Rayon.

### Tauri boundary (inbound adapter + composition root)

- Seventeen commands, kept deliberately thin (`deserialize → validate boundary → call use case → map result`): `bootstrap`, `update_config`, `reset_config`, `choose_output_directory`, `open_output_directory`, `choose_images`, `register_font`, `remove_font`, `register_overlay`, `read_task_exif`, `start_tasks`, `preview_task`, `cancel_task`, `clear_tasks`, `minimize_window`, `close_window`, `open_external_url`. Native drag-drop enters through the same `RegisterImages` use case as the dialog.
- Electron-era callbacks are removed, not rebuilt: text-render callbacks, shadow-render callbacks, path joining, arbitrary path info, generic directory/shell/filesystem/protocol access.
- DTOs live only in the Tauri layer (inner crates never derive serde for IPC convenience): `BootstrapDto`, `PublicConfigDto` (no private paths), `TaskDescriptorDto`, `ResourceDescriptorDto`, `MetadataDto`, `CommandErrorDto` (`{code, message}` only). Explicit `From`/`TryFrom` anti-corruption mapping; the small duplication is intentional. Rust DTOs are the single source of truth for the boundary; `ts-rs` (dev-only) generates and drift-checks the TS types.
- One typed event, `task-status`: tagged union of `progress` (integer percent) / `completed` (output or preview resource descriptor) / `failed` (stable code + safe message) / `cancelled`. Images and binary content never travel in command responses or events; events are read-only notifications — command results and `bootstrap` remain the reconciliation source of truth.
- Resource protocol `yiyin://resource/<opaque-id>`: opaque IDs are runtime capabilities, not path encodings, not persistent across restarts. Every request re-parses the exact scheme/host/single-segment path, rejects traversal/encoded separators/queries/unknown IDs, re-verifies registry containment and MIME, and serves with a fixed content type. It cannot read arbitrary local files.
- Native integration: dialog and opener plugins sit only behind dedicated commands (their generic JS APIs are not exposed); external URLs are a closed enum with URLs hardcoded in Rust (GitHub repo/issues/current release, Bilibili profile/feedback); single-instance plugin registered first, second-instance callback restores and focuses the main window; production DevTools disabled.
- Composition order: single-instance → log → dialog → opener → protocol → managed state → invoke handler → drag-drop listener → setup.

### React presentation layer

- A `platform` module is the only place importing `@tauri-apps/api`; a typed `PlatformClient` encapsulates command names, request/response DTOs, errors, and event subscription; a fake adapter serves Vitest and browser Playwright.
- `useAppController` with `useReducer`: Rust snapshots are authoritative, task events reconcile by ID, selection is kept only while the task exists, unknown event IDs trigger re-bootstrap, unsubscribe on unmount. No global client state library.
- React Hook Form (no resolvers) for dialog-style parameter/template/font edit transactions; instant-save for global controls, with Rust as the validation and persistence authority; slider drags debounce but flush on blur/unmount.
- React never copies domain rules, never receives paths or `File` objects, never reads EXIF, never renders text into images, never builds output files. Images display only via `yiyin://resource/<id>` URLs. React must be replaceable without touching the inner three layers.

### Security

- `withGlobalTauri` disabled; no remote scripts/styles, no `eval`, no arbitrary network; production CSP allows only bundled assets and the custom resource protocol (dev policy separately allows loopback Vite only); navigation and new-window requests denied unless converted to whitelisted Rust open commands.
- Minimal custom capabilities: no generic `dialog:`/`opener:`/`fs:`/`shell:`/`http:`/wide-protocol permissions for the WebView.
- The old remote release check is removed: the footer keeps the version text and explicit release-page interaction, but the app never calls the GitHub API.

### Dependency baseline

- Toolchains: Rust `1.97.0` (pinned), Node.js `24`, pnpm `11.13.0`. Frontend: React `19.2.7`, `@tauri-apps/api` `2.11.1`, Base UI, React Hook Form; tooling Vite 8, TypeScript 5.9 (strict + `noUncheckedIndexedAccess`), Biome, Vitest, Testing Library, Playwright. Rust: Tauri `2.11.5`, dialog/opener/log/single-instance plugins, serde, thiserror, Tokio, UUID, `image` (jpeg/png/webp only), `kamadak-exif`, `cosmic-text`. Dev-only: `ts-rs`, `tempfile`.
- Selection priority: minimal dependency & supply-chain surface → ecosystem maturity & DX → performance & build speed.
- Explicit exclusions (require an approved design change to add): Zustand, Tailwind, shadcn/ui, Sass, Zod/resolvers, Radix, React Router, TanStack Query, Axios, Sonner, Lucide, Electron, Svelte, Sharp, `db-ui`, Node sidecars, `imageproc`, Rayon, `async-trait`, WebdriverIO/Selenium npm packages, prerelease deps, Git deps.
- Supply chain: exact pins, frozen/locked installs, 24-hour minimum release-age policy for npm and Cargo, `pnpm audit --audit-level high`, `cargo deny check`, CodeQL (JS/TS + Rust), GitHub Actions pinned to full commit SHAs, Renovate per-ecosystem with no auto-merge.

## Testing Decisions

Good tests assert external behavior through the layer's public interface, never implementation details. Each layer is tested at its own seam — domain through pure function contracts, application through hand-written fake ports, infrastructure through real files/codecs, the Tauri adapter through DTO/protocol/capability behavior, React through a typed fake platform adapter.

- **Domain**: value-object range validation; template parsing/substitution/omission; normalized-EXIF consumption; output-name conflicts; geometry and rounding fixtures (constants copied from the frozen legacy manifest); task-state transitions and cancellation.
- **Application**: every use case against fake ports (no Tauri, no real filesystem) — bootstrap and legacy-import outcomes, legal/illegal config updates (invalid config never persists), image registration with unsupported input, font/overlay registration, explicit vs. quick-output starts, per-task frozen configuration, preview supersession, cancel/fail/clear semantics, safe-error mapping.
- **Infrastructure**: versioned JSON migration, corrupt-file backup, durable-write failure injection at every stage; legacy-import idempotency and source preservation; containment and symlink/race re-validation; real JPEG/PNG/WebP decode and EXIF orientation; vendor normalization fixtures (generic/Nikon/Sony); resource registration; **golden renderer compatibility** — exact geometry plus perceptual comparison (SSIM ≥ 0.97, changed-pixel ratio ≤ 8%) with human-inspectable diff artifacts on failure; atomic output publish and cleanup after failure/cancellation.
- **Tauri adapter**: DTO↔domain mapping and `ts-rs` drift check; command boundary validation; stable error-code mapping and sanitization (a source error containing a private path must never appear in serialized DTOs or events); capability whitelist; protocol attack matrix (traversal, encoded separators, extra segments, queries, unknown IDs, symlink escape, wrong MIME); plugin registration order and focus callback.
- **React**: Biome, strict TypeScript, Vitest + Testing Library component and workflow tests against the fake adapter; Playwright browser parity tests for layout and workflows (explicitly not claiming to be the native runtime); user-visible error copy; task reconciliation from command results + events.
- **Desktop and packaging**: macOS unsigned bundle build/launch/observe/quit/inspect; Windows unsigned bundle plus a desktop smoke where Edge WebDriver attaches to the running app over the WebView2 remote debugging port, driven by a minimal internal W3C client (test code only — no new npm WebDriver framework); a compile-time-only `e2e-fixture` Cargo feature (off by default in production, proven by policy tests); bundle verification proving no Electron/Node/Sharp/Svelte/`db-ui`/sidecar/remote assets/generic capabilities.
- **CI gates**: frontend quality; Rust fmt/clippy/tests; security & dependency policy (audit, cargo-deny, release-age); golden compatibility; macOS packaging smoke; Windows packaging + desktop smoke.

Prior art: the legacy Electron renderer outputs are captured once as frozen fixtures (14 scenarios: portrait/landscape/webp defaults, EXIF orientation 6, explicit 3:2 ratio, portrait-to-landscape, solid-white-no-shadow, blurred-shadow-radius, both built-in focal templates, light/dark logos, forced custom text, bundled custom font) and are the sole benchmark for the golden tests — never regenerated from Rust.

## Out of Scope

- No UI or product redesign; no updater, telemetry, analytics, remote services, arbitrary network access, or background cloud features.
- No signing, notarization, release publishing, or auto-release workflows — macOS/Windows bundles are unsigned.
- No mobile or Linux targets in this migration.
- No Electron→Tauri compatibility layer, no dual-runtime period, no Node sidecar or bundled Node runtime.
- No pixel- or byte-identical renderer output requirement (perceptual compatibility with documented tolerances instead).
- No speculative routing, server-state, notification, state-management, or component-system abstractions.
- The rewrite PR is not auto-merged or published; passing checks, review approval, and push completion do not authorize merge or release.

## Further Notes

- **Approved decision record:** (1) clean single-PR cutover, dual-runtime and phased shell/UI migration rejected; (2) Rust owns the product, React is display-only; (3) multi-crate clean architecture as the production-grade, compiler-enforced reference; (4) native Rust traits over abstraction-supporting dependencies (no `async-trait`); (5) perceptual renderer compatibility accepted over pixel parity; (6) dependency-baseline priority: minimal surface → maturity/DX → performance; (7) single-instance behavior, minimal capabilities, opaque resources, and Rust-owned native integration are mandatory.
- **Delivery governance:** work stayed on the short-lived `codex/tauri-rewrite` branch from clean `origin/main`; PR title in English Conventional-Commit style with an assignee; PR body and review discussion in English + 简体中文 sections; repo docs, code, identifiers, commit messages, and PR titles in English; at most one immediate checks snapshot after creating or updating the PR — no long watch loops; deliver but never merge or publish.
- **Provenance:** this spec was converted from the superpowers-style design document (`docs/superpowers/specs/2026-07-15-tauri-rust-rewrite-design.md`, now removed) into the mattpocock/skills spec format. The companion implementation plan was converted into per-ticket files under `docs/tickets/tauri-rust-rewrite/`. All tickets are complete as of PR #3.
