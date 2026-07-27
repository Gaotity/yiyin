# 04 — Domain: Render Plans, Output Naming, and Task State

**What to build:** The renderer-agnostic planning half of the domain: `RenderRequest`, `RenderPlan`, `RenderStage`, `TaskState`/`TaskStatus`, and `OutputNameResolver`. The exact geometry pipeline is reproduced as named steps with explicit rounding helpers (explicit ratio → portrait-to-landscape swap → main-image width expansion → minimum/shadow top margin → three-quarter text spacing → 2.7% bottom text offset → centering → radius/shadow percentages → 10240 shadow-plane cap). Output naming resolves conflicts to `<stem>-<N>.jpg` with gap skipping and non-numeric-suffix handling. The task state machine only allows legal transitions, with progress milestones `1, 10, 20, 30, 50, 60, 70, 90, 100` encoded on the stages. Geometry test expectations are copied as Rust constants from the frozen manifest (no JSON parsing in domain).

**Blocked by:** 01 — Capture and Freeze Legacy Compatibility Fixtures; 03 — Domain: Configuration, Templates, Metadata, and Resources

**Status:** completed

- [x] `cargo test -p yiyin-domain --locked` passes: exact-geometry fixtures (canvas, main image, text lines, radius, shadow, offsets, integer rounding) all green
- [x] Output-naming conflict tests pass (numeric suffix increment, gap skipping, non-numeric suffixes)
- [x] Illegal task-state transitions are rejected by construction and by tests
