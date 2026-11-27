#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-only
# Moves the github-action-benchmark chart page and its data from bench/ to bench/raw/ in a gh-pages checkout, so that
# bench/index.html can be the new overview. Nothing is deleted: the files are moved with git, history and all.
#   sh site/bench-migrate.sh <gh-pages checkout>
set -eu
cd "$1"
if [ -f bench/data.js ] && [ ! -f bench/raw/data.js ]; then
  mkdir -p bench/raw
  git mv bench/data.js bench/raw/data.js
  [ -f bench/index.html ] && git mv bench/index.html bench/raw/index.html
  echo "moved the raw benchmark page to bench/raw/"
fi
