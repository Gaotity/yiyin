# Issue tracker: Linear `Yiyin` project

The canonical issue tracker for this repo is the Linear
[`Yiyin`](https://linear.app/wg-studio/project/yiyin-3ba958517518/overview)
project (wg-studio workspace, team `C-level`). Pull requests remain on GitHub —
use the `gh` CLI for PR operations. Local files under `docs/tickets/` are the
**frozen pre-migration record** (migrated 2026-07-30; see
`docs/agents/linear-migration.md`); new work is tracked in Linear only.

## Access

Use the Linear GraphQL API at `https://api.linear.app/graphql` with the
`LINEAR_PERSONAL_PROJECT_FULL_ACCESS_API_KEY` environment variable as the
`Authorization` header. The key lives locally; never commit it, never print it.

Useful ids (not secrets):

- Team `C-level`: `bca72391-8cfe-4be9-9bcd-81a65184df34`
- Project `Yiyin`: `d48a0d5f-f9cc-491b-9b43-5d816dfff6eb`

## Conventions

- **One issue per ticket** in the Yiyin project; title `[<feature-slug>] <title>`.
- **Descriptions are bilingual**: complete English block, a `---` divider, the
  complete Simplified Chinese block, then one shared `## References` section at
  the bottom (external-collaboration rule).
- **States**: `Backlog` (new/unprioritized) → `Todo` → `In Progress` (claimed)
  → `In Review` (delivering PR open) → `Done` (PR merged). `Canceled` is the
  terminal wontfix state.
- **Labels**: triage roles per `docs/agents/triage-labels.md`;
  `cluster:<feature-slug>` groups a feature's tickets; `architecture-backlog`
  for architecture-review candidates; `migrated-from-repo` marks migrated
  history.
- **Blocking edges**: native Linear issue relations (`blocks` / `blocked by`)
  via `issueRelationCreate` — a ticket is unblocked when every blocking issue
  is `Done`.

## When a skill says "publish to the issue tracker"

Create the issue in the Yiyin project via `issueCreate` (team `C-level`,
project `Yiyin`), following the conventions above. Do **not** create local
files under `docs/tickets/` and do **not** call `gh issue create`.

## When a skill says "fetch the relevant ticket"

Query Linear: fetch a single issue by identifier (`{ issue(id: "C-42") { ... } }`)
or search by title (`{ issues(filter: { title: { contains: "..." } }) { nodes { ... } } }`).

## List open tickets

Project issues whose state is neither `Done` nor `Canceled`:

```graphql
{
  project(id: "yiyin-3ba958517518") {
    issues(filter: { state: { type: { nin: ["completed", "canceled"] } } }) {
      nodes { identifier title state { name } labels { nodes { name } } }
    }
  }
}
```

## Wayfinding operations

Used by `/wayfinder`. The **map** is a parent issue; child tickets are its
sub-issues.

- **Map**: one issue titled `[<feature-slug>] Map: <name>` holding the Notes /
  Decisions-so-far / Fog body in its description.
- **Child ticket**: an issue created with `parentId` set to the map issue.
- **Blocking**: `issueRelationCreate` with type `blocks`. A child is unblocked
  when every related blocker is `Done`.
- **Frontier query**: among the map's sub-issues, those not `Done`/`Canceled`
  whose blockers are all `Done`; first by sort order wins.
- **Claim**: move the child to `In Progress` (assign the operator).
- **Resolve**: record the answer in the child's description (or a bilingual
  comment), move it to `Done`, and append a context pointer to the map issue's
  Decisions-so-far section.

## Pull requests as a triage surface

**PRs as a request surface: no.** External collaboration happens on Linear
issues, not GitHub PRs.
