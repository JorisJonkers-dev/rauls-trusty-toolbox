#!/usr/bin/env bash
# Fails when a submodule points at a commit that is neither a release tag of its repository nor on the branch named
# for it in .gitmodules (nv-rs: fork, the others: main). Asks GitHub, so no deep fetch is needed.
# Usage: scripts/check-submodules.sh   (needs `gh` with read access, GH_TOKEN in CI, and `jq`)
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
fail=0
while read -r key path; do
  name="${key#submodule.}"; name="${name%.path}"
  url="$(git config -f .gitmodules "submodule.$name.url")"
  branch="$(git config -f .gitmodules "submodule.$name.branch" || echo main)"
  repo="$(sed -E 's#(git@github.com:|https://github.com/)##; s#\.git$##' <<<"$url")"
  # The Component follower reads the pinned release from components/<dir>.version (see scripts/bump-component.sh).
  # loot-rs has none: it moves with the nvmod release that links it (ADR-0015).
  if [ "${path#components/}" != "loot-rs" ]; then
    pin="$path.version"
    if [ ! -f "$pin" ]; then
      echo "FAIL $pin is missing; the Component follower would never see $path; run scripts/bump-component.sh ${path#components/} <tag>"; fail=1
    elif ! want="$(scripts/component-version.sh "${path#components/}" | sed 's/^v//')" || [ -z "$want" ]; then
      echo "FAIL cannot read the release of $path from its release-please manifest"; fail=1
    else
      have="$(tr -d '[:space:]' < "$pin")"
      [ "$want" = "$have" ] || { echo "FAIL $pin says $have but $path is at $want; run scripts/bump-component.sh ${path#components/} v$want"; fail=1; }
    fi
  fi
  sha="$(git ls-tree HEAD "$path" | awk '{print $3}')"
  if git ls-remote --tags "$url" | awk '{print $1}' | grep -qx "$sha"; then
    tag="$(git ls-remote --tags "$url" | awk -v s="$sha" '$1==s{print $2}' | sed 's#refs/tags/##; s#\^{}##' | head -1)"
    echo "ok   $path $sha (tag $tag)"
    continue
  fi
  status="$(gh api "repos/$repo/compare/$branch...$sha" --jq .status 2>/dev/null || echo missing)"
  case "$status" in
    identical|behind) echo "ok   $path $sha (on $branch)" ;;
    *) echo "FAIL $path $sha is not a tag of $repo and not on its $branch branch ($status)"; fail=1 ;;
  esac
done < <(git config -f .gitmodules --get-regexp '\.path$')
exit "$fail"
