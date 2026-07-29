# Issue tracker: local files under `docs/tickets/`

Tickets and PRDs for this repo are **local Markdown files committed with the code**. GitHub Issues is enabled on `Gaotity/yiyin` but is **not** the canonical tracker: the legacy-tree audit issues #26–#39 were converted to `docs/tickets/legacy-electron-audit/` on 2026-07-29, and new work is tracked locally. Pull requests remain on GitHub — use the `gh` CLI for PR operations.

## Conventions

- **Layout**: one directory per feature (`docs/tickets/<feature-slug>/`), one file per ticket named `NN-<slug>.md`, numbered from `01` in dependency order (e.g. `docs/tickets/tauri-rust-rewrite/`).
- **Ticket format**: a `# NN — <title>` heading, a `**What to build:**` paragraph, `**Blocked by:**` (ticket numbers or `None`), `**Status:**`, and a `- [ ]` checklist.
- **Status values**: `needs-triage` for raw incoming items, `in progress` once claimed, `completed` when the delivering PR merges. The triage-role vocabulary lives in `docs/agents/triage-labels.md`.
- **Create a ticket**: write the file in the feature directory and commit it — either with the work it describes or as a standalone docs commit.
- **Read a ticket**: read the file.
- **List open tickets**: `grep -r '^\*\*Status:\*\*' docs/tickets` — anything not `completed` is open.
- **Close a ticket**: set `**Status:** completed` and check every checkbox (with evidence notes) in the PR that delivers the work, so the ticket state lands on `main` together with the code.

## Pull requests as a triage surface

**PRs as a request surface: no.** _(Set to `yes` if this repo treats external PRs as feature requests; `/triage` reads this flag.)_

When set to `yes`, PRs run through the same roles as tickets, using the `gh pr` equivalents:

- **Read a PR**: `gh pr view <number> --comments` and `gh pr diff <number>` for the diff.
- **List external PRs for triage**: `gh pr list --state open --json number,title,body,labels,author,authorAssociation` then keep only `authorAssociation` of `CONTRIBUTOR`, `FIRST_TIME_CONTRIBUTOR`, or `NONE` (drop `OWNER`/`MEMBER`/`COLLABORATOR`).
- **Comment / label / close**: `gh pr comment`, `gh pr edit --add-label`, `gh pr close`.

GitHub shares one number space across issues and PRs, so a bare `#42` may be either — resolve with `gh pr view 42` and fall back to `gh issue view 42`.

## When a skill says "publish to the issue tracker"

Write the local ticket file under `docs/tickets/<feature-slug>/`. Do not call `gh issue create`.

## When a skill says "fetch the relevant ticket"

Read the ticket file under `docs/tickets/`.

## Wayfinding operations

Used by `/wayfinder`. The **map** is a hub ticket file in the feature directory (e.g. `docs/tickets/legacy-electron-audit/00-tracking-hub.md`) holding the Notes / Decisions-so-far / Fog body and an index of its child tickets.

- **Map**: a `00-<slug>.md` hub file in the feature directory.
- **Child ticket**: a numbered ticket file listed in the hub's index. Where a finding cluster has no hub yet, create the directory with the first ticket and add the hub when the map forms.
- **Blocking**: the `**Blocked by:**` line in each child file. A ticket is unblocked when every listed blocker reads `**Status:** completed`.
- **Frontier query**: scan the hub's index for child files whose status is not `completed` and whose blockers are all completed; first in hub order wins.
- **Claim**: set the child's `**Status:** in progress` — the session's first write to that file.
- **Resolve**: record the answer in the child file, set `**Status:** completed`, then append a context pointer to the hub's Decisions-so-far.
