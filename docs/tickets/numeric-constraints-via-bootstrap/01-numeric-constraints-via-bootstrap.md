# 01 — Land ADR 0003: Numeric Constraints Ship to the Frontend via Bootstrap

**What to build:** Make the domain the single source of truth for numeric
option constraints (min/max/decimals), per
`docs/adr/0003-numeric-constraints-via-bootstrap.md`. Today the React settings
UI clamps against a hand-maintained `NUMBER_SETTINGS` table — five
hand-copied clamp sites in total — while the domain's `bounded_*!` value
objects reject out-of-range values, so the domain's rejection path is dead
code for UI traffic and the two sides drift silently (ts-rs syncs structure,
not values). The constraints are generated into the bootstrap payload and the
frontend consumes them for clamping; clamping itself stays a frontend UX
responsibility. Also correct the stale blur help copy (says default 15, the
real default is 100 — proof the two sources diverged).

**Blocked by:** None

**Status:** ready-for-agent

## Agent Brief

**Category:** architecture

**Current behavior:**
- `NUMBER_SETTINGS` (`src/features/settings/RenderingSettings.tsx`)
  hand-maintains `minimum`/`maximum`/`decimals` for the 7 numeric options —
  plus four more hand-copied clamp sites: the blur slider's `min`/`max`/`step`
  attributes and `scheduleBlur`'s inline `(raw, 0, 100, 0)`, and the ratio
  width/height inputs' inline `(0, MAX_SAFE_INTEGER, 3)`.
- The domain already owns the same bounds in `bounded_integer!` /
  `bounded_decimal!` macro invocations (`crates/yiyin-domain/src/config.rs`),
  but nothing ships them to the frontend.
- Divergence evidence: the blur help text claims "默认值: 15" while the
  domain default is `BackgroundBlur(100)` (and `fake.ts` agrees with the
  domain).
- `BackgroundRatio` is not a bounded value object — the domain enforces only
  non-negative finite components; the UI's 3-decimal clamp is pure UX.

**Desired behavior:**
- The macros emit `pub const CONSTRAINT: NumericConstraint` from the same
  arguments that drive `TryFrom`, so the shipped values structurally cannot
  diverge from the validation bounds.
- A `NumericOption` registry enum (7 variants) maps each option to its VO's
  `CONSTRAINT` and enumerates `ALL`; the application `Bootstrap` use case
  collects them into the snapshot with zero hardcoded values.
- The adapter serializes the map into `BootstrapDto.constraints` keyed by the
  camelCase DTO field names (the adapter owns that naming); ts-rs regenerates
  the TypeScript types.
- The frontend threads constraints snapshot → reducer state →
  `RenderingSettings` prop. `NUMBER_SETTINGS` keeps only `key`/`label`/`help`;
  every clamp reads from the prop; the blur slider derives `min`/`max`/`step`
  (`step = 10^-decimals`) from the same prop.
- Ratio inputs are unchanged: ratio is not a bounded value object and its
  clamp is UX-only.
- The blur help copy is corrected to "默认值: 100".

**Key interfaces:**
- `yiyin_domain::NumericConstraint { minimum: f64, maximum: f64, decimals: u8 }`
- `bounded_integer!` / `bounded_decimal!` — generate `CONSTRAINT` (decimals 0
  for integers, derived from scale for decimals: 10→1, 100→2)
- `yiyin_domain::NumericOption` — 7 variants, `ALL`, `constraint()`
- `BootstrapSnapshot::constraints` — all 7 entries
- `NumericConstraintDto`, `BootstrapDto::constraints` — camelCase keys,
  ts-rs-exported
- `RenderingSettings` — new `constraints` prop; `NUMBER_SETTINGS` reduced to
  `key`/`label`/`help`

**Acceptance criteria:**
- [ ] Each macro-generated `CONSTRAINT` equals the VO's `TryFrom` bounds for
      all 7 value objects (domain unit tests pin the exact values, e.g.
      `MainImageWidth` → `{1, 100, 0}`, `Radius` → `{0, 50, 1}`,
      `TextMargin` → `{0, 10000, 2}`)
- [ ] `NumericOption::ALL` holds exactly the 7 variants and `constraint()`
      returns the corresponding VO constant (name mapping only, no value
      copies)
- [ ] The bootstrap snapshot and `BootstrapDto` carry all 7 constraints with
      camelCase keys (application bootstrap test + dto serialization test)
- [ ] `RenderingSettings` clamps solely from the prop: a test renders with
      constraint values that deliberately differ from the domain truth and
      asserts clamping follows the prop; the blur slider's `min`/`max`/`step`
      derive from the same prop
- [ ] `NUMBER_SETTINGS` no longer contains `minimum`/`maximum`/`decimals` and
      no bounds literals survive in the component's clamp sites (grep)
- [ ] The blur help copy reads "默认值: 100"
- [ ] `defaultBootstrap()` in `fake.ts` ships the 7 constraints as a fixture
      with a comment pointing at the pinning Rust test
- [ ] Full Rust suite (`fmt`/`clippy -D warnings`/`cargo test --workspace`)
      and `pnpm ci` stay green

**Out of scope:**
- Ratio constraints (UX-only clamp; `BackgroundRatio` is not a bounded VO)
- Shipping *defaults* via bootstrap (current values already travel inside
  `PublicConfigDto.options`; the input displays them)
- The preview quality hardcode (`70_u8`) — ADR-0001 tail
- Changing any actual bound or default value
