# Linear migration record (2026-07-30)

On 2026-07-30 every local ticket under `docs/tickets/` was migrated into the
canonical tracker — the Linear [`Yiyin`](https://linear.app/wg-studio/project/yiyin-3ba958517518/overview)
project (wg-studio workspace, team `Engineering` / `ENG`) — and the local files
were frozen as the historical record. Migrated issues carry the
`migrated-from-repo` label and a `cluster:<feature-slug>` label; each
description is bilingual (English block, `---`, Simplified Chinese block) with
the source path under `## References`. (The issues were first created under the
`C-level` team as `C-5…C-49` and moved to `Engineering` the same day; the table
below uses the current `ENG-*` identifiers.)

- `completed` tickets → Linear state **Done**
- `wontfix` tickets → Linear state **Canceled**
- The six open architecture-review backlog candidates were created fresh with
  the `architecture-backlog` label in state **Backlog**

| Local ticket (frozen) | Linear issue |
| --- | --- |
| `docs/tickets/tauri-rust-rewrite/01-capture-legacy-compatibility-fixtures.md` | [ENG-32](https://linear.app/wg-studio/issue/ENG-32) |
| `docs/tickets/tauri-rust-rewrite/02-establish-toolchains-workspace-frontend-shell.md` | [ENG-31](https://linear.app/wg-studio/issue/ENG-31) |
| `docs/tickets/tauri-rust-rewrite/03-domain-config-templates-metadata-resources.md` | [ENG-30](https://linear.app/wg-studio/issue/ENG-30) |
| `docs/tickets/tauri-rust-rewrite/04-domain-render-plans-output-naming-task-state.md` | [ENG-29](https://linear.app/wg-studio/issue/ENG-29) |
| `docs/tickets/tauri-rust-rewrite/05-application-ports-errors-config-resource-use-cases.md` | [ENG-28](https://linear.app/wg-studio/issue/ENG-28) |
| `docs/tickets/tauri-rust-rewrite/06-application-task-orchestration.md` | [ENG-27](https://linear.app/wg-studio/issue/ENG-27) |
| `docs/tickets/tauri-rust-rewrite/07-durable-config-and-legacy-import.md` | [ENG-26](https://linear.app/wg-studio/issue/ENG-26) |
| `docs/tickets/tauri-rust-rewrite/08-resource-registry-and-exif-adapter.md` | [ENG-25](https://linear.app/wg-studio/issue/ENG-25) |
| `docs/tickets/tauri-rust-rewrite/09-rust-renderer-and-golden-comparisons.md` | [ENG-24](https://linear.app/wg-studio/issue/ENG-24) |
| `docs/tickets/tauri-rust-rewrite/10-concurrency-two-task-queue.md` | [ENG-23](https://linear.app/wg-studio/issue/ENG-23) |
| `docs/tickets/tauri-rust-rewrite/11-tauri-boundary-dto-error-protocol-composition.md` | [ENG-22](https://linear.app/wg-studio/issue/ENG-22) |
| `docs/tickets/tauri-rust-rewrite/12-react-typed-client-app-shell-chrome.md` | [ENG-21](https://linear.app/wg-studio/issue/ENG-21) |
| `docs/tickets/tauri-rust-rewrite/13-react-settings-fields-templates-fonts.md` | [ENG-20](https://linear.app/wg-studio/issue/ENG-20) |
| `docs/tickets/tauri-rust-rewrite/14-react-tasks-exif-preview-output-workflows.md` | [ENG-19](https://linear.app/wg-studio/issue/ENG-19) |
| `docs/tickets/tauri-rust-rewrite/15-security-supply-chain-ci-packaging-gates.md` | [ENG-18](https://linear.app/wg-studio/issue/ENG-18) |
| `docs/tickets/tauri-rust-rewrite/16-remove-legacy-runtime-full-acceptance.md` | [ENG-17](https://linear.app/wg-studio/issue/ENG-17) |
| `docs/tickets/legacy-electron-audit/00-tracking-hub.md` | [ENG-16](https://linear.app/wg-studio/issue/ENG-16) |
| `docs/tickets/legacy-electron-audit/01-image-pipeline.md` | [ENG-38](https://linear.app/wg-studio/issue/ENG-38) |
| `docs/tickets/legacy-electron-audit/02-exif-formatting.md` | [ENG-42](https://linear.app/wg-studio/issue/ENG-42) |
| `docs/tickets/legacy-electron-audit/03-config-migration.md` | [ENG-35](https://linear.app/wg-studio/issue/ENG-35) |
| `docs/tickets/legacy-electron-audit/04-runtime-robustness.md` | [ENG-37](https://linear.app/wg-studio/issue/ENG-37) |
| `docs/tickets/legacy-electron-audit/05-ipc-boundary.md` | [ENG-43](https://linear.app/wg-studio/issue/ENG-43) |
| `docs/tickets/legacy-electron-audit/06-test-completeness.md` | [ENG-39](https://linear.app/wg-studio/issue/ENG-39) |
| `docs/tickets/legacy-electron-audit/07-renderer-isolation-guardrails.md` | [ENG-44](https://linear.app/wg-studio/issue/ENG-44) |
| `docs/tickets/legacy-electron-audit/08-main-process-security-model.md` | [ENG-45](https://linear.app/wg-studio/issue/ENG-45) |
| `docs/tickets/legacy-electron-audit/09-types-and-lint.md` | [ENG-41](https://linear.app/wg-studio/issue/ENG-41) |
| `docs/tickets/legacy-electron-audit/10-dependencies-supply-chain.md` | [ENG-40](https://linear.app/wg-studio/issue/ENG-40) |
| `docs/tickets/legacy-electron-audit/11-build-and-packaging.md` | [ENG-36](https://linear.app/wg-studio/issue/ENG-36) |
| `docs/tickets/legacy-electron-audit/12-web-ui-completeness.md` | [ENG-33](https://linear.app/wg-studio/issue/ENG-33) |
| `docs/tickets/legacy-electron-audit/13-docs-accuracy.md` | [ENG-34](https://linear.app/wg-studio/issue/ENG-34) |
| `docs/tickets/post-rewrite-hardening/00-map.md` | [ENG-15](https://linear.app/wg-studio/issue/ENG-15) |
| `docs/tickets/post-rewrite-hardening/01-registration-without-full-decodes.md` | [ENG-14](https://linear.app/wg-studio/issue/ENG-14) |
| `docs/tickets/post-rewrite-hardening/02-sub-second-shutter-display.md` | [ENG-13](https://linear.app/wg-studio/issue/ENG-13) |
| `docs/tickets/post-rewrite-hardening/03-renderer-import-boundary-guardrail.md` | [ENG-12](https://linear.app/wg-studio/issue/ENG-12) |
| `docs/tickets/post-rewrite-hardening/04-dangling-v1.7.0-tag.md` | [ENG-11](https://linear.app/wg-studio/issue/ENG-11) |
| `docs/tickets/post-rewrite-hardening/05-graceful-legacy-config-import.md` | [ENG-10](https://linear.app/wg-studio/issue/ENG-10) |
| `docs/tickets/actions-usage/01-optimize-actions-usage.md` | [ENG-9](https://linear.app/wg-studio/issue/ENG-9) |
| `docs/tickets/numeric-constraints-via-bootstrap/01-numeric-constraints-via-bootstrap.md` | [ENG-8](https://linear.app/wg-studio/issue/ENG-8) |
| `docs/tickets/output-directory-write-path/01-output-directory-dedicated-write-path.md` | [ENG-7](https://linear.app/wg-studio/issue/ENG-7) |
| `architecture-backlog/config-update-validation` | [ENG-6](https://linear.app/wg-studio/issue/ENG-6) |
| `architecture-backlog/durable-publish-convergence` | [ENG-1](https://linear.app/wg-studio/issue/ENG-1) |
| `architecture-backlog/preview-orchestration-controller` | [ENG-4](https://linear.app/wg-studio/issue/ENG-4) |
| `architecture-backlog/config-edit-intents` | [ENG-3](https://linear.app/wg-studio/issue/ENG-3) |
| `architecture-backlog/renderer-filesystem-seam` | [ENG-2](https://linear.app/wg-studio/issue/ENG-2) |
| `architecture-backlog/single-newness-diff` | [ENG-5](https://linear.app/wg-studio/issue/ENG-5) |
