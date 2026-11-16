---
status: accepted
date: 2026-11-16
---
# One bench front door: `raul bench`, with in-game results from the engine's harness

`raul bench` is the one place results are reported and kept: the synthetic suite, Component micro-benchmarks and
in-game results. It never starts the game. In-game numbers come from the nv-rs fork's harness, which runs only under
the machine-wide bench lock, in a nested headless session with the VRAM watchdog on, and writes a versioned bench
summary Contract; `raul bench ingest` reads that file into the same report and history.

The in-game harness stays in nv-rs, where its scenarios and drivers live, and is ported from Python to a fork-owned
Rust crate once its outputs have a versioned Contract. Until then it is the one recorded exception to the Rust-native
rule for release gates.

## Consequences

- One machine-wide lock serves every in-game measurement; Release tests take it first, then the performance work.
- Every gate run records its lock time.
