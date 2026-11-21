#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# A release must never be published, or pass a dry run, without its installers (issue #505: v0.5.0 was published with
# no assets because a build job failed and the publish job was skipped, and nothing said so).
# Usage: scripts/check-release-assets.sh files DIR VERSION   every installer of VERSION is in DIR
#        scripts/check-release-assets.sh release TAG         the GitHub release TAG lists every installer
#        scripts/check-release-assets.sh --self-test         checks both on synthetic input
set -euo pipefail

# The installers of one version, one name per line (release.yml names them "raul-VERSION-SYSTEM.EXT").
expected() {
  local v="$1"
  printf '%s\n' \
    "raul-$v-linux-x64.AppImage" "raul-$v-linux-x64.deb" "raul-$v-linux-x64.tar.gz" \
    "raul-$v-windows-x64.setup.exe" "raul-$v-windows-x64-portable.zip" \
    "raul-$v-macos-universal.dmg" "raul-$v-macos-universal.tar.gz" \
    THIRD-PARTY-NOTICES.md
}

# missing VERSION PRESENT: the expected names that are not in PRESENT (one name per line).
missing() {
  local want
  want="$(expected "$1")"
  grep -vxF -f <(printf '%s\n' "$2") <<<"$want" || true
}

fail_if_missing() { # fail_if_missing VERSION PRESENT WHERE
  local gone
  gone="$(missing "$1" "$2")"
  if [ -n "$gone" ]; then
    echo "::error title=Release without its installers::$3 lacks:" >&2
    echo "$gone" >&2
    return 1
  fi
}

files() {
  fail_if_missing "${2#v}" "$(cd "$1" && find . -maxdepth 1 -type f -printf '%f\n')" "$1"
}

release() {
  local names
  names="$(gh release view "$1" --repo "${GITHUB_REPOSITORY:?}" --json assets --jq '.assets[].name')"
  fail_if_missing "${1#v}" "$names" "release $1" || {
    echo "::error::Release $1 has no complete set of installers. Run the Release workflow by hand from main with the input tag=$1 to build and attach them." >&2
    return 1
  }
  for extra in SHA256SUMS raul.cdx.json; do
    grep -qxF "$extra" <<<"$names" || { echo "::error::release $1 lacks $extra" >&2; return 1; }
  done
}

self_test() {
  local d; d="$(mktemp -d)"; trap 'rm -rf "$d"' RETURN
  while read -r f; do : > "$d/$f"; done < <(expected 1.2.3)
  files "$d" 1.2.3 || { echo "complete set rejected" >&2; return 1; }
  rm "$d/raul-1.2.3-macos-universal.dmg"
  if files "$d" v1.2.3 2>/dev/null; then echo "a missing dmg was accepted" >&2; return 1; fi
  if files "$(mktemp -d)" 1.2.3 2>/dev/null; then echo "an empty folder was accepted" >&2; return 1; fi
  echo "check-release-assets self-test ok"
}

case "${1:-}" in
  files) files "${2:?folder}" "${3:?version}" ;;
  release) release "${2:?tag}" ;;
  --self-test) self_test ;;
  *) sed -n 2,9p "$0" >&2; exit 2 ;;
esac
