# 05 — IPC Boundary: shadowRender Events Leak Host Paths to Renderer

> **Source:** GitHub issue #31 — https://github.com/Gaotity/yiyin/issues/31 (labels: bug, audit, P0, area:ipc)
> **Audit context:** adversarial audit of the pre-rewrite Electron/Svelte tree
> at `7e115bb` (`codex/harden-project-foundation`); that tree was deleted from
> `main` by the Tauri/Rust rewrite (PR #3). Findings quote legacy `file:line`
> paths that no longer exist — re-validate against current `main` before acting.

**What to build:** Harden the IPC boundary beyond channel validation, which is
already solid (13/13 channels zod-strict, sender anti-spoofing, sanitized
errors). Dimension score: **7/10**. The fixes would stop host absolute paths
leaking into the renderer via `shadowRender` events, deeply freeze the
`platform` preload bridge, validate payloads on the renderer side, cap
`completeTextRender` payload size, and prune dead contract surface.

**Blocked by:** None

**Status:** needs-triage

- [ ] **P0** `shadowRender` events leak host absolute paths into the renderer
  via `...item` / `...this.material.bg` spreads (`material.main/bg` carry
  `path`), contradicting the hardening plan's "host paths never cross into the
  renderer" — `electron/src/modules/image-tool/index.ts:431-441`
- [ ] `Object.freeze(platform)` is shallow; the five namespace objects
  (`app/config/files/tasks/events`) remain mutable in the main world —
  `electron/preload/index.ts:52`
- [ ] Renderer consumes event payloads and invoke returns with zero runtime
  validation (validation covers renderer→main only)
- [ ] `completeTextRender` allows ~400MB per call (20 × 20MB dataURLs) and
  `taskId` is not checked against pending renders —
  `electron/ipc/schemas.ts:17`, `electron/ipc/register.ts:137-140`
- [ ] Dead contract surface: `ipcSchemas.outputOptions` has no channel; error
  codes `CANCELLED/CONFIG_INVALID/FORBIDDEN/TASK_NOT_FOUND` are never produced
  by main — `electron/ipc/schemas.ts:11`, `common/platform/result.ts:2-10`
