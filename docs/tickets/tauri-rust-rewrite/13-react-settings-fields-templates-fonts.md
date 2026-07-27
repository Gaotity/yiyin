# 13 — React: Rendering Settings, Fields, Templates, and Fonts

**What to build:** The settings surface of the app, rebuilt with full behavioral parity: rendering-settings controls (controlled inputs + switches) with numeric clamping/rounding, ratio swapping, landscape-disable logic, instant save, solid-color, preview and quick-output toggles — every legal change calls `updateConfig` and replaces the local draft with the returned canonical config, while continuous slider drags debounce and flush on blur/unmount. Field and template dialogs use React Hook Form transactions (no resolvers): save/cancel, visibility, forced values, text/image types, light/dark overlay registration, font overrides, stable custom keys, system-template deletion protection, custom deletion, reordering, placeholder insertion, baseline/center alignment — with Rust as the final validator. Font management covers bundled fonts, native picking, duplicate names, missing sources, removal, selected-font fallback, and refresh-on-change; React only ever receives font IDs and resource URLs, never font paths.

**Blocked by:** 12 — React: Typed Client, App Shell, and Chrome Parity

**Status:** completed

- [x] `pnpm vitest run` settings tests pass with exact copy: numeric clamping, transaction cancel never calls `updateConfig`, all field/template/font workflows
- [x] `pnpm typecheck` passes
- [x] No private path appears in any rendered output
