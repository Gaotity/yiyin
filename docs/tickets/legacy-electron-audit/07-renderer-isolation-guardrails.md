# 07 — Renderer Isolation Guardrails

> **Source:** GitHub issue #33 — https://github.com/Gaotity/yiyin/issues/33 (labels: bug, audit, P0, area:renderer)
> **Audit context:** adversarial audit of the pre-rewrite Electron/Svelte tree
> at `7e115bb` (`codex/harden-project-foundation`); that tree was deleted from
> `main` by the Tauri/Rust rewrite (PR #3). Findings quote legacy `file:line`
> paths that no longer exist — re-validate against current `main` before acting.

**What to build:** Automated guardrails for renderer isolation in the build
and lint setup (dimension score: **8/10**). Actual isolation held up in the
audit — zero Node/Electron API usage in `web/` and `common/`, clean bundle,
tight CSP — but nothing prevents a regression: the fixes would stop a stray
import from bundling main-process code into the renderer and keep
build-machine paths out of generated env files.

**Blocked by:** None

**Status:** needs-triage

- [ ] **P0** Renderer `resolve.alias` mixes in all electron aliases
  (`@root`/`@src`/`@config`/`@modules`/`@utils`): one stray import bundles
  main-process code into the renderer, and no rollup `external` backstop
  exists — `vite.config.ts:138`
- [ ] Add an eslint boundary rule (`no-restricted-imports` or equivalent):
  `web/` and `common/` must not import `electron/` or `node:*` — currently
  enforced only by review discipline — `eslint.config.ts`
- [ ] `.env.local` is written with absolute build-machine paths
  (`VITE_DIST_ELECTRON`/`VITE_WEB`) on every build — `vite.config.ts:45`
