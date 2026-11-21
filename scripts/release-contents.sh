#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Print the "Contents" section of the release notes (issue #169): what each installer holds, with its SHA-256.
# The checksums are read from the release's own SHA256SUMS, so the notes cannot disagree with the files.
# Usage: scripts/release-contents.sh SHA256SUMS      reads the Component versions from the submodules
#        scripts/release-contents.sh --install       the "Install" section that opens the notes (issue #516)
#        scripts/release-contents.sh --strip         stdin: notes that already got our sections, without them again
#        scripts/release-contents.sh --self-test     checks the rendering on synthetic input
set -euo pipefail

# install: what a person who lands on the release page needs first, before the list of changes.
install() {
  echo "## Install"
  echo
  echo "Download the file for your system below and follow the Install section of the"
  echo "[README](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox#install). To check a download, put it next to"
  echo "\`SHA256SUMS\` and run \`sha256sum -c SHA256SUMS --ignore-missing\` (Linux; macOS: \`shasum -a 256 <file>\`, Windows"
  echo "PowerShell: \`Get-FileHash <file>\`, then compare with the line of that file)."
  echo
}

# strip: the notes release-please wrote, from notes that a run of this workflow already added the Install, Contents and
# Signing sections to (a run for the same tag is repeated by hand), so a repeat does not add them twice.
strip() {
  awk '
    /^## Contents$/ { exit }
    /^## Install$/ { skip = 1; next }
    skip && /^## / { skip = 0 }
    !skip { print }
  '
}

# render SUMS COMPONENTS: SUMS is a SHA256SUMS file; COMPONENTS has one "name version role" line per Component.
render() {
  echo "## Contents"
  echo
  echo "Every installer holds raul with the linked Components built in (ADR-0016) and the engine (nv-rs) too."
  echo "No separate engine program is included; the game starts as raul's own play mode."
  echo "Nothing of ours is downloaded on first run."
  echo
  echo "| Part | Version | How it is in the installer |"
  echo "| --- | --- | --- |"
  while read -r name version role; do
    if [ -n "$name" ]; then echo "| $name | $version | $role |"; fi
  done <"$2"
  echo
  echo "\`SHA256SUMS\` lists the files below; check a download against it."
  echo
  echo "| File | SHA-256 |"
  echo "| --- | --- |"
  while read -r sum file; do
    if [ -n "$sum" ]; then echo "| \`${file#\*}\` | \`$sum\` |"; fi
  done <"$1"
}

if [ "${1:-}" = "--install" ]; then
  install
  exit 0
fi

if [ "${1:-}" = "--strip" ]; then
  strip
  exit 0
fi

if [ "${1:-}" = "--self-test" ]; then
  tmp="$(mktemp -d "${TMPDIR:-/tmp}/release-contents.XXXXXX")"
  trap 'rm -rf "$tmp"' EXIT
  a="$(printf a | sha256sum | cut -d' ' -f1)"
  b="$(printf b | sha256sum | cut -d' ' -f1)"
  printf '%s  raul-1.0.0-linux-x64.AppImage\n%s *raul-1.0.0-windows-x64.setup.exe\n' "$a" "$b" >"$tmp/sums"
  printf 'nvmod-rs v1.2.3 linked into raul\nnv-rs v4.5.6 linked into raul\n' >"$tmp/components"
  out="$(render "$tmp/sums" "$tmp/components")"
  fail() { echo "self-test: $1" >&2; exit 1; }
  grep -qF "| \`raul-1.0.0-linux-x64.AppImage\` | \`$a\` |" <<<"$out" || fail "a checksum row is missing"
  grep -qF "| \`raul-1.0.0-windows-x64.setup.exe\` | \`$b\` |" <<<"$out" || fail "the binary-mode marker was not stripped"
  grep -qF '| nv-rs | v4.5.6 | linked into raul |' <<<"$out" || fail "a Component row is missing"
  ! grep -qi 'packaged engine' <<<"$out" || fail "a packaged engine is still named"
  grep -qF 'No separate engine program is included' <<<"$out" || fail "the no-engine sentence is missing"
  [ "$(grep -c '^| `' <<<"$out")" = 2 ] || fail "the file table has the wrong number of rows"
  grep -qF 'SHA256SUMS' <<<"$out" || fail "the table does not say which file to check against"
  top="$(install)"
  grep -qF '## Install' <<<"$top" || fail "the install section is missing"
  grep -qF 'sha256sum -c SHA256SUMS' <<<"$top" || fail "the install section has no check command"
  grep -qF '#install' <<<"$top" || fail "the install section does not link the README"
  base=$'## [1.0.0] (2026-01-01)\n\n### Features\n\n* a change\n'
  again="$(printf '%s\n%s\n%s\n' "$top" "$base" "$out" | strip)"
  [ "$again" = "$(printf '%s' "$base")" ] || fail "a repeat does not give back the notes release-please wrote"
  [ "$(printf '%s\n' "$base" | strip)" = "$(printf '%s' "$base")" ] || fail "notes without our sections changed"
  echo "self-test ok"
  exit 0
fi

sums="$(realpath "${1:?usage: scripts/release-contents.sh SHA256SUMS}")"
cd "$(git rev-parse --show-toplevel)"
tmp="$(mktemp "${TMPDIR:-/tmp}/components.XXXXXX")"
trap 'rm -f "$tmp"' EXIT
# The Components are all compiled into raul, the engine (nv-rs) included.
for c in nvmod-rs nv-rs loot-rs lodwright-rs ttwright-rs; do echo "$c $(scripts/component-version.sh "$c") linked into raul"; done >"$tmp"
render "$sums" "$tmp"
