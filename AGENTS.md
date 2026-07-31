# yiyin

## Coding instructions

All coding work in this repo follows the [mattpocock/skills](https://github.com/mattpocock/skills) engineering skills as the source of truth for process and conventions. Reach for them by default:

- **`tdd`** — features and bug fixes are built test-first (red-green-refactor).
- **`implement`** — spec/ticket-driven implementation; **`diagnosing-bugs`** — hard bugs and regressions.
- **`code-review`** — review a branch or PR before wrapping up; **`grill-me` / `grilling`** — stress-test a plan before building.
- **`domain-modeling` / `ubiquitous-language`** — domain terms live in `CONTEXT.md`; decisions are recorded as ADRs (see `docs/agents/domain.md`).
- **`to-spec`, `to-tickets`, `triage`, `qa`, `wayfinder`** — issue-tracker workflows per `docs/agents/issue-tracker.md` and `docs/agents/triage-labels.md`.

## Agent skills

### Issue tracker

Tickets live in Linear, in the [`Yiyin`](https://linear.app/wg-studio/project/yiyin-3ba958517518/overview) project (wg-studio workspace, team `Engineering` / `ENG`). Local files under `docs/tickets/` are the frozen pre-migration record (see `docs/agents/linear-migration.md`). See `docs/agents/issue-tracker.md` for tracker mechanics and `docs/agents/project-governance.md` for day-to-day conventions (milestones, labels, relations, due dates, estimates, priority).

### Triage labels

Default five-role vocabulary (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context — `CONTEXT.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.
