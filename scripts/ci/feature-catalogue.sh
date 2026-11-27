#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# The feature catalogue (catalogue/, docs/COVERAGE.md "Feature catalogue"), run from the repository root by the
# `catalogue` job of ci.yml.
#
# On a pull request: the catalogue is valid (ids, tags, the generator copy) and, against the base revision, no id was
# removed without `retired` or an alias. A pull request may not carry the generated reports under docs/coverage/.
# On a push to main, with the JUnit results of all three systems in $JUNIT: an item is covered when its tagged tests
# passed on each, and the ratchet holds (the covered and tagged counts do not drop below the base revision's report).
# The reports are rendered in a clean worktree at $RUNNER_TEMP/render; when they changed, `reports=changed` goes to
# $GITHUB_OUTPUT for the `reports` job.
#
# tally is a pinned release binary checked against its SHA-256; it is not a crate dependency.
# Environment: TALLY_VERSION, TALLY_SHA256, REPORT_FILES, BASE, JUNIT, and the GitHub Actions defaults.
set -euo pipefail
: "${TALLY_VERSION:?}" "${TALLY_SHA256:?}" "${REPORT_FILES:?}" "${RUNNER_TEMP:?}" "${GITHUB_EVENT_NAME:?}"
summary="${GITHUB_STEP_SUMMARY:-/dev/stdout}"
output="${GITHUB_OUTPUT:-/dev/null}"

# The files the `reports` job commits on main must not be ignored, or its `git add` fails there.
# shellcheck disable=SC2086
if ignored="$(git check-ignore --no-index -v $REPORT_FILES)"; then
  echo "::error title=Feature catalogue::report files are ignored by .gitignore: $ignored"; exit 1
fi

tally="$RUNNER_TEMP/tally"
curl -fsSL --retry 3 --retry-all-errors -o "$tally" \
  "https://github.com/JorisJonkers-dev/tally/releases/download/${TALLY_VERSION}/tally-x86_64-unknown-linux-gnu"
echo "${TALLY_SHA256}  $tally" | sha256sum -c -
chmod +x "$tally"

base_ok=false
if [ -n "${BASE:-}" ] && [ "$BASE" != 0000000000000000000000000000000000000000 ] && git cat-file -e "${BASE}^{commit}" 2>/dev/null; then
  base_ok=true
fi
# Pull requests never carry the generated reports, so none can lower the bar the ratchet measures against.
# The first catalogue pull request brings the first copy (the base has none yet).
if [ "$GITHUB_EVENT_NAME" = pull_request ] && $base_ok && git cat-file -e "${BASE}:docs/coverage/tally-report.json" 2>/dev/null \
   && git diff --name-only "$BASE" HEAD -- docs/coverage | grep -q .; then
  echo "::error title=Feature catalogue::docs/coverage/ is generated on main; leave it out of the pull request"; exit 1
fi
if $base_ok; then
  "$tally" check --base "$BASE"
elif [ "$GITHUB_EVENT_NAME" = pull_request ]; then
  echo "::error title=Feature catalogue::the base revision $BASE is not in the checkout"; exit 1
else
  "$tally" check
fi
[ "$GITHUB_EVENT_NAME" = push ] || exit 0

# On main: the results of all three systems are needed, and the ratchet holds against the base report.
: "${JUNIT:?}"
for os in linux windows macos; do
  [ -f "$JUNIT/junit-$os.xml" ] || { echo "::error title=Feature catalogue::the JUnit results of $os are missing"; exit 1; }
done
first=true
for os in linux windows macos; do
  files=(--junit "$JUNIT/junit-$os.xml")
  [ -f "$JUNIT/junit-$os-gui.xml" ] && files+=(--junit "$JUNIT/junit-$os-gui.xml")
  if $first; then "$tally" ingest --os "$os" "${files[@]}" --out "$RUNNER_TEMP/results.json"; first=false
  else "$tally" ingest --os "$os" "${files[@]}" --out "$RUNNER_TEMP/results.json" --append; fi
done
args=(--results "$RUNNER_TEMP/results.json")
ratchet=()
if $base_ok && git show "${BASE}:docs/coverage/tally-report.json" >"$RUNNER_TEMP/base-report.json" 2>/dev/null; then
  ratchet=(--ratchet "$RUNNER_TEMP/base-report.json")
fi
"$tally" check "${args[@]}" "${ratchet[@]}"
"$tally" report --depth 2 --format md "${args[@]}" >>"$summary"

# Rendered in a clean worktree of the tested commit, so nothing else in the checkout is picked up.
git worktree add --quiet --detach "$RUNNER_TEMP/render" HEAD
cd "$RUNNER_TEMP/render"
mkdir -p docs/coverage
{
  echo "# Feature coverage"
  echo
  echo "Generated on main by tally from \`catalogue/\` (docs/COVERAGE.md, \"Feature catalogue\"). Do not edit."
  echo
  "$tally" report --depth 3 --format md "${args[@]}"
} >docs/coverage/FEATURES.md
"$tally" report --depth 3 --format json "${args[@]}" --out docs/coverage/tally-report.json
"$tally" report --depth 1 --format svg "${args[@]}" --out docs/coverage/tally.svg

# Only the report files may change; anything else stops the run.
# shellcheck disable=SC2086
printf '%s\n' $REPORT_FILES >"$RUNNER_TEMP/allowed"
git status --porcelain --untracked-files=all | cut -c4- >"$RUNNER_TEMP/changed"
if [ ! -s "$RUNNER_TEMP/changed" ]; then
  echo "The catalogue reports are up to date: nothing to commit." >>"$summary"
  exit 0
fi
other="$(grep -vxF -f "$RUNNER_TEMP/allowed" "$RUNNER_TEMP/changed" || true)"
if [ -n "$other" ]; then
  echo "::error title=Feature catalogue::files other than the reports changed: $other"; exit 1
fi
echo "reports=changed" >>"$output"
{ echo "Catalogue reports changed:"; sed 's/^/- /' "$RUNNER_TEMP/changed"; } >>"$summary"
