# Numeric constraints ship to the frontend via bootstrap

Numeric option constraints (min/max/decimals) were defined twice with divergent policies: the React settings UI clamps against a hand-maintained `NUMBER_SETTINGS` table, while the domain's `bounded_*!` value objects reject out-of-range values — so the domain's rejection path was dead code for UI traffic, and changing a bound on one side silently broke the other (ts-rs syncs structure, not constraints). We decided the domain is the single source of truth for constraints: they are generated into the bootstrap payload (via the ts-rs boundary), and the frontend consumes them for its clamping behavior instead of maintaining its own table. Clamping stays a frontend UX responsibility; the constraint values never diverge.

## Considered Options

- Frontend sends raw values and surfaces domain rejection errors — single source, but typing a wrong character becomes an error round-trip; rejected on UX grounds.
- Keep dual definitions with a contract test comparing both tables — smallest change, but still two hand-synced sources; rejected.
