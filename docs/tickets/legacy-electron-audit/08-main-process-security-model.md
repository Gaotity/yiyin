# 08 — Main-Process Security Model: TOCTOU on Main Path, Missing Resource Lifecycle

> **Source:** GitHub issue #34 — https://github.com/Gaotity/yiyin/issues/34 (labels: bug, audit, P1, area:security)
> **Audit context:** adversarial audit of the pre-rewrite Electron/Svelte tree
> at `7e115bb` (`codex/harden-project-foundation`); that tree was deleted from
> `main` by the Tauri/Rust rewrite (PR #3). Findings quote legacy `file:line`
> paths that no longer exist — re-validate against current `main` before acting.

**What to build:** Close the remaining gaps in the main-process security model
(dimension score: **8/10**). webPreferences, permission handlers, CSP, and
protocol path resolution all held up under adversarial probing; the fixes
would eliminate a TOCTOU on the main processing path, add a missing resource
lifecycle, and tighten redirect handling, dead code, and protocol response
limits.

**Blocked by:** None

**Status:** needs-triage

- [ ] Watermark task uses the renderer-supplied `file.path` instead of the
  registry-canonicalized path (`registry` computes realPath then discards
  it) — TOCTOU; invariant 6 ("re-validate before every operation") is not
  honored on the main processing path — `electron/ipc/register.ts:73`
- [ ] No unregister/release lifecycle anywhere: `removeFont` deletes the file
  but leaves a dangling registry entry; `registerOverlay` copies with a new id
  each time and never cleans old slot files (disk leak) —
  `electron/resources/registry.ts`, `electron/ipc/register.ts:98-114`
- [ ] `will-redirect` is not handled — a 302 can bypass the navigation
  whitelist (dev-server exposure only) — `electron/main/create-window.ts:24`
- [ ] Delete dead `canLoadApplicationUrl` (no callers; can net.fetch arbitrary
  URLs) — `electron/main/protocol.ts:72-75`
- [ ] Protocol responses read the whole file into memory with no serve-time
  size cap (registration cap can be bypassed by swapping the file) —
  `electron/main/protocol.ts:54`
