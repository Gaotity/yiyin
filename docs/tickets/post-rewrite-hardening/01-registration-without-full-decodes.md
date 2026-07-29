# 01 — Register Images Without Full-Pixel Decodes

**What to build:** Image registration currently fully decodes every image —
once for dimensions/validation and again for the supportedness check — even
though only header information is needed, and the image-picker (`choose_images`)
batch path runs that work inline on the async command context while the
drag-drop path already offloads to blocking threads. Registration should probe
dimensions and supportedness from headers only, batch registration should never
run pixel decodes on the async runtime, and the dead original-dimensions config
surface should stop crossing to the UI. Restated from the legacy audit
(`legacy-electron-audit/01-image-pipeline.md`, triaged 2026-07-29).

**Blocked by:** None

**Status:** ready-for-agent

## Agent Brief

**Category:** bug

**Current behavior:**
Registering an image decodes the full pixel data just to obtain dimensions and
validate the file, and the supportedness probe performs a second full decode.
In the image-picker flow this batch work executes on the async command context;
the drag-drop flow already uses blocking threads correctly. Separately, the
legacy `origin_wh_output` config key has no behavioral effect yet still crosses
to the UI as `originalDimensions` on the config DTO with zero consumers.

**Desired behavior:**
Dimension and supportedness probing reads image headers only — no full-pixel
decode anywhere in registration. Batch registration through the image picker
runs its heavy work on blocking threads, mirroring the drag-drop path. The
original-dimensions value no longer appears on the config DTO; legacy config
import keeps accepting `origin_wh_output` and ignores it.

**Key interfaces:**
- `ResourceRegistry` registration path — dimension/validation probing must be
  header-only
- `is_supported_image` — must not perform its own full decode
- The `choose_images` command path — heavy registration work moves to blocking
  execution like the drag-drop path
- Config DTO (`originalDimensions`) and domain config (`origin_wh_output`) —
  DTO field removed; legacy import tolerates the key as ignored

**Acceptance criteria:**
- [ ] A fixture with valid headers but corrupted pixel data registers
      successfully (proves registration never fully decodes)
- [ ] Supportedness checking performs no full decode of its own
- [ ] Batch registration via the image picker executes off the async runtime,
      consistent with the drag-drop path
- [ ] `originalDimensions` is absent from the generated config DTO; legacy
      import and schema-compat tests pass
- [ ] Golden-rendering and legacy-parity suites stay green

**Out of scope:**
- The render pipeline's single decode per render (accepted as designed)
- EXIF normalization rules and metadata fields
