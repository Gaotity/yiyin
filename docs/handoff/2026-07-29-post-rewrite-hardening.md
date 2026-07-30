# Handoff — yiyin post-rewrite hardening

Date: 2026-07-29. Session: long-running implement/triage marathon on the `yiyin` repo (Tauri + Rust + React desktop app, public, `Gaotity/yiyin`).

## Where things stand

- **Worktree in use:** `/Users/terrence-tang/Coding/yiyin-main` — detached HEAD at `31ede93` (= `origin/main`). Use it for all repo work. (The sibling `/Users/terrence-tang/Coding/yiyin` worktree belongs to another workstream — do not touch.)
- **Done and merged** (all squash-merged to main, branches deleted):
  - PR #45 — ticket 01 registration without full-pixel decodes
  - PR #46 — ticket 02 sub-second shutter display
  - PR #47 — ticket 03 renderer import-boundary guardrail
  - PR #48 — ticket 05 graceful legacy config import
- **Remaining work — exactly one ticket:**
  - `docs/tickets/post-rewrite-hardening/04-dangling-v1.7.0-tag.md` — **ready-for-human**, blocked on the user's decision, not agent-delegable:
    - Option A: delete the dangling `v1.7.0` tag (`f7b493a`, points at the deleted legacy tree, not an ancestor of main) and keep the 1.6.x version line.
    - Option B: keep the tag as legacy history and bump version metadata (`package.json`, `src-tauri/tauri.conf.json`, `CHANGELOG.md`) past 1.7.0 so the next release cannot collide.
  - After the user picks: record the decision in the ticket, align version metadata per the ticket's acceptance criteria, mark it completed in the delivering PR. `00-map.md` in the same directory is the cluster index to update.
- **Unrelated open item:** dotfiles PR `Gaotity/dotfiles#147` (global `AGENTS.md` language preference: keep English technical terms in Chinese replies) awaits the user's merge.

## How this repo works (do not re-derive)

- **Tickets are local files**, the canonical tracker: `docs/agents/issue-tracker.md`. Statuses: `needs-triage` / `in progress` / `completed` / `wontfix` (both terminal). Frontier = first non-terminal, unblocked ticket in hub order. `ready-for-human` is never claimed by an agent.
- **Implement flow used for all four tickets:** claim ticket (`Status: in progress`) → TDD red→green slices at public seams → full verification → two-axis code-review (parallel read-only sub-agents: Standards vs Spec, fixed point `origin/main`) → fix review findings → ticket checkboxes + `Status: completed` in the same PR → bilingual PR (English block, `---`, Chinese block, shared `## References`) with `--assignee Gaotity` → `@cursor review` + `@codex review` comments → `codex-pr-finalize <url>` → background read-only monitor for checks → **merge only on explicit user instruction**, `--squash`.
- **Verification commands:** `cargo test --workspace --locked` (includes ~2.5min golden parity suite), `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `pnpm test`, `pnpm typecheck`, `pnpm format:check`, `pnpm lint`.

## Environment traps already learned

- **`codex-pr-finalize` flips the gh active account to `terrence-kira` without restoring it.** Run `gh auth switch --user Gaotity` afterwards. The user has said PR author identity does not matter (work computer, mixed accounts) — do not chase or "fix" authorship.
- **Ruleset "Protect main":** required checks (Frontend/Rust/Security quality + macOS/Windows package smoke), strict up-to-date policy, and **required review-thread resolution**. `mergeStateStatus: BLOCKED` usually means unresolved codex review threads → evaluate them (they have all been valid so far), fix, reply bilingually, resolve via GraphQL `resolveReviewThread`. `BEHIND` → `git rebase origin/main && git push --force-with-lease`, checks re-run (~12 min).
- **macOS DMG bundling flakes** (`bundle_dmg.sh`/hdiutil) — retry the failed job (`gh run rerun <run-id> --failed`) before investigating.
- **Native package gate:** docs-only and `tests/security`-only PRs skip the macOS/Windows smoke jobs (required checks still satisfied); code PRs run them for real (~4 min macOS, ~10 min Windows).
- **Rust lints:** `clippy::float_cmp` denies `==` on f64 — compare `< 1.5`-style bounds or typed values instead (see `format_shutter` in `crates/yiyin-infrastructure/src/metadata.rs`).
- zune-jpeg gray-fills truncated scans (non-strict) — a truncated JPEG cannot discriminate full-decode vs header-only; use PNG truncation (see the registration test in `crates/yiyin-infrastructure/tests/resources.rs`).

## Communication

- Reply in Simplified Chinese; keep technical terms (tickets, PR, checks, worktree, branch, commit) in English — now a global `AGENTS.md` rule.
- GitHub PR bodies and PR comments **must** be bilingual (English block first, then Chinese, shared References section). Bot triggers (`@cursor review`, `@codex review`) stay exact.

## Suggested skills

- **`/implement`** — for executing ticket 04 once the user picks option A or B (it is small: tag command + version metadata + docs).
- **`/tdd`** — any code change inside that execution (version consistency tests likely exist in `tests/security/policy.test.ts`).
- **`/code-review`** — before committing, two-axis review of the diff against `origin/main`.
- **`/triage`** — only if new raw issues arrive; nothing pending.
- **`/ask-matt`** — if unsure which flow fits the next piece of work.

## Suggested first actions next session

1. Ask the user for the ticket 04 decision if not given (Option A vs B — do not guess; tag mutation is outward-facing).
2. Execute per the ticket, update `00-map.md`, open the PR with the conventions above.
3. Optionally nudge the user about dotfiles PR #147.
