# 06 — Application: Task Orchestration

**What to build:** The task-facing use cases and the port signatures the infrastructure queue and renderer will implement: `StartTasks`, `PreviewTask`, `CancelTask`, `ClearTasks`, with extended `TaskQueue`/`TaskEventSink`/`ImageRenderer` port contracts. Semantics: starting a task freezes a full copy of the render request (later config edits don't affect in-flight work); quick-output mode enqueues on registration while explicit mode waits; preview B cancels preview A through a dedicated replace-latest slot that never publishes an output; clear removes only registered/queued snapshots; conflict-safe output names are reserved before enqueue; completion results carry no paths.

**Blocked by:** 05 — Application: Ports, Errors, and Configuration/Resource Use Cases

**Status:** completed

- [x] `cargo test -p yiyin-application --locked` passes: frozen-request isolation (config edits after start have no effect), queue semantics, preview supersession, clear semantics
- [x] No filesystem path appears in any public snapshot or result
