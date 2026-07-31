# Project governance (Linear `Yiyin`)

Conventions for running the Linear [`Yiyin`](https://linear.app/wg-studio/project/yiyin-3ba958517518/overview)
project day to day, as set up on 2026-07-30. For tracker mechanics (API access,
skill-verb mappings, wayfinding) see `docs/agents/issue-tracker.md`.

## Milestones

Five project milestones group the work. Delivered milestones carry their actual
completion date as `targetDate`; open ones carry the current plan.

| Milestone | Contents | targetDate policy |
| --- | --- | --- |
| `Legacy Electron audit` | The 14 audit findings (Done/Canceled) | Actual triage date (2026-07-29) |
| `Tauri/Rust rewrite v2.0.0` | The 16 rewrite tickets | Actual completion (2026-07-27) |
| `Post-rewrite hardening` | The 6 hardening tickets | Actual completion (2026-07-29) |
| `ADR landings` | Tickets landing recorded ADRs (0002, 0003, …) | Delivery date of the batch |
| `Architecture deepening` | Open `architecture-backlog` candidates | Rolling plan, adjusted as candidates complete |

New work joins the milestone that fits; architecture-review output always lands
in `Architecture deepening`, ADR-landing tickets in `ADR landings`.

## Labels

All labels are scoped to the `Engineering` team — no workspace-level labels.

- **Triage roles** (`needs-triage`, `needs-info`, `ready-for-agent`,
  `ready-for-human`): per `docs/agents/triage-labels.md`. `wontfix` is the
  `Canceled` state, not a label.
- **`cluster:<feature-slug>`**: groups tickets of one feature cluster.
- **`architecture-backlog`**: open candidates from architecture reviews.
- **`migrated-from-repo`**: historical marker for the 2026-07-30 migration;
  never applied to new issues.
- **Linear defaults** (`Bug`, `Improvement`, `Feature`): the issue's kind.

## Relations

- **`blocks` / `blocked by`**: real dependency edges only — a ticket is
  unblocked when every blocking issue is `Done`. New `/to-tickets` output must
  declare its edges as native relations.
- **`related`**: supersession and soft links (e.g. a wontfix audit finding
  superseded by a hardening ticket).
- **Duplicates**: move to the `Duplicate` state and add a `related` edge to the
  surviving issue.

## Due dates

- Delivered work: `dueDate` is the actual completion (merge/triage) date.
- Open work: `dueDate` is a proposed target, set when the issue is scheduled
  and adjusted explicitly — never a silently slipping date.

## Estimates

Fibonacci points (`1, 2, 3, 5, 8`), set when an issue becomes
`ready-for-agent`/`ready-for-human`:

- `1` — a one-liner class change (docs, config, a single constant)
- `2` — single file, single concern
- `3` — one layer, moderate surface (a use case plus tests)
- `5` — multi-layer feature slice
- `8` — large multi-layer build with a heavy test surface

Completed historical issues carry retroactive estimates for calibration; do not
treat them as measurements.

## Priority

- Backlog architecture candidates: `High` for Strong-rated seams, `Normal` for
  Worth-exploring ones.
- Everything else starts at `None`; triage may raise it (`Urgent` is for
  user-facing breakage only).
- Done/Canceled history stays at `None`.
