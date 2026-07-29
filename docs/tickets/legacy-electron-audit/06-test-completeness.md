# 06 — Test Completeness — Coverage Whitelist Theater, E2E Absent from CI

> **Source:** GitHub issue #32 — https://github.com/Gaotity/yiyin/issues/32 (labels: bug, audit, P0, area:tests)
> **Audit context:** adversarial audit of the pre-rewrite Electron/Svelte tree
> at `7e115bb` (`codex/harden-project-foundation`); that tree was deleted from
> `main` by the Tauri/Rust rewrite (PR #3). Findings quote legacy `file:line`
> paths that no longer exist — re-validate against current `main` before acting.

**What to build:** Close the test-completeness gaps in the legacy Electron/Svelte
test suite (dimension score: **4/10**). The existing tests are high quality — no
snapshots, no mock-only tests, real adversarial cases — but the fixes would make
coverage measurement honest, put integration tests into CI, and add unit tests
for the critical boundary code that is currently unmeasured.

**Blocked by:** None

**Status:** needs-triage

- [ ] **P0** Coverage whitelist theater: the committed 96.11% summary measures only
  206/2927 lines (6 whitelisted files); real denominator coverage is ~7% — widen
  `include` or stop committing the misleading summary — `vitest.config.ts:21-28`,
  `coverage/coverage-summary.json`
- [ ] Zero unit tests for the most critical boundary code: `electron/ipc/register.ts`,
  `electron/resources/registry.ts`, `electron/src/config.ts` (atomic write),
  `electron/main/*`, `electron/src/modules/image-tool/index.ts` (569-line core pipeline)
- [ ] Playwright integration tests are not in CI (`pnpm ci` and
  `.github/workflows/ci.yml` have no playwright step)
- [ ] Integration suite was red in its last run (`electron-security.spec.ts:31`
  timeout) — re-verify when the Svelte 5 migration is redone `[lost-changes]`
- [ ] `schemas.test.ts` covers only 3/10 schemas with generic `toThrow` assertions
- [ ] `blur.test.ts` clamp case only asserts `resolves.toBeInstanceOf(Buffer)` —
  `tests/image/blur.test.ts:25-26`
- [ ] `migrate.ts` font.map path-traversal branch untested (file at 76% line coverage)
- [ ] No tests import `common/` contracts (bridge/schema) or anything in `web/`
