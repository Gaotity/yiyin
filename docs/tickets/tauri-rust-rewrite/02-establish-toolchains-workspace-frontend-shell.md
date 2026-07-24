# 02 — Establish Toolchains, Workspace Boundaries, and Frontend Shell

**What to build:** The compilable skeleton of the new runtime: Rust 1.97.0 pinned via toolchain file, pnpm 11.13.0 via Corepack, a Cargo workspace (resolver 2, edition 2024, `unsafe_code` forbidden, GPL-3.0-only) with four crate stubs (`yiyin-domain`, `yiyin-application`, `yiyin-infrastructure`, `yiyin-desktop` for the Tauri adapter), all approved dependency pins applied exactly (with registry re-verification of availability, 24-hour age, and compatibility — including the version selection for cargo-deny and tauri-driver used later by CI), and a Svelte-free React entry point with strict TypeScript, Biome, Vitest, and the full script surface. The dependency-direction boundary test goes from red to green.

**Blocked by:** None — can start immediately.

**Status:** completed

- [x] All crate stubs compile with `cargo check --workspace --locked`
- [x] Dependency-boundary test passes (domain has zero dependencies, application depends only on domain, only the Tauri adapter consumes tauri)
- [x] `pnpm typecheck` and `pnpm build` produce a single Svelte-free React bundle
- [x] Every pinned version exists in its registry and is older than 24 hours
