#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Fails when a package licensed GPL-3.0-or-later sits outside components/nv-rs. deny.toml allows that licence
# id globally (the nv-rs crates are not publish = false, so cargo-deny does not treat them as private); this keeps
# the allowance from covering any other dependency.
# Usage: scripts/check-gpl-scope.sh                 checks the umbrella and the GUI workspace (cargo metadata)
#        scripts/check-gpl-scope.sh FILE...         checks cargo metadata JSON files (used by --self-test)
#        scripts/check-gpl-scope.sh --self-test     checks the check itself on synthetic metadata
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
root="$PWD"

stray() { # metadata JSON on stdin: prints "name manifest" for each stray package
  jq -r --arg ok "$root/components/nv-rs/" '
    .packages[]
    | select((.license // "") | test("(^|[^L])GPL-3\\.0-or-later"))
    | select(.manifest_path | startswith($ok) | not)
    | "\(.name) \(.manifest_path)"'
}

if [ "${1:-}" = "--self-test" ]; then
  good='{"packages":[{"name":"host","license":"GPL-3.0-or-later","manifest_path":"'"$root"'/components/nv-rs/crates/host/Cargo.toml"},{"name":"x","license":"LGPL-3.0-or-later","manifest_path":"/r/x/Cargo.toml"}]}'
  bad='{"packages":[{"name":"stray","license":"GPL-3.0-or-later","manifest_path":"/r/stray/Cargo.toml"}]}'
  [ -z "$(stray <<<"$good")" ] || { echo "self-test: a nv-rs crate was flagged" >&2; exit 1; }
  [ "$(stray <<<"$bad")" = "stray /r/stray/Cargo.toml" ] || { echo "self-test: a stray crate was not flagged" >&2; exit 1; }
  echo "self-test ok"
  exit 0
fi

fail=0
check() { # label, metadata JSON file
  local found
  found="$(stray <"$2")"
  if [ -n "$found" ]; then
    echo "FAIL $1: GPL-3.0-or-later outside components/nv-rs:" >&2
    echo "$found" >&2
    fail=1
  else
    echo "ok   $1"
  fi
}

if [ "$#" -gt 0 ]; then
  for f in "$@"; do check "$f" "$f"; done
else
  tmp="$(mktemp)"
  trap 'rm -f "$tmp"' EXIT
  for manifest in Cargo.toml crates/raul-gui/Cargo.toml; do
    cargo metadata --locked --format-version 1 --manifest-path "$manifest" >"$tmp"
    check "$manifest" "$tmp"
  done
fi
exit "$fail"
