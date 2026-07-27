# 09 — Infrastructure: Rust Renderer and Golden Comparisons

**What to build:** The `ImageRenderer` adapter that replaces Sharp/Canvas: it consumes a frozen `RenderRequest`, resolved resource records, and the domain `RenderPlan`, and produces an atomically published `RenderResult` (dimensions, density, resource kind). Backgrounds are solid or stretched-blur using only the `image` crate (no `imageproc`), preserving brightness-overlay thresholds; text and logos are shaped and rasterized with `cosmic-text` against bundled + registered fonts with field/template font merging, case conversion, and baseline/center image-slot alignment; compositing draws the shadow alpha mask directly (intermediate plane ≤ 10240), applies rounded-corner masking, and stacks background/main/text in the established order using the plan's integer coordinates. JPEG output encodes at source density and chosen quality via same-directory temp file → sync → atomic rename; cancellation or error removes temp/cache files and never touches existing outputs; previews use quality 70 and never publish files. Golden tests decode both sides with `image`, assert exact dimensions, and compute luma SSIM and changed-pixel ratio against the frozen manifest thresholds — emitting human-inspectable diff artifacts on failure.

**Blocked by:** 01 — Capture and Freeze Legacy Compatibility Fixtures; 06 — Application: Task Orchestration

**Status:** completed

- [x] `cargo test -p yiyin-infrastructure --test rendering --locked` passes
- [x] `cargo test -p yiyin-infrastructure --test golden --locked` passes: exact geometry/density plus perceptual thresholds (SSIM ≥ 0.97, changed pixels ≤ 8%) across all 14 frozen scenarios
- [x] Preview renders at quality 70 without publishing an output file; successful runs leave no diff artifacts; failures write actual/expected/diff images under the golden-diffs directory
