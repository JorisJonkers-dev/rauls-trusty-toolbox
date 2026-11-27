#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Decides what a CI run has to build and test, from the files a pull request changes. Writes these to $GITHUB_OUTPUT
# (stdout outside Actions):
#   code          true when any changed file is not documentation (the checks that read the repository run either way)
#   umbrella      true when the umbrella workspace has something to build and test
#   umbrella_args the cargo selection for it: `--workspace`, or `-p a -p b` (the changed crates and their dependents)
#   gui           the same for the GUI workspace (crates/raul-gui, its own lock file)
#   gui_args
#   platforms     a matrix (`{"include":[...]}`) of the Windows and macOS legs to run; empty when none
#   allowed_skips the jobs `Pipeline Complete` accepts as skipped (what this script decided not to run)
#
# A push to main, a manual run, or a run with no base revision selects everything on every system. A pull request
# selects by what it changes:
#   - a file inside a crate selects that crate and, through `cargo metadata`, every workspace member that depends on
#     it (dev-dependencies included), in either workspace;
#   - a Component moving (the submodule pointer or components/<name>.version) selects the members that depend on a
#     crate of that Component;
#   - a file every crate can read (manifests, lock files, toolchain, contracts/, lists/, catalogue/, CI scripts and
#     the CI workflow) selects everything; so does any file this script does not know;
#   - documentation, the site and the release tooling select no Rust at all.
# Windows and macOS run on a pull request only for platform-specific changes (packaging, installers, release scripts,
# the toolchain and lock files, a file with cfg(windows), cfg(unix) or cfg(target_os)) or with the `ci:all-os` label.
# Linux runs whatever is selected.
#
# Usage: BASE=<sha> HEAD=<sha> scripts/ci/changes.sh | scripts/ci/changes.sh --self-test
# Environment: GITHUB_EVENT_NAME, GITHUB_OUTPUT, and for the label PR_NUMBER and GITHUB_REPOSITORY (gh reads GH_TOKEN).
set -euo pipefail

# Reads changed paths on stdin; succeeds when one of them is not documentation.
touches_code() { grep -qvE '^(docs/|LICENSE[^/]*$|[^/]*\.md$|.*/[^/]*\.md$)'; }

# Paths that select no Rust build or test: the site, release and repository tooling, and the other workflows (each has
# its own path filters; the policy job lints all of them).
IGNORED='^(docs/|site/|LICENSE[^/]*$|[^/]*\.md$|.*/[^/]*\.md$|renovate\.json$|release-please-config\.json$|\.release-please-manifest\.json$|\.gitleaks\.toml$|\.editorconfig$|\.gitattributes$|\.gitignore$|scripts/(apply-ruleset|bump-component|check-deb-install|check-gpl-scope|check-public-text|check-release-assets|check-submodules|component-version|installed-files|measure-build|release-contents)|scripts/ruleset_bypass\.jq$|scripts/component-locks\.allow$|\.github/(workflows/(bench|components|engine|installer-smoke|pages|release|repository-hygiene)\.yml|renovate-components\.json|ISSUE_TEMPLATE/|CODEOWNERS|dependabot\.yml))'
# Paths that decide the GUI workspace on its own: its manifest and lock (the umbrella never reads them).
GUI_ONLY='^crates/raul-gui/(Cargo\.(toml|lock)|deny\.toml|\.config/)'
# Platform-specific changes: these run Windows and macOS on a pull request.
PLATFORM='^(packaging/|rust-toolchain\.toml$|Cargo\.lock$|crates/raul-gui/Cargo\.lock$|scripts/(check-deb-install|check-release-assets|installed-files|release-contents)|\.github/(actions/|workflows/(ci|gui|release|installer-smoke)\.yml))'
PLATFORM_CFG='cfg(_attr)?!?\(.*(windows|unix|macos|target_os|target_family)'

# select_members <metadata.json> <paths-file>: ids of the packages a change reaches, through reverse dependencies.
# A package is hit when a changed path is its directory, inside it, or contains it (a submodule pointer). Directories
# are relative to the repository root, $TOP, because the GUI workspace uses packages outside its own root.
select_members() {
  jq -r --rawfile paths "$2" --arg top "${TOP:-/r}" '
    ($paths | split("\n") | map(select(length > 0))) as $p
    | (.packages | map({id, dir: (.manifest_path | sub("/Cargo.toml$"; "") | ltrimstr($top + "/"))})) as $pk
    | [ $pk[] | select(. as $k | any($p[]; . as $x | $x == $k.dir or ($x | startswith($k.dir + "/")) or ($k.dir | startswith($x + "/")))) | .id ] as $hit
    | (reduce (.resolve.nodes[] | . as $n | $n.dependencies[] | {k: ., v: $n.id}) as $e ({}; .[$e.k] += [$e.v])) as $rev
    | {seen: ($hit | map({(.): true}) | add // {}), todo: $hit}
    | until(.todo | length == 0;
        . as $s
        | ([ $s.todo[] | ($rev[.] // [])[] | select($s.seen[.] | not) ] | unique) as $next
        | {seen: ($s.seen + ($next | map({(.): true}) | add // {})), todo: $next})
    | .seen | keys[]' "$1"
}

# member_names <metadata.json> <paths-file>: the selected workspace members' names, sorted, one per line.
member_names() {
  local ids
  ids="$(select_members "$1" "$2")"
  jq -r --arg ids "$ids" '
    ($ids | split("\n") | map(select(length > 0))) as $sel
    | . as $m
    | [ $m.workspace_members[] | select(. as $w | $sel | index($w)) ]
    | map(. as $w | $m.packages[] | select(.id == $w) | .name) | sort | .[]' "$1"
}

# members_all <metadata.json>: number of workspace members.
member_count() { jq -r '.workspace_members | length' "$1"; }

# args_for <metadata.json> <paths-file> <force:true|false>: prints "<run> <args>" for one workspace.
args_for() {
  local meta="$1" paths="$2" force="$3" names total n
  total="$(member_count "$meta")"
  if [ "$force" = true ]; then echo "true --workspace"; return; fi
  names="$(member_names "$meta" "$paths")"
  n="$(printf '%s' "$names" | grep -c . || true)"
  if [ "$n" -eq 0 ]; then echo "false "; return; fi
  if [ "$n" -ge "$total" ]; then echo "true --workspace"; return; fi
  echo "true $(printf '%s\n' "$names" | sed 's/^/-p /' | paste -sd' ' -)"
}

if [ "${1:-}" = --self-test ]; then
  fail=0
  expect() { # expect <true|false> <paths...>: touches_code
    local want="$1"; shift
    if printf '%s\n' "$@" | touches_code; then got=true; else got=false; fi
    [ "$got" = "$want" ] || { echo "FAIL expected code=$want for: $*"; fail=1; }
  }
  expect false docs/PLAN.md
  expect false README.md crates/raul-core/README.md LICENSE LICENSE-THIRD-PARTY
  expect true crates/raul-core/src/lib.rs
  expect true docs/PLAN.md Cargo.lock
  expect true components/nvmod-rs
  expect true .github/workflows/ci.yml
  expect true catalogue/setup.toml
  ign() { # ign <yes|no> <path>
    if printf '%s\n' "$2" | grep -qE "$IGNORED"; then got=yes; else got=no; fi
    [ "$got" = "$1" ] || { echo "FAIL expected ignored=$1 for: $2"; fail=1; }
  }
  ign yes site/build.py; ign yes docs/wiki/a.md; ign yes .github/workflows/pages.yml; ign yes scripts/release-contents.sh
  ign no .github/workflows/ci.yml; ign no scripts/ci/changes.sh; ign no crates/raul-core/src/lib.rs; ign no Cargo.toml
  ign no contracts/a.json; ign no scripts/ci/coverage-gate.sh
  plat() { # plat <yes|no> <path>
    if printf '%s\n' "$2" | grep -qE "$PLATFORM"; then got=yes; else got=no; fi
    [ "$got" = "$1" ] || { echo "FAIL expected platform=$1 for: $2"; fail=1; }
  }
  plat yes packaging/README.md; plat yes Cargo.lock; plat yes .github/workflows/release.yml; plat no crates/raul-core/src/lib.rs
  # The crate graph: app -> core <- util, test-only dev-dependency testkit -> core, plus a Component crate dep.
  tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
  cat >"$tmp/meta.json" <<'JSON'
{"workspace_root":"/r","workspace_members":["core","util","app","testkit"],
 "packages":[
  {"id":"core","name":"core","manifest_path":"/r/crates/core/Cargo.toml"},
  {"id":"util","name":"util","manifest_path":"/r/crates/util/Cargo.toml"},
  {"id":"app","name":"app","manifest_path":"/r/crates/app/Cargo.toml"},
  {"id":"testkit","name":"testkit","manifest_path":"/r/crates/testkit/Cargo.toml"},
  {"id":"comp","name":"comp","manifest_path":"/r/components/c-rs/crates/comp/Cargo.toml"}],
 "resolve":{"nodes":[
  {"id":"core","dependencies":["comp"]},{"id":"util","dependencies":["core"]},
  {"id":"app","dependencies":["util"]},{"id":"testkit","dependencies":["core"]},{"id":"comp","dependencies":[]}]}}
JSON
  sel() { printf '%s\n' "$@" >"$tmp/p"; args_for "$tmp/meta.json" "$tmp/p" false; }
  check() { local want="$1"; shift; local got; got="$(sel "$@")"; [ "$got" = "$want" ] || { echo "FAIL want '$want' got '$got' for: $*"; fail=1; }; }
  check "true -p app -p util" crates/app/src/main.rs crates/util/src/lib.rs
  check "true -p app" crates/app/src/main.rs
  check "true --workspace" crates/core/src/lib.rs
  check "true --workspace" components/c-rs
  check "false " crates/unknown/readme.txt
  [ "$(args_for "$tmp/meta.json" "$tmp/p" true)" = "true --workspace" ] || { echo "FAIL force"; fail=1; }
  [ "$fail" = 0 ] && echo "changes.sh self-test passed"
  exit "$fail"
fi

TOP="$(git rev-parse --show-toplevel)"
out="${GITHUB_OUTPUT:-/dev/stdout}"
event="${GITHUB_EVENT_NAME:-}"
head="${HEAD:-HEAD}"

# A leg of the platform matrix, for the workspaces that run.
matrix() { # matrix <run_umbrella> <umbrella_args> <run_gui> <gui_args>
  local legs=() os
  for os in windows-latest macos-latest; do
    if [ "$1" = true ]; then
      legs+=("$(jq -cn --arg os "$os" --arg args "$2" '{os:$os,workspace:"umbrella",dir:".",root:".",suffix:"",args:$args}')")
    fi
    if [ "$3" = true ]; then
      legs+=("$(jq -cn --arg os "$os" --arg args "$4" '{os:$os,workspace:"gui",dir:"crates/raul-gui",root:"../..",suffix:"-gui",args:$args}')")
    fi
  done
  if [ "${#legs[@]}" -eq 0 ]; then echo ""; else printf '%s\n' "${legs[@]}" | jq -sc '{include: .}'; fi
}

emit() { # emit <code> <umbrella> <umbrella_args> <gui> <gui_args> <all_os>
  local code="$1" u="$2" ua="$3" g="$4" ga="$5" aos="$6" plat skips=()
  plat=""
  if [ "$aos" = true ]; then plat="$(matrix "$u" "$ua" "$g" "$ga")"; fi
  [ "$u" = true ] || skips+=(lint test-linux)
  [ "$g" = true ] || skips+=(gui)
  [ -n "$plat" ] || skips+=(platforms)
  {
    echo "code=$code"
    echo "umbrella=$u"; echo "umbrella_args=$ua"
    echo "gui=$g"; echo "gui_args=$ga"
    echo "platforms=$plat"
    echo "allowed_skips=$(IFS=,; echo "${skips[*]:-}")"
  } >>"$out"
  echo "code=$code umbrella=$u ($ua) gui=$g ($ga) all-os=$aos platforms=${plat:-none}" >&2
}

if [ "$event" = push ] || [ "$event" = workflow_dispatch ] || [ "$event" = schedule ]; then
  emit true true --workspace true --workspace true; echo "a $event: building everything" >&2; exit 0
fi
if [ -z "${BASE:-}" ] || [ "$BASE" = 0000000000000000000000000000000000000000 ] || ! git cat-file -e "$BASE^{commit}" 2>/dev/null; then
  emit true true --workspace true --workspace true; echo "no base to compare with: building everything" >&2; exit 0
fi

changed="$(git diff --name-only "$BASE...$head")"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
code=false
if printf '%s\n' "$changed" | touches_code; then code=true; fi

# components/<name>.version moves with the submodule pointer: it stands for the Component.
printf '%s\n' "$changed" | sed -E 's#^(components/[^/]+)\.version$#\1#' | sort -u >"$tmp/paths"
# Everything that is neither ignored nor a crate or Component path can be read by any crate: select everything.
global=false; gui_global=false
while IFS= read -r p; do
  [ -n "$p" ] || continue
  if printf '%s\n' "$p" | grep -qE "$IGNORED"; then continue; fi
  if printf '%s\n' "$p" | grep -qE "$GUI_ONLY"; then gui_global=true; continue; fi
  case "$p" in crates/*|components/*) ;; *) global=true; echo "shared input changed: $p" >&2 ;; esac
done <"$tmp/paths"

pick() { # pick <manifest> <force> <extra cargo flags>
  jq --version >/dev/null
  # shellcheck disable=SC2086
  cargo metadata --format-version 1 --locked --manifest-path "$1" $3 >"$tmp/meta.json"
  args_for "$tmp/meta.json" "$tmp/paths" "$2"
}
read -r u ua < <(pick Cargo.toml "$global" "")
read -r g ga < <(pick crates/raul-gui/Cargo.toml "$([ "$global" = true ] || [ "$gui_global" = true ] && echo true || echo false)" --all-features)

# Platform-specific?
aos=false
if grep -qE "$PLATFORM" "$tmp/paths"; then aos=true; echo "platform-specific path changed" >&2; fi
if [ "$aos" = false ]; then
  while IFS= read -r p; do
    case "$p" in *.rs) [ -f "$p" ] && grep -qE "$PLATFORM_CFG" "$p" && { aos=true; echo "platform cfg in $p" >&2; break; } ;; esac
  done <"$tmp/paths"
fi
if [ "$aos" = false ] && [ -n "${PR_NUMBER:-}" ] && [ -n "${GH_TOKEN:-}" ]; then
  if gh api "repos/${GITHUB_REPOSITORY:?}/issues/$PR_NUMBER/labels" --jq '.[].name' | grep -qxF 'ci:all-os'; then
    aos=true; echo "label ci:all-os" >&2
  fi
fi
# Windows and macOS only run what Linux runs.
[ "$u" = true ] || [ "$g" = true ] || aos=false
emit "$code" "$u" "$ua" "$g" "$ga" "$aos"
