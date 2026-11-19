#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Print the "Contents" section of the release notes (issue #169): what each installer holds, with its SHA-256.
# The checksums are read from the release's own SHA256SUMS, so the notes cannot disagree with the files.
# Usage: scripts/release-contents.sh SHA256SUMS      reads the Component versions from the submodules
#        scripts/release-contents.sh --self-test     checks the rendering on synthetic input
set -euo pipefail

# render SUMS COMPONENTS: SUMS is a SHA256SUMS file; COMPONENTS has one "name version role" line per Component.
render() {
  echo "## Contents"
  echo
  echo "Every installer holds raul with the linked Components built in (ADR-0016) and the packaged engine."
  echo "Nothing of ours is downloaded on first run."
  echo
  echo "| Part | Version | How it is in the installer |"
  echo "| --- | --- | --- |"
  while read -r name version role; do
    if [ -n "$name" ]; then echo "| $name | $version | $role |"; fi
  done <"$2"
  echo
  echo "| File | SHA-256 |"
  echo "| --- | --- |"
  while read -r sum file; do
    if [ -n "$sum" ]; then echo "| \`${file#\*}\` | \`$sum\` |"; fi
  done <"$1"
}

if [ "${1:-}" = "--self-test" ]; then
  tmp="$(mktemp -d "${TMPDIR:-/tmp}/release-contents.XXXXXX")"
  trap 'rm -rf "$tmp"' EXIT
  a="$(printf a | sha256sum | cut -d' ' -f1)"
  b="$(printf b | sha256sum | cut -d' ' -f1)"
  printf '%s  raul-1.0.0-linux-x64.AppImage\n%s *raul-1.0.0-windows-x64.setup.exe\n' "$a" "$b" >"$tmp/sums"
  printf 'nvmod-rs v1.2.3 linked into raul\nnv-rs v4.5.6 packaged engine build\n' >"$tmp/components"
  out="$(render "$tmp/sums" "$tmp/components")"
  fail() { echo "self-test: $1" >&2; exit 1; }
  grep -qF "| \`raul-1.0.0-linux-x64.AppImage\` | \`$a\` |" <<<"$out" || fail "a checksum row is missing"
  grep -qF "| \`raul-1.0.0-windows-x64.setup.exe\` | \`$b\` |" <<<"$out" || fail "the binary-mode marker was not stripped"
  grep -qF '| nv-rs | v4.5.6 | packaged engine build |' <<<"$out" || fail "a Component row is missing"
  [ "$(grep -c '^| `' <<<"$out")" = 2 ] || fail "the file table has the wrong number of rows"
  echo "self-test ok"
  exit 0
fi

sums="$(realpath "${1:?usage: scripts/release-contents.sh SHA256SUMS}")"
cd "$(git rev-parse --show-toplevel)"
tmp="$(mktemp "${TMPDIR:-/tmp}/components.XXXXXX")"
trap 'rm -f "$tmp"' EXIT
# The linked Components are compiled into raul. nv-rs is the engine's packaged build until it is linked too (#260).
for c in nvmod-rs loot-rs lodwright-rs ttwright-rs; do echo "$c $(scripts/component-version.sh "$c") linked into raul"; done >"$tmp"
echo "nv-rs $(scripts/component-version.sh nv-rs) packaged engine build (Linux and Windows)" >>"$tmp"
render "$sums" "$tmp"
