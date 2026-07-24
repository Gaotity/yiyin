# 10 — Infrastructure: Concurrency-Two Task Queue and Cancellation

**What to build:** The `TaskQueue` adapter that runs exports the way users expect: a Tokio semaphore caps concurrency at exactly 2 (preserving current behavior), CPU-heavy rendering runs on `spawn_blocking` so the async runtime never stalls, and each task gets its own cancellation token checked at stage boundaries. Typed domain statuses flow through `TaskEventSink`; join errors map to `INTERNAL` without panicking. Preview cancellation is best-effort at stage boundaries so a superseded preview can never overwrite the current selection. Shutdown rejects new tasks, cancels active ones, and waits a bounded grace period (5s) for cleanup.

**Blocked by:** 06 — Application: Task Orchestration; 09 — Infrastructure: Rust Renderer and Golden Comparisons

**Status:** completed

- [x] `cargo test -p yiyin-infrastructure --test tasks --locked` passes: three released tasks observed at exactly 2 max concurrency
- [x] Cancellation is a terminal state; a superseded preview completing late cannot win
- [x] Progress is monotonic; shutdown rejects new tasks and cleans up within the grace bound
