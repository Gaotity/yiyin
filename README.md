# Yiyin

Yiyin (壹印) is a desktop photo-framing and watermark composition tool. This repository is maintained at [Gaotity/yiyin](https://github.com/Gaotity/yiyin).

## Security model

The Electron renderer is sandboxed and has no Node.js access. Local files remain in the main process and are represented in the renderer by opaque resource identifiers and restricted `yiyin://` URLs. File ingestion validates signatures, size, batch count, and decoded image dimensions before processing.

The application performs image processing locally with Sharp and reads metadata in-process with ExifReader. It does not bundle or execute ExifTool or FFmpeg binaries, and it does not contain an automatic update or release-publishing path.

See [the security policy](SECURITY.md) for private vulnerability reporting and [the hardening plan](docs/security-hardening-plan.md) for the enforced boundaries and residual risks.

## Development

Requirements:

- Node.js 24
- pnpm 11.13.0 through Corepack

Install and validate:

```sh
corepack pnpm install --frozen-lockfile
corepack pnpm run ci
```

Run the desktop application in development:

```sh
corepack pnpm dev
```

Additional commands are documented in [CONTRIBUTING.md](CONTRIBUTING.md).

## Packaging quarantine

Local and CI package outputs are unsigned test artifacts only. Public distribution is intentionally blocked until application branding and asset provenance are approved, Apple and Windows signing are configured, notarization is available, and a separately reviewed release workflow is authorized.

The repository does not publish unsigned installers or create GitHub Releases from its packaging workflows.

## License

Yiyin is licensed under `GPL-3.0-only`. See [LICENSE](LICENSE).
