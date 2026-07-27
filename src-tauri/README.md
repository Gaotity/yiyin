# Desktop adapter and composition root

`src-tauri` is the outermost adapter for the Yiyin desktop application. It
contains Tauri-specific commands, DTO mappings, events, native integration,
the custom resource protocol, and dependency composition. Business rules do
not live here.

## Dependency direction

The workspace enforces this inward-only dependency graph:

```text
yiyin-domain <- yiyin-application <- yiyin-infrastructure <- yiyin-desktop
```

- `yiyin-domain` owns pure entities, value objects, invariants, and render
  plans. It has no serialization, filesystem, image, async, or Tauri dependency.
- `yiyin-application` owns use cases and inbound/outbound ports and depends
  only on the domain.
- `yiyin-infrastructure` implements persistence, resource, EXIF, rendering,
  filesystem, and task adapters.
- `yiyin-desktop` maps the Tauri boundary and wires concrete adapters into use
  cases.

Run `bash tests/architecture/dependency-boundaries.sh` after changing crate
dependencies.

## Tracing a command

Trace a desktop action in this order:

1. Find the typed method in `src/platform/client.ts`.
2. Find its dedicated Tauri command in `src/commands/`.
3. Follow the command into the matching application use case held by
   `AppState`.
4. Follow the outbound port to its infrastructure adapter.
5. Follow the explicit DTO/error mapping back to React.

Commands stay thin. They may select a native input, map DTOs, call one use
case, and map the result, but they do not implement rendering or persistence
rules. Task progress uses the single path-free `task-status` event contract.

## Ownership and async boundaries

`AppState` owns shared use cases and adapter handles. Long-running render work
is owned by the Rust task queue, which caps export concurrency at two and uses
replace-latest cancellation for previews. Blocking dialogs, filesystem work,
and image work must not execute on the WebView event loop; commands use the
Tauri async runtime or the queue boundary as appropriate.

React never receives a filesystem path. Resource DTOs contain opaque IDs and
`yiyin://resource/<opaque-id>` URLs. Every protocol request resolves and
revalidates the registered resource inside Rust.

## DTO types

The Rust DTO modules under `src/dto/` are the source of truth. Test builds use
`ts-rs` to assemble the corresponding declarations through
`dto::generated_types()`. The checked-in `src/platform/types.ts` file is
compared byte-for-byte by `src-tauri/tests/dto.rs` and must contain no private
path fields.

After a DTO change, update `src/platform/types.ts` from the Rust declarations
and run:

```bash
cargo test -p yiyin-desktop --test dto --locked
pnpm typecheck
```

Do not hand-add platform fields only on the TypeScript side.

## Validation

From the repository root, run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
bash tests/architecture/dependency-boundaries.sh
pnpm typecheck
pnpm vitest run
```

Production packages must be built without the `e2e-fixture` feature. That
feature exists only for the Windows WebDriver smoke job and adds no
command or path-bearing DTO.
