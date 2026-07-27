# 15 — Security, Supply-Chain, CI, and Native Packaging Gates

**What to build:** The enforcement layer that keeps the rewrite shippable: six required CI jobs (frontend quality, Rust quality, security, golden compatibility, macOS packaging smoke, Windows packaging + desktop smoke). A release-age check script fails on npm or Cargo direct dependencies younger than 24 hours; a bundle-verification script checks product name, identifier, version, icon, unsigned status, and the absence of Electron/Node/Sharp/Svelte/`db-ui`/sidecars/remote URLs/source maps/generic capabilities. A compile-time-only `e2e-fixture` Cargo feature (off by default, proven by policy tests) pre-registers fixture images through `RegisterImages` for desktop smoke tests — no commands, no paths to React. A minimal W3C client built on Node's native `fetch` drives `tauri-driver` for the Windows desktop smoke (no WebdriverIO/Selenium). Workflows pin every Action to a full 40-character SHA, install cargo-deny and tauri-driver as workflow-level tools, use frozen/locked installs, run CodeQL for JS/TS + Rust, and keep Dependabot split per ecosystem with no auto-merge.

**Blocked by:** All of tickets 02–14 (gates need every manifest, script, and artifact to exist)

**Status:** completed

- [x] `pnpm audit --audit-level high` clean; release-age check passes; `cargo deny check` passes
- [x] Security-policy and package tests pass: every Action pinned to a full SHA; Dependabot isolated per ecosystem; production builds contain no `e2e-fixture` feature, paths, or environment variables
- [x] All six CI jobs green, including macOS launch smoke and Windows `tauri-driver` desktop smoke
