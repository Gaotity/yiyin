# 07 — Infrastructure: Durable Configuration and Idempotent Legacy Import

**What to build:** The configuration persistence adapter and the one-time Electron migration. Versioned JSON storage (envelope with explicit mapping, existing field names preserved, missing built-ins appended, unknown future-schema fields never silently reinterpreted, private paths never serialized) with durable writes — temp file → flush+sync → `.bak` of the previous file → atomic rename → directory sync — and corrupt files preserved as `.invalid.bak`. The legacy import runs only when no committed Tauri config exists, probes the platform-specific legacy user-data locations in order, stages in a temp directory, validates, atomically publishes, and records a migration marker; it verifies legacy source hashes unchanged, survives interruption, is idempotent on rerun, and degrades single unsupported resources to warnings. Platform paths arrive from the Tauri layer; infrastructure never queries Tauri itself.

**Blocked by:** 05 — Application: Ports, Errors, and Configuration/Resource Use Cases

**Status:** completed

- [x] `cargo test -p yiyin-infrastructure --test config_persistence --locked` passes: failure injection at every write stage leaves the previous file loadable; atomic replace and directory sync verified
- [x] `cargo test -p yiyin-infrastructure --test legacy_import --locked` passes: dual-directory probe order, copy-once idempotency, unchanged source hashes, interruption-safe retry
