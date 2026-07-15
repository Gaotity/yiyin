# Project Foundation Security Hardening Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish a maintainable, least-privilege Electron foundation for Yiyin while preserving the existing desktop workflows and keeping application distribution quarantined.

**Architecture:** Pure TypeScript models and the `PlatformBridge` contract live in `common/`. Electron owns trusted paths, resource registration, image processing, configuration persistence, IPC validation, and protocol handling. The renderer receives only validated data, opaque resource identifiers, and safe application URLs.

**Tech Stack:** Node.js 24, pnpm 11.13.0, Electron 43.1.1, electron-builder 26.15.3, Svelte 4.2.20, Vite 5.4.21, TypeScript 5.9.3, Sharp 0.35.3, ExifReader, Zod, Vitest, Playwright, and GitHub Actions.

## Global Constraints

- Keep the existing desktop-oriented directory layout; do not convert the repository into a monorepo.
- Use `GPL-3.0-only` consistently in package and documentation metadata.
- Use `Gaotity/yiyin`, `io.github.gaotity.yiyin`, and the existing product name `壹印`.
- Pin `@ggchivalrous/db-ui` to exactly `1.3.1` and document its provenance and maintenance risk.
- Do not publish releases, public unsigned installers, signing credentials, or automatic client updates.
- Do not add an upstream remote or reintroduce original-project release endpoints and tokens.
- Remove bundled assets that lack a documented redistribution basis; use system fonts and text-only camera brands.
- Require explicit runtime validation at every trust boundary and stable typed errors at every IPC boundary.
- Keep external collaboration content bilingual, with English first and Simplified Chinese second.

## Security Invariants

1. Renderer processes run with `nodeIntegration: false`, `contextIsolation: true`, `sandbox: true`, and `webSecurity: true`.
2. Renderer code never imports Electron or Node APIs and never receives an absolute local path.
3. The preload exposes only the explicit `PlatformBridge`; it never exposes `ipcRenderer`, Electron events, or dynamic route maps.
4. Every IPC request is schema-validated and accepted only from the main frame at an approved application origin.
5. IPC failures use stable error codes and redacted messages; stack traces, host paths, and Electron objects never cross into the renderer.
6. File work operates on registered opaque resource IDs. The main process retains canonical paths and revalidates containment before each operation.
7. Image and font ingestion verifies signatures and extensions, enforces file, batch, and pixel limits, and rejects traversal and decompression-bomb inputs.
8. Production pages load through the restricted `yiyin://` protocol with a strict Content Security Policy. Development traffic binds only to `127.0.0.1`.
9. Navigation, window creation, permission requests, and external URL opening default to deny. Only explicitly allowed HTTPS destinations may open externally.
10. Packaged applications use ASAR and Electron Fuses to disable RunAsNode, `NODE_OPTIONS`, and the CLI inspector while enforcing ASAR integrity and app-only ASAR loading.
11. Configuration has an explicit schema version. A valid previous configuration is backed up before a migrated configuration is committed atomically.
12. CI and packaging run without release secrets and use read-only GitHub token permissions unless a narrowly scoped job documents otherwise.

## Migration Sequence

### Task 1: Repository and engineering baseline

**Files:** `package.json`, `pnpm-lock.yaml`, `pnpm-workspace.yaml`, `.gitignore`, `.editorconfig`, `AGENTS.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CODEOWNERS`, `docs/third-party-assets.md`, `tsconfig*.json`, `eslint.config.ts`.

- [ ] Pin Node.js, pnpm, framework, Electron, builder, Sharp, and UI dependency versions.
- [ ] Replace legacy scripts with the standardized development, validation, build, packaging, and CI scripts.
- [ ] Split shared, web, Electron, and test TypeScript configurations with the required strictness flags.
- [ ] Add repository contribution, security reporting, ownership, and third-party asset documentation.
- [ ] Generate and verify a frozen pnpm lockfile.

**Verification:** `corepack pnpm install --frozen-lockfile`, `corepack pnpm lint`, and `corepack pnpm typecheck`.

### Task 2: Shared platform contract and typed errors

**Files:** `common/platform/bridge.ts`, `common/platform/result.ts`, `common/platform/resources.ts`, `common/config/schema.ts`, `web/global.env.d.ts`.

- [ ] Define `PlatformBridge` namespaces for `app`, `config`, `files`, `tasks`, and `events`.
- [ ] Define `ResourceDescriptor` using `resourceId`, `displayName`, and `resourceUrl` only.
- [ ] Define stable discriminated success and failure results without implementation-specific payloads.
- [ ] Version the public configuration model independently of Electron persistence details.
- [ ] Type `window.platform` against the shared contract and remove renderer Electron declarations.

**Verification:** compile the shared and web projects without Node or Electron ambient types.

### Task 3: Electron window, protocol, and navigation boundary

**Files:** `electron/main/create-window.ts`, `electron/main/security.ts`, `electron/main/protocol.ts`, `electron/main/index.ts`, `electron/src/path.ts`, `web/main/index.html`, `vite.config.ts`.

- [ ] Register the privileged `yiyin` scheme before application readiness and serve only contained renderer assets.
- [ ] Enforce the BrowserWindow security preferences and global renderer sandbox.
- [ ] Install default-deny navigation, popup, permission, and external-link handlers.
- [ ] Apply a strict CSP compatible with the packaged application and the loopback-only development server.
- [ ] Verify the production protocol never resolves outside the packaged web root.

**Verification:** unit tests cover protocol containment and URL allowlisting; Electron integration tests prove Node APIs and remote scripts are unavailable.

### Task 4: Preload and IPC boundary

**Files:** `electron/preload/index.ts`, `electron/ipc/channels.ts`, `electron/ipc/schemas.ts`, `electron/ipc/register.ts`, `electron/ipc/sender.ts`, `electron/ipc/errors.ts`, `electron/router-config.ts` (remove after migration).

- [ ] Replace dynamic route generation with an explicit immutable bridge implementation.
- [ ] Validate every request with Zod before dispatch.
- [ ] Verify the invoking sender URL, main-frame identity, and expected application origin.
- [ ] Map internal exceptions to redacted stable error results.
- [ ] Return an unsubscribe closure for each renderer event subscription.

**Verification:** unit tests cover valid requests, malformed requests, hostile senders, subframes, error redaction, and repeated subscribe/unsubscribe cycles.

### Task 5: Resources, image limits, and configuration persistence

**Files:** `electron/resources/registry.ts`, `electron/resources/policy.ts`, `electron/resources/protocol.ts`, `electron/config/store.ts`, `electron/config/migrate.ts`, related router modules, and renderer stores/components that currently hold paths.

- [ ] Register user-selected and application-managed files under random resource IDs.
- [ ] Canonicalize and revalidate contained paths while rejecting symlink escapes and traversal.
- [ ] Verify JPEG, PNG, WebP, and supported font signatures; enforce fixed file, batch, and pixel limits.
- [ ] Open the configured output directory without accepting a renderer-supplied path.
- [ ] Back up the previous configuration, migrate by schema version, and atomically replace only validated configuration.

**Verification:** unit tests cover containment, signatures, thresholds, decompression-bomb metadata, migration, backup, and recovery.

### Task 6: Native toolchain removal and pure-library image processing

**Files:** `electron/src/modules/exiftool/**`, `electron/src/modules/image-tool/**`, `scripts/install-exiftool.ts`, `vite.config.ts`, `package.json`, release scripts, and bundled archives.

- [ ] Replace shell-based ExifTool and `exif-parser` fallbacks with ExifReader normalization.
- [ ] Replace fluent-ffmpeg blur work with Sharp and verify output dimensions and blur behavior.
- [ ] Remove ExifTool, FFmpeg, archive extraction, release automation, Octokit, and MIME-only dependencies and scripts.
- [ ] Remove automatic update checks and original-project release endpoints.
- [ ] Confirm the source and packaged application contain no shell-spliced image paths, release tokens, or bundled unknown executables.

**Verification:** EXIF and Sharp unit tests pass; repository and package scans find none of the removed tools or archives.

### Task 7: Asset and platform-boundary cleanup

**Files:** `web/public/**`, `static/**`, `icon/**`, UI components and style sheets that refer to removed assets, `docs/third-party-assets.md`.

- [ ] Remove donation QR codes, community details, bundled fonts, camera logos, legacy screenshots, samples, and original application icons without clear authorization evidence.
- [ ] Use system font stacks and text-only brand labels.
- [ ] Keep icon configuration unset or use builder defaults; do not introduce a production brand asset.
- [ ] Record all retained third-party packages and assets, including license source and residual maintenance risk.
- [ ] Keep mobile reuse limited to shared pure TypeScript contracts and models.

**Verification:** source and built output contain no removed assets or references; shared TypeScript has no Electron imports.

### Task 8: CI, packaging quarantine, and dependency automation

**Files:** `.github/workflows/ci.yml`, `.github/workflows/build.yml`, `.github/workflows/security.yml`, `.github/workflows/package.yml`, `.github/dependabot.yml`, `scripts/verify-package.mjs`.

- [ ] Replace the five legacy workflows with quality CI, cross-platform build, security review, and manual artifact-only packaging.
- [ ] Pin every action to a complete commit SHA and set default permissions to `contents: read`.
- [ ] Publish stable aggregate jobs named `CI / required` and `Security / required`.
- [ ] Run lint, typecheck, tests with coverage, high-level audit, production build, and platform package smoke checks.
- [ ] Verify ASAR, Fuses, identity metadata, and the absence of disallowed archives, tokens, and executables.
- [ ] Configure weekly grouped pnpm and GitHub Actions dependency updates and security updates.

**Verification:** validate workflow syntax locally where feasible and inspect GitHub check names after the pull request is created.

### Task 9: GitHub repository governance

**Remote settings:** repository security features, Actions permissions, and a `main` branch ruleset.

- [ ] Enable Dependabot alerts, automated security fixes, secret scanning, push protection, code scanning, and private vulnerability reporting where supported.
- [ ] Set Actions workflow permissions to read-only and disable Actions-created or Actions-approved pull requests.
- [ ] Create an active `main` ruleset requiring pull requests, resolved discussions, linear history, `CI / required`, and `Security / required`, with deletion and force pushes blocked and administrators included.
- [ ] Require `@Gaotity` ownership for `.github/workflows/**`.

**Verification:** query the GitHub API after mutation and save a redacted settings snapshot in the delivery notes.

### Task 10: Final verification and pull request delivery

**Files:** all changed files and the final Git commit.

- [ ] Run `corepack pnpm ci` from a clean dependency state.
- [ ] Run package smoke checks for the locally available platform and ensure GitHub covers Windows.
- [ ] Review the complete diff against every requirement in this plan.
- [ ] Commit as `chore: harden project foundation` and push only `codex/harden-project-foundation`.
- [ ] Create a bilingual pull request with the English block first and Simplified Chinese block second.
- [ ] Take one immediate pull request status snapshot; do not merge, publish, or start a long-running watch.

## Residual Risks

- `@ggchivalrous/db-ui@1.3.1` is MIT-licensed but has concentrated maintainer risk and remains a future replacement candidate.
- Existing UI and image-generation behavior may require focused regression work after strict path and resource-ID boundaries replace direct path access.
- Unsigned local packages may trigger operating-system warnings and are test artifacts only.
- Dependency freshness is deliberately delayed by 24 hours; urgent security releases require an explicit reviewed exception.
- Cross-platform packaging correctness ultimately depends on GitHub-hosted macOS and Windows runners.
- A new icon, full asset provenance review, Apple notarization, and Windows signing remain outside this change.

## Release Isolation Conditions

Public distribution remains blocked until all of the following are complete:

- A newly commissioned or fully licensed application icon and store asset set is approved.
- Retained third-party assets have verifiable redistribution records.
- Apple signing and notarization credentials are configured in a protected release workflow.
- Windows code signing is configured in a protected release workflow.
- A release workflow is separately reviewed, authorized, and restricted to trusted maintainers and environments.
- Signed installers pass malware scanning, update-channel validation, and manual desktop regression tests.
- The repository owner explicitly authorizes a public release after reviewing these controls.
