# 01 — Reduce GitHub Actions Resource Usage

**What to build:** Remove the duplicate macOS golden-rendering job while keeping
the golden suite inside the required Ubuntu Rust job. Run native package smoke
jobs only for pull requests that change desktop inputs, keep their existing
required-check names, and upload unsigned packages only for manual runs. Retain
manual packages for seven days and failure diagnostics for three days. Group
Dependabot minor and patch updates per ecosystem, and cancel superseded runs for
the same pull request.

**Blocked by:** None

**Status:** in progress

- [x] Repository policy tests define the optimized workflow contract — the
  "GitHub automation policy" block in `tests/security/policy.test.ts` (12 tests
  green on `origin/main`)
- [x] CI, CodeQL, native packaging, and dependency-update configuration satisfy
  it — the Dependabot grouping item was superseded by the Renovate migration
  (#25): `dependabot.yml` is deleted and `renovate.json` groups minor+patch per
  ecosystem (npm, cargo, github-actions) with no auto-merge
- [x] The obsolete golden required check is removed from the main ruleset —
  verified via API: ruleset "Protect main" (id 18975693) requires only Frontend
  quality, Rust quality, Security and supply chain, macOS package smoke, and
  Windows package and desktop smoke
- [x] Existing unsigned package artifacts are deleted and storage is verified —
  artifact listing shows no stale auto-uploaded packages; the only unsigned
  package is a manually dispatched `yiyin-windows-unsigned` (2026-07-27,
  7-day retention) that self-expires
- [ ] Pull-request checks validate the resulting workflow behavior
