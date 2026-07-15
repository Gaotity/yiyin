# Tauri and Rust Rewrite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task under the active DIRECT contract. Do not create subagents in this task; subagent-driven execution requires an explicitly created fresh MANAGED task.

**Goal:** Replace the Electron and Svelte application with a behavior-compatible Tauri 2, Rust, and React desktop application in one clean cutover.

**Architecture:** A Cargo workspace enforces Clean Architecture: `yiyin-domain` contains pure rules, `yiyin-application` contains use cases and ports, `yiyin-infrastructure` implements outbound adapters, and `src-tauri` is the Tauri inbound adapter and composition root. React is a thin presentation client over generated DTO types, dedicated commands, one task event, and the opaque `yiyin://resource/<id>` protocol.

**Tech Stack:** Rust 1.97.0, Tauri 2.11.5, Tokio, `image`, `kamadak-exif`, `cosmic-text`, React 19.2.7, TypeScript 5.9.3, Vite 8.1.4, Base UI, React Hook Form, Biome, Vitest, Testing Library, Playwright, cargo-deny, and CodeQL.

**Approved design:** `docs/superpowers/specs/2026-07-15-tauri-rust-rewrite-design.md`

## Global Constraints

- Work only on `codex/tauri-rewrite`, whose base is `origin/main`; never touch or rely on the original checkout's abandoned work.
- Preserve product name `壹印`, application identifier `io.github.gaotity.yiyin`, version `1.6.0`, fixed production window `900 x 730`, current Simplified Chinese copy, layout, and interactions.
- Change repository and package license metadata to `GPL-3.0-only` in the final cutover.
- Rust owns image processing, text rendering, shadows, EXIF, resources, persistence, task scheduling, dialogs, filesystem access, and output generation. React never receives paths or generic platform authority.
- Preserve configuration and template semantics, normalized EXIF fields, JPEG/PNG/WebP input, parameter limits, output geometry, `.jpg` naming/collision behavior, quality, density, task concurrency `2`, progress milestones, and preview supersession.
- Preserve the default output directory `Pictures/watermark` and any valid imported user-selected output directory.
- Preserve `yiyin://resource/<opaque-id>`, the stable error codes, and one `task-status` event.
- Rendering geometry is exact. Sharp/Canvas versus Rust pixels are perceptually compatible, measured with deterministic legacy fixtures and committed diff thresholds.
- Register `tauri-plugin-single-instance@2.4.2` before every other plugin and focus/restore the existing main window on second launch.
- Expose no generic dialog, opener, filesystem, shell, HTTP, or arbitrary protocol API to React. Production has no remote code, `eval`, arbitrary network, or DevTools.
- Use the exact dependency pins in the approved design. Reverify registry age and compatibility immediately before changing manifests; stop for design review if a pin is unavailable, less than 24 hours old, or incompatible.
- Do not add Zustand, Tailwind, shadcn/ui, Sass, Zod/resolvers, Radix, Router, Query, Axios, Sonner, Lucide, Electron, Sharp, Svelte, `db-ui`, a Node sidecar, `imageproc`, Rayon, `async-trait`, WebdriverIO, Selenium, prerelease dependencies, or Git dependencies to the final tree.
- Keep `ts-rs@12.0.1` and `tempfile@3.27.0` development-only.
- Pin GitHub Actions to full commit SHAs, use frozen pnpm and locked Cargo builds, enforce a 24-hour release-age policy, and separate Dependabot ecosystems without auto-merge.
- Do not merge, publish, sign, notarize, add telemetry, add an updater, or create a network service.
- Every task below ends with focused verification and a separate English Conventional Commit.

## Target File Map

### Root and governance

- `Cargo.toml`: workspace members and shared package/lint metadata.
- `rust-toolchain.toml`: Rust 1.97.0 minimal profile and required components.
- `deny.toml`: advisory, license, source, and duplicate-crate policy.
- `package.json`, `pnpm-lock.yaml`, `pnpm-workspace.yaml`: exact frontend/tooling dependency graph and scripts.
- `biome.json`, `tsconfig.json`, `vite.config.mjs`, `index.html`: React build and quality configuration. The JavaScript Vite config avoids adding Node type declarations to the application TypeScript graph.
- `.github/dependabot.yml`: separate pnpm, Cargo, and Actions update groups without auto-merge.
- `.github/workflows/ci.yml`, `.github/workflows/codeql.yml`, `.github/workflows/package.yml`: required quality, security, rendering, and desktop jobs.
- `scripts/check-release-age.mjs`: npm and Cargo 24-hour release-age enforcement.
- `scripts/verify-bundle.mjs`: identity, bundle, and forbidden-payload inspection.
- `scripts/w3c-smoke.mjs`: minimal Windows W3C client for `tauri-driver`.

### Domain crate

- `crates/yiyin-domain/src/config.rs`: validated configuration and rendering-option value objects.
- `crates/yiyin-domain/src/template.rs`: fields, templates, substitution, font merging, and row plans.
- `crates/yiyin-domain/src/metadata.rs`: normalized EXIF model.
- `crates/yiyin-domain/src/resource.rs`: opaque resource and task identifiers and resource kinds.
- `crates/yiyin-domain/src/render.rs`: image dimensions, output naming, layout geometry, and `RenderPlan`.
- `crates/yiyin-domain/src/task.rs`: task state machine, progress stages, and frozen `RenderRequest`.
- `crates/yiyin-domain/src/error.rs`: domain validation errors.

### Application crate

- `crates/yiyin-application/src/models.rs`: application snapshots and safe outcomes.
- `crates/yiyin-application/src/ports.rs`: object-safe inbound dependencies and outbound ports.
- `crates/yiyin-application/src/error.rs`: stable application error categories.
- `crates/yiyin-application/src/use_cases/bootstrap.rs`: startup and migration orchestration.
- `crates/yiyin-application/src/use_cases/config.rs`: update and reset use cases.
- `crates/yiyin-application/src/use_cases/resources.rs`: image, font, overlay, and metadata use cases.
- `crates/yiyin-application/src/use_cases/tasks.rs`: start, preview, cancel, and clear use cases.

### Infrastructure crate

- `crates/yiyin-infrastructure/src/config/json_repository.rs`: durable JSON persistence.
- `crates/yiyin-infrastructure/src/config/legacy_import.rs`: idempotent Electron import.
- `crates/yiyin-infrastructure/src/filesystem.rs`: atomic writes and containment-safe helpers.
- `crates/yiyin-infrastructure/src/resources.rs`: resource registry and owned resource storage.
- `crates/yiyin-infrastructure/src/metadata.rs`: `kamadak-exif` extraction and normalization.
- `crates/yiyin-infrastructure/src/rendering/background.rs`: solid and blurred backgrounds.
- `crates/yiyin-infrastructure/src/rendering/text.rs`: `cosmic-text` shaping and rasterization.
- `crates/yiyin-infrastructure/src/rendering/composite.rs`: rounded mask, shadow, composition, and JPEG encoding.
- `crates/yiyin-infrastructure/src/rendering/mod.rs`: complete renderer adapter.
- `crates/yiyin-infrastructure/src/tasks.rs`: concurrency-two Tokio queue, cancellation, and events.

### Tauri adapter

- `src-tauri/src/dto/*`: serde/`ts-rs` command and event boundary types.
- `src-tauri/src/commands/*`: thin bootstrap, config, resource, task, native, and window commands.
- `src-tauri/src/error.rs`: application-to-IPC error mapping and redaction.
- `src-tauri/src/events.rs`: the single typed `task-status` event sink.
- `src-tauri/src/protocol.rs`: `yiyin://resource/<id>` parser and responder.
- `src-tauri/src/native.rs`: dedicated dialog, opener, drag/drop, external-link, and window operations.
- `src-tauri/src/state.rs`: injected application services and registries.
- `src-tauri/src/app.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`: plugin order, composition, and entrypoints.
- `src-tauri/capabilities/main.json`, `src-tauri/tauri.conf.json`: minimal capability and package policy.
- `src-tauri/README.md`: architecture navigation and Rust contributor workflow.

### React presentation

- `src/platform/types.ts`: generated DTO types checked into the repository.
- `src/platform/client.ts`: the only direct Tauri invoke/listen wrapper.
- `src/platform/fake.ts`: typed browser test adapter.
- `src/app/App.tsx`, `src/app/useAppController.ts`: composition and bootstrap reconciliation.
- `src/features/chrome/*`: title bar, help, feedback, footer, and window controls.
- `src/features/settings/*`: rendering settings, custom fields, templates, and font transactions.
- `src/features/tasks/*`: image registration, selection, EXIF, queue status, preview, and output actions.
- `src/components/*`: narrow Base UI wrappers with owned CSS.
- `src/styles/*`: extracted visual tokens and parity styles.

### Compatibility fixtures

- `tests/fixtures/input/*`: deterministic JPEG, PNG, WebP, orientation, and font inputs.
- `tests/fixtures/legacy/manifest.json`: scenario configuration, exact geometry, density, and perceptual thresholds.
- `tests/fixtures/legacy/output/*.jpg`: old-renderer reference outputs.
- `tests/fixtures/legacy/metadata/*.json`: normalized old-renderer metadata.

---

### Task 1: Capture and Freeze Legacy Compatibility Fixtures

**Files:**
- Create: `tests/legacy/capture.spec.ts`
- Create: `tests/legacy/scenarios.ts`
- Create: `tests/fixtures/legacy/manifest.json`
- Create: `tests/fixtures/legacy/output/*.jpg`
- Create: `tests/fixtures/legacy/metadata/*.json`
- Modify temporarily: `package.json`
- Modify temporarily: `electron/main/app.ts`

**Interfaces:**
- Produces: `LegacyScenario { id, input, options, templates, expected_output, expected_metadata, exact_geometry, perceptual_threshold }` consumed by Rust golden tests.
- Produces: immutable binary outputs from the current Sharp/Canvas renderer; later tasks must never regenerate them with Rust.

- [ ] **Step 1: Add deterministic fixture mode behind a test-only environment guard**

Add `node:fs` and `node:path` imports plus `YIYIN_FIXTURE_MODE=1` handling in `electron/main/app.ts` that disables remote version checks and accepts a fixture output directory. The guard must throw in packaged production when the variable is present, and normal startup must remain unchanged.

```ts
if (app.isPackaged && process.env.YIYIN_FIXTURE_MODE) {
  throw new Error('Fixture mode is unavailable in packaged builds')
}

const fixtureMode = !app.isPackaged && process.env.YIYIN_FIXTURE_MODE === '1'

if (fixtureMode) {
  const output = process.env.YIYIN_FIXTURE_OUTPUT
  if (!output) throw new Error('YIYIN_FIXTURE_OUTPUT is required')
  config.output = path.resolve(output)
  config.cacheDir = path.join(config.output, '.cache')
  fs.mkdirSync(config.cacheDir, { recursive: true })
}

if (!fixtureMode) {
  this.win.on('ready-to-show', () => this.checkAssetsUpdate())
}
```

- [ ] **Step 2: Define the exact scenario matrix**

In `tests/legacy/scenarios.ts`, export exactly these scenario IDs: `portrait-default`, `landscape-default`, `webp-default`, `exif-orientation-6`, `explicit-ratio-3x2`, `portrait-to-landscape`, `solid-white-no-shadow`, `blurred-shadow-radius`, `built-in-equivalent-focal`, `built-in-original-focal`, `logo-light`, `logo-dark`, `custom-text-forced`, and `bundled-custom-font`. Start from `static/最终效果.jpg`, `static/最终效果.png`, and `static/最终效果-竖转横.jpeg`; use the current Sharp dependency to create deterministic JPEG/PNG/WebP copies. After the legacy Vite build extracts its bundled ExifTool, write orientation, density, generic, Nikon, and Sony tags into dedicated copies and verify them with the old `getExitInfo` route. Copy `千图小兔体.ttf` into the fixture directory during capture.

```ts
export interface LegacyScenario {
  id: string
  input: string
  options: Record<string, unknown>
  templateKeys: string[]
  perceptualThreshold: { minSsim: number; maxChangedPixelRatio: number }
}

export const legacyScenarios: LegacyScenario[] = [
  { id: 'portrait-default', input: 'static/最终效果.png', options: {}, templateKeys: ['make-model', 'exif-params'], perceptualThreshold: { minSsim: 0.97, maxChangedPixelRatio: 0.08 } },
  { id: 'blurred-shadow-radius', input: 'static/最终效果.jpg', options: { bg_blur: 100, radius: 2.1, shadow: 6, shadow_show: true }, templateKeys: ['make-model', 'exif-params'], perceptualThreshold: { minSsim: 0.97, maxChangedPixelRatio: 0.08 } },
  { id: 'portrait-to-landscape', input: 'static/最终效果-竖转横.jpeg', options: { landscape: true }, templateKeys: ['make-model', 'exif-params'], perceptualThreshold: { minSsim: 0.97, maxChangedPixelRatio: 0.08 } },
]
```

The exported array contains one explicit entry for every ID above; use `{ minSsim: 0.97, maxChangedPixelRatio: 0.08 }` for each visual output, and record exact geometry/density independently. Do not hide multiple geometry dimensions inside one aggregate scenario.

- [ ] **Step 3: Write the Playwright Electron capture test**

Add temporary exact dev dependency `@playwright/test@1.61.1`. Use Playwright's Electron launcher, wait for the existing renderer, call only the existing `window.api` boundary, and wait for progress `100` or failure. For each scenario, load the current config, merge the scenario options and template selections into a complete request, register one input, start the queue, copy normalized EXIF, and record output dimensions and density.

```ts
test('captures every legacy scenario', async () => {
  for (const scenario of legacyScenarios) {
    const result = await page.evaluate(async ({ scenario }) => {
      const current = await window.api.getConfig()
      if (current.code !== 0) throw new Error(current.message)
      await window.api.setConfig({
        ...current.data,
        options: { ...current.data.options, ...scenario.options },
        temps: current.data.temps.map((item) => ({ ...item, use: scenario.templateKeys.includes(item.key) })),
      })
      const name = scenario.input.split(/[\\/]/).at(-1) ?? scenario.input
      const added = await window.api.addTask([{ path: scenario.input, name }])
      if (added.code !== 0) throw new Error(added.message)
      await window.api.startTask()
      return added.data[0]
    }, { scenario })
    expect(result.id).toBeTruthy()
  }
})
```

- [ ] **Step 4: Run capture and verify repeatability**

Run the legacy build and capture twice into two temporary directories, then compare SHA-256 for metadata/manifests and decoded dimensions for images.

```bash
pnpm install --lockfile-only
pnpm install --frozen-lockfile
pnpm exec vite build
YIYIN_FIXTURE_MODE=1 YIYIN_FIXTURE_OUTPUT="$PWD/tests/fixtures/legacy/output" pnpm exec playwright test tests/legacy/capture.spec.ts
```

Expected: all scenarios complete, every reference JPEG opens successfully, exact dimensions/density are present, and a second run produces the same manifest and geometry. JPEG byte hashes may differ and are not used as compatibility assertions.

- [ ] **Step 5: Commit only fixture inputs, outputs, manifest, and temporary capture harness**

```bash
git add package.json pnpm-lock.yaml electron/main/app.ts tests/legacy tests/fixtures
git commit -m "test: capture legacy rendering fixtures"
```

### Task 2: Establish Toolchains, Workspace Boundaries, and Frontend Shell

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `deny.toml`
- Create: `crates/yiyin-{domain,application,infrastructure}/Cargo.toml`
- Create: `crates/yiyin-{domain,application,infrastructure}/src/lib.rs`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/src/{lib.rs,main.rs}`
- Create: `pnpm-workspace.yaml`, `biome.json`, `index.html`, `src/main.tsx`
- Replace: `package.json`, `tsconfig.json`
- Create: `vite.config.mjs`
- Test: `tests/architecture/dependency-boundaries.sh`

**Interfaces:**
- Produces: Cargo dependency direction described in the design.
- Produces: React build entry and exact dependency baseline used by every later task.

- [ ] **Step 1: Bootstrap the approved Rust toolchain without changing the design**

If `rustup` is absent, install it from the official `https://sh.rustup.rs` endpoint with the minimal profile and no default toolchain. Then install exactly Rust 1.97.0 with rustfmt and Clippy. Use Node 24's Corepack entrypoint for pnpm 11.13.0 rather than installing a floating global pnpm.

```bash
command -v rustup >/dev/null || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain none
rustup toolchain install 1.97.0 --profile minimal --component rustfmt --component clippy
rustup run 1.97.0 rustc --version
corepack pnpm@11.13.0 --version
```

Expected: `rustc 1.97.0`, rustfmt and Clippy are available, and pnpm reports `11.13.0`.

- [ ] **Step 2: Reverify all approved dependency pins and release age**

Use official registries. For npm, record `version` and `time`; for Cargo, use crates.io metadata. Do not edit manifests until every direct dependency is stable and older than 24 hours.

```bash
pnpm view react@19.2.7 version time --json
pnpm view @tauri-apps/cli@2.11.4 version time --json
cargo search tauri --limit 1
cargo info tauri@2.11.5
```

Expected: exact versions exist and satisfy the age policy. Any mismatch stops execution for user-approved design adjustment.

- [ ] **Step 3: Write the boundary test first**

`tests/architecture/dependency-boundaries.sh` must assert that domain has no non-workspace dependencies, application depends only on domain, infrastructure depends inward, and `src-tauri` is the only Tauri consumer.

```bash
#!/usr/bin/env bash
set -euo pipefail
metadata="$(cargo metadata --format-version 1 --no-deps)"
jq -e '.packages[] | select(.name == "yiyin-domain") | .dependencies | length == 0' <<<"$metadata"
jq -e '.packages[] | select(.name == "yiyin-application") | [.dependencies[].name] == ["yiyin-domain"]' <<<"$metadata"
! rg -n 'tauri|serde|image|tokio' crates/yiyin-domain crates/yiyin-application
```

- [ ] **Step 4: Run the boundary test to verify RED**

```bash
bash tests/architecture/dependency-boundaries.sh
```

Expected: FAIL because the Cargo workspace does not exist.

- [ ] **Step 5: Create exact manifests and crate stubs**

Set workspace resolver `2`, shared edition `2024`, Rust version `1.97.0`, `unsafe_code = "forbid"`, and warning lints. Keep domain dependency-free and application dependent only on domain. Put serde, image, EXIF, text, Tokio, and UUID only in infrastructure or Tauri as approved.

Use these exact Rust pins: `tauri@2.11.5`, `tauri-build@2.6.3`, `tauri-plugin-dialog@2.7.1`, `tauri-plugin-opener@2.5.4`, `tauri-plugin-log@2.9.0`, `tauri-plugin-single-instance@2.4.2`, `log@0.4.33`, `serde@1.0.228`, `serde_json@1.0.150`, `thiserror@2.0.18`, `tokio@1.52.3`, `uuid@1.23.5`, `image@0.25.10`, `kamadak-exif@0.6.1`, and `cosmic-text@0.19.0`. Configure `image` with default features disabled and only `jpeg`, `png`, and `webp`. Keep `ts-rs@12.0.1` and `tempfile@3.27.0` in dev dependencies only. Cargo manifests use exact requirements (`=version`) for direct third-party crates, and `Cargo.lock` freezes the full graph.

```toml
[workspace]
members = ["crates/yiyin-domain", "crates/yiyin-application", "crates/yiyin-infrastructure", "src-tauri"]
resolver = "2"

[workspace.package]
version = "1.6.0"
edition = "2024"
rust-version = "1.97.0"
license = "GPL-3.0-only"

[workspace.lints.rust]
unsafe_code = "forbid"
```

Name the `src-tauri` package and library `yiyin-desktop`; all later `cargo test -p yiyin-desktop` commands depend on that exact name.

- [ ] **Step 6: Replace frontend manifests and create the empty React entry**

Use exact runtime pins `react@19.2.7`, `react-dom@19.2.7`, `@tauri-apps/api@2.11.1`, `@base-ui/react@1.6.0`, and `react-hook-form@7.81.0`. Use exact tooling pins `vite@8.1.4`, `@vitejs/plugin-react@6.0.3`, `typescript@5.9.3`, `@types/react@19.2.17`, `@types/react-dom@19.2.3`, `@biomejs/biome@2.5.4`, `vitest@4.1.10`, `@testing-library/react@16.3.2`, `@testing-library/dom@10.4.1`, `jsdom@29.1.1`, `@playwright/test@1.61.1`, and `@tauri-apps/cli@2.11.4`. Keep all tooling packages in dev dependencies. Use ordinary CSS, strict TypeScript, and `noUncheckedIndexedAccess`. Configure Vitest with `environment: 'jsdom'`. Add scripts `format:check`, `lint`, `typecheck`, `test`, `test:ui`, `build`, `tauri`, `audit`, and `ci`.

```tsx
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { App } from './app/App'
import './styles/index.css'

createRoot(document.getElementById('root')!).render(<StrictMode><App /></StrictMode>)
```

- [ ] **Step 7: Generate lockfiles and verify GREEN**

Run:

```bash
pnpm install --no-frozen-lockfile
pnpm install --frozen-lockfile
bash tests/architecture/dependency-boundaries.sh
cargo check --workspace --locked
pnpm typecheck
pnpm build
```

Expected: one Svelte-free React bundle, all crate stubs compile, and the boundary test passes.

- [ ] **Step 8: Commit the workspace foundation**

```bash
git add Cargo.toml Cargo.lock rust-toolchain.toml deny.toml crates src-tauri package.json pnpm-lock.yaml pnpm-workspace.yaml biome.json tsconfig.json vite.config.mjs index.html src tests/architecture
git commit -m "build: establish Tauri Rust workspace"
```

### Task 3: Implement Domain Configuration, Templates, Metadata, and Resources

**Files:**
- Create: `crates/yiyin-domain/src/{config,template,metadata,resource,error}.rs`
- Modify: `crates/yiyin-domain/src/lib.rs`
- Test: unit tests colocated in each module

**Interfaces:**
- Produces: `Config`, `RenderOptions`, `Template`, `TemplateField`, `FontSpec`, `Metadata`, `ResourceId`, `TaskId`, `ResourceKind`, and validated newtypes.
- Consumes: no external crate.

- [ ] **Step 1: Write failing tests for defaults and exact option ranges**

Test every default from the design and boundary failures for main image width, radius, shadow, quality, margin, blur, and background ratio.

```rust
#[test]
fn defaults_match_v1_6() {
    let config = Config::default();
    assert_eq!(config.options.quality.get(), 100);
    assert_eq!(config.options.radius.get(), 2.1);
    assert_eq!(config.options.shadow.get(), 6.0);
    assert_eq!(config.options.main_image_width.get(), 90);
    assert_eq!(config.options.font.as_str(), "PingFang SC");
}

#[test]
fn quality_rejects_out_of_range_values() {
    assert_eq!(Quality::try_from(0), Err(DomainError::OutOfRange("quality")));
    assert!(Quality::try_from(100).is_ok());
}
```

- [ ] **Step 2: Implement validated newtypes and complete configuration**

Use private fields and `TryFrom` constructors; do not let raw `f64` or `u8` bypass validation. Keep `origin_wh_output` in the schema model even though it has no current geometry effect.

```rust
pub struct Quality(u8);

impl TryFrom<u8> for Quality {
    type Error = DomainError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        (1..=100).contains(&value).then_some(Self(value)).ok_or(DomainError::OutOfRange("quality"))
    }
}
```

- [ ] **Step 3: Write failing template substitution tests**

Cover hidden fields, forced custom values, absent EXIF, image/text slots, case conversion, system-template deletion protection, and empty-row omission.

```rust
#[test]
fn hidden_or_empty_fields_collapse_without_literal_placeholders() {
    let rows = plan_rows("{Make} {Model}", &fields_with_hidden_make());
    assert_eq!(rows[0].plain_text(), "z8");
    assert!(!rows[0].plain_text().contains('{'));
}
```

- [ ] **Step 4: Implement templates and normalized metadata**

Define all fifteen established fields, three default templates, field override precedence, light/dark image variants, and normalized `Metadata`. Domain accepts normalized values; vendor parsing stays outside.

```rust
pub fn plan_rows(templates: &[Template], fields: &FieldValues, background: BackgroundKind) -> Vec<TextRowPlan> {
    templates.iter().filter(|template| template.enabled()).filter_map(|template| template.plan(fields, background)).collect()
}
```

- [ ] **Step 5: Implement opaque identifier and resource types**

`ResourceId` and `TaskId` wrap non-empty opaque strings; `ResourceKind` is a closed enum for bundled asset, font, overlay, input, preview, and output.

```rust
pub enum ResourceKind { BundledAsset, Font, Overlay, Input, Preview, Output }
pub struct ResourceId(String);
pub struct TaskId(String);
```

- [ ] **Step 6: Run domain tests and commit**

Run: `cargo test -p yiyin-domain --locked`

Expected: all defaults, ranges, templates, metadata, and identifier tests pass with no external dependency in `cargo tree -p yiyin-domain`.

```bash
git add crates/yiyin-domain
git commit -m "feat(domain): model configuration and templates"
```

### Task 4: Implement Domain Render Plans, Output Naming, and Task State

**Files:**
- Create: `crates/yiyin-domain/src/{render,task}.rs`
- Modify: `crates/yiyin-domain/src/lib.rs`
- Test: `crates/yiyin-domain/tests/legacy_geometry.rs`

**Interfaces:**
- Consumes: domain configuration, templates, metadata, and identifiers from Task 3.
- Produces: `RenderRequest`, `RenderPlan`, `RenderStage`, `TaskState`, `TaskStatus`, `OutputNameResolver`.

- [ ] **Step 1: Write failing exact-geometry tests from the manifest**

Copy the reviewed exact numeric expectations from the committed manifest into dependency-free Rust test constants, convert them into domain inputs, and assert canvas, main image, text rows, radius, shadow, offsets, and all integer rounding. Do not add serde or a build script to the domain crate; the infrastructure golden test in Task 9 is responsible for parsing the JSON manifest and cross-checking the same scenario IDs.

```rust
#[test]
fn portrait_default_geometry_matches_legacy() {
    let fixture = fixture("portrait-default");
    let plan = RenderPlan::build(fixture.request).unwrap();
    assert_eq!(plan.canvas, fixture.exact_geometry.canvas);
    assert_eq!(plan.main_rect, fixture.exact_geometry.main_rect);
    assert_eq!(plan.text_rows, fixture.exact_geometry.text_rows);
}
```

- [ ] **Step 2: Implement the exact calculation order**

Implement explicit ratio, landscape swap, width-rate expansion, minimum/shadow top spacing, three-quarter text spacing, 2.7% bottom text offset, centering, radius/shadow percentages, and the `10240` shadow surface cap. Use named integer-rounding helpers so tests make every `ceil`, `floor`, and `round` site explicit.

```rust
let mut background = ratio_adjusted_dimensions(input, options);
background = apply_landscape(background, options);
background = expand_for_main_width(background, input, options.main_image_width);
let spacing = vertical_spacing(background.height, input.height, text_rows, options);
let canvas = recenter_content(background, input, text_rows, spacing);
```

- [ ] **Step 3: Write and implement collision-name tests**

Cover `name.jpg`, `name-1.jpg`, gaps, existing nonnumeric suffixes, and source extensions. The resolver accepts an abstract set of existing file names and returns only the file name.

```rust
assert_eq!(resolve("photo.png", &BTreeSet::from(["photo.jpg"])), "photo-1.jpg");
assert_eq!(resolve("photo.png", &BTreeSet::from(["photo.jpg", "photo-4.jpg"])), "photo-5.jpg");
```

- [ ] **Step 4: Write and implement task state-machine tests**

Allow only registered to queued/running, running to completed/failed/cancelled, and preview supersession cancellation. Encode progress milestones `1,10,20,30,50,60,70,90,100` as `RenderStage::percent()`.

```rust
pub fn transition(from: TaskState, to: TaskState) -> Result<TaskState, DomainError> {
    match (&from, &to) {
        (TaskState::Registered, TaskState::Queued | TaskState::Running)
        | (TaskState::Queued, TaskState::Running | TaskState::Cancelled)
        | (TaskState::Running, TaskState::Completed | TaskState::Failed | TaskState::Cancelled) => Ok(to),
        _ => Err(DomainError::IllegalTaskTransition),
    }
}
```

- [ ] **Step 5: Verify and commit**

Run: `cargo test -p yiyin-domain --locked`

Expected: exact geometry fixtures, naming, and illegal-transition tests pass.

```bash
git add crates/yiyin-domain
git commit -m "feat(domain): define rendering and task rules"
```

### Task 5: Define Application Ports, Errors, and Configuration/Resource Use Cases

**Files:**
- Create: `crates/yiyin-application/src/{models,ports,error}.rs`
- Create: `crates/yiyin-application/src/use_cases/{mod,bootstrap,config,resources}.rs`
- Modify: `crates/yiyin-application/src/lib.rs`
- Test: `crates/yiyin-application/tests/config_resources.rs`

**Interfaces:**
- Consumes: domain types from Tasks 3-4.
- Produces object-safe ports: `ConfigRepository`, `ResourceRepository`, `MetadataReader`, `OutputDirectoryGateway`, `Clock`, `IdGenerator`, and `TaskQueue`.
- Produces safe application models including `RenderResult`, `ResourceSnapshot`, and `BootstrapSnapshot`.
- Produces use cases: `Bootstrap`, `UpdateConfig`, `ResetConfig`, `RegisterImages`, `RegisterFont`, `RemoveFont`, `RegisterOverlay`, and `ReadTaskExif`.

- [ ] **Step 1: Write fake ports and failing use-case tests**

Use handwritten `Arc<Mutex<...>>` fakes. Cover bootstrap snapshot, config validation, reset, file registration, duplicate font, missing file, overlay registration, and safe EXIF lookup.

```rust
#[test]
fn update_config_validates_before_persisting() {
    let repo = FakeConfigRepository::default();
    let use_case = UpdateConfig::new(repo.clone());
    let error = use_case.execute(invalid_public_config()).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ConfigInvalid);
    assert_eq!(repo.write_count(), 0);
}
```

- [ ] **Step 2: Define exact stable application errors**

Use a closed enum with `CANCELLED`, `CONFIG_INVALID`, `FILE_INVALID`, `FILE_NOT_FOUND`, `FORBIDDEN`, `INTERNAL`, `INVALID_REQUEST`, `RESOURCE_NOT_FOUND`, and `TASK_NOT_FOUND`. Store internal source errors without exposing their text through `safe_message()`.

```rust
pub enum ErrorCode { Cancelled, ConfigInvalid, FileInvalid, FileNotFound, Forbidden, Internal, InvalidRequest, ResourceNotFound, TaskNotFound }

impl ErrorCode {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Cancelled => "CANCELLED",
            Self::ConfigInvalid => "CONFIG_INVALID",
            Self::FileInvalid => "FILE_INVALID",
            Self::FileNotFound => "FILE_NOT_FOUND",
            Self::Forbidden => "FORBIDDEN",
            Self::Internal => "INTERNAL",
            Self::InvalidRequest => "INVALID_REQUEST",
            Self::ResourceNotFound => "RESOURCE_NOT_FOUND",
            Self::TaskNotFound => "TASK_NOT_FOUND",
        }
    }
}
```

- [ ] **Step 3: Define object-safe synchronous ports**

```rust
pub trait ConfigRepository: Send + Sync {
    fn load(&self) -> Result<Config, ApplicationError>;
    fn store(&self, config: &Config) -> Result<(), ApplicationError>;
    fn import_legacy_if_needed(&self) -> Result<ImportOutcome, ApplicationError>;
}

pub trait ResourceRepository: Send + Sync {
    fn register_input(&self, source: &Path) -> Result<ResourceRecord, ApplicationError>;
    fn register_owned(&self, kind: ResourceKind, source: &Path) -> Result<ResourceRecord, ApplicationError>;
    fn resolve(&self, id: &ResourceId) -> Result<ResourceRecord, ApplicationError>;
    fn snapshot(&self) -> Vec<ResourceRecord>;
}

pub trait CancellationProbe: Send + Sync {
    fn is_cancelled(&self) -> bool;
}

pub trait ImageRenderer: Send + Sync {
    fn render(
        &self,
        request: &RenderRequest,
        cancellation: &dyn CancellationProbe,
        progress: &mut dyn FnMut(RenderStage),
    ) -> Result<RenderResult, ApplicationError>;
}
```

Keep path types inside ports only; no path appears in public snapshots.

- [ ] **Step 4: Implement minimal orchestration**

Construct complete public snapshots from port results, freeze safe display names, and preserve font outcomes as explicit errors rather than numeric magic values. Bootstrap calls legacy import before load and returns migration warnings separately from fatal errors.

```rust
pub fn execute(&self) -> Result<BootstrapSnapshot, ApplicationError> {
    let import = self.config.import_legacy_if_needed()?;
    let config = self.config.load()?;
    let resources = self.resources.snapshot().into_iter().map(ResourceSnapshot::from).collect();
    Ok(BootstrapSnapshot::new(config, resources, self.tasks.snapshot(), import.warnings))
}
```

- [ ] **Step 5: Verify dependency direction and commit**

Run:

```bash
cargo test -p yiyin-application --locked
bash tests/architecture/dependency-boundaries.sh
```

Expected: all fake-port tests pass and application depends only on domain.

```bash
git add crates/yiyin-application tests/architecture
git commit -m "feat(application): add configuration and resource use cases"
```

### Task 6: Implement Application Task Orchestration

**Files:**
- Create: `crates/yiyin-application/src/use_cases/tasks.rs`
- Modify: `crates/yiyin-application/src/{ports,models}.rs`
- Test: `crates/yiyin-application/tests/tasks.rs`

**Interfaces:**
- Produces: `StartTasks`, `PreviewTask`, `CancelTask`, `ClearTasks`.
- Extends: `TaskQueue`, `TaskEventSink`, and `ImageRenderer` port signatures.

- [ ] **Step 1: Write failing tests for frozen requests and queue semantics**

Test concurrency-independent orchestration: configuration is copied at start, quick output triggers enqueue, explicit mode waits, preview B cancels preview A, clear removes registered/queued snapshots, and completion does not expose a path.

```rust
#[test]
fn configuration_is_frozen_when_task_starts() {
    let harness = Harness::with_quality(80);
    let id = harness.register("photo.jpg");
    harness.start(&[id.clone()]).unwrap();
    harness.set_quality(40);
    assert_eq!(harness.queue.request(&id).options.quality.get(), 80);
}
```

- [ ] **Step 2: Finalize task port signatures**

```rust
pub trait TaskQueue: Send + Sync {
    fn register(&self, task: RegisteredTask) -> Result<(), ApplicationError>;
    fn enqueue(&self, request: RenderRequest) -> Result<(), ApplicationError>;
    fn preview(&self, request: RenderRequest) -> Result<(), ApplicationError>;
    fn cancel(&self, id: &TaskId) -> Result<(), ApplicationError>;
    fn clear(&self) -> Result<(), ApplicationError>;
    fn snapshot(&self) -> Vec<TaskSnapshot>;
}

pub trait TaskEventSink: Send + Sync {
    fn publish(&self, status: TaskStatus);
}
```

- [ ] **Step 3: Implement the four use cases**

Validate requested task IDs, build frozen requests through domain constructors, reserve collision-safe output names before enqueue, and return updated snapshots. Preview uses a dedicated replace-latest slot and never writes final output.

```rust
for id in ids {
    let task = self.tasks.require_registered(id)?;
    let request = self.requests.freeze(task, self.config.load()?)?;
    self.queue.enqueue(request)?;
}
Ok(self.queue.snapshot())
```

- [ ] **Step 4: Verify and commit**

Run: `cargo test -p yiyin-application --locked`

Expected: all task, cancellation, and no-path-leak tests pass.

```bash
git add crates/yiyin-application
git commit -m "feat(application): orchestrate rendering tasks"
```

### Task 7: Implement Durable Configuration and Idempotent Legacy Import

**Files:**
- Create: `crates/yiyin-infrastructure/src/config/{mod,json_repository,legacy_import}.rs`
- Create: `crates/yiyin-infrastructure/src/filesystem.rs`
- Modify: `crates/yiyin-infrastructure/src/lib.rs`
- Test: `crates/yiyin-infrastructure/tests/config_persistence.rs`
- Test: `crates/yiyin-infrastructure/tests/legacy_import.rs`

**Interfaces:**
- Implements: `ConfigRepository`.
- Consumes: platform paths supplied by `src-tauri`; never queries Tauri directly.

- [ ] **Step 1: Write failing durability and recovery tests**

Use `tempfile` to test initial write, `.bak`, `.invalid.bak`, failed temp write, atomic replacement, and directory sync behavior. Inject a filesystem fault at each write stage and assert the prior valid file remains loadable.

```rust
#[test]
fn failed_atomic_replace_preserves_the_prior_config() {
    let harness = ConfigHarness::with_valid_config();
    harness.fail_at(FaultPoint::Rename);
    assert!(harness.repository().store(&changed_config()).is_err());
    assert_eq!(harness.reload().unwrap(), harness.original_config());
}
```

- [ ] **Step 2: Implement versioned serde DTOs and explicit mappings**

Infrastructure owns `StoredConfigV1`; map it to/from domain `Config`. Preserve established JSON field names, append missing built-ins, keep unknown future schema from being silently interpreted, and never serialize runtime private paths into the public DTO.

```rust
#[derive(Serialize, Deserialize)]
struct StoredConfigEnvelope {
    version: u32,
    config: serde_json::Value,
}

const CURRENT_CONFIG_VERSION: u32 = 1;
```

- [ ] **Step 3: Implement atomic writes**

Write `<name>.tmp`, flush, `sync_all`, copy the prior valid file to `.bak`, rename atomically, then sync the directory where supported. Clean a stale temp file only after validating it is inside the config directory.

```rust
write_and_sync(&temporary_path, bytes)?;
backup_valid_file(&config_path, &backup_path)?;
filesystem.rename(&temporary_path, &config_path)?;
filesystem.sync_parent(&config_path)?;
```

- [ ] **Step 4: Write failing import tests for both legacy names**

Check ordered candidates `Application Support/壹印` then `Application Support/yiyin` on macOS and `%APPDATA%/壹印` then `%APPDATA%/yiyin` on Windows. Assert config, `font.json`, font files, and static resources copy once; source hashes remain unchanged; interrupted import retries safely.

```rust
assert_eq!(macos_candidates(&support), [support.join("壹印"), support.join("yiyin")]);
assert_eq!(windows_candidates(&app_data), [app_data.join("壹印"), app_data.join("yiyin")]);
```

- [ ] **Step 5: Implement import transaction and marker**

Import only when no committed Tauri config exists. Stage copies under a temporary import directory, validate all copied records, atomically publish config/resources, and write a migration marker containing source location fingerprint and completed schema version. Unsupported individual resources become warnings.

```rust
pub struct MigrationMarker {
    pub source_fingerprint: String,
    pub completed_schema_version: u32,
}
```

- [ ] **Step 6: Verify and commit**

Run:

```bash
cargo test -p yiyin-infrastructure --test config_persistence --locked
cargo test -p yiyin-infrastructure --test legacy_import --locked
```

Expected: durability, recovery, idempotency, and source-preservation tests pass.

```bash
git add crates/yiyin-infrastructure
git commit -m "feat(infrastructure): persist and migrate configuration"
```

### Task 8: Implement Resource Registry and EXIF Adapter

**Files:**
- Create: `crates/yiyin-infrastructure/src/{resources,metadata}.rs`
- Test: `crates/yiyin-infrastructure/tests/{resources,metadata}.rs`

**Interfaces:**
- Implements: `ResourceRepository`, `MetadataReader`, and `IdGenerator`.
- Produces: `ResourceRecord { id, kind, canonical_path, mime, allowed_root }` for Tauri protocol use.

- [ ] **Step 1: Write failing resource security tests**

Cover JPEG/PNG/WebP signature checks, extension mismatch, unsupported content, symlink escape, source deletion, custom font/overlay copy, unknown ID, and re-canonicalization after registration.

```rust
#[test]
fn resolve_rejects_a_registered_symlink_that_now_escapes() {
    let record = harness.register_input("inside/photo.jpg").unwrap();
    harness.replace_with_external_symlink("inside/photo.jpg");
    assert_eq!(harness.resolve(&record.id).unwrap_err().code(), ErrorCode::Forbidden);
}
```

- [ ] **Step 2: Implement the registry**

Generate UUID v4 IDs, store records behind `RwLock<HashMap<ResourceId, ResourceRecord>>`, copy owned resources with atomic publication, and revalidate canonical containment on every resolve. Inputs may reference registered source files but never cross the React boundary.

```rust
pub struct ResourceRegistry {
    records: RwLock<HashMap<ResourceId, ResourceRecord>>,
    owned_root: PathBuf,
}
```

- [ ] **Step 3: Write failing EXIF normalization fixtures**

Assert generic maker/model, Nikon `Z` to `ℤ` and Roman suffix, Sony `ILCE-` to `α`, fractional shutter, focal lengths, date/time, white balance, exposure program, metering mode, orientation, and all-empty `None`.

```rust
assert_eq!(normalize_nikon_model("NIKON", "NIKON Z 7_2"), " ℤ 7 Ⅱ");
assert_eq!(normalize_sony_model("ILCE-7RM5"), "α7rm5");
assert_eq!(format_shutter(1, 125), "1/125");
```

- [ ] **Step 4: Implement `kamadak-exif` extraction**

Read only required tags, convert values through named normalization functions, and return domain `Metadata`. Unsupported or malformed tags become empty fields rather than panics; file/decode failures map to stable application errors.

```rust
let metadata = Metadata::new(
    normalized_camera(&exif),
    normalized_lens(&exif),
    normalized_exposure(&exif),
    normalized_capture_time(&exif),
    normalized_orientation(&exif),
);
```

- [ ] **Step 5: Verify and commit**

Run: `cargo test -p yiyin-infrastructure --test resources --test metadata --locked`

Expected: resource containment and every normalized EXIF fixture pass.

```bash
git add crates/yiyin-infrastructure tests/fixtures/input
git commit -m "feat(infrastructure): register resources and normalize EXIF"
```

### Task 9: Implement the Rust Renderer and Golden Comparisons

**Files:**
- Create: `crates/yiyin-infrastructure/src/rendering/{mod,background,text,composite}.rs`
- Test: `crates/yiyin-infrastructure/tests/rendering.rs`
- Test: `crates/yiyin-infrastructure/tests/golden.rs`
- Create: `crates/yiyin-infrastructure/tests/support/perceptual.rs`

**Interfaces:**
- Implements: `ImageRenderer`.
- Consumes: frozen `RenderRequest`, resolved resource records, and domain `RenderPlan`.
- Produces: `RenderResult { dimensions, density, resource_kind }` after atomic publication.

- [ ] **Step 1: Write failing stage tests**

Test JPEG/PNG/WebP decode, EXIF orientation, solid background, stretched blur, text rows, logo variants, corner mask, shadow, density, preview quality `70`, export quality, and temp cleanup.

```rust
#[test]
fn preview_uses_quality_seventy_without_publishing_output() {
    let result = harness.render_preview(default_request()).unwrap();
    assert_eq!(result.encoder_quality, 70);
    assert!(!harness.output_path().exists());
}
```

- [ ] **Step 2: Implement background generation with `image` only**

Use `image` resize and blur operations; do not add `imageproc`. Preserve the old stretched-background geometry and brightness overlay thresholds. Operate on `RgbaImage` and return an owned buffer.

```rust
pub fn render_background(source: &RgbaImage, plan: &BackgroundPlan) -> RgbaImage {
    let stretched = image::imageops::resize(source, plan.width, plan.height, plan.filter);
    image::imageops::blur(&stretched, plan.blur_sigma)
}
```

- [ ] **Step 3: Implement deterministic text and logo rasterization**

Create a `cosmic-text` font system seeded with bundled and registered fonts. Shape each planned row, rasterize glyphs into transparent `RgbaImage`, apply field/template font merging and case conversion, and place image slots using baseline or center alignment. Golden tests always specify repository fonts.

```rust
pub fn rasterize_rows(
    font_system: &mut FontSystem,
    rows: &[PlannedRow],
    resources: &dyn ResourceRepository,
) -> Result<RgbaImage, ApplicationError>;
```

- [ ] **Step 4: Implement rounded mask, shadow, and composition**

Draw the shadow alpha mask directly into an RGBA surface, cap the intermediate width at `10240`, apply the planned radius, clear the main-image opening, composite background/main/text in the established order, and use explicit integer coordinates from `RenderPlan`.

```rust
assert!(plan.shadow_surface_width <= 10_240);
overlay(&mut canvas, &shadow, plan.shadow_origin.x, plan.shadow_origin.y);
overlay(&mut canvas, &rounded_main, plan.main_origin.x, plan.main_origin.y);
overlay(&mut canvas, &text, plan.text_origin.x, plan.text_origin.y);
```

- [ ] **Step 5: Implement JPEG encode and atomic publication**

Encode to a same-directory temporary file with selected quality and source density. On success sync and rename; on cancellation/error delete temporary/cache files and leave existing output untouched.

```rust
let temporary = output.with_extension("jpg.tmp");
encode_jpeg(&temporary, &canvas, request.quality, request.density)?;
cancellation.ensure_active()?;
filesystem.rename(&temporary, &output)?;
```

- [ ] **Step 6: Implement perceptual comparison without a new dependency**

Decode both images with `image`, assert exact dimensions, calculate luminance SSIM and changed-pixel ratio in test support, and use each manifest threshold. Write `actual.png`, `expected.png`, and amplified `diff.png` under `target/golden-diffs/<scenario>` on failure.

```rust
assert_eq!(actual.dimensions(), expected.dimensions());
assert!(metrics.ssim >= scenario.min_ssim);
assert!(metrics.changed_pixel_ratio <= scenario.max_changed_pixel_ratio);
```

- [ ] **Step 7: Verify and commit**

Run:

```bash
cargo test -p yiyin-infrastructure --test rendering --locked
cargo test -p yiyin-infrastructure --test golden --locked
```

Expected: exact geometry/density and all perceptual thresholds pass; no diff artifact on success.

```bash
git add crates/yiyin-infrastructure tests/fixtures/legacy
git commit -m "feat(infrastructure): render compatible watermarks"
```

### Task 10: Implement the Concurrency-Two Task Queue and Cancellation

**Files:**
- Create: `crates/yiyin-infrastructure/src/tasks.rs`
- Test: `crates/yiyin-infrastructure/tests/tasks.rs`

**Interfaces:**
- Implements: `TaskQueue`.
- Consumes: `Arc<dyn ImageRenderer>`, `Arc<dyn TaskEventSink>`, and frozen render requests.

- [ ] **Step 1: Write deterministic queue tests with a controlled renderer**

Use barriers/channels to prove no more than two exports run, explicit mode stays idle, quick mode enqueues, preview replace-latest cancels stale work, progress order is monotonic, clear removes pending entries, and shutdown prevents new work.

```rust
renderer.release_all();
assert_eq!(renderer.maximum_concurrency(), 2);
assert_eq!(events.for_task(&stale_preview).last().unwrap().state, TaskState::Cancelled);
```

- [ ] **Step 2: Implement task records and cancellation probes**

Store tasks in `Arc<RwLock<HashMap<TaskId, TaskRecord>>>`. Each active task owns a Tokio cancellation token implemented with `watch` or `AtomicBool`; do not add a cancellation crate. The renderer checks at stage boundaries.

```rust
pub struct AtomicCancellation(AtomicBool);

impl CancellationProbe for AtomicCancellation {
    fn is_cancelled(&self) -> bool { self.0.load(Ordering::Acquire) }
}
```

- [ ] **Step 3: Implement scheduling on blocking workers**

Use a Tokio semaphore of `2` and `spawn_blocking` for image work. Emit typed domain status through `TaskEventSink`; catch join errors and map them to `INTERNAL` without panicking. Shutdown rejects new tasks, cancels active work, and waits at most the named `SHUTDOWN_GRACE` constant of five seconds for cleanup.

```rust
let permit = self.semaphore.clone().acquire_owned().await.map_err(internal)?;
let result = tokio::task::spawn_blocking(move || renderer.render(&request, &cancel, &mut progress)).await;
drop(permit);
let result = result.map_err(internal)??;
```

```rust
const SHUTDOWN_GRACE: Duration = Duration::from_secs(5);
```

- [ ] **Step 4: Verify and commit**

Run: `cargo test -p yiyin-infrastructure --test tasks --locked`

Expected: maximum observed concurrency is exactly two when three tasks are released, cancellation is terminal, and stale preview completion cannot win.

```bash
git add crates/yiyin-infrastructure
git commit -m "feat(infrastructure): schedule rendering tasks"
```

### Task 11: Build the Tauri DTO, Error, Protocol, Native, and Composition Boundaries

**Files:**
- Create: `src-tauri/src/dto/{mod,config,resource,metadata,task,error}.rs`
- Create: `src-tauri/src/commands/{mod,bootstrap,config,resources,tasks,native,window}.rs`
- Create: `src-tauri/src/{error,events,protocol,native,state,app}.rs`
- Modify: `src-tauri/src/{lib,main}.rs`
- Create: `src-tauri/capabilities/main.json`, `src-tauri/tauri.conf.json`
- Test: `src-tauri/tests/{dto,error,protocol,capabilities,plugin_order}.rs`

**Interfaces:**
- Produces commands: `bootstrap`, `update_config`, `reset_config`, `choose_output_directory`, `open_output_directory`, `choose_images`, `register_font`, `remove_font`, `register_overlay`, `read_task_exif`, `start_tasks`, `preview_task`, `cancel_task`, `clear_tasks`, `minimize_window`, `close_window`, `open_external_url`.
- Produces event: `task-status`.
- Produces protocol: `yiyin://resource/<opaque-id>`.

- [ ] **Step 1: Write DTO generation and mapping tests**

Derive serde and `ts-rs` only on Tauri DTOs. Assert no DTO has a field named `path`, `dir`, `cache_dir`, or `static_dir`; generate `src/platform/types.ts` and fail if the checked-in output drifts.

```rust
#[test]
fn generated_types_have_no_private_path_fields() {
    let generated = generate_types();
    for forbidden in ["path:", "dir:", "cache_dir:", "static_dir:"] {
        assert!(!generated.contains(forbidden));
    }
}
```

- [ ] **Step 2: Implement stable error mapping and redaction**

Map every application error code one-to-one. `CommandErrorDto` contains only `{ code, message }`; tests inject source errors containing `/Users/private/secret.jpg` and assert the serialized DTO and event omit it.

```rust
#[derive(Serialize, TS)]
pub struct CommandErrorDto {
    pub code: String,
    pub message: String,
}
```

- [ ] **Step 3: Write protocol attack tests, then implement the handler**

Reject unknown IDs, `..`, encoded separators, extra path segments, query substitution, symlink escape, wrong MIME, and stale records. Parse exactly scheme `yiyin`, host `resource`, and one opaque segment; resolve through the registry on every request.

```rust
let segments = request.uri().path().split('/').filter(|segment| !segment.is_empty()).collect::<Vec<_>>();
if request.uri().host() != Some("resource") || segments.len() != 1 || request.uri().query().is_some() {
    return forbidden_response();
}
```

- [ ] **Step 4: Implement dedicated native operations**

Use Rust plugin APIs behind commands. Dialogs return registered task/resource descriptors, drag/drop calls the same registration use case, output opening accepts no path argument, and external links accept an enum destination rather than a raw URL.

```rust
pub enum ExternalDestination { Repository, Issues, CurrentRelease, BilibiliProfile, BilibiliFeedback }
```

Map each variant inside Rust: repository `https://github.com/ggchivalrous/yiyin`, issues `https://github.com/ggchivalrous/yiyin/issues`, current release `https://github.com/ggchivalrous/yiyin/releases/tag/v1.6.0`, Bilibili profile `https://space.bilibili.com/94829489`, and Bilibili feedback `https://message.bilibili.com/#/whisper/mid94829489`.

- [ ] **Step 5: Implement thin commands and the single event sink**

Commands deserialize DTOs, validate the boundary, call one use case, and map results. `TauriTaskEventSink` emits only `task-status`; no image bytes or paths appear.

```rust
#[tauri::command]
pub fn cancel_task(state: State<'_, AppState>, request: CancelTaskRequestDto) -> CommandResult<TaskDescriptorDto> {
    state.cancel_task.execute(request.id.into()).map(Into::into).map_err(Into::into)
}
```

- [ ] **Step 6: Compose plugins in exact order**

In `app.rs`, register single-instance first, then log, dialog, opener, protocol, managed state, invoke handler, drag/drop listener, and setup. The second-instance callback restores and focuses `main`.

```rust
tauri::Builder::default()
    .plugin(single_instance_plugin())
    .plugin(tauri_plugin_log::Builder::new().build())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_opener::init());
```

- [ ] **Step 7: Write minimal capabilities and production policy**

Disable `withGlobalTauri` and DevTools, use local-only CSP, and grant no generic plugin command to the WebView. Tests parse JSON and assert forbidden capability identifiers are absent.

```rust
for forbidden in ["dialog:", "opener:", "fs:", "shell:", "http:"] {
    assert!(!capability_permissions.iter().any(|permission| permission.starts_with(forbidden)));
}
```

- [ ] **Step 8: Verify and commit**

Run:

```bash
cargo test -p yiyin-desktop --locked
cargo check --workspace --locked
```

Expected: DTO generation, redaction, protocol attacks, capabilities, and plugin-order tests pass.

```bash
git add src-tauri src/platform/types.ts
git commit -m "feat(desktop): expose secure Tauri boundaries"
```

### Task 12: Implement the Typed React Client, App Shell, and Chrome Parity

**Files:**
- Create: `src/platform/{client,fake}.ts`
- Create: `src/app/{App,useAppController}.tsx`
- Create: `src/features/chrome/{TitleBar,HelpPopover,FeedbackPopover,Footer}.tsx`
- Create: `src/components/{Dialog,Drawer,Popover,Select,Switch,RadioGroup}.tsx`
- Create: `src/styles/{tokens,index,chrome}.css`
- Test: `src/app/App.test.tsx`, `src/platform/client.test.ts`

**Interfaces:**
- Consumes: generated `src/platform/types.ts` and exact Tauri command/event names.
- Produces: `PlatformClient` interface used by every UI feature.

- [ ] **Step 1: Write a failing typed-client contract test**

Mock `invoke` and `listen`; assert exact command names, DTO payloads, event cleanup, and `CommandErrorDto` parsing. No feature may import `@tauri-apps/api` directly.

```ts
export interface PlatformClient {
  bootstrap(): Promise<BootstrapDto>
  updateConfig(input: UpdateConfigRequestDto): Promise<PublicConfigDto>
  startTasks(ids: string[]): Promise<TaskDescriptorDto[]>
  onTaskStatus(listener: (event: TaskStatusEventDto) => void): Promise<() => void>
}
```

- [ ] **Step 2: Implement real and fake adapters**

The real adapter centralizes invoke/listen and safe error parsing. The fake stores deterministic bootstrap/config/task state for Vitest and browser Playwright tests.

```ts
export const tauriClient: PlatformClient = {
  bootstrap: () => invoke<BootstrapDto>('bootstrap'),
  updateConfig: (input) => invoke<PublicConfigDto>('update_config', { input }),
  startTasks: (ids) => invoke<TaskDescriptorDto[]>('start_tasks', { input: { ids } }),
  onTaskStatus: (listener) => listen<TaskStatusEventDto>('task-status', ({ payload }) => listener(payload)),
}
```

- [ ] **Step 3: Write app bootstrap/reconciliation tests**

Test loading, successful bootstrap, migration warning, fatal bootstrap error, task event update, event-before-bootstrap race, and unsubscribe on unmount.

```ts
it('reconciles an event received before bootstrap resolves', async () => {
  const fake = createDeferredPlatformClient()
  render(<App client={fake.client} />)
  fake.emit(taskStatus('task-1', 'running'))
  fake.resolveBootstrap(bootstrapWithTask('task-1'))
  expect(await screen.findByText('处理中')).toBeVisible()
})
```

- [ ] **Step 4: Implement `useAppController` with `useReducer`**

Keep Rust snapshots authoritative. Reconcile task events by ID, preserve current selection only while it exists, and re-bootstrap after an unknown event ID. Do not add global state dependencies.

```ts
type AppAction =
  | { type: 'bootstrapped'; snapshot: BootstrapDto }
  | { type: 'task-status'; event: TaskStatusEventDto }
  | { type: 'bootstrap-failed'; error: CommandErrorDto }
```

- [ ] **Step 5: Rebuild title bar and footer with exact text/interactions**

Use current DOM dimensions and CSS tokens. Rust-owned destination enums back repository/issues/release/Bilibili links. Preserve copy-QQ behavior, help content, donation images, font selector placement, reset, minimize, and close.

```tsx
<TitleBar onMinimize={client.minimizeWindow} onClose={client.closeWindow} />
<Footer onOpen={(destination) => client.openExternalUrl({ destination })} />
```

- [ ] **Step 6: Verify and commit**

Run:

```bash
pnpm lint
pnpm typecheck
pnpm vitest run src/platform/client.test.ts src/app/App.test.tsx
```

Expected: no direct Tauri import outside `src/platform/client.ts`, all shell interactions pass.

```bash
git add src
git commit -m "feat(ui): add typed React application shell"
```

### Task 13: Rebuild Rendering Settings, Fields, Templates, and Fonts

**Files:**
- Create: `src/features/settings/{RenderingSettings,FieldDrawer,FieldDialog,TemplateDrawer,TemplateDialog,FontSelect,FontDialog}.tsx`
- Create: `src/features/settings/settings.css`
- Test: `src/features/settings/settings.test.tsx`

**Interfaces:**
- Consumes: `PublicConfigDto` and `PlatformClient.updateConfig/registerFont/removeFont/registerOverlay`.
- Produces: user intents only; Rust returns the canonical updated config.

- [ ] **Step 1: Write failing parity tests for every setting**

Assert exact defaults, labels, help text, numeric clamps/rounding, ratio swap, landscape disablement, immediate save, solid color, preview toggle, and quick-output toggle.

```ts
it.each([
  ['quality', '101', 100],
  ['density', '0', 1],
  ['blurRadius', '3.6', 4],
])('canonicalizes %s through Rust', async (field, input, expected) => {
  await changeSetting(field, input)
  expect(fake.lastConfig()[field]).toBe(expected)
})
```

- [ ] **Step 2: Implement rendering controls**

Use owned controlled inputs and Base UI switches. On valid changes call `updateConfig`; replace local draft with the returned canonical config. Debounce continuous blur slider changes but flush on blur/unmount.

```tsx
const save = async (next: PublicConfigDto) => {
  const canonical = await client.updateConfig({ config: next })
  setDraft(canonical)
}
```

- [ ] **Step 3: Write failing field/template transaction tests**

Cover save/cancel, show/hide, force use, text/image type, light/dark overlay registration, font overrides, custom-key stability, system-template deletion denial, custom deletion, ordering, placeholder insertion, and baseline/center alignment.

```ts
it('discards a cancelled template edit', async () => {
  await openTemplate('default')
  await userEvent.type(screen.getByLabelText('名称'), ' changed')
  await userEvent.click(screen.getByRole('button', { name: '取消' }))
  expect(fake.updateConfig).not.toHaveBeenCalled()
})
```

- [ ] **Step 4: Implement dialogs with React Hook Form**

Initialize from DTOs, validate user-visible form constraints, submit one complete update command, and discard on cancel. Rust remains final validation. Do not add Zod or a resolver.

```tsx
const form = useForm<TemplateFormValues>({ defaultValues: toFormValues(template) })
const submit = form.handleSubmit((values) => onSave(toCompleteConfig(config, values)))
```

- [ ] **Step 5: Write and implement font tests**

Cover bundled fonts, native file selection, duplicate name, missing source, deletion, selected-font fallback, and refresh after mutation. React receives only font IDs/resource URLs, never font paths.

```ts
expect(await client.registerFont()).toEqual({ id: 'font-custom', displayName: 'Fixture Sans' })
expect(JSON.stringify(fake.bootstrapSnapshot())).not.toContain('/Users/')
```

- [ ] **Step 6: Verify and commit**

Run: `pnpm vitest run src/features/settings/settings.test.tsx && pnpm typecheck`

Expected: every existing settings workflow passes with exact copy and no private path in rendered output.

```bash
git add src/features/settings src/components src/styles
git commit -m "feat(ui): rebuild settings and templates"
```

### Task 14: Rebuild Image Tasks, EXIF, Preview, Drag/Drop, and Output Workflows

**Files:**
- Create: `src/features/tasks/{TaskWorkspace,TaskList,TaskItem,PreviewPane,OutputControls}.tsx`
- Create: `src/features/tasks/tasks.css`
- Test: `src/features/tasks/tasks.test.tsx`
- Test: `tests/ui/workflows.spec.ts`

**Interfaces:**
- Consumes: task DTOs and `PlatformClient` task/resource methods.
- Produces: no renderer data; images use only `yiyin://resource/<id>` URLs.

- [ ] **Step 1: Write failing task workflow tests**

Cover native add, drag/drop registration event, selected-first behavior, progress milestones, success/failure, EXIF load/copy, clear, explicit start, quick output, output directory choose/open, and error messages.

```ts
it('starts selected tasks before unselected tasks', async () => {
  await addTasks(['one.jpg', 'two.jpg'])
  await selectTask('two.jpg')
  await userEvent.click(screen.getByRole('button', { name: '开始输出' }))
  expect(fake.startedTaskIds()).toEqual(['task-two', 'task-one'])
})
```

- [ ] **Step 2: Write failing preview supersession tests**

Use fake deferred promises: config/selection changes debounce `300 ms`, request B supersedes A, cancelled A cannot replace B, preview off clears the URL, and failed preview displays current failure copy.

```ts
await advanceTimersByTimeAsync(300)
fake.resolvePreview('preview-b', resourceUrl('resource-b'))
fake.resolvePreview('preview-a', resourceUrl('resource-a'))
expect(screen.getByRole('img', { name: '预览' })).toHaveAttribute('src', resourceUrl('resource-b'))
```

- [ ] **Step 3: Implement task workspace and reconciliation**

Render task state from the app controller, select by opaque ID, call only dedicated client methods, and use typed task events for progress. Clipboard receives only normalized JSON returned by `readTaskExif`.

```tsx
<TaskList tasks={state.tasks} selectedId={state.selectedTaskId} onSelect={actions.selectTask} />
<PreviewPane resourceUrl={state.preview?.resourceUrl ?? null} status={state.preview?.status} />
```

- [ ] **Step 4: Implement browser parity tests with the fake adapter**

Playwright loads the React SPA with injected fake adapter and verifies fixed layout, drawers, image list, preview, settings, templates, and errors. It does not claim native dialog or WebView coverage.

```ts
test('completes the browser-only task workflow', async ({ page }) => {
  await page.goto('/?platform=fake')
  await expect(page.getByRole('heading', { name: '壹印' })).toBeVisible()
  await page.getByRole('button', { name: '开始输出' }).click()
  await expect(page.getByText('输出完成')).toBeVisible()
})
```

- [ ] **Step 5: Verify and commit**

Run:

```bash
pnpm vitest run src/features/tasks/tasks.test.tsx
pnpm playwright test tests/ui/workflows.spec.ts
pnpm build
```

Expected: complete browser workflow parity passes and the production bundle contains no Svelte chunk.

```bash
git add src/features/tasks tests/ui
git commit -m "feat(ui): rebuild image task workflows"
```

### Task 15: Add Security, Supply-Chain, CI, and Native Packaging Gates

**Files:**
- Create: `scripts/{check-release-age,verify-bundle,w3c-smoke}.mjs`
- Create: `src-tauri/src/e2e.rs`
- Create: `.github/workflows/{ci,codeql,package}.yml`
- Create: `.github/dependabot.yml`
- Modify: `deny.toml`, `package.json`, `src-tauri/tauri.conf.json`
- Test: `tests/security/policy.test.ts`, `tests/package/package.test.ts`

**Interfaces:**
- Produces six required jobs: frontend quality, Rust quality, security, golden compatibility, macOS package smoke, Windows package plus desktop smoke.

- [ ] **Step 1: Write failing policy tests**

Parse manifests/workflows and assert exact npm pins, Cargo lock presence, no Git/prerelease dependencies, no forbidden packages, full SHA Actions, ecosystem-separated Dependabot, no auto-merge, production DevTools off, and no generic capabilities.

```ts
expect(allActionUses(workflows).every((use) => /^[^@]+@[0-9a-f]{40}$/.test(use))).toBe(true)
expect(packageNames).not.toEqual(expect.arrayContaining(['electron', 'sharp', 'svelte', 'webdriverio']))
```

- [ ] **Step 2: Implement release-age and audit scripts**

`check-release-age.mjs` reads direct npm pins from `package.json` and direct Cargo packages from metadata/lockfile, fetches registry publish timestamps, and fails under 24 hours. Cache registry responses only inside CI temp directories.

```js
const minimumAgeMs = 24 * 60 * 60 * 1000
if (Date.now() - Date.parse(publishedAt) < minimumAgeMs) {
  throw new Error(`${name}@${version} is less than 24 hours old`)
}
```

- [ ] **Step 3: Implement bundle verification**

Inspect macOS `.app` and Windows package contents for product name, identifier, version, icons, unsigned state, and absence of Electron, Node, Sharp, Svelte, `db-ui`, sidecars, remote URLs, source maps, and forbidden generic capabilities.

```bash
node scripts/verify-bundle.mjs --platform macos --artifact src-tauri/target/release/bundle/macos/壹印.app
node scripts/verify-bundle.mjs --platform windows --artifact src-tauri/target/release/bundle
```

- [ ] **Step 4: Add a compile-time-only desktop fixture seed**

Define Cargo feature `e2e-fixture` with no production default. Under that feature, `src-tauri/src/e2e.rs` reads a CI-provided fixture directory at startup and registers the same JPEG/PNG/WebP files through `RegisterImages`; it adds no command and sends no path to React. Add `#[cfg(feature = "e2e-fixture")]` at the single composition call site. Policy tests must prove the production build has the feature disabled and contains no fixture path or environment variable name.

```rust
#[cfg(feature = "e2e-fixture")]
e2e::seed_registered_images(&application, &fixture_directory_from_ci()?)?;
```

- [ ] **Step 5: Implement the minimal Windows W3C client**

Use Node built-in `fetch` only. Start the `e2e-fixture` desktop binary through `tauri-driver`, create a session, locate the pre-registered task, assert title/size, exercise settings, templates, preview, and export, then delete the session. Build the real production package separately without the feature. Do not add WebdriverIO or Selenium.

```js
const session = await command('POST', '/session', { capabilities: { alwaysMatch: { 'tauri:options': { application } } } })
try {
  await assertDesktopWorkflow(session.value.sessionId)
} finally {
  await command('DELETE', `/session/${session.value.sessionId}`)
}
```

- [ ] **Step 6: Create full-SHA workflows**

Resolve each selected stable action tag with `git ls-remote`, verify the commit belongs to the official repository, and write the resulting 40-character SHA with a trailing tag comment. During the Task 2 registry preflight, also select and record exact stable versions older than 24 hours for the CI-only `cargo-deny` and official `tauri-driver` binaries as workflow variables `CARGO_DENY_VERSION` and `TAURI_DRIVER_VERSION`; install both with the commands below and do not add them to workspace manifests. Use Node 24, pnpm 11.13.0, Rust 1.97.0, frozen/locked installs, CodeQL JavaScript/TypeScript plus Rust, golden artifacts, macOS launch smoke, and Windows `tauri-driver` smoke.

```bash
git ls-remote https://github.com/actions/checkout.git refs/tags/v4 refs/tags/v4^{}
git ls-remote https://github.com/actions/setup-node.git refs/tags/v4 refs/tags/v4^{}
git ls-remote https://github.com/github/codeql-action.git refs/tags/v3 refs/tags/v3^{}
cargo install cargo-deny --locked --version "$CARGO_DENY_VERSION"
cargo install tauri-driver --locked --version "$TAURI_DRIVER_VERSION"
```

- [ ] **Step 7: Verify all local policy gates and commit**

Run:

```bash
pnpm audit --audit-level high
pnpm run check:release-age
cargo deny check
pnpm vitest run tests/security/policy.test.ts tests/package/package.test.ts
```

Expected: no high/critical advisory, policy test passes, every Action uses a full SHA, and Dependabot has three isolated ecosystems.

```bash
git add scripts .github deny.toml package.json pnpm-lock.yaml src-tauri/tauri.conf.json tests/security tests/package
git commit -m "ci: enforce Tauri security and packaging"
```

### Task 16: Remove the Legacy Runtime, Update Documentation, and Run Full Acceptance

**Files:**
- Delete: `electron/`, `web/`, `common/`, `dev.js`, `release.js`, `auto-release.config.ts`, `svelte.config.ts`, `eslint.config.ts`, old Vite/Electron scripts, bundled FFmpeg/ExifTool archives, generated logger code, and old workflows.
- Delete: temporary `tests/legacy/capture.spec.ts` and Electron fixture mode after verifying committed artifacts.
- Modify: `README.md`, `LICENSE`, `.gitignore`, `package.json`, `pnpm-lock.yaml`
- Create: `src-tauri/README.md`
- Create: `tests/security/final-tree.test.ts`
- Test: entire repository

**Interfaces:**
- Produces: the final single-runtime repository described by the design.

- [ ] **Step 1: Write the final forbidden-payload test before deleting files**

Assert no tracked runtime file or dependency contains Electron, Svelte, Sharp, FFmpeg, ExifTool binary, `db-ui`, Node sidecar, old renderer callback name, or old IPC router name. Explicitly allow references inside the design, implementation plan, legacy fixture manifest provenance, and test assertions.

```ts
const forbiddenRuntimeEntries = ['electron', 'svelte', 'sharp', 'ffmpeg', 'exiftool', 'db-ui']
const hits = scanTrackedRuntimeFiles({ forbiddenRuntimeEntries, allow: documentationAndFixtureProvenance })
expect(hits).toEqual([])
```

- [ ] **Step 2: Run it to verify RED**

Run: `pnpm vitest run tests/security/final-tree.test.ts`

Expected: FAIL listing current legacy directories and dependencies.

```bash
pnpm vitest run tests/security/final-tree.test.ts
```

- [ ] **Step 3: Remove the old runtime and temporary capture harness**

Delete only legacy-owned files identified above. Preserve icons, screenshots, logos, donation images, bundled fonts, README product content, fixture inputs, and reference outputs by moving them into the new asset/fixture locations before deletion.

```bash
git rm -r electron web common
git rm dev.js release.js auto-release.config.ts svelte.config.ts eslint.config.ts tests/legacy/capture.spec.ts
pnpm vitest run tests/security/final-tree.test.ts
```

- [ ] **Step 4: Update license and contributor documentation**

Replace `LICENSE` with the canonical GPL-3.0-only text, update package/Cargo metadata, update README commands and unsigned package instructions, and write `src-tauri/README.md` explaining crate direction, command tracing, ownership/async boundaries, DTO generation, and fmt/clippy/test commands.

```bash
rg -n 'GPL-3.0-only|io.github.gaotity.yiyin|version = "1.6.0"' LICENSE package.json Cargo.toml src-tauri/Cargo.toml README.md
```

- [ ] **Step 5: Run the complete local verification matrix**

Run:

```bash
pnpm install --frozen-lockfile
pnpm format:check
pnpm lint
pnpm typecheck
pnpm vitest run
pnpm playwright test
pnpm audit --audit-level high
pnpm run check:release-age
pnpm build
cargo fmt --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
cargo deny check
bash tests/architecture/dependency-boundaries.sh
pnpm vitest run tests/security/final-tree.test.ts
```

Expected: every command passes with no tracked change generated by tests or type generation.

- [ ] **Step 6: Build and inspect the unsigned macOS package locally**

Run:

```bash
pnpm tauri build --bundles app,dmg
pnpm run verify:bundle -- --platform macos
```

Expected: app launches, shows one `900 x 730` `壹印` window, closes cleanly, and contains no forbidden payload. Perform manual drag/drop, settings drawer, template drawer, preview, export, output-directory, and second-launch focus acceptance.

- [ ] **Step 7: Commit the clean cutover**

```bash
git add -A
git commit -m "refactor: replace Electron with Tauri"
git status --short --branch
```

Expected: clean worktree; final tree contains Tauri/Rust/React plus compatibility fixtures only.

- [ ] **Step 8: Prepare delivery without merging or releasing**

Rebase on current `origin/main`, rerun changed-risk checks after rebase, push only `codex/tauri-rewrite`, create a new assigned PR with English title and block-separated English/Chinese body, and take one immediate checks snapshot. Do not watch, merge, or publish.

```bash
git fetch origin main
git rebase origin/main
git push -u origin HEAD:codex/tauri-rewrite
gh pr create --base main --head codex/tauri-rewrite --assignee @me --title "refactor: replace Electron with Tauri"
gh pr checks
```

## Plan Self-Review Checklist

- Every approved design section maps to at least one task: compatibility fixtures (1, 4, 9), architecture (2-6), persistence/import (7), resources/EXIF (8), rendering (9), tasks (10), Tauri/security (11, 15), React parity (12-14), packaging/CI/removal/delivery (15-16).
- Stable identifiers and signatures are introduced before use: domain types in Tasks 3-4, ports/use cases in Tasks 5-6, adapters in Tasks 7-10, DTO/client boundary in Tasks 11-12.
- No task asks an implementer to invent a dependency, command name, event name, error code, path, state owner, or test threshold.
- Temporary Electron fixture code is explicitly removed in Task 16 and cannot survive final-tree verification.
- The plan contains no implementation authorization; execution begins only after the user chooses a compatible execution option.

## Execution Handoff

Two execution options are compatible with the current no-subagent requirement:

1. **Inline DIRECT execution in this task (recommended):** invoke `superpowers:executing-plans`, execute tasks in order with review checkpoints, and keep agent budget `0`.
2. **Fresh DIRECT task:** create a new clean-context task on `codex/tauri-rewrite` with a sanitized handoff, then invoke `superpowers:executing-plans` there.

Subagent-driven execution is intentionally unavailable in this task. It requires the user to create and approve a fresh MANAGED task with a new execution contract.
