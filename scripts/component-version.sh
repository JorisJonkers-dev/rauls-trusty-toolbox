#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Print the release tag (vX.Y.Z) of a Component as its submodule is pinned: its release-please manifest says the
# version of the commit checked out under components/. Usage: scripts/component-version.sh <nv-rs|nvmod-rs|lodwright-rs|ttwright-rs|loot-rs>
set -euo pipefail
dir="components/${1:?usage: scripts/component-version.sh <component>}"
jq -er '.["."] | "v" + .' "$dir/.release-please-manifest.json"
