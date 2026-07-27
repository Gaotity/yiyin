# 08 — Infrastructure: Resource Registry and EXIF Adapter

**What to build:** The adapters behind resources and metadata: `FileResourceRepository`, `ExifMetadataReader`, `UuidGenerator`. The registry issues UUID v4 opaque IDs with records (kind, canonical path, MIME, allowed root) for the Tauri protocol to consume; owned resources are copied in atomically; containment is re-verified on every resolve so symlink escapes fail with `FORBIDDEN`; JPEG/PNG/WebP are validated by magic bytes with extension mismatches rejected; source file references never cross the React boundary. The EXIF adapter reads only the needed tags and reproduces the full normalization contract — vendor `CORPORATION` stripping and Title Case, Nikon `Z 7_2` → `ℤ 7 Ⅱ`, Sony `ILCE-` → lowercase `α`, fractional shutter → `1/N`, focal rounding, datetime/white-balance/program/metering strings, orientation handling, all-empty → none — with malformed tags blanking fields instead of panicking.

**Blocked by:** 05 — Application: Ports, Errors, and Configuration/Resource Use Cases

**Status:** completed

- [x] `cargo test -p yiyin-infrastructure --test resources --locked` passes: magic-byte/extension validation, symlink escape → `FORBIDDEN`, source deletion, unknown IDs
- [x] `cargo test -p yiyin-infrastructure --test metadata --locked` passes: all normalization fixtures (generic, Nikon, Sony, shutter fractions, all-empty → none)
