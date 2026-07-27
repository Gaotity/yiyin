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

- [ ] Repository policy tests define the optimized workflow contract
- [ ] CI, CodeQL, native packaging, and Dependabot configuration satisfy it
- [ ] The obsolete golden required check is removed from the main ruleset
- [ ] Existing unsigned package artifacts are deleted and storage is verified
- [ ] Pull-request checks validate the resulting workflow behavior
