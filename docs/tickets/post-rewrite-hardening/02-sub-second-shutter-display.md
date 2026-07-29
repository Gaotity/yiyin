# 02 — Display Sub-Second Shutter Speeds as Decimal Seconds

**What to build:** Shutter formatting reciprocal-rounds every exposure below
one second, so real exposures in the ~0.67–1s range display as `1/1` — a
misleading value users never set. Sub-second exposures that would round to
`1/1` should display as decimal seconds (e.g. `0.7`) instead. Restated from
the legacy audit (`legacy-electron-audit/02-exif-formatting.md`, triaged
2026-07-29).

**Blocked by:** None

**Status:** ready-for-agent

## Agent Brief

**Category:** bug

**Current behavior:**
The shutter normalizer computes `1/round(1/seconds)` for any exposure below
one second. Exposures between ~0.67s and 1s therefore render as `1/1`. The
interval is untested, and frozen legacy-capture fixtures may pin the `1/1`
output for such inputs.

**Desired behavior:**
Exposures whose reciprocal would round to `1/1` display as decimal seconds
(`0.7`, `0.8`, …) with trailing zeros trimmed, matching the existing
normalized-display-value conventions. Exposures of one second or more keep
their current display (`2`, `1.5`-style behavior unchanged), and fast
reciprocals (`1/125`) are untouched. Frozen fixtures that pin the old `1/1`
output for the affected interval are updated deliberately with the new
expected values.

**Key interfaces:**
- The shutter formatting path in the metadata adapter (`format_shutter`
  concept) — gains a decimal-seconds branch for the affected interval
- Frozen legacy-capture fixtures — updated only where they pin the `1/1`
  output for sub-second exposures in the affected interval

**Acceptance criteria:**
- [ ] Exposures in the (~0.67, 1.0)s interval display as trimmed decimal
      seconds, not `1/1`
- [ ] One second and above, and fast reciprocals like `1/125`, are unchanged
- [ ] Unit tests pin both boundaries of the affected interval
- [ ] Frozen fixtures referencing the old `1/1` output are updated with a
      deliberate expectation change; the rest of the parity suite stays green

**Out of scope:**
- The reciprocal convention for exposures below the affected interval
- Any other metadata field normalization
