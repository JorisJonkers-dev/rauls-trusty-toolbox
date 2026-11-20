#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Fails when a public file of the Toolbox names the owner's private workshop repository. Decisions that came from the
# workshop are described in plain words ("the earlier workshop decision on ..."), never by its name, path or tickets.
# Usage: scripts/check-public-text.sh               checks every tracked file outside components/
#        scripts/check-public-text.sh --self-test   checks the check itself on synthetic text
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

# The name is spelled with a character class so this file does not match itself.
pattern='nv[-_ ]?lin[u]x'

if [ "${1:-}" = "--self-test" ]; then
  hit() { printf '%s\n' "$1" | grep -qiE "$pattern"; }
  hit "see the NV-Linux workshop" || { echo "self-test: the name was not flagged" >&2; exit 1; }
  hit "ADR-0009 of nv_linux" || { echo "self-test: a variant was not flagged" >&2; exit 1; }
  if hit "the private workshop and nv-rs"; then echo "self-test: plain words were flagged" >&2; exit 1; fi
  echo "self-test ok"
  exit 0
fi

found=$(git ls-files -z -- . ':!components' ':!scripts/check-public-text.sh' | xargs -0 grep -InE -i -- "$pattern" || true)
if [ -n "$found" ]; then
  echo "A public file names the private workshop; describe it in plain words instead:" >&2
  echo "$found" >&2
  exit 1
fi
echo "no public file names the private workshop"
