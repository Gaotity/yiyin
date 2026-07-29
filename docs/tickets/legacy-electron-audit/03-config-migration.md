# 03 — Config & Migration — Failure Path Can Overwrite Real Config with Fake Defaults

> **Source:** GitHub issue #29 — https://github.com/Gaotity/yiyin/issues/29 (labels: bug, audit, P1, area:config)
> **Audit context:** adversarial audit of the pre-rewrite Electron/Svelte tree
> at `7e115bb` (`codex/harden-project-foundation`); that tree was deleted from
> `main` by the Tauri/Rust rewrite (PR #3). Findings quote legacy `file:line`
> paths that no longer exist — re-validate against current `main` before acting.

**What to build:** Harden the legacy config load/migration path so failures
cannot silently destroy user settings: a `getConfig()` failure must not leave
the store `initialized` and later overwrite the real main-process config with
fake defaults, defaults must stop drifting between the web store and the
Electron model, and migration plus corrupt-recovery must degrade gracefully
instead of resetting all blocks or crashing the main process. Dimension score:
**7/10**.

**Blocked by:** None

**Status:** needs-triage

- [ ] `getConfig()` failure still sets `initialized` in `finally`; any later
  edit then overwrites the real main-process config with fake defaults
  (persistent data loss) — `web/store/config.ts:57-70`
- [ ] Default-value drift between web store and electron model
  (`origin_wh_output` true/false, `bg_rate_show` true/false, `options.font`
  `'system-ui'`/`''`) — `web/store/config.ts:20-26` vs
  `electron/config/model.ts:46-53`
- [ ] Migration uses a single `safeParse` over four blocks: one out-of-range
  legacy field resets **all** blocks to defaults —
  `electron/config/migrate.ts:17-28`
- [ ] `FieldInfoItem.font` contract is wider than the schema (all-optional
  type vs required `size`/`font` in `common/config/schema.ts:7-8`)
- [ ] Legacy `version` string is preserved but never updated; no
  version-dispatched migration chain (the hardening plan claims one)
- [ ] Corrupt-recovery catch can re-throw if `writeAtomically` fails (disk
  full/EACCES) → main-process startup crash — `electron/src/config.ts:45-50`
- [ ] Add tests for backup/recovery paths (`writeAtomically`,
  `.invalid.bak`) — currently untested
