#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Runs the five area scripts and the demo's own test walk. Exits non-zero if any fails.
# Needs Playwright to be resolvable from node (for example NODE_PATH pointing at a node_modules with playwright).
set -u
cd "$(dirname "$0")/.."
out="${TMPDIR:-/tmp}"
fail=0
for a in start mods tests pages shell; do
  echo "== scripts/$a.js"
  node "scripts/$a.js" > "$out/redesign-$a.out" 2>&1 && echo "pass $a" || { echo "FAIL $a (see $out/redesign-$a.out)"; tail -5 "$out/redesign-$a.out"; fail=1; }
done
echo "== ../design/demo/test/run.js"
(cd ../design/demo && node test/run.js > "$out/redesign-demo.out" 2>&1) && echo "pass demo" || { echo "FAIL demo (see $out/redesign-demo.out)"; tail -5 "$out/redesign-demo.out"; fail=1; }
exit $fail
