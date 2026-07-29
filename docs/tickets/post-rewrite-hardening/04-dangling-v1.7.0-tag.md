# 04 — Resolve the Dangling `v1.7.0` Tag Before the Next Release

**What to build:** The `v1.7.0` tag (`f7b493a`, from the legacy pre-rewrite
line) is not an ancestor of current `main`, while version metadata still reads
`1.6.0`. Any future release numbered ≥1.7.0 from this line collides with the
dangling tag. Restated from the legacy audit
(`legacy-electron-audit/13-docs-accuracy.md`, triaged 2026-07-29).

**Blocked by:** None

**Status:** ready-for-human

## Agent Brief

**Category:** bug

**Current behavior:**
`git merge-base --is-ancestor v1.7.0 HEAD` fails: the tag points at the
removed legacy tree. `package.json` and `src-tauri/tauri.conf.json` still say
`1.6.0`, and the changelog's unreleased section follows `v1.6.0`.

**Desired behavior:**
The maintainer picks one resolution, records it in this ticket, and the repo
is left consistent:

- **Option A** — delete the `v1.7.0` tag (it describes a tree that no longer
  exists) and keep the current version line.
- **Option B** — keep the tag as legacy history and bump version metadata past
  `1.7.0` so the next release cannot collide.

**Why a human:** deleting or re-pointing a published tag mutates shared repo
refs and is hard to reverse, and choosing the next version line is a
release-policy decision — neither is delegable to an agent.

**Acceptance criteria:**
- [ ] The maintainer's decision (A or B) is recorded in this ticket
- [ ] `package.json`, `src-tauri/tauri.conf.json`, and `CHANGELOG.md` are
      mutually consistent with the decision
- [ ] The next planned release number is verified free of tag collisions

**Out of scope:**
- Publishing any release or changing the release workflow
- Rewriting legacy changelog history
