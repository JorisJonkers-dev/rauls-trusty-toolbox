#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# The changed-line coverage gate (docs/COVERAGE.md): 70% of the executable lines a change adds or edits must run in a
# test. Run it from the workspace root that wrote the LCOV file. Without a base revision in the checkout (a first push,
# a shallow clone) there is nothing to compare with and the gate passes.
# Usage: BASE=<sha> scripts/ci/coverage-gate.sh <lcov file>
set -euo pipefail
lcov="${1:?usage: coverage-gate.sh <lcov file>}"
summary="${GITHUB_STEP_SUMMARY:-/dev/stdout}"
if [ -z "${BASE:-}" ] || [ "$BASE" = 0000000000000000000000000000000000000000 ] || ! git cat-file -e "$BASE^{commit}" 2>/dev/null; then
  echo "No base commit to compare with: nothing to gate." | tee -a "$summary"
  exit 0
fi
top="$(git rev-parse --show-toplevel)"
cargo run --quiet --locked --manifest-path "$top/tools/coverage-gate/Cargo.toml" -- \
  --lcov "$lcov" --base "$BASE" --min 70 --summary "$summary"
