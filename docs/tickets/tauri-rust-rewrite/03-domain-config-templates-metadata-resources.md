# 03 — Domain: Configuration, Templates, Metadata, and Resources

**What to build:** The pure domain core the whole product validates against: `Config`, `RenderOptions`, `Template`, `TemplateField`, `FontSpec`, normalized `Metadata`, and identifier value objects (`ResourceId`, `TaskId`, `ResourceKind`) with validated newtypes (`TryFrom`, private fields — invalid states unrepresentable). Preserves v1.6 semantics exactly: default values (quality 100, radius 2.1, shadow 6, main-image width 90, `PingFang SC`), the 15 built-in EXIF fields, the 3 default templates, field override precedence, light/dark image variants, hidden/empty field collapsing, case conversion, system-template deletion protection, and empty-line omission. The domain receives only normalized values and depends on nothing outside itself.

**Blocked by:** 02 — Establish Toolchains, Workspace Boundaries, and Frontend Shell

**Status:** completed

- [x] `cargo test -p yiyin-domain --locked` passes: defaults, ranges, templates, metadata, and identifier tests all green
- [x] `cargo tree -p yiyin-domain` shows zero external dependencies
- [x] v1.6 default values and field/template semantics reproduced exactly
