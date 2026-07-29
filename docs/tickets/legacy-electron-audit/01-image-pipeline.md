# 01 — Image Pipeline: Resource Leaks and Dead Cancel Semantics

> **Source:** GitHub issue #27 — https://github.com/Gaotity/yiyin/issues/27 (labels: bug, audit, P0, area:image)
> **Audit context:** adversarial audit of the pre-rewrite Electron/Svelte tree
> at `7e115bb` (`codex/harden-project-foundation`); that tree was deleted from
> `main` by the Tauri/Rust rewrite (PR #3). Findings quote legacy `file:line`
> paths that no longer exist — re-validate against current `main` before acting.

**What to build:** Harden the legacy Electron image-tool pipeline (dimension
score: **4/10**). Fixes would stop cache intermediates from leaking on
failure/cancel paths, make cancellation real instead of dead code, prevent
same-name outputs from silently overwriting each other, and remove synchronous
main-process I/O so large batches no longer block the event loop — alongside
the remaining decode-cost, pixel-limit, dead-config, and preview-queue
findings.

**Blocked by:** None

**Status:** needs-triage

- [ ] **P0** Cache intermediates (`_bg/_main/_mask`) leak on failure/cancel
  paths — `delCacheFile()` only runs on success —
  `electron/src/modules/image-tool/index.ts:139-194`
- [ ] Cancel semantics are dead: `cancel()` has zero callers; `isCancelled` is
  never checked in `genWatermark`; `tasks.clear` only drains pending items
- [ ] Same-name outputs silently overwrite each other within a batch (filename
  dedup happens at construction, before any write)
- [ ] `init()` fully decodes + re-encodes a 100MP image just to obtain rotated
  dimensions — `electron/src/modules/image-tool/index.ts:100-101`
- [ ] Pixel limit vs downstream mismatch: policy allows 100MP but shadow
  rendering only caps width → canvas/dataURL rejection surfaces as a false 20s
  "timeout"
- [ ] `origin_wh_output` is a dead config key (zero references across the pipeline)
- [ ] Synchronous I/O in the main process (`writeFileSync`/`rmSync`/`renameSync`,
  sync base64 decode) blocks the event loop on large batches —
  `index.ts:360,382,406,413-414`
- [ ] Error path leaves `${mask}catch.png`, not covered by `delCacheFile` — `index.ts:412-414`
- [ ] `genPreview` bypasses the queue entirely: no concurrency cap, double full decode per preview
