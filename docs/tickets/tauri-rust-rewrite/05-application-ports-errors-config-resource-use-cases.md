# 05 — Application: Ports, Errors, and Configuration/Resource Use Cases

**What to build:** The application layer's contracts and its non-task use cases: object-safe synchronous ports (`ConfigRepository`, `ResourceRepository`, `MetadataReader`, `OutputDirectoryGateway`, `Clock`, `IdGenerator`, `TaskQueue`), safe snapshot models (`RenderResult`, `ResourceSnapshot`, `BootstrapSnapshot` — no paths), the closed error enum with nine stable codes and `safe_message()` sanitization, and the use cases `Bootstrap` (legacy import before load, warnings separated from fatal errors), `UpdateConfig`, `ResetConfig`, `RegisterImages`, `RegisterFont`, `RemoveFont`, `RegisterOverlay`, `ReadTaskExif`. All tested against hand-written fake ports — no Tauri, no real filesystem.

**Blocked by:** 03 — Domain: Configuration, Templates, Metadata, and Resources; 04 — Domain: Render Plans, Output Naming, and Task State

**Status:** completed

- [x] `cargo test -p yiyin-application --locked` passes: bootstrap snapshot, config validation, reset, image/font/overlay registration, duplicate font, missing file, safe EXIF exposure
- [x] Invalid config updates never reach the repository (write count stays zero)
- [x] Dependency-boundary test still passes: application depends only on domain
