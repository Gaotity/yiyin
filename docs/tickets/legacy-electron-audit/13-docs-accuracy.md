# 13 — Docs Accuracy: Hardening Plan Facts and Checkboxes Are Stale

> **Source:** GitHub issue #39 — https://github.com/Gaotity/yiyin/issues/39 (labels: documentation, audit, P1, area:docs)
> **Audit context:** adversarial audit of the pre-rewrite Electron/Svelte tree
> at `7e115bb` (`codex/harden-project-foundation`); that tree was deleted from
> `main` by the Tauri/Rust rewrite (PR #3). Findings quote legacy `file:line`
> paths that no longer exist — re-validate against current `main` before acting.

**What to build:** Bring the hardening plan and CHANGELOG back in line with
reality so the docs are trustworthy again (audit dimension score: **8/10**).
Most verifiable claims held up (engines, no ExifTool/FFmpeg bundling, no
auto-update, CONTRIBUTING commands, GPL-3.0, third-party asset provenance);
the fixes correct the stale stack versions, overclaimed guarantees, and
unchecked delivered work.

**Blocked by:** None

**Status:** needs-triage

- [ ] Fix stack versions in the hardening plan: it claims Svelte 4.2.20 /
  Vite 5.4.21; actual is 5.56.4 / 6.4.3 — `docs/security-hardening-plan.md:9`
- [ ] Reconcile plan claims with reality: "host paths never cross into the
  renderer" (broken — see the IPC issue), subscribe/unsubscribe cycle tests and
  config backup/recovery tests (do not exist), "migrate by schema version"
  (no version dispatch implemented) — `docs/security-hardening-plan.md:29,86,96,98`
- [ ] Check off the delivered checkboxes in the plan (all 52 are unchecked
  although the work landed in `af693ca`)
- [ ] Add a 1.6.0 entry to CHANGELOG; resolve the non-ancestor `v1.7.0` tag
  before any future ≥1.7.0 release
