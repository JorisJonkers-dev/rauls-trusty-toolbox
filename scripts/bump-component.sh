#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Move a Component to a release the way a bump pull request does: its submodule goes to the tag, and its pin file
# components/<dir>.version follows. When the Component is nvmod, the loot-rs tag it links moves too (submodule and the
# workspace Cargo tag, ADR-0015). Lock files are not touched: the caller runs `cargo update` after.
# Usage: scripts/bump-component.sh <nv-rs|nvmod|lodwright-rs|ttwright-rs> vX.Y.Z
set -euo pipefail
component="${1:?usage: scripts/bump-component.sh <component> <tag>}"
tag="${2:?usage: scripts/bump-component.sh <component> <tag>}"
tag="v${tag#v}"
[[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "$tag is not a vX.Y.Z tag" >&2; exit 1; }
case "$component" in
  nv-rs | lodwright-rs | ttwright-rs) dir="$component" ;;
  nvmod | nvmod-rs) dir="nvmod-rs" ;;
  *) echo "$component is not a Component" >&2; exit 1 ;;
esac
checkout_tag() {
  git -C "components/$1" fetch --quiet --tags origin
  git -C "components/$1" checkout --quiet "refs/tags/$2"
}
checkout_tag "$dir" "$tag"
# The self-hosted Renovate run (.github/renovate-components.json) reads the pinned release from this tracked file,
# because it lists the repository's files with git and never sees inside a submodule.
version="$(scripts/component-version.sh "$dir" | sed 's/^v//')"
[ -n "$version" ] || { echo "cannot read the release of components/$dir" >&2; exit 1; }
printf '%s\n' "$version" > "components/$dir.version"
if [ "$dir" = nvmod-rs ]; then
  loot="$(sed -n '/^name = "loot-rs"/,/^\[tool\]/p; /^\[tool.source\]/,/^$/p' components/nvmod-rs/tools.lock.toml | sed -n 's/^tag = "\(v[0-9.]*\)".*/\1/p' | head -n1)"
  [ -n "$loot" ] || { echo "nvmod's tools.lock.toml has no loot-rs tag" >&2; exit 1; }
  checkout_tag loot-rs "$loot"
  sed -i -E "s#^(loot-rs = \{ git = \"https://github.com/JorisJonkers-dev/loot-rs\", tag = \")v[0-9.]+(\" \})#\1$loot\2#" Cargo.toml
  echo "moved $component $tag, loot-rs $loot"
else
  echo "moved $component $tag"
fi
