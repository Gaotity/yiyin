# Security Policy

## Supported versions

Security fixes currently target the latest commit on `main`. No public binary release is supported while distribution remains quarantined.

## Reporting a vulnerability

Use GitHub Private Vulnerability Reporting for sensitive reports. Do not open a public issue containing exploit details, private paths, user content, credentials, or signing material.

Include the affected commit, platform, reproduction steps, impact, and any suggested mitigation. The maintainer will acknowledge a complete report as capacity permits and coordinate disclosure after a fix is available.

## Security boundaries

The renderer is untrusted. Electron IPC, custom protocol requests, file resources, image metadata, fonts, and persisted configuration are validated before trusted operations. Reports that demonstrate a bypass of these boundaries are in scope.
