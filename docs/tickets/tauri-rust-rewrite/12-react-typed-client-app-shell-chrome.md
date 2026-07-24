# 12 — React: Typed Client, App Shell, and Chrome Parity

**What to build:** The React side of the boundary and the app's frame: a `PlatformClient` interface with a real adapter that centralizes invoke/listen and error parsing (feature code is forbidden from importing `@tauri-apps/api` directly) and a fake adapter for Vitest and browser Playwright. `useAppController` drives state with `useReducer` — Rust snapshots are authoritative, task events reconcile by ID, selection survives only while the task exists, unknown event IDs trigger re-bootstrap, subscriptions clean up on unmount — with no global state library. The title bar and footer are rebuilt to match the existing DOM dimensions and CSS tokens: external links through the enum, the QQ-copy behavior, help content, donation image, font selector placement, reset, minimize, close.

**Blocked by:** 11 — Tauri Boundary: DTOs, Errors, Protocol, Native Operations, and Composition

**Status:** completed

- [x] `pnpm lint` and `pnpm typecheck` pass; no direct Tauri imports outside the platform client
- [x] `pnpm vitest run` shell tests pass: bootstrap, migration-warning vs fatal-error display, event-before-bootstrap race, unsubscribe on unmount
- [x] Title bar and footer match the legacy layout and copy exactly
