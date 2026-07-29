# 09 — Types & Lint: Any Escapes on the EXIF Chain, Schema/Type Drift

> **Source:** GitHub issue #35 — https://github.com/Gaotity/yiyin/issues/35 (labels: bug, audit, P1, area:types)
> **Audit context:** adversarial audit of the pre-rewrite Electron/Svelte tree
> at `7e115bb` (`codex/harden-project-foundation`); that tree was deleted from
> `main` by the Tauri/Rust rewrite (PR #3). Findings quote legacy `file:line`
> paths that no longer exist — re-validate against current `main` before acting.

**What to build:** Close the remaining type-safety and lint gaps in the legacy
tree (audit dimension score: **5/10**). `strict` + `noUncheckedIndexedAccess`
are genuinely on and there are zero `@ts-ignore`; the fixes remove the `any`
escapes that defeat indexed-access checking on the EXIF chain, repair zod ↔ TS
drift in the config models, and resolve an undocumented lint exclusion.

**Blocked by:** None

**Status:** needs-triage

- [ ] Remove `any` escapes that defeat `noUncheckedIndexedAccess`:
  `common/modules/exif-format/index.ts:6,10,27` (whole EXIF chain),
  `common/utils/base.ts:5` (`catch (e: any)`), `common/utils/base.ts:59`,
  `electron/src/modules/image-tool/index.ts:380` (`as any`)
- [ ] Fix zod ↔ TS drift: `FontSettingsOverride` is all-optional while the
  schema requires `size`/`font` (type allows what runtime rejects); template
  `font.use` exists in schema but not in the TS type —
  `common/models/config.ts:12,33`, `common/config/schema.ts:7-8,34`
- [ ] Justify or remove the undocumented `electron/src/modules/logger/**` lint
  exclusion — `eslint.config.ts:8`
- [ ] If the Svelte 5 migration is redone: clear the 51 pre-existing
  svelte-check errors and keep `pnpm ci` green before committing `[lost-changes]`
