# 00 — Map: Post-Rewrite Hardening

Hub for the findings that survived the legacy-audit triage (PR #44), restated
against the post-rewrite Tauri/Rust/React stack. Source audit record:
`docs/tickets/legacy-electron-audit/00-tracking-hub.md`.

**Status:** in progress

## Ticket index

| Ticket | Summary | Status | Blocked by |
|---|---|---|---|
| `01-registration-without-full-decodes.md` | Header-only probing and blocking-context registration; drop dead DTO field | completed (PR #45) | None |
| `02-sub-second-shutter-display.md` | 0.67–1s exposures render as `1/1` | completed (PR #46) | None |
| `03-renderer-import-boundary-guardrail.md` | Automated `@tauri-apps/api` / `node:*` import boundary for `src/` | completed (PR #47) | None |
| `04-dangling-v1.7.0-tag.md` | Non-ancestor tag vs 1.6.0 version metadata | ready-for-human | None |
| `05-graceful-legacy-config-import.md` | Legacy import must not abort startup on invalid values | ready-for-agent | None |

## Notes

- All five tickets are independent; any one can be picked up first.
- `04` is maintainer-only: it decides the tag/version line before any release.
