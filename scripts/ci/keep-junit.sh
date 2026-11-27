#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Copies the JUnit file that `cargo nextest --profile ci` (or `cargo llvm-cov nextest` with NEXTEST_PROFILE=ci)
# left under ./target to $RUNNER_TEMP/junit/<name>.xml, for the feature catalogue (docs/COVERAGE.md). Run it from the
# workspace root. A missing file is a warning: the test step already failed, or did not run.
# Usage: scripts/ci/keep-junit.sh <name>    e.g. junit-linux, junit-macos-gui
set -euo pipefail
name="${1:?usage: keep-junit.sh <name>}"
dir="${RUNNER_TEMP:?RUNNER_TEMP is not set}/junit"
mkdir -p "$dir"
junit="$(find target -path '*nextest/ci/junit.xml' 2>/dev/null | head -n 1 || true)"
if [ -n "$junit" ]; then
  cp "$junit" "$dir/$name.xml"
  echo "kept $junit as $dir/$name.xml"
else
  echo "::warning title=JUnit::no JUnit file below $(pwd)/target for $name"
fi
