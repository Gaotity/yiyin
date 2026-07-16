#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repository_root"

metadata="$(cargo metadata --format-version 1 --no-deps)"

jq -e '
  .packages[]
  | select(.name == "yiyin-domain")
  | .dependencies
  | length == 0
' <<<"$metadata" >/dev/null

jq -e '
  .packages[]
  | select(.name == "yiyin-application")
  | [.dependencies[].name]
  | sort == ["yiyin-domain"]
' <<<"$metadata" >/dev/null

jq -e '
  .packages[]
  | select(.name == "yiyin-infrastructure")
  | [.dependencies[] | select(.path != null) | .name]
  | sort == ["yiyin-application", "yiyin-domain"]
' <<<"$metadata" >/dev/null

jq -e '
  [
    .packages[]
    | select(any(.dependencies[]; .name == "tauri" or (.name | startswith("tauri-plugin-"))))
    | .name
  ]
  | unique == ["yiyin-desktop"]
' <<<"$metadata" >/dev/null

! rg -n \
  '(^|[^[:alnum:]_-])(tauri|serde|serde_json|image|tokio|uuid|exif|cosmic-text)([^[:alnum:]_-]|$)' \
  crates/yiyin-domain crates/yiyin-application
