#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repository_root"

metadata="$(cargo metadata --format-version 1 --no-deps)"

command -v rg >/dev/null || {
  echo "ripgrep (rg) is required for the production-purity source scan" >&2
  exit 1
}

jq -e '
  .packages[]
  | select(.name == "yiyin-domain")
  | [.dependencies[] | select(.kind == null)]
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

# Production code stays pure; dev-only test dependencies (e.g. parsing the
# frozen legacy fixtures) are allowed and covered by the jq assertion above.
! rg -n \
  '((use|extern crate)[[:space:]]+(tauri|serde|serde_json|image|tokio|uuid|exif|cosmic_text)(::|[[:space:]]|;)|(^|[^[:alnum:]_])(tauri|serde|serde_json|image|tokio|uuid|exif|cosmic_text)::)' \
  crates/yiyin-domain/src crates/yiyin-application/src \
  --glob '*.rs'
