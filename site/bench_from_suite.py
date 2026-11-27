#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Turns the `suite.json` of an `nv-rs-scene bench` run into a toolbox-bench/1 record (see the wiki page Benchmarks).

    python3 site/bench_from_suite.py suite.json --date 2026-11-27 --machine "..." [--commit SHA] > record.json

Nothing is added to the numbers: the original game's series is written as "not measured" until someone measures it.
A scenario the tool marked CAPPED (its fps within 2% of the cap) is flagged, because that fps is not a headline number.
"""
import argparse
import json
import sys


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("suite")
    ap.add_argument("--date", required=True)
    ap.add_argument("--machine", required=True)
    ap.add_argument("--commit")
    ap.add_argument("--version")
    ap.add_argument("--source", default="")
    ap.add_argument("--id", default="quick-suite")
    a = ap.parse_args()
    suite = json.load(open(a.suite))
    rows = suite["rows"]
    if not rows:
        sys.exit("the suite has no rows")
    first = rows[0]
    orig = lambda what: {"id": "original", "label": "Original game (FalloutNV.exe)", "role": "against", "tool": "FalloutNV.exe",
                         "status": "not-measured", "note": what}
    note = "No run of the original executable on these places exists yet."
    cases_fps, cases_ms = [], []
    for r in rows:
        name = r["scenario"].split(":", 1)[-1]
        flags = {"avg": "capped", "low": "capped"} if r.get("capped") else {}
        if r.get("noisy"):
            flags = {"avg": "noisy", "low": "noisy"}
        cases_fps.append({"name": name, "values": {"avg": r["fps"], "low": r["low1_fps"]}, "flags": flags})
        cases_ms.append({"name": name, "values": {"p50": r["p50_ms"], "p99": r["p99_ms"]}})
    out = {
        "schema": "toolbox-bench/1",
        "project": "nv-rs",
        "run": {
            "id": a.id, "date": a.date, "commit": a.commit, "version": a.version, "machine": a.machine, "source": a.source,
            "method": f"nv-rs-scene bench, suite {suite['name'].split('-')[0]}: one engine start visits each place on the real clock "
                      f"under the exclusive game lock, warm-up then the median of five passes; mode {first['mode']}, {first['window']}. "
                      "A place marked CAPPED measured within 2% of the frame cap, so its fps says nothing about the engine's speed.",
        },
        "groups": [
            {"id": "fps", "title": "Frame rate per place", "metric": "Average frames per second and the 1% low", "unit": "fps", "better": "higher",
             "series": [{"id": "avg", "label": "nv-rs average", "role": "ours"}, {"id": "low", "label": "nv-rs 1% low", "role": "ours"}, orig(note)],
             "cases": cases_fps},
            {"id": "frame-time", "title": "Frame time per place", "metric": "Median (p50) and 99th percentile frame time", "unit": "ms", "better": "lower",
             "series": [{"id": "p50", "label": "nv-rs p50", "role": "ours"}, {"id": "p99", "label": "nv-rs p99", "role": "ours"}, orig(note)],
             "cases": cases_ms},
        ],
        "pending": [],
    }
    json.dump(out, sys.stdout, indent=2)
    print()


if __name__ == "__main__":
    main()
