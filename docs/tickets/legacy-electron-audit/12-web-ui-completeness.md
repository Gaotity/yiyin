# 12 — Web UI: Task Progress Never Updates on Screen

> **Source:** GitHub issue #38 — https://github.com/Gaotity/yiyin/issues/38 (labels: bug, audit, P0, area:web-ui)
> **Audit context:** adversarial audit of the pre-rewrite Electron/Svelte tree
> at `7e115bb` (`codex/harden-project-foundation`); that tree was deleted from
> `main` by the Tauri/Rust rewrite (PR #3). Findings quote legacy `file:line`
> paths that no longer exist — re-validate against current `main` before acting.

**What to build:** Fix the web UI's reactivity and small logic bugs (dimension
score: **5/10**). All UI entries are wired (no dead buttons), so the fixes
would make task progress update live on screen and clear the remaining
dead-code, TODO, and state-merge defects.

**Blocked by:** None

**Status:** needs-triage

- [ ] **P0** Task progress never updates: `imgInfoRecord[id].progress = n`
  is a property mutation that does not trigger Svelte reactivity — progress
  %, `handleCount`, and completion icons stay stale until `fileInfoList`
  changes — `web/main/components/actions/index.svelte:95`
- [ ] Delete dead code: `web/util/model-map.ts` (no importers),
  `calcAverageBrightness`, unused `toRoman`/`charToNumberChar`
- [ ] Resolve TODOs: canvas height still hardcoded — `web/modules/text-tool/index.ts:172,175`
- [ ] `font-select` reactive `system-ui` fallback breaks `clearable` and
  overwrites parent `conf.font` via `bind:value` —
  `web/components/font-select/index.svelte:16`
- [ ] Config store subscribe has no debounce → IPC write amplification
  while dragging sliders — `web/store/config.ts:57`
- [ ] `param-dialog` merges booleans/enums with `||`; model missing
  `caseType` → Radio group loses selection after edit —
  `web/main/components/param-dialog/index.svelte:37-46`
- [ ] `insertText` fallback path skips name→key conversion —
  `web/main/components/temp-setting/dialog.svelte:91,99`
- [ ] Failed `importFont` permanently blacklists the font in `loadFonts`
  (no retry) — `web/util/util.ts:93-101`
- [ ] `{@html item.desc}` rendering surface (static local source, low risk) —
  `web/main/components/header/index.svelte:38`
