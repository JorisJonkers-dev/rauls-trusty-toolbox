#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# The checksum list of an install unit (ADR-0020, #179): INSTALLED-FILES.sha256 holds one `sha256sum` line for every file
# of a portable package except itself. `raul update` checks an unpacked update against it, and the release smoke test
# checks the packages with `check`.
# Usage: scripts/installed-files.sh generate DIR    write DIR/INSTALLED-FILES.sha256
#        scripts/installed-files.sh check DIR       fail unless DIR matches its list exactly
#        scripts/installed-files.sh --self-test     checks both on a synthetic folder
set -euo pipefail

LIST=INSTALLED-FILES.sha256

hasher() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum -- "$@"; else shasum -a 256 -- "$@"; fi
}

# Every file below the current folder except the list, one relative path per line, in a fixed order.
files() {
  find . -type f ! -name "$LIST" | sed 's|^\./||' | LC_ALL=C sort
}

generate() {
  local dir="$1" tmp
  tmp="$(mktemp "${TMPDIR:-/tmp}/installed-files.XXXXXX")"
  (
    cd "$dir"
    files | while IFS= read -r f; do hasher "$f"; done >"$tmp"
  )
  [ -s "$tmp" ] || { echo "installed-files: no files in $dir" >&2; rm -f "$tmp"; return 1; }
  mv "$tmp" "$dir/$LIST"
}

check() {
  local dir="$1" listed present
  [ -f "$dir/$LIST" ] || { echo "installed-files: $dir has no $LIST" >&2; return 1; }
  if command -v sha256sum >/dev/null 2>&1; then
    (cd "$dir" && sha256sum -c --quiet "$LIST" >&2) || return 1
  else
    (cd "$dir" && shasum -a 256 -c "$LIST" >/dev/null) || return 1
  fi
  listed="$(sed -E 's/^[0-9a-fA-F]{64} [ *]//' "$dir/$LIST" | LC_ALL=C sort)"
  present="$(cd "$dir" && files)"
  if [ "$listed" != "$present" ]; then
    echo "installed-files: the files in $dir are not the files in its list:" >&2
    diff <(printf '%s\n' "$listed") <(printf '%s\n' "$present") >&2 || true
    return 1
  fi
}

if [ "${1:-}" = "--self-test" ]; then
  tmp="$(mktemp -d "${TMPDIR:-/tmp}/installed-files-test.XXXXXX")"
  trap 'rm -rf "$tmp"' EXIT
  fail() { echo "self-test: $1" >&2; exit 1; }
  mkdir -p "$tmp/u/components/nv-rs/bin" "$tmp/u/dir with space"
  printf 'raul' >"$tmp/u/raul"
  printf 'viewer' >"$tmp/u/components/nv-rs/bin/nv-viewer"
  printf 'x' >"$tmp/u/dir with space/file"
  generate "$tmp/u"
  [ "$(wc -l <"$tmp/u/$LIST" | tr -d ' ')" = 3 ] || fail "the list should have three lines"
  grep -q "  raul\$" "$tmp/u/$LIST" || fail "raul is not listed"
  grep -q "$LIST" <(cut -c67- "$tmp/u/$LIST") && fail "the list lists itself"
  check "$tmp/u" || fail "a fresh list does not check"
  printf 'tampered' >"$tmp/u/raul"
  check "$tmp/u" 2>/dev/null && fail "a changed file passed"
  printf 'raul' >"$tmp/u/raul"
  check "$tmp/u" || fail "the restored file does not check"
  printf 'extra' >"$tmp/u/extra"
  check "$tmp/u" 2>/dev/null && fail "an unlisted file passed"
  rm "$tmp/u/extra" "$tmp/u/raul"
  check "$tmp/u" 2>/dev/null && fail "a missing file passed"
  echo "self-test ok"
  exit 0
fi

case "${1:-}" in
  generate) generate "$(realpath "${2:?usage: scripts/installed-files.sh generate DIR}")" ;;
  check) check "$(realpath "${2:?usage: scripts/installed-files.sh check DIR}")" ;;
  *) echo "usage: scripts/installed-files.sh generate DIR | check DIR | --self-test" >&2; exit 2 ;;
esac
