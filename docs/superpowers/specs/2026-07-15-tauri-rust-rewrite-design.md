# Tauri and Rust Rewrite Design

## Status

Approved design for a clean replacement of the Electron and Svelte application with a Tauri 2, Rust, and React desktop application.

This document is the architecture and compatibility contract for the rewrite. It is not an implementation plan. Implementation sequencing will be defined only after this specification is reviewed and approved in its committed form.

## Summary

Yiyin will become a Tauri- and Rust-based desktop application. Rust owns every product capability that reads or writes files, interprets or persists configuration, manages resources, extracts metadata, schedules tasks, or produces image output. React is a thin WebView presentation layer that renders Rust-owned state and submits user intent through a typed Tauri adapter.

The rewrite is a single cutover from `origin/main`. It does not inherit the closed, unmerged PR #1 branch or the abandoned Svelte and Vite work in the original checkout. It does not temporarily support two desktop runtimes, introduce a Node sidecar, or redesign the product.

The implementation uses compiler-enforced Clean Architecture across a Cargo workspace. Domain rules and application use cases remain independent of Tauri, serialization, the filesystem, and image libraries. Infrastructure implements the outbound ports. `src-tauri` is the inbound adapter and composition root.

Compatibility is exact for product behavior, persisted semantics, geometry, naming, task behavior, and boundary contracts. The new renderer is allowed perceptual differences from Sharp and browser Canvas because the Rust libraries use different text metrics, blur algorithms, and JPEG encoders. Existing-renderer fixtures and deterministic golden comparisons make that exception measurable.

## Goals

- Replace Electron, Svelte, Sharp, browser Canvas rendering, FFmpeg, ExifTool, and Node runtime code with Tauri 2, Rust, and React.
- Make Rust the only business and platform core.
- Preserve product identity, visible text, layout, controls, workflows, and output behavior.
- Preserve existing configuration, templates, custom fields, fonts, logos, EXIF normalization, rendering geometry, and output naming semantics.
- Import legacy Electron user data once without modifying or deleting its source.
- Enforce dependency direction through Cargo crate boundaries.
- Expose a narrow, typed, capability-based boundary to React.
- Keep the dependency and supply-chain surface intentionally small.
- Produce unsigned macOS and Windows packages that can be validated in CI without publishing a release.

## Non-goals

- No UI or product redesign.
- No updater, telemetry, analytics, remote service, arbitrary network access, or background cloud feature.
- No application signing, notarization, release publication, or automatic release workflow.
- No mobile or Linux target in this migration.
- No Electron-to-Tauri compatibility layer and no dual-runtime period.
- No Node sidecar or bundled Node runtime.
- No pixel-identical or byte-identical output requirement between the old and new renderers.
- No speculative routing, server-state, notification, state-management, or component-system abstraction.
- No automatic merge or release after the rewrite PR is created.

## Product Compatibility Baseline

### Identity and desktop behavior

- Product name: `壹印`.
- Application identifier: `io.github.gaotity.yiyin`.
- Migration version: `1.6.0`.
- License: `GPL-3.0-only`.
- Main window: one frameless `900 x 730` desktop window with the current fixed-size production behavior.
- The existing custom title bar, minimize action, close action, footer, help, feedback, donation, font selector, and reset action remain in the same locations and retain their current text and behavior.
- The official Rust-only single-instance plugin is registered before every other Tauri plugin. A second launch restores the existing window if minimized and focuses it instead of creating another instance.
- Production DevTools are disabled.

### User workflows

The React application preserves these workflows:

- Add multiple images through a native selection dialog.
- Add images through native drag and drop.
- Reject unsupported inputs with the current user-facing failure behavior.
- Display the image list, selection, EXIF availability, copy-EXIF action, progress, success, and failure state.
- Start queued output explicitly with `生成印框`.
- Start output automatically when `快速输出` is enabled.
- Enable debounced `实时预览`, cancel the previous preview when a newer request supersedes it, and display loading, success, or failure state.
- Clear queued and displayed tasks without terminating already completed output files.
- Choose and open the output directory.
- Edit global rendering parameters.
- Add, edit, show, hide, override, and remove custom parameter fields.
- Add, edit, enable, disable, reorder, and remove custom text templates while preserving system templates.
- Select bundled fonts, register custom fonts, and remove custom fonts.
- Reset configuration to defaults.
- Open only approved external project links through a Rust-owned allowlist.

The existing Simplified Chinese UI strings are product data and remain unchanged unless a compatibility fixture proves that the source text itself has changed on `origin/main` before implementation begins.

### Configuration semantics

The persisted model retains the current field names and meanings:

- `version`
- `output`
- `options`
- `tempFields`
- `customTempFields`
- `temps`
- font records and registered font resources
- legacy version-update data only for migration compatibility; the new application does not perform update checks

The public configuration returned to React contains only presentation-safe values and opaque resource identifiers. It never exposes internal configuration, cache, font, static-resource, or source file paths.

Default rendering options remain:

| Option | Default | Valid UI range or rule |
| --- | ---: | --- |
| `iot` | `false` | Boolean |
| `landscape` | `false` | Disabled while explicit background ratio is active |
| `solid_bg` | `false` | Boolean |
| `solid_color` | `#fff` | Valid supported color |
| `origin_wh_output` | `false` | Preserved for schema compatibility |
| `radius` | `2.1` | `0..=50`, one decimal place |
| `radius_show` | `true` | Boolean |
| `shadow` | `6` | `0..=50`, one decimal place |
| `shadow_show` | `true` | Boolean |
| `bg_rate_show` | `false` | Requires positive width and height to alter geometry |
| `bg_rate.w` | `0` | Non-negative number |
| `bg_rate.h` | `0` | Non-negative number |
| `font` | `PingFang SC` | Registered or bundled font name |
| `main_img_w_rate` | `90` | `1..=100`, integer |
| `text_margin` | `0.4` | `0..=10000`, two decimal places |
| `quality` | `100` | `1..=100`, integer |
| `mini_top_bottom_margin` | `0` | `0..=100`, two decimal places |
| `bg_blur` | `100` | `0..=100`, integer |
| `preview_show` | `false` | Boolean |

Unknown fields from a newer schema are not silently reinterpreted. Missing known fields receive their current defaults. Missing built-in EXIF fields and system templates are appended without removing user-defined entries. Reset restores the complete default model.

Global controls keep their current immediate-save behavior: a valid change updates Rust-owned configuration and is persisted without a separate save action. Dialog-based parameter, template, and font edits retain their explicit save or cancel transaction. React may debounce command traffic, but Rust remains the validation and persistence authority.

### Template and field semantics

The built-in fields remain:

`PersonalSign`, `Make`, `Model`, `LensMake`, `LensModel`, `ExposureTime`, `FNumber`, `ISO`, `FocalLength`, `FocalLengthIn35mmFormat`, `ExposureProgram`, `DateTimeOriginal`, `ExposureCompensation`, `MeteringMode`, and `WhiteBalance`.

Field behavior remains:

- A field can be shown or hidden.
- A user value can be enabled and can optionally force replacement of extracted EXIF data.
- A field can render as text or as the light/dark image variant.
- A field can override font family, size, weight, italic style, case conversion, and color.
- Custom field keys remain stable once created.
- Template placeholders continue to use `{FieldKey}` syntax.
- Missing or hidden fields collapse from the rendered template rather than leaving placeholder text.
- A template with no visible content produces no text row.
- Templates preserve order, enabled state, system/custom type, vertical alignment, optional height, and font options.
- System templates cannot be deleted.

The default templates remain the current `Make + Model`, equivalent-focal-length EXIF row, and original-focal-length EXIF row, including their current enabled states and formatting strings.

### EXIF normalization

The Rust metadata adapter must reproduce the normalized values currently consumed by templates and the copy-EXIF action:

- Strip `CORPORATION` from maker names before vendor-specific formatting.
- Title-case the generic maker form while preserving the existing vendor-specific logo lookup key.
- Preserve Nikon model conversion, including `Z` to `ℤ` and numeric suffix to Roman numeral behavior.
- Preserve Sony `ILCE-` to lowercase `α` model behavior.
- Normalize fractional exposure time to the current `1/N` form when applicable.
- Normalize focal length and 35 mm equivalent focal length to the existing rounded display value.
- Normalize aperture, ISO, exposure compensation, exposure program, date/time, white balance, and metering mode to the strings currently used by templates.
- Return no EXIF object when every supported value is empty or zero.
- Apply EXIF orientation before calculating output geometry.

The Rust adapter may read additional tags internally, but only the established normalized fields cross the application boundary.

### Input and output contracts

- Accepted image formats are JPEG, PNG, and WebP, verified from file content rather than extension or browser MIME alone.
- Multiple images can be registered in one action.
- Parameter limits are the exact ranges listed in this specification.
- The rendered output is JPEG.
- Output density preserves the source density when available.
- Final export quality uses the configured quality, falling back to `100` under the current compatibility rule.
- Preview quality is `70`.
- Output uses the original file stem with a `.jpg` extension.
- On collision, naming remains `<stem>-<N>.jpg`, where `N` is one greater than the greatest matching numeric suffix found under the existing algorithm.
- Rendering writes to a same-directory temporary file and atomically renames only after a complete successful encode.
- Failure or cancellation removes temporary and cache artifacts and never replaces an existing successful output.
- The default output directory remains `Pictures/watermark`.

## Architecture

### Workspace and dependency direction

```text
Cargo.toml
crates/
  yiyin-domain/
  yiyin-application/
  yiyin-infrastructure/
src-tauri/
src/
```

The allowed dependency graph is:

```text
src-tauri                 -> yiyin-application
src-tauri                 -> yiyin-infrastructure
yiyin-infrastructure      -> yiyin-application
yiyin-infrastructure      -> yiyin-domain
yiyin-application         -> yiyin-domain
yiyin-domain              -> no project crate
```

No reverse edge is allowed. Cargo manifests are an architectural enforcement mechanism, not only package organization.

Every owned Rust crate uses `#![forbid(unsafe_code)]`. Production code does not use `unwrap()` or `expect()` for recoverable states. Tests may use them when failure is the assertion mechanism.

### `yiyin-domain`

The domain crate contains pure entities, value objects, invariants, state transitions, and renderer-independent plans.

Representative modules:

```text
src/
  config/
  template/
  resource/
  task/
  render/
  metadata/
  error.rs
```

Representative types include:

- `Config`, `RenderOptions`, `Template`, `TemplateField`, and normalized `Metadata` entities.
- `ResourceId`, `TaskId`, `OutputDirectory`, `ImageDimensions`, `ImageDensity`, `Quality`, and percentage newtypes.
- `TaskState` and legal state transitions.
- `RenderRequest` and `RenderPlan`.
- Text and logo slots, layout geometry, background strategy, corner radius, shadow plan, and output naming value objects.

The domain crate does not depend on Tauri, Tokio, serde, the filesystem, image codecs, font engines, UUID generation, or wall-clock APIs. It does not serialize itself or emit events. Invalid ranges and illegal task states are unrepresentable after construction.

### `yiyin-application`

The application crate defines use cases and the ports needed to execute them.

Inbound use cases:

- `Bootstrap`
- `UpdateConfig`
- `ResetConfig`
- `RegisterImages`
- `RegisterFont`
- `RemoveFont`
- `RegisterOverlay`
- `ReadTaskExif`
- `StartTasks`
- `PreviewTask`
- `CancelTask`
- `ClearTasks`

Outbound ports:

- `ConfigRepository`
- `ResourceRepository`
- `MetadataReader`
- `ImageRenderer`
- `TaskQueue`
- `TaskEventSink`
- `OutputDirectoryGateway`
- `Clock`
- `IdGenerator`

Use cases own orchestration and transaction boundaries. They do not know whether configuration is JSON, an image is decoded by the `image` crate, tasks are scheduled by Tokio, or events are delivered through Tauri.

Ports use native Rust traits. Synchronous local operations remain synchronous. The design does not add `async-trait`; asynchronous scheduling is owned by the adapter that already controls the runtime.

### `yiyin-infrastructure`

The infrastructure crate implements outbound ports and contains all platform-independent I/O details.

Representative modules:

```text
src/
  config/
  resources/
  metadata/
  rendering/
  tasks/
  filesystem/
```

Representative adapters:

- `JsonConfigRepository`
- `FileResourceRepository`
- `ExifMetadataReader`
- `CosmicTextImageRenderer`
- `TokioTaskQueue`
- `UuidGenerator`

This is the first layer permitted to depend on serde, serde_json, `image`, `kamadak-exif`, `cosmic-text`, Tokio, UUID, and concrete filesystem APIs. Adapter errors are translated into application errors before they cross inward.

### `src-tauri`

`src-tauri` is the Tauri inbound adapter and composition root.

```text
src/
  commands/
  dto/
  events/
  protocol/
  state.rs
  app.rs
  lib.rs
  main.rs
```

It owns:

- Tauri application and window lifecycle.
- Plugin registration.
- Managed state construction.
- Native dialogs, drag and drop, output-directory opening, window controls, and external URL opening.
- Command DTO deserialization and boundary validation.
- Explicit DTO-to-application and application-to-DTO mappings.
- Application error to stable IPC error mapping.
- The typed task-status event adapter.
- The `yiyin://resource/<opaque-id>` protocol.
- Capability files and Tauri configuration.

Commands are intentionally thin:

```text
deserialize -> validate boundary -> call use case -> map result
```

They contain no rendering, persistence, task scheduling, or template business logic.

### React presentation

React owns only presentation and short-lived interaction state.

```text
src/
  app/
  components/
  features/
  platform/
  styles/
```

- `platform` contains the only direct uses of `@tauri-apps/api`.
- A typed client wraps command names, request DTOs, responses, errors, and event subscription.
- Rust DTOs are the boundary source of truth. `ts-rs` is used only in test/dev code to generate and verify TypeScript boundary types.
- React does not duplicate domain rules, accept filesystem paths, read EXIF, render text into images, generate shadows, or create output files.
- React local state and `useReducer` handle ephemeral UI state. React Hook Form handles parameter and template editing transactions.
- No global client-state library is used. Persistent configuration and task truth remain in Rust.

React must be replaceable without changing the domain, application, or infrastructure crates.

## Tauri Boundary

### Commands

The command surface preserves behavior without copying Electron callback-shaped IPC:

- `bootstrap`
- `update_config`
- `reset_config`
- `choose_output_directory`
- `open_output_directory`
- `choose_images`
- `register_font`
- `remove_font`
- `register_overlay`
- `read_task_exif`
- `start_tasks`
- `preview_task`
- `cancel_task`
- `clear_tasks`
- `minimize_window`
- `close_window`
- `open_external_url`

Native drag and drop enters through Tauri window events and calls the same `RegisterImages` use case as `choose_images`. React receives task descriptors, never browser `File` objects or absolute paths.

The following Electron-era browser-render callbacks are removed rather than recreated:

- text-render request and completion callbacks
- shadow-render request and completion callbacks
- path joining
- arbitrary path information
- generic directory, shell, filesystem, or protocol access

All text rendering, logo loading, shadow creation, and composition execute in Rust.

### DTOs

DTOs belong exclusively to `src-tauri`. Inner crates do not derive serde traits for IPC convenience.

Boundary responses include:

- `BootstrapDto`: public configuration, resource descriptors, registered fonts, and task snapshot.
- `PublicConfigDto`: editable configuration without private paths.
- `TaskDescriptorDto`: opaque task ID, display name, current state, percentage, EXIF availability, preview resource, output resource, and safe failure.
- `ResourceDescriptorDto`: opaque ID, resource kind, and `yiyin://resource/<id>` URL.
- `MetadataDto`: the established normalized EXIF fields.
- `CommandErrorDto`: stable code and safe user-facing message.

Explicit `From`/`TryFrom` mappings form an anti-corruption layer. The small duplication between DTOs and inner models is intentional.

### Error contract

Commands return idiomatic `Result<T, CommandErrorDto>`. Stable error codes are:

```text
CANCELLED
CONFIG_INVALID
FILE_INVALID
FILE_NOT_FOUND
FORBIDDEN
INTERNAL
INVALID_REQUEST
RESOURCE_NOT_FOUND
TASK_NOT_FOUND
```

The mapping preserves the current distinction between success and command failure while replacing the generic Electron `0`/`500` envelope with typed codes. Font registration preserves its observable cases: incomplete request, duplicate name, missing source file, and success. React maps codes to the current user-facing messages.

Errors returned across IPC contain no absolute path, library error, backtrace, or sensitive platform detail. Detailed causal chains go only to the local Rust log.

### Task event

The WebView subscribes to one typed event named `task-status`.

The payload is a tagged union with:

- task ID
- `progress`, `completed`, `failed`, or `cancelled`
- integer percentage for progress
- output or preview resource descriptor for completion when applicable
- stable error code and safe message for failure

Images and binary content never travel in command responses or events. The event is read-only notification; command results and `bootstrap` remain the reconciliation source of truth.

### Resource protocol

Runtime resources use exactly:

```text
yiyin://resource/<opaque-id>
```

An opaque ID is a runtime capability, not a path encoding and not a durable cross-restart identifier.

On every request, the protocol adapter:

1. Parses the exact scheme, host, and single opaque path segment.
2. Rejects traversal, separators, percent-decoded path syntax, query-based path substitution, and unknown IDs.
3. Looks up the ID in the Rust-owned registry.
4. Re-canonicalizes the target.
5. Verifies that it remains inside its registered allowlisted root.
6. Verifies the registered resource kind and permitted MIME type.
7. Serves it with a fixed content type and restrictive response headers.

The protocol serves bundled static assets, registered custom fonts and overlays, controlled previews, and other explicitly registered application resources. It cannot read arbitrary local files.

## Rendering Design

### Frozen render request

Registering an image creates a task descriptor. Starting or previewing it freezes a complete `RenderRequest`, including configuration, templates, resource references, normalized metadata, output directory, and collision-resolved destination. Later configuration edits do not change an in-flight request.

### Pipeline

Each preview and export uses the same renderer:

1. Validate the registered resource, file signature, and supported format.
2. Decode JPEG, PNG, or WebP.
3. Apply EXIF orientation.
4. Normalize metadata.
5. Build a pure `RenderPlan` in the domain crate.
6. Produce either the configured solid background or a stretched, blurred source-image background.
7. Shape and rasterize template text with `cosmic-text`.
8. Load registered logo/image slots.
9. Compose main image, overlay, shadow, rounded mask, text rows, and bottom spacing.
10. Encode JPEG with source density and selected quality.
11. Atomically publish the result or register the preview resource.
12. Clean temporary data and emit the terminal task status.

Preview differs only in destination, cancellation policy, and quality `70`. A new preview request cancels the previous preview. It writes to controlled cache and returns a registered resource URL instead of a data URL.

### Exact geometry rules

The domain `RenderPlan` preserves the current calculation order and rounding behavior:

- Explicit background ratio adjusts the reset dimension based on original orientation.
- Landscape mode swaps reset width and height only for portrait images and is inactive while explicit background ratio controls geometry.
- Background width and height scale from the selected ratio and content height.
- If the main image would exceed `main_img_w_rate` of background width, the background expands proportionally.
- Main image is centered horizontally.
- Minimum top spacing is `background_height * mini_top_bottom_margin / 100`.
- When shadow is enabled, top spacing is at least `ceil(main_image_height * shadow / 100)`.
- With text rows, main-image vertical spacing is reduced to three quarters of the no-text value and the existing `2.7%` bottom text offset is added.
- Text rows stack from the bottom in template order using their calculated heights.
- The final content height is the ceiling of text height, main-image height, and vertical spacing.
- The final background recenters the complete content block vertically.
- Shadow and radius are calculated from main-image height using the current percentage rules.
- The shadow intermediate surface is limited to width `10240` and is scaled back to final dimensions.
- The existing brightness thresholds and overlay opacity behavior for non-solid backgrounds are retained as compatibility rules.

Integer rounding is part of the geometry contract and is covered by domain tests against recorded old-renderer plans.

### Text rendering

Text rendering preserves:

- Template order and conditional omission.
- Field-level and template-level font merging.
- Case conversion.
- Baseline and center alignment for text/image mixtures.
- Light and dark logo variants based on background mode.
- Font size as a percentage of background height.
- Text margin as a percentage of background height.
- Bundled font preference, registered custom fonts, and deterministic fallback.

Golden tests use repository-owned fixed fonts. System fonts are permitted for interactive fallback but are not used as golden-test dependencies.

### Renderer compatibility policy

Exact requirements:

- Output dimensions and orientation.
- Main-image, text-row, logo, margin, radius, and shadow geometry.
- Template substitution and EXIF values.
- Output naming and collision behavior.
- JPEG quality selection and density behavior.
- Task progress ordering and terminal states.

Perceptual requirements:

- Color and composition remain visually equivalent.
- Text and logo placement stay within exact geometry while glyph edges and metrics may differ within a documented tolerance.
- Blur and shadow appearance may differ within a documented perceptual tolerance.
- JPEG bytes and decoded pixels need not be identical.

Before the old renderer is removed, its output is captured for fixed fixtures covering landscape, portrait, EXIF rotation, explicit ratio, solid and blurred backgrounds, radius, shadow, every built-in template mode, logos, and custom fonts. Test failures produce a human-inspectable diff artifact.

## Task Scheduling

- The queue defaults to concurrency `2`, preserving the current product behavior.
- The queue does not begin export until `start_tasks` is called unless quick output is enabled.
- CPU-intensive image work runs on controlled blocking workers and does not block the Tauri async runtime.
- Tokio coordinates cancellation, task state, and concurrency. Rayon is not added.
- Every task follows legal domain transitions such as registered, queued, running, completed, failed, or cancelled.
- Progress preserves the current observable milestones: `1`, `10`, `20`, `30`, `50`, `60`, `70`, `90`, and `100` where the new stage mapping has an equivalent.
- Cancelling a preview is best-effort at stage boundaries and prevents a stale preview from replacing the current selection.
- Clearing removes queued tasks and the UI snapshot; it does not delete successful exported images.
- Application shutdown requests cancellation, waits for bounded cleanup, and prevents new work.

## Persistence and Legacy Import

### New storage

- Configuration is stored in Tauri app data as versioned JSON.
- Registered fonts and overlay resources are copied into application-owned directories and referred to by opaque IDs.
- Preview and intermediate data live in application cache.
- Logs live in application log data.
- No private storage path is returned to React.

### Durable writes

For every configuration write:

1. Validate and normalize into a complete domain config.
2. Serialize to a same-directory temporary file.
3. Flush and sync the temporary file.
4. Preserve the previous valid file as `.bak`.
5. Atomically rename the temporary file.
6. Sync the directory where the platform supports it.

Invalid existing JSON is preserved as `.invalid.bak` before defaults are recovered. A failed write leaves the previous valid config available.

### One-time Electron import

On first Tauri startup only, when no Tauri configuration has been committed:

- Locate the legacy Electron user-data directory through platform-specific known locations for the existing application identity.
- Import `config.json`, font metadata, font files, and static/template image resources.
- Apply ordered schema migrations and current normalization rules.
- Copy required data into Tauri-owned storage.
- Record migration completion only after the new configuration and resources are durable.
- Never modify, rename, or delete any legacy source file.
- Re-running after a partial failure is idempotent.

The importer retains user output-directory selection, option values, fields, templates, order, and custom resources. Unsupported or corrupt individual resources produce a safe migration warning without destroying other valid imported state.

## Native Integration

- The Rust dialog plugin is used only behind dedicated commands. Its generic JavaScript API is not exposed.
- The Rust opener plugin is used only behind the output-directory and approved-external-link commands. Its generic JavaScript API is not exposed.
- Native drag/drop paths are consumed immediately by Rust registration and are never forwarded to React.
- Window minimize, close, and focus behavior is implemented in Rust commands and lifecycle hooks.
- External URL opening uses an exact allowlist for the existing repository, issues, release, Bilibili profile, and Bilibili feedback destinations. The allowed origins are `https://github.com` and `https://space.bilibili.com` plus the exact existing `https://message.bilibili.com/#/whisper/mid94829489` destination; path rules restrict the GitHub origin to the Yiyin repository and its issue/release pages. No caller-supplied arbitrary URL is opened.
- The single-instance plugin is registered first, and the second-instance callback restores and focuses `main`.

## Security

### WebView boundary

- `withGlobalTauri` is disabled.
- React can access only the typed wrapper around approved commands and the task-status event.
- No remote script, remote style, `eval`, or arbitrary network connection is allowed.
- Production Content Security Policy permits only packaged application assets and the custom resource protocol required by the UI.
- Development policy separately permits only the loopback Vite origin needed for local development.
- Navigation and new-window requests are denied unless transformed into an allowlisted Rust external-open command.
- Production DevTools are disabled.

The old remote release check is removed. The footer retains the current version text and its explicit release-page interaction, but the application does not call the GitHub API, fetch update metadata, or display a remotely discovered version.

### Capabilities

Capabilities are minimal and custom. They grant only the Tauri primitives required for the dedicated Rust-owned behavior and the application's own commands. React does not receive generic filesystem, shell, HTTP, dialog, opener, or broad protocol permissions.

### Filesystem and resources

- All untrusted paths are canonicalized and containment-checked at the last responsible moment.
- File type is verified from content.
- Configuration and output writes use controlled directories and atomic publication.
- Resource IDs are unguessable opaque UUIDs with registry metadata.
- Protocol lookup cannot turn an ID into an arbitrary path.
- Errors and events do not leak private paths.

## Dependency Baseline

All listed npm dependencies are exact pins. Cargo dependencies are locked in `Cargo.lock`, and direct dependency versions are constrained to the approved stable release. Exact releases and the 24-hour age policy must be revalidated from authoritative registries before implementation begins.

### Toolchains

- Rust `1.97.0`, pinned by `rust-toolchain.toml`.
- Node.js `24`.
- pnpm `11.13.0`.

### Frontend runtime

- `react@19.2.7`
- `react-dom@19.2.7`
- `@tauri-apps/api@2.11.1`
- `@base-ui/react@1.6.0`
- `react-hook-form@7.81.0`

### Frontend tooling and tests

- `vite@8.1.4`
- `@vitejs/plugin-react@6.0.3`
- `typescript@5.9.3`
- `@types/react@19.2.17`
- `@types/react-dom@19.2.3`
- `@biomejs/biome@2.5.4`
- `vitest@4.1.10`
- `@testing-library/react@16.3.2`
- `@testing-library/dom@10.4.1`
- `jsdom@29.1.1`
- `@playwright/test@1.61.1`
- `@tauri-apps/cli@2.11.4`

### Rust runtime

- `tauri@2.11.5`
- `tauri-build@2.6.3`
- `tauri-plugin-dialog@2.7.1`
- `tauri-plugin-opener@2.5.4`
- `tauri-plugin-log@2.9.0`
- `tauri-plugin-single-instance@2.4.2`
- `log@0.4.33`
- `serde@1.0.228`
- `serde_json@1.0.150`
- `thiserror@2.0.18`
- `tokio@1.52.3`
- `uuid@1.23.5`
- `image@0.25.10` with default features disabled and only `jpeg`, `png`, and `webp`
- `kamadak-exif@0.6.1`
- `cosmic-text@0.19.0`

### Development-only Rust dependencies

- `ts-rs@12.0.1`
- `tempfile@3.27.0`

### Explicit exclusions

Do not add the following without a later approved design change:

- Zustand
- Tailwind CSS
- shadcn/ui
- Sass
- Zod or React Hook Form resolvers
- Radix
- React Router
- TanStack Query
- Axios
- Sonner
- Lucide
- Electron
- Svelte
- Sharp
- `db-ui`
- Node sidecars
- `imageproc`
- Rayon
- `async-trait`
- WebdriverIO or Selenium npm packages
- prerelease dependencies
- Git dependencies

CSS uses ordinary CSS and existing visual assets. Base UI supplies only the complex unstyled accessible interactions that the product actually needs.

## Testing Strategy

### Domain

- Value-object range validation.
- Template parsing, substitution, and omission.
- EXIF normalized-value consumption.
- Output naming collisions.
- Geometry and rounding fixtures.
- Task state transitions and cancellation.

### Application

Handwritten fake ports test each use case without Tauri or real filesystem dependencies, including:

- bootstrap and legacy-import outcomes
- valid and invalid configuration updates
- image registration and unsupported input
- font and overlay registration
- explicit and quick-output task starts
- frozen configuration per task
- preview supersession
- cancellation, failure, and clear semantics
- safe error mapping inputs

### Infrastructure

- Versioned JSON migration, invalid backup, durable write, and recovery.
- Legacy import idempotency and source preservation.
- Filesystem containment and symlink/race revalidation.
- Real JPEG, PNG, and WebP decoding with EXIF orientation.
- EXIF normalization fixtures for generic, Nikon, and Sony data.
- Font and overlay resource registration.
- Old-renderer golden compatibility, exact geometry, perceptual comparison, and diff artifacts.
- Atomic output publication and cleanup after failure or cancellation.

### Tauri adapter

- DTO conversions and `ts-rs` drift check.
- Command boundary validation.
- Stable error-code mapping and redaction.
- Capability allowlist and absence of generic plugin grants.
- Custom protocol parsing, registry lookup, containment, MIME type, and unknown-ID behavior.
- Single-instance registration order and focus callback.
- Production configuration disables DevTools and remote access.

### React

- Biome format and lint.
- Strict TypeScript with `noUncheckedIndexedAccess`.
- Vitest and Testing Library component and workflow tests using a typed fake Tauri adapter.
- Browser Playwright parity tests for the current layout and workflows, without pretending the browser is the native desktop runtime.
- User-visible errors and task reconciliation from command results plus events.

### Desktop and packaging

- macOS: build unsigned package, launch, observe main window/process, close cleanly, and inspect the bundle.
- Windows: build unsigned package and run `tauri-driver` smoke through a minimal internal W3C client.
- The internal W3C client is test code, not a new npm WebDriver framework dependency.
- Local macOS manual acceptance covers drag/drop, parameter drawer, template drawer, preview, export, output directory, and single-instance focus.
- Bundle inspection proves that Electron, Node, Sharp, Svelte, `db-ui`, sidecars, and forbidden remote assets are absent.

## CI and Supply Chain

Required CI jobs:

1. Frontend quality.
2. Rust formatting, Clippy, and tests.
3. Security and dependency policy.
4. Golden rendering compatibility.
5. macOS package smoke.
6. Windows package and `tauri-driver` smoke.

The checks include:

- frozen pnpm installation
- exact npm dependency pins
- `pnpm audit --audit-level high` over the full dependency graph
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --locked`
- `cargo deny check`
- Cargo advisory, license, source, and duplicate checks
- a 24-hour minimum release-age policy for npm and Cargo dependencies
- rejection of Git and prerelease dependencies
- CodeQL for both `javascript-typescript` and `rust`
- GitHub Actions pinned to full commit SHAs
- Dependabot groups separated for pnpm, Cargo, and GitHub Actions
- no Dependabot auto-merge

Manual packaging workflows upload short-lived private artifacts only. They do not create releases.

## Packaging

- Product name remains `壹印`.
- Application identifier is `io.github.gaotity.yiyin`.
- Version remains `1.6.0` for the migration.
- License metadata and repository license become `GPL-3.0-only`.
- Existing macOS and Windows icons are retained.
- macOS and Windows packages are unsigned.
- Artifact naming preserves the existing `yiyin-<version>-<platform>-<arch>.<ext>` convention where Tauri packaging permits it.
- No updater, signing, notarization, release upload, or publishing credential is configured.

## Cutover

The rewrite occurs in one PR from a clean `origin/main` base:

1. Capture old-renderer fixtures before deleting the old renderer.
2. Establish the Cargo workspace, Tauri composition root, typed boundary, and React shell.
3. Implement and validate inner layers, adapters, renderer, persistence, UI parity, security, and packaging.
4. Remove Electron, Svelte, Sharp, FFmpeg, ExifTool, `db-ui`, old build scripts, old workflows, and obsolete generated logger code in the same cutover.
5. Verify that only the new runtime and compatibility fixtures remain.

The implementation plan will define a safe test-driven sequence, but there will be no committed hybrid release or compatibility sidecar.

## Delivery Governance

- The implementation remains on the short-lived `codex/tauri-rewrite` branch based on `origin/main`.
- The design specification is committed independently before the implementation plan or code.
- The rewrite is delivered through a new pull request with an English Conventional Commit-style title and an assignee.
- Pull request descriptions, ordinary comments, reviews, and status updates use block-separated English followed by Simplified Chinese. Repository documentation, code, identifiers, commit messages, and the pull request title remain English.
- The main task takes at most one immediate checks snapshot after creating or updating the pull request and does not run a long watch loop.
- Passing checks, review approval, and push completion do not authorize merging or releasing.

## Acceptance Criteria

The rewrite is complete only when:

- Cargo dependency direction matches this document and prohibited dependencies are absent.
- Rust owns all image, text, shadow, EXIF, resource, persistence, task, dialog, filesystem, and output behavior.
- React has no path, file, binary rendering, or generic platform authority.
- Existing configuration and user resources import once without source mutation.
- Product text, layout, and interactions pass parity tests and manual acceptance.
- Geometry and behavioral golden contracts pass exactly.
- Perceptual rendering comparisons pass their documented tolerances with deterministic fonts.
- Output naming, quality, density, and atomic-write behavior match the compatibility baseline.
- The resource protocol exposes only registered opaque capabilities.
- Task concurrency, progress, preview cancellation, success, failure, cancellation, and clear behavior match the contract.
- macOS and Windows unsigned packaging smoke tests pass.
- Security, CodeQL, audit, cargo-deny, lockfile, action pinning, and release-age checks pass.
- Package inspection finds no Electron, Node, Sharp, Svelte, `db-ui`, sidecar, forbidden generic capability, or remote runtime content.
- No updater, telemetry, signing, publishing, network service, or speculative product feature is introduced.

## Approved Decision Record

- A clean one-PR cutover was selected over dual runtime and staged shell/UI migrations.
- Rust owns the product; React is presentation only.
- A multi-crate Clean Architecture was selected as a production-grade, compiler-enforced reference architecture.
- Native Rust traits are preferred over abstraction-support dependencies.
- Perceptual renderer compatibility was accepted in place of pixel identity.
- The dependency baseline prioritizes minimum dependency and supply-chain surface, then ecosystem maturity and developer experience, then performance and build speed.
- Single-instance behavior, minimal capabilities, opaque resources, and Rust-owned native integration are mandatory.
- The migration stays on version `1.6.0`, produces unsigned packages, and does not publish or merge automatically.
