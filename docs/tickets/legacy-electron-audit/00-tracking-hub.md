# 00 — Tracking Hub: Adversarial Audit Findings (Legacy Electron Tree)

> **Source:** GitHub issue #26 — <https://github.com/Gaotity/yiyin/issues/26>
> (converted to a local ticket on 2026-07-29; sub-issues #27–#39 are now
> tickets `01`–`13` in this directory)

**Status:** needs-triage

> **Read first — codebase validity.** This audit ran against `7e115bb` on
> `codex/harden-project-foundation`, the **pre-rewrite Electron/Svelte tree**
> that was deleted from `main` by the Tauri/Rust rewrite (PR #3, ticket
> `tauri-rust-rewrite/16`). Every finding references files that no longer exist
> on `main`. Triage each ticket as: (a) obsolete — drop; (b) portable lesson —
> restate against the Tauri/Rust/React stack and keep; or (c) still relevant
> cross-cutting concern (tests, docs, supply chain). Do not implement against
> the quoted `file:line` evidence directly.

## Context

An adversarial 14-dimension audit of HEAD (`7e115bb`, branch
`codex/harden-project-foundation`) was run with parallel read-only auditors.
Each dimension assumed the implementation was broken and tried to prove it with
`file:line` evidence. **Average score: 6.0/10.**

The findings were split into the sub-issues (all actionable checkboxes live
there). This file is the tracking hub. Note: the working tree was externally
reverted mid-audit and the uncommitted Svelte 5 migration changes were lost;
items marked `[lost-changes]` in the tickets only apply if that work is redone.

## P0 findings → where they live

| Finding | Ticket |
|---|---|
| `Queue.close()` doesn't await in-flight tasks; exit truncates JPEG writes | `04-runtime-robustness.md` |
| Duplicate paths in a batch silently drop a task | `04-runtime-robustness.md` |
| Cache intermediates leak on failure/cancel paths | `01-image-pipeline.md` |
| `shadowRender` events leak host absolute paths to the renderer | `05-ipc-boundary.md` |
| Coverage theater: 96.11% summary measures 206/2927 lines | `06-test-completeness.md` |
| Renderer build can import main-process code via electron aliases | `07-renderer-isolation-guardrails.md` |
| Task progress UI never updates | `12-web-ui-completeness.md` |

## Ticket index

| Ticket | Source issue | Dimension | Score | Priority |
|---|---|---|---|---|
| `01-image-pipeline.md` | #27 | Image pipeline | 4/10 | P0 |
| `02-exif-formatting.md` | #28 | EXIF reading & formatting | 5/10 | P1 |
| `03-config-migration.md` | #29 | Config & migration | 7/10 | P1 |
| `04-runtime-robustness.md` | #30 | Runtime robustness | 5/10 | P0 |
| `05-ipc-boundary.md` | #31 | IPC boundary | 7/10 | P0 |
| `06-test-completeness.md` | #32 | Test completeness | 4/10 | P0 |
| `07-renderer-isolation-guardrails.md` | #33 | Renderer isolation guardrails | 8/10 | P0 |
| `08-main-process-security-model.md` | #34 | Main-process security model | 8/10 | P1 |
| `09-types-and-lint.md` | #35 | Types & lint | 5/10 | P1 |
| `10-dependencies-supply-chain.md` | #36 | Dependencies & supply chain | 7/10 | P1 |
| `11-build-and-packaging.md` | #37 | Build & packaging | 7/10 | P1 |
| `12-web-ui-completeness.md` | #38 | Web UI completeness | 5/10 | P0 |
| `13-docs-accuracy.md` | #39 | Docs accuracy | 8/10 | P1 |

Not split out (no actionable items): renderer isolation actual state (verified
clean), uncommitted-changes audit (4/10, moot — the changes were lost; lessons
folded into tickets 06 and 09).

## What held up in the legacy tree (verified, not just claimed)

- webPreferences (`sandbox`/`contextIsolation`/no `nodeIntegration`),
  default-deny permission handlers, tight CSP, `yiyin://` path-traversal
  resistance under adversarial probing
- All 13 IPC channels zod-strict with sender anti-spoofing and sanitized errors
- Release quarantine: no publish config, `--publish never`, fuses asserted on
  real artifacts in CI, SHA-pinned actions
- Production dependency hygiene: 3 pinned deps, `audit --prod` clean, zero
  exotic lockfile resolutions
- Atomic config writes with `.bak` and `.invalid.bak` recovery; EXIF reader
  malformed-input tolerance
