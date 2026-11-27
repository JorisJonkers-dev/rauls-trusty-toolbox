#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Decides whether a CI run has to build and test the Rust workspaces. Writes `code=true` or `code=false` to
# $GITHUB_OUTPUT (stdout outside Actions). A push, a run with no base revision, or any changed file outside
# documentation builds; a pull request that only changes documentation (docs/, Markdown, LICENSE*) does not.
# The checks that read documentation (public text, catalogue) run either way.
# Usage: BASE=<sha> HEAD=<sha> scripts/ci/changes.sh | scripts/ci/changes.sh --self-test
set -euo pipefail

# Reads changed paths on stdin; succeeds when one of them is not documentation.
touches_code() { grep -qvE '^(docs/|LICENSE[^/]*$|[^/]*\.md$|.*/[^/]*\.md$)'; }

if [ "${1:-}" = --self-test ]; then
  fail=0
  expect() { # expect <true|false> <paths...>
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
  [ "$fail" = 0 ] && echo "changes.sh self-test passed"
  exit "$fail"
fi

out="${GITHUB_OUTPUT:-/dev/stdout}"
if [ "${GITHUB_EVENT_NAME:-}" = push ]; then
  echo "code=true" >>"$out"; echo "a push: building"; exit 0
fi
if [ -z "${BASE:-}" ] || [ "$BASE" = 0000000000000000000000000000000000000000 ] || ! git cat-file -e "$BASE^{commit}" 2>/dev/null; then
  echo "code=true" >>"$out"; echo "no base to compare with: building"; exit 0
fi
if git diff --name-only "$BASE...${HEAD:-HEAD}" | touches_code; then
  echo "code=true" >>"$out"; echo "code changed: building"
else
  echo "code=false" >>"$out"; echo "only documentation changed: the Rust build and test jobs are skipped"
fi
