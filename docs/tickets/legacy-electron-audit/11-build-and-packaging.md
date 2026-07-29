# 11 — Build & Packaging: verify-package Silently Skips Fuse Checks

> **Source:** GitHub issue #37 — https://github.com/Gaotity/yiyin/issues/37 (labels: bug, audit, P1, area:build)
> **Audit context:** adversarial audit of the pre-rewrite Electron/Svelte tree
> at `7e115bb` (`codex/harden-project-foundation`); that tree was deleted from
> `main` by the Tauri/Rust rewrite (PR #3). Findings quote legacy `file:line`
> paths that no longer exist — re-validate against current `main` before acting.

**What to build:** Harden the build & packaging verification path (dimension
score: **7/10**). Release quarantine is real (no publish config,
`--publish never` everywhere, fuses asserted on real artifacts in CI, all
actions SHA-pinned); the remaining fixes would close the gaps in
`verify-package` fail-closed behavior, binary blacklist coverage, artifact
upload policy, and audit-flag parity between `pnpm ci` and CI.

**Blocked by:** None

**Status:** needs-triage

- [ ] `pnpm verify:package` without an executable argument silently skips all
  fuse checks yet still prints "Verified" — make it fail-closed —
  `scripts/verify-package.mjs:7,29`
- [ ] Binary blacklist (`^exiftool$`/`^ffmpeg`) is bypassable by renaming; >2MB
  files skip text scanning; `app.asar` contents are never inspected —
  `scripts/verify-package.mjs:8,23`
- [ ] `package.yml` uploads unsigned artifacts with 7-day retention — a soft
  distribution channel on a public repo; reconsider or document —
  `.github/workflows/package.yml:55-61`
- [ ] Keep `pnpm ci` audit flags in sync with CI (`--prod` vs full audit
  diverged in the lost working-tree changes) `[lost-changes — re-check if
  re-applied]`
