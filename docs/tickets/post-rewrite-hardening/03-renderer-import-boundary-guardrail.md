# 03 — Automated Renderer Import-Boundary Guardrail

**What to build:** The renderer's privilege boundary — `src/platform/` is the
only layer allowed to talk to the host — currently holds by review discipline
alone. Nothing automated stops a stray `@tauri-apps/api` or `node:*` import
elsewhere in `src/`, the same "guardrails missing" weakness the legacy audit
flagged for the Electron tree. Add an automated check in the repo's existing
policy-test style. Restated from the legacy audit
(`legacy-electron-audit/07-renderer-isolation-guardrails.md`, triaged
2026-07-29).

**Blocked by:** None

**Status:** completed

## Agent Brief

**Category:** enhancement

**Current behavior:**
`src/platform/client.ts` is the sole importer of `@tauri-apps/api` in `src/`,
enforced only by convention. Biome runs its recommended preset with no
restricted-imports rule, and the final-tree/policy tests scan for legacy
runtimes (electron, svelte, sharp) but not for host-API imports inside `src/`.

**Desired behavior:**
A policy test fails when any module under `src/` outside the platform layer
imports `@tauri-apps/api` or a `node:*` builtin. The check runs with the
existing policy suite (`tests/security/policy.test.ts` conventions). The
current tree passes.

**Key interfaces:**
- `tests/security/policy.test.ts` — gains an import-boundary case in the
  existing file-scanning style
- `src/platform/` — stays the single door for host APIs; no restructuring

**Acceptance criteria:**
- [x] The policy test scans `src/` and fails on `@tauri-apps/api` or `node:*`
      imports outside `src/platform/` — `findHostApiImports` walks the tree
      (platform pruned) matching static, side-effect, and dynamic imports
- [x] The test is proven red against a planted violation (fixture or temporary
      file), then green on the real tree — a temp-dir fixture with
      `@tauri-apps/api/core` and `node:fs` violations is flagged (platform
      import allowed), and the real-tree assertion returns empty
- [x] Full frontend suite (`pnpm test`) and policy suite stay green — 56
      tests pass across the frontend, policy, package, and final-tree suites
      (14 in the policy file)

**Out of scope:**
- Runtime isolation (capabilities, CSP — already locked by policy tests)
- A Biome `noRestrictedImports` mirror (optional nicety, not required)
- Changing the platform client's API surface
