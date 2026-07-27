# 14 — React: Image Tasks, EXIF, Preview, Drag/Drop, and Output Workflows

**What to build:** The main task workspace, rebuilt end to end: native add, drag-drop registration events, selection-first start, progress milestones, success/failure states, EXIF load and copy (clipboard receives only normalized JSON), clear, explicit start, quick output, output-directory choose/open, and the existing error copy. Preview supersession works exactly as specified — 300ms debounce on config/selection changes, preview B cancels preview A, a cancelled A can never overwrite B, closing preview clears the URL, failures show the existing copy — and images display only via `yiyin://resource/<id>` URLs. Playwright browser parity tests inject the fake adapter and verify fixed layout, drawers, image list, preview, settings, templates, and errors, explicitly not claiming coverage of native dialogs or the WebView.

**Blocked by:** 13 — React: Rendering Settings, Fields, Templates, and Fonts

**Status:** completed

- [x] `pnpm vitest run` task tests pass: selection-first start, 300ms debounced preview supersession (cancelled A cannot overwrite B), EXIF copy
- [x] `pnpm playwright test` browser workflow parity passes
- [x] `pnpm build` produces a bundle with no Svelte chunk
