# 01 — Capture and Freeze Legacy Compatibility Fixtures

**What to build:** Before the old renderer is touched, capture its exact behavior as immutable reference material: a test-only fixture mode in the legacy Electron app (enabled via environment variable, hard-disabled in packaged builds) drives the old Sharp/Canvas renderer through the existing `window.api` boundary across 14 precise scenarios (portrait/landscape/webp defaults, EXIF orientation 6, explicit 3:2 ratio, portrait-to-landscape, solid-white-no-shadow, blurred-shadow-radius, both built-in focal templates, light/dark logo variants, forced custom text, bundled custom font). The captured reference JPEGs, a manifest with exact geometry/density/perceptual thresholds (SSIM ≥ 0.97, changed-pixel ratio ≤ 8%), and normalized EXIF JSON become the sole benchmark for all later golden tests — never regenerated from Rust.

**Blocked by:** None — can start immediately.

**Status:** completed

- [x] All 14 scenarios captured; every reference JPEG opens with complete dimensions and density recorded
- [x] Two consecutive capture runs produce identical manifest and geometry data (SHA-256 compared; JPEG bytes intentionally not asserted)
- [x] Fixture mode refuses to run in packaged builds
- [x] Manifest, reference outputs, and normalized EXIF JSON committed as frozen fixtures
