# 05 — Legacy Config Import Must Degrade Gracefully, Not Block Startup

**What to build:** When an upgrading user's legacy config contains any invalid
value — a single out-of-range option is enough — the whole legacy decode
fails and the error propagates through app state composition, aborting
startup. The legacy app opened with defaults in this situation; the rewrite
must not strand upgraders at a dead process. Import should salvage what it
can and never prevent startup because of legacy config content. Restated from
`legacy-electron-audit/03-config-migration.md` after codex review on PR #44
(2026-07-29).

**Blocked by:** None

**Status:** completed

## Agent Brief

**Category:** bug

**Current behavior:**
Legacy import deserializes the entire stored legacy config in one shot; a
single invalid field fails the domain conversion, and the error bubbles from
`decode_legacy_config` through `import_legacy_if_needed` into
`AppState::compose`, so the app exits during setup. The legacy source file is
preserved untouched (that part is handled), but the user gets a dead startup
instead of their remaining valid settings.

**Desired behavior:**
Legacy import degrades per configuration block: valid blocks import normally,
invalid blocks or fields fall back to their defaults, and every skipped value
produces a warning. Startup never fails because of legacy config content.
Only the legacy-import path degrades — corrupt *current-format* config keeps
the existing backup/recovery behavior.

**Key interfaces:**
- `decode_legacy_config` / `DecodedLegacyConfig` — per-block fallible decode
  instead of one atomic failure
- `import_if_needed` / `ImportOutcome` — collects warnings rather than
  propagating legacy decode errors
- `AppState::compose` — legacy import failure becomes logged warnings, never
  a setup abort

**Acceptance criteria:**
- [x] A legacy config with one out-of-range field imports the remaining valid
      configuration; the invalid field falls back to its default; startup
      succeeds — `decode_legacy_config` now salvages per option field (merge
      into defaults, skip + warning on failure):
      `one_invalid_option_falls_back_to_its_default_without_losing_valid_settings`
      keeps `iot`/`radius`, resets `quality` to 100
- [x] A wholly undecodable legacy config falls back to defaults; startup
      succeeds; the legacy source file is untouched —
      `an_unreadable_legacy_config_imports_as_defaults_with_a_warning`
- [x] Every skipped value produces a recorded warning visible in logs — decode
      warnings flow into `ImportOutcome::warnings` (named per skipped value:
      `Skipped invalid legacy option/template/field…`, plus block-shape
      mismatches), and `AppState::compose` now logs each one via `log::warn!`
      instead of discarding the outcome
- [x] Existing legacy-import and config-persistence suites stay green, with
      new tests covering both degradation cases — 7/7 legacy_import (4
      pre-existing) plus `a_broken_template_is_skipped_while_valid_templates_import`

**Out of scope:**
- Changing current-format config error handling (backup/recovery stays)
- Redesigning the legacy schema mapping
