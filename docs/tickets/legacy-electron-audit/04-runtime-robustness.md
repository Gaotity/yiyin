# 04 — Runtime Robustness: Exit Truncates In-Flight Writes, Unbounded Logs

> **Source:** GitHub issue #30 — https://github.com/Gaotity/yiyin/issues/30 (labels: bug, audit, P0, area:runtime)
> **Audit context:** adversarial audit of the pre-rewrite Electron/Svelte tree
> at `7e115bb` (`codex/harden-project-foundation`); that tree was deleted from
> `main` by the Tauri/Rust rewrite (PR #3). Findings quote legacy `file:line`
> paths that no longer exist — re-validate against current `main` before acting.

**What to build:** Harden the runtime lifecycle of the legacy Electron app
(dimension score: **5/10**). Fixes would make quit await in-flight image
writes instead of truncating JPEGs, stop duplicate batch paths from silently
dropping tasks, bound log growth with rotation or size caps, and make startup
and crash handling fail cleanly instead of leaving zombie processes, fake
crash records, or stack-less crash logs.

**Blocked by:** None

**Status:** needs-triage

- [ ] **P0** `Queue.close()` does not await in-flight tasks (`Promise.all` over
  a Map yields `[k,v]` entries that resolve immediately), and `before-quit`
  calls it with `void` → in-flight sharp writes are killed on exit, producing
  truncated JPEGs — `electron/src/modules/queue/index.ts:84`,
  `electron/main/app.ts:35-39`
- [ ] **P0** Duplicate paths in one batch silently drop a task: same
  `resourceId` → `queue.set` overwrites the earlier entry —
  `electron/src/modules/queue/index.ts:40`
- [ ] Logs append forever with no rotation or size cap (~10 lines per image) —
  `electron/src/modules/logger/index.ts:66-73`
- [ ] Config load failure leaves a zombie process (no window, no exit) —
  `electron/main/index.ts:15` lacks `.catch`
- [ ] `closeAllLogger` is an empty function registered as quit cleanup —
  `electron/src/modules/logger/index.ts:78`
- [ ] Launching a second instance always writes a fake crash record (`must be
  initialized`) — `electron/main/app.ts:23-26` vs `electron/main/index.ts:33`
- [ ] `uncaughtException` handler logs and **continues** (Node advises exit);
  double-writes via `uncaughtExceptionMonitor`; `format('%s', e)` drops the
  stack; crash.log is 0644 vs application.log 0600 —
  `electron/main/index.ts:36-46`
