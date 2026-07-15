# Contributing

## Local setup

1. Install Node.js 24.
2. Enable Corepack.
3. Run `pnpm install --frozen-lockfile`.
4. Run `pnpm electron:install` to install the pinned local Electron runtime.
5. Run `pnpm dev` for the desktop development server.

The package manager version is fixed in `package.json`. Do not use npm or Yarn to update dependencies.

## Change requirements

- Keep pull requests focused and use Conventional Commits.
- Add or update tests for behavior and security-boundary changes.
- Run `pnpm run ci` before requesting review.
- Run `pnpm test:integration` for Electron boundary changes.
- Do not commit secrets, signing material, local paths, generated packages, or runtime data.
- Do not weaken the BrowserWindow, protocol, navigation, IPC, or resource validation policies.

## Release status

Public distribution is quarantined. Local and CI package artifacts are for testing only until signing, notarization, brand assets, and asset licensing are approved.
