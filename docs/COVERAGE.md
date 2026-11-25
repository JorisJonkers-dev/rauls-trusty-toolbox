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

# Feature catalogue

`catalogue/` lists what raul itself promises, in areas, groups and items with stable ids (`setup.journal.resume`), and
says for each one whether it is implemented, missing, gated (it needs game data or real hardware that CI does not have)
or deferred. [tally](https://github.com/JorisJonkers-dev/tally) reads it; its format is tally's `docs/FORMAT.md`. The
Components under `components/` keep their own catalogues in their own repositories.

- **Tests say what they prove.** A test names the items it exercises and asserts on with
  `covers(&[ids::setup::journal::RESUME])`, from the `raul-testkit` crate (a dev-dependency only). The ids are written
  by `crates/raul-testkit/build.rs` from the catalogue, so an id that does not exist does not compile. A feature a
  test only uses as setup is not tagged, and neither is a test that is ignored or needs game data.
- **States.** *tagged*: a test that is not ignored tags it. *passing*: a tagging test passed on one system.
  *covered*: tagging tests passed, and none failed, on Linux, Windows and macOS.
- **Three systems.** The workspace tests run through `cargo nextest --profile ci` (`.config/nextest.toml`: no retries,
  JUnit with every skipped test listed) on Linux (`Pipeline Complete` and the GUI workflow), Windows and macOS
  (`platforms`). Each leaves a `junit-<system>` artifact.
- **On a pull request** `Pipeline Complete` runs `tally check`: the catalogue is valid, ids are never reused (one that
  goes away is kept with `retired`), tags name live ids inside tests, and the generator copy is the pinned tally's.
  A pull request may not edit `docs/coverage/`.
- **On main** the step ingests the three JUnit files, runs `tally check --ratchet` against the last report (the tagged
  and covered counts may not drop) and renders `docs/coverage/FEATURES.md`, `tally-report.json` and `tally.svg`.
  The `reports` job commits only those three files, as raul-progress-bot, with the skip-ci marker so the commit does
  not start another run.

Adding a promise: add the item to its group file in `catalogue/`, tag the test that proves it, and run
`tally check` (the pinned binary is downloaded in CI; the version and SHA-256 are `TALLY_VERSION` and `TALLY_SHA256`
in `.github/workflows/ci.yml`). Bumping tally: also run `tally ids --generator catalogue/tally_ids_gen.rs`.
