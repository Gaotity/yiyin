# 10 — Dependencies & Supply Chain: db-ui Pulls EOL svelte 3.59.2 into Renderer Tree

> **Source:** GitHub issue #36 — https://github.com/Gaotity/yiyin/issues/36 (labels: dependencies, audit, P1, area:dependencies)
> **Audit context:** adversarial audit of the pre-rewrite Electron/Svelte tree
> at `7e115bb` (`codex/harden-project-foundation`); that tree was deleted from
> `main` by the Tauri/Rust rewrite (PR #3). Findings quote legacy `file:line`
> paths that no longer exist — re-validate against current `main` before acting.

**What to build:** Close the remaining dependency supply-chain gaps in the
legacy tree (audit dimension score: **7/10**). Production deps are already
clean — 3 pinned deps, `audit --prod` passes, zero exotic resolutions,
`.env.local` untracked; the fixes target the EOL svelte runtime pulled into
the renderer tree by the UI library, high-severity dev-chain audit findings,
an overdue `minimumReleaseAge` exception, and a build-time network check.

**Blocked by:** None

**Status:** needs-triage

- [ ] `@ggchivalrous/db-ui@1.3.1` pulls EOL `svelte@3.59.2` into the renderer
  dependency tree (mXSS advisory); dual svelte runtimes in the renderer
  bundle — evaluate an override or replacement
- [ ] Full `pnpm audit` reports 3 high in dev chains: `fast-uri` (via
  electron-builder>ajv), `brace-expansion` (via eslint chain), vite dev-server
  advisories — track upgrades
- [ ] Remove the overdue electron `minimumReleaseAge` exception (self-described
  as temporary, still present 12 days later) — `pnpm-workspace.yaml:3`
- [ ] Disclose or neutralize `simple-update-notifier` (build-time network check
  via electron-builder) — conflicts with the no-network hardening posture
