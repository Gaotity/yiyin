# 01 — Land ADR 0002: Output Directory Writable Only Through a Dedicated Use Case

**What to build:** Make the output directory a domain value object and close
the general-config-update backdoor, per `docs/adr/0002-output-directory-dedicated-write-path.md`.
`choose_output_directory` currently mutates `Config.output` (a bare `String`)
and pushes the whole `Config` through the general `UpdateConfig` use case —
exactly the convention-based boundary the ADR retires. The DTO side already
holds (`PublicConfigDto` carries no output field; the UI never displays the
path), so this ticket is Rust-side only: value object, port extension, one
dedicated use case, and rewiring the two write sites.

**Blocked by:** None

**Status:** completed

## Agent Brief

**Category:** architecture

**Current behavior:**
- `Config.output` is a bare `String` (`crates/yiyin-domain/src/config.rs`),
  validated only as non-empty in `use_cases/config.rs`.
- `choose_output_directory` (`src-tauri/src/commands/native.rs`) calls
  `NativeOutputDirectory::set_root` (create dir + swap root + silently clear
  reservations), then mutates `config.output` and persists via the general
  `UpdateConfig` use case. If persistence fails after `set_root` succeeded,
  the runtime root and the persisted config diverge until restart.
- Relative-to-home resolution is duplicated between `resolve_output_root`
  (`src-tauri/src/state.rs`, bootstrap) and `reset_output_root`
  (`src-tauri/src/commands/native.rs`, reset).
- Root mutation lives only on the concrete `NativeOutputDirectory`; the
  `OutputDirectoryGateway` port has no root-changing capability.

**Desired behavior:**
- `Config.output` is an `OutputDirectory` value object wrapping the
  *configured* value (relative paths allowed; default stays
  `Pictures/watermark`). Minimal invariant: reject strings that are empty
  after trim. Path usability is verified by the filesystem probe, not the VO.
  The persisted JSON format is unchanged (plain string).
- `OutputDirectoryGateway` gains root mutation: `ensure_root` (probe:
  `create_dir_all`) and `change_root` (swap the shared root; documented to
  drop all reservations — a new root is a new naming namespace). Both take
  the `OutputDirectory`; the native adapter resolves relative paths against
  the home directory captured at construction.
- A dedicated `SetOutputDirectory` use case orchestrates the write in three
  steps — probe → persist → swap — so every failure window is benign: probe
  failure changes nothing; persist failure leaves at most a created
  directory; swap can only fail on a poisoned lock.
- `choose_output_directory` goes through `SetOutputDirectory`; it no longer
  touches `Config.output` or the general update path. `reset_config`
  composes `ResetConfig` + `SetOutputDirectory` (default VO) — all root
  changes flow through the single orchestration path.
- `resolve_output_root` and `reset_output_root` are deleted; resolution is
  single-sourced inside `NativeOutputDirectory`. The renderer's
  `shared_root()` channel (resolved absolute path behind the lock) is
  unchanged.

**Key interfaces:**
- `yiyin_domain::OutputDirectory` — new VO (`TryFrom`, private field),
  `Config.output: OutputDirectory`
- `yiyin_application::OutputDirectoryGateway` — `+ ensure_root(&OutputDirectory)`,
  `+ change_root(&OutputDirectory)` (reservation-drop contract in docs)
- `yiyin_application::SetOutputDirectory` —
  `new(ConfigRepository, OutputDirectoryGateway)`,
  `execute(OutputDirectory) -> Result<Config, ApplicationError>`
- `NativeOutputDirectory` — captures home dir at construction; resolves the
  VO internally for bootstrap, probe, and swap
- Commands: `choose_output_directory` (dialog → string → VO → use case,
  non-UTF-8 still `FILE_INVALID`), `reset_config` (two-use-case composition)

**Acceptance criteria:**
- [x] `Config.output` is `OutputDirectory`; construction rejects empty and
      whitespace-only strings (domain unit tests
      `output_directory_rejects_blank_values`,
      `output_directory_keeps_relative_and_absolute_values`)
- [x] Persisted `config.json` keeps output as a plain string; existing
      config-persistence and legacy-import suites stay green after the type
      change (`StoredConfigV1` maps via `as_str`/`TryFrom`; legacy salvage
      converts at the boundary, whitespace-only output still falls back to
      the default with a warning)
- [x] `SetOutputDirectory` calls probe → persist → swap in order; a failing
      probe never reaches persist, a failing persist never reaches swap
      (`set_output_directory_probes_persists_then_swaps`,
      `a_failing_probe_never_persists_or_swaps`,
      `a_failing_persist_never_swaps_the_root` with call-order-logging fakes)
- [x] `change_root` creates the directory, swaps the root, and drops
      reservations (`change_root_creates_swaps_and_drops_reservations`: a
      name reserved pre-swap is absent from `existing_names` post-swap);
      relative paths resolve against the captured home
      (`relative_roots_resolve_against_the_captured_home`), absolute paths
      pass through (`absolute_roots_pass_through`)
- [x] `choose_output_directory` and `reset_config` write output only via
      `SetOutputDirectory`; the bootstrap/reset resolvers are gone and
      resolution lives solely inside `NativeOutputDirectory` (grep finds a
      single `resolve_output_root` — the adapter's private helper)
- [x] Full Rust suite (`cargo fmt`/`clippy -D warnings`/`cargo test
      --workspace`) + frontend CI (`pnpm ci`) pass; no TypeScript changes

**Out of scope:**
- The pre-existing in-flight render race when the root changes mid-render
- The ADR-0001 tail (`format_shutter`/`normalize_make` still in
  infrastructure, hardcoded preview quality `70`) — collected by whichever
  ADR ticket lands it
- ADR-0003 (numeric constraints via bootstrap) and any DTO/frontend change
