# 02 — EXIF Formatting (5/10): User-Visible Output Errors, Zero Tests

> **Source:** GitHub issue #28 — https://github.com/Gaotity/yiyin/issues/28 (labels: bug, audit, P1, area:exif)
> **Audit context:** adversarial audit of the pre-rewrite Electron/Svelte tree
> at `7e115bb` (`codex/harden-project-foundation`); that tree was deleted from
> `main` by the Tauri/Rust rewrite (PR #3). Findings quote legacy `file:line`
> paths that no longer exist — re-validate against current `main` before acting.

**What to build:** Fix the EXIF formatting layer, which has user-visible
output bugs and no tests even though the reader is solid (audit dimension
score: **5/10**). The fixes would correct the malformed values users see in
watermarks and put the whole `exif-format` layer under test.

**Blocked by:** None

**Status:** needs-triage

- [ ] `ExposureCompensation` prints raw SRATIONAL text, e.g.
  `0.3333333333333333` for 1/3 EV — `common/modules/exif-format/base.ts:92-94`
- [ ] 0.7–0.9s shutter speeds are formatted as `1/1` — `common/modules/exif-format/base.ts:37-40`
- [ ] Add tests for the whole `exif-format` layer (currently zero; excluded from coverage `include`)
- [ ] Delete dead `web/util/model-map.ts` (no importers; logic has drifted, e.g. `α 7m4` with a space)
- [ ] Nikon `Model.replace(Make,'')` without trim → leading space in watermark;
  missing `Model` throws TypeError — `common/modules/exif-format/nikon.ts:7`
- [ ] `Flash()`/`ExposureMode()` are dead methods (fields never extracted by
  the reader) — `base.ts:106-115`
- [ ] `init()` mutates the caller's exif `Make` in place — `common/modules/exif-format/index.ts:34`
- [ ] No i18n layer: English program/metering/white-balance descriptions leak
  into watermarks; `translate.ts` is misnamed (only `toRoman`)
- [ ] `Math.round` returns number, violating the all-string `ExifData` contract
  (hidden by `any`) — `base.ts:54`
