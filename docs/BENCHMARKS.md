# Benchmarks

`raul bench` runs a standard suite on synthetic fixtures built by nvmod's test kit. No game data is read,
downloaded or kept.

| Case | What it measures |
| --- | --- |
| `sort/small`, `sort/medium`, `sort/ttw-sized` | loot-rs sorting 60, 300 and 1000 synthetic plugins with an empty masterlist |
| `plugin-headers/ttw-sized` | reading 1000 synthetic plugin files |
| `install/zip` | nvmod `init`, `profile create` and `install` of an 8 MiB synthetic zip |
| `lod/generate` | skipped until a synthetic lodwright plan fixture exists |
| `frame-time/camera-path` | skipped until nv-rs has a scripted camera path and a frame-time log |

Each case reports the median, minimum and maximum of its timed runs, and a throughput where it has one.

## Run it locally

```sh
cargo run --release -p raul-cli -- bench --out target/bench        # 15 runs per case
cargo run --release -p raul-cli -- bench --quick --out target/bench # 3 runs per case
```

It writes `bench.json` (the report) and `bench.md` (the same as a table).

## Compare two branches

Run both on the same machine, one after the other, with nothing else heavy running:

```sh
git switch main && cargo run --release -p raul-cli -- bench --out /tmp/bench-main
git switch my-branch && cargo run --release -p raul-cli -- bench --out /tmp/bench-branch
cargo run --release -p raul-cli -- bench compare /tmp/bench-main/bench.json /tmp/bench-branch/bench.json --threshold 10
```

A case slower than the threshold is printed as a warning. The command never fails because of one.

## In CI

- Add the `bench` label to a pull request when it is opened, or later. The Bench workflow runs main and the pull
  request in one job and comments the comparison.
- Every push to main adds a run to the history on the `gh-pages` branch (`bench/`), through
  github-action-benchmark, with an alert threshold of 125%. Alerts do not fail anything.

Shared runners are noisy: treat a single slower run as a reason to rerun, not as a regression. A dedicated runner
would allow hard thresholds; none is configured.

Micro-benchmarks inside the Components (plugin parsing, BSA reads, LOD steps) are proposed as Component pull
requests and wait for the agreed order (docs/PLAN.md, section 8).
