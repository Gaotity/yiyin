# Repository Agent Guidelines

## Scope

These rules apply to the entire `yiyin` repository.

## Working Agreement

- Write documentation, code identifiers, and Git commits in English.
- Use Node.js 24 and the exact pnpm version declared in `package.json`.
- Keep renderer code browser-only. Do not import Node.js or Electron APIs from `web/` or `common/`.
- Treat `common/platform/bridge.ts` as the public cross-platform contract. Electron IPC details belong under `electron/`.
- Validate all IPC and file-boundary inputs at runtime. Return stable typed errors without stack traces or local paths.
- Do not add release publishing, signing credentials, automatic updates, or unsigned public artifacts.
- Keep changes surgical and preserve existing desktop behavior unless a security invariant requires a migration.

## Validation

Run `pnpm ci` before delivery. Run `pnpm package` only for an explicit packaging check; generated installers are local test artifacts and must not be published.
