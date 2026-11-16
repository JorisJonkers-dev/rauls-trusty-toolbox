# Changed-line coverage gate

Every pull request must run at least **70%** of the executable lines it adds or edits in the Toolbox's own crates
in a test. The step "Changed-line coverage gate (70%)" of `Pipeline Complete` enforces it, so a pull request below
the line cannot merge. The rule and the number are the estate's (nv-rs `docs/fork/COVERAGE.md`, ttwright-rs
`docs/COVERAGE.md`, which this copies).

1. `cargo llvm-cov nextest --workspace --lcov --output-path lcov.info` runs the workspace tests once, instrumented.
   The workspace is the `raul-*` crates; the Components under `components/` are their own workspaces and are
   measured in their own repositories.
2. `tools/coverage-gate` (Rust, no dependencies, its own workspace) reads the lines the change adds or edits from
   `git diff -U0 -M <base>...HEAD -- '*.rs'` and keeps those the report counts as executable.
3. Fewer than 70% of them ran: the step fails. The job summary lists each file with the line numbers no test ran.

A rename counts only its edited lines; a change with no executable line (documentation, workflows, a submodule
move) passes; without a base commit (a manual run) there is nothing to compare with and the gate passes.

Locally:

```sh
mise exec -- cargo llvm-cov nextest --workspace --locked --lcov --output-path lcov.info
cargo run --manifest-path tools/coverage-gate/Cargo.toml -- --lcov lcov.info --base origin/main --min 70
```
