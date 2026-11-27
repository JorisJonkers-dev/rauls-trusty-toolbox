<!-- SPDX-License-Identifier: GPL-3.0-only -->
# Benchmarks

The [benchmark pages](https://jorisjonkers-dev.github.io/rauls-trusty-toolbox/bench/) show each project on its own, with bars
set beside the tool it replaces: nv-rs against the original game, loot-rs against LOOT and libloot, ttwright-rs against the
official TTW installer, lodwright-rs against xLODGen and DynDOLOD, nvmod-rs against Mod Organizer 2, and raul's own
`raul bench` history.

The rule is simple: **a bar is a number somebody measured.** Every card says on which machine, with which command and at
which commit. A comparison nobody has measured is drawn as "not measured yet", with an issue that says what to measure.

## The record format

One JSON file is one recorded run of one project (`toolbox-bench/1`). Many files of one project make its history.

```json
{
  "schema": "toolbox-bench/1",
  "project": "loot-rs",
  "run": {
    "id": "sort-vs-libloot", "date": "2026-11-15", "commit": "c7c42f2", "version": "loot-rs 0.x, libloot 0.29.7",
    "machine": "16 threads, release build", "method": "what was run, how many times, what the number is (median of 5)",
    "source": "https://github.com/.../docs/RESOURCE-LIMITS.md"
  },
  "groups": [{
    "id": "sort-time", "title": "Sorting a load order", "metric": "Wall time of one sort", "unit": "ms", "better": "lower",
    "series": [
      {"id": "ours", "label": "loot-rs", "role": "ours"},
      {"id": "libloot", "label": "libloot 0.29.7", "role": "against", "tool": "libloot", "version": "0.29.7"},
      {"id": "other", "label": "Some tool", "role": "against", "status": "not-measured", "note": "why there is no number"}
    ],
    "cases": [{"name": "133 plugins", "values": {"ours": 29, "libloot": 69}, "flags": {"ours": "capped"}}]
  }],
  "pending": [{"tool": "LOOT application", "what": "what to measure", "issue": "https://github.com/.../issues/1"}]
}
```

- `better` is `lower` or `higher`; the page says it on every chart.
- A series with `"role": "ours"` is the project's own number; `"against"` is the tool it replaces. A series with
  `"status": "not-measured"` and no values is drawn as an empty dashed bar.
- A case with a missing value for an `against` series that is not marked `not-measured` simply shows no bar.
- `flags` (`capped`, `noisy`) stripe a bar that is not a headline number.
- Groups with the same `id` in several records of one project form a **history line** (one line per case, each against its
  own median). A group can also carry its own `history` list of `{date, commit, values: {case: {series: value}}}`.
- The build stops with an error when a record does not match the format.

## Where results come from

| Source | How |
| --- | --- |
| A recorded run | A maintainer puts a record in `docs/bench/<project>/` in this repository, by pull request. `site/bench_from_suite.py` turns the `suite.json` of an `nv-rs-scene bench` run into a record. |
| A component's CI | The job writes a record and pushes it to the `gh-pages` branch as `bench/results/<project>/<date>-<sha>.json` (the Toolbox's Bench workflow does this for `raul bench`). The next site build reads that folder. |
| `raul bench` | Every push to main also writes the github-action-benchmark history. Its raw chart page stays at `/bench/raw/`; the site draws the same history on the Raul's Toolbox page. |

Never put game data in a record. Say what the machine was, and leave a value out rather than estimate it.
