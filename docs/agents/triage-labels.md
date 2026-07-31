# Triage Labels

The skills speak in terms of five canonical triage roles. This file maps those roles to the actual constructs used in this repo's issue tracker (Linear `Yiyin` project — see `docs/agents/issue-tracker.md`).

| Label in mattpocock/skills | Construct in our tracker | Meaning                                  |
| -------------------------- | ------------------------ | ---------------------------------------- |
| `needs-triage`             | `needs-triage` label     | Maintainer needs to evaluate this issue  |
| `needs-info`               | `needs-info` label       | Waiting on reporter for more information |
| `ready-for-agent`          | `ready-for-agent` label  | Fully specified, ready for an AFK agent  |
| `ready-for-human`          | `ready-for-human` label  | Requires human implementation            |
| `wontfix`                  | `Canceled` state         | Will not be actioned (terminal)          |

When a skill mentions a role (e.g. "apply the AFK-ready triage label"), use the corresponding label string from this table. `wontfix` has no label: move the issue to the `Canceled` workflow state instead, with the evidence recorded in a bilingual comment.

Edit the right-hand column to match whatever vocabulary you actually use.
