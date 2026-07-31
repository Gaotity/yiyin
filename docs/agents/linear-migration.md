# Linear migration record (2026-07-30)

On 2026-07-30 every local ticket under `docs/tickets/` was migrated into the
canonical tracker — the Linear [`Yiyin`](https://linear.app/wg-studio/project/yiyin-3ba958517518/overview)
project (wg-studio workspace, team `C-level`) — and the local files were frozen
as the historical record. Migrated issues carry the `migrated-from-repo` label
and a `cluster:<feature-slug>` label; each description is bilingual
(English block, `---`, Simplified Chinese block) with the source path under
`## References`.

- `completed` tickets → Linear state **Done**
- `wontfix` tickets → Linear state **Canceled**
- The six open architecture-review backlog candidates were created fresh with
  the `architecture-backlog` label in state **Backlog**

| Local ticket (frozen) | Linear issue |
| --- | --- |
| `docs/tickets/tauri-rust-rewrite/01-capture-legacy-compatibility-fixtures.md` | [C-5](https://linear.app/wg-studio/issue/C-5/tauri-rust-rewrite-capture-and-freeze-legacy-compatibility-fixtures) |
| `docs/tickets/tauri-rust-rewrite/02-establish-toolchains-workspace-frontend-shell.md` | [C-6](https://linear.app/wg-studio/issue/C-6/tauri-rust-rewrite-establish-toolchains-workspace-boundaries-and) |
| `docs/tickets/tauri-rust-rewrite/03-domain-config-templates-metadata-resources.md` | [C-7](https://linear.app/wg-studio/issue/C-7/tauri-rust-rewrite-domain-configuration-templates-metadata-and) |
| `docs/tickets/tauri-rust-rewrite/04-domain-render-plans-output-naming-task-state.md` | [C-8](https://linear.app/wg-studio/issue/C-8/tauri-rust-rewrite-domain-render-plans-output-naming-and-task-state) |
| `docs/tickets/tauri-rust-rewrite/05-application-ports-errors-config-resource-use-cases.md` | [C-9](https://linear.app/wg-studio/issue/C-9/tauri-rust-rewrite-application-ports-errors-and-configurationresource) |
| `docs/tickets/tauri-rust-rewrite/06-application-task-orchestration.md` | [C-10](https://linear.app/wg-studio/issue/C-10/tauri-rust-rewrite-application-task-orchestration) |
| `docs/tickets/tauri-rust-rewrite/07-durable-config-and-legacy-import.md` | [C-11](https://linear.app/wg-studio/issue/C-11/tauri-rust-rewrite-infrastructure-durable-configuration-and-idempotent) |
| `docs/tickets/tauri-rust-rewrite/08-resource-registry-and-exif-adapter.md` | [C-12](https://linear.app/wg-studio/issue/C-12/tauri-rust-rewrite-infrastructure-resource-registry-and-exif-adapter) |
| `docs/tickets/tauri-rust-rewrite/09-rust-renderer-and-golden-comparisons.md` | [C-13](https://linear.app/wg-studio/issue/C-13/tauri-rust-rewrite-infrastructure-rust-renderer-and-golden-comparisons) |
| `docs/tickets/tauri-rust-rewrite/10-concurrency-two-task-queue.md` | [C-14](https://linear.app/wg-studio/issue/C-14/tauri-rust-rewrite-infrastructure-concurrency-two-task-queue-and) |
| `docs/tickets/tauri-rust-rewrite/11-tauri-boundary-dto-error-protocol-composition.md` | [C-15](https://linear.app/wg-studio/issue/C-15/tauri-rust-rewrite-tauri-boundary-dtos-errors-protocol-native) |
| `docs/tickets/tauri-rust-rewrite/12-react-typed-client-app-shell-chrome.md` | [C-16](https://linear.app/wg-studio/issue/C-16/tauri-rust-rewrite-react-typed-client-app-shell-and-chrome-parity) |
| `docs/tickets/tauri-rust-rewrite/13-react-settings-fields-templates-fonts.md` | [C-17](https://linear.app/wg-studio/issue/C-17/tauri-rust-rewrite-react-rendering-settings-fields-templates-and-fonts) |
| `docs/tickets/tauri-rust-rewrite/14-react-tasks-exif-preview-output-workflows.md` | [C-18](https://linear.app/wg-studio/issue/C-18/tauri-rust-rewrite-react-image-tasks-exif-preview-dragdrop-and-output) |
| `docs/tickets/tauri-rust-rewrite/15-security-supply-chain-ci-packaging-gates.md` | [C-19](https://linear.app/wg-studio/issue/C-19/tauri-rust-rewrite-security-supply-chain-ci-and-native-packaging-gates) |
| `docs/tickets/tauri-rust-rewrite/16-remove-legacy-runtime-full-acceptance.md` | [C-20](https://linear.app/wg-studio/issue/C-20/tauri-rust-rewrite-remove-the-legacy-runtime-update-documentation-and) |
| `docs/tickets/legacy-electron-audit/00-tracking-hub.md` | [C-21](https://linear.app/wg-studio/issue/C-21/legacy-electron-audit-tracking-hub-adversarial-audit-findings-legacy) |
| `docs/tickets/legacy-electron-audit/01-image-pipeline.md` | [C-22](https://linear.app/wg-studio/issue/C-22/legacy-electron-audit-image-pipeline-resource-leaks-and-dead-cancel) |
| `docs/tickets/legacy-electron-audit/02-exif-formatting.md` | [C-23](https://linear.app/wg-studio/issue/C-23/legacy-electron-audit-exif-formatting-510-user-visible-output-errors) |
| `docs/tickets/legacy-electron-audit/03-config-migration.md` | [C-24](https://linear.app/wg-studio/issue/C-24/legacy-electron-audit-config-and-migration-failure-path-can-overwrite) |
| `docs/tickets/legacy-electron-audit/04-runtime-robustness.md` | [C-25](https://linear.app/wg-studio/issue/C-25/legacy-electron-audit-runtime-robustness-exit-truncates-in-flight) |
| `docs/tickets/legacy-electron-audit/05-ipc-boundary.md` | [C-26](https://linear.app/wg-studio/issue/C-26/legacy-electron-audit-ipc-boundary-shadowrender-events-leak-host-paths) |
| `docs/tickets/legacy-electron-audit/06-test-completeness.md` | [C-27](https://linear.app/wg-studio/issue/C-27/legacy-electron-audit-test-completeness-coverage-whitelist-theater-e2e) |
| `docs/tickets/legacy-electron-audit/07-renderer-isolation-guardrails.md` | [C-28](https://linear.app/wg-studio/issue/C-28/legacy-electron-audit-renderer-isolation-guardrails) |
| `docs/tickets/legacy-electron-audit/08-main-process-security-model.md` | [C-29](https://linear.app/wg-studio/issue/C-29/legacy-electron-audit-main-process-security-model-toctou-on-main-path) |
| `docs/tickets/legacy-electron-audit/09-types-and-lint.md` | [C-30](https://linear.app/wg-studio/issue/C-30/legacy-electron-audit-types-and-lint-any-escapes-on-the-exif-chain) |
| `docs/tickets/legacy-electron-audit/10-dependencies-supply-chain.md` | [C-31](https://linear.app/wg-studio/issue/C-31/legacy-electron-audit-dependencies-and-supply-chain-db-ui-pulls-eol) |
| `docs/tickets/legacy-electron-audit/11-build-and-packaging.md` | [C-32](https://linear.app/wg-studio/issue/C-32/legacy-electron-audit-build-and-packaging-verify-package-silently) |
| `docs/tickets/legacy-electron-audit/12-web-ui-completeness.md` | [C-33](https://linear.app/wg-studio/issue/C-33/legacy-electron-audit-web-ui-task-progress-never-updates-on-screen) |
| `docs/tickets/legacy-electron-audit/13-docs-accuracy.md` | [C-34](https://linear.app/wg-studio/issue/C-34/legacy-electron-audit-docs-accuracy-hardening-plan-facts-and) |
| `docs/tickets/post-rewrite-hardening/00-map.md` | [C-35](https://linear.app/wg-studio/issue/C-35/post-rewrite-hardening-map-post-rewrite-hardening) |
| `docs/tickets/post-rewrite-hardening/01-registration-without-full-decodes.md` | [C-36](https://linear.app/wg-studio/issue/C-36/post-rewrite-hardening-register-images-without-full-pixel-decodes) |
| `docs/tickets/post-rewrite-hardening/02-sub-second-shutter-display.md` | [C-37](https://linear.app/wg-studio/issue/C-37/post-rewrite-hardening-display-sub-second-shutter-speeds-as-decimal) |
| `docs/tickets/post-rewrite-hardening/03-renderer-import-boundary-guardrail.md` | [C-38](https://linear.app/wg-studio/issue/C-38/post-rewrite-hardening-automated-renderer-import-boundary-guardrail) |
| `docs/tickets/post-rewrite-hardening/04-dangling-v1.7.0-tag.md` | [C-39](https://linear.app/wg-studio/issue/C-39/post-rewrite-hardening-resolve-the-dangling-v170-tag-before-the-next) |
| `docs/tickets/post-rewrite-hardening/05-graceful-legacy-config-import.md` | [C-40](https://linear.app/wg-studio/issue/C-40/post-rewrite-hardening-legacy-config-import-must-degrade-gracefully) |
| `docs/tickets/actions-usage/01-optimize-actions-usage.md` | [C-41](https://linear.app/wg-studio/issue/C-41/actions-usage-reduce-github-actions-resource-usage) |
| `docs/tickets/numeric-constraints-via-bootstrap/01-numeric-constraints-via-bootstrap.md` | [C-42](https://linear.app/wg-studio/issue/C-42/numeric-constraints-via-bootstrap-land-adr-0003-numeric-constraints) |
| `docs/tickets/output-directory-write-path/01-output-directory-dedicated-write-path.md` | [C-43](https://linear.app/wg-studio/issue/C-43/output-directory-write-path-land-adr-0002-output-directory-writable) |
| `architecture-backlog/config-update-validation` | [C-44](https://linear.app/wg-studio/issue/C-44/architecture-backlog-unify-config-update-validation-in-the-application) |
| `architecture-backlog/durable-publish-convergence` | [C-45](https://linear.app/wg-studio/issue/C-45/architecture-backlog-converge-the-three-durable-publish) |
| `architecture-backlog/preview-orchestration-controller` | [C-46](https://linear.app/wg-studio/issue/C-46/architecture-backlog-consolidate-preview-orchestration-into-the) |
| `architecture-backlog/config-edit-intents` | [C-47](https://linear.app/wg-studio/issue/C-47/architecture-backlog-replace-structuredclone-surgery-with-config-edit) |
| `architecture-backlog/renderer-filesystem-seam` | [C-48](https://linear.app/wg-studio/issue/C-48/architecture-backlog-close-the-filesystem-seam-leaks-in-the-renderer) |
| `architecture-backlog/single-newness-diff` | [C-49](https://linear.app/wg-studio/issue/C-49/architecture-backlog-compute-the-registration-newness-diff-exactly) |
