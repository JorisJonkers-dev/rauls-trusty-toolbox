#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Builds the single self-contained index.html from src/.
set -euo pipefail
cd "$(dirname "$0")"
{
  printf '<!doctype html>\n<html lang="en">\n<head>\n<meta charset="utf-8">\n<meta name="viewport" content="width=device-width, initial-scale=1">\n<title>Raul'"'"'s Toolbox</title>\n<style>\n'
  cat src/style.css
  printf '</style>\n</head>\n<body>\n'
  cat src/body.html
  printf '<script>\n"use strict";\n'
  for f in icons data core art pages wizard overlays actions; do cat "src/$f.js"; printf '\n'; done
  printf '</script>\n</body>\n</html>\n'
} > index.html
echo "built index.html ($(wc -c < index.html) bytes)"
