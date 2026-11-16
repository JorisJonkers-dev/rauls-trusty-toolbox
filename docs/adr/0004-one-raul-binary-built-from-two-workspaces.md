---
status: accepted
date: 2026-11-16
---
# One `raul` binary, built from two workspaces

Amended by [ADR-0012](0012-egui-is-the-gui-toolkit.md): with egui instead of GPUI, the reason for a separate GUI
workspace is re-measured in #13; one `raul` binary stays.

Users get one program: `raul` with no arguments opens the GUI, `raul <command>` is the CLI. A cargo feature alone
would put GPUI's roughly 560 crates into the main `Cargo.lock` even when off (Cargo locks optional dependencies), so
the main workspace builds a CLI-only `raul` (lean lock, fast pull-request CI, benches, headless use) and a nested GUI
workspace with its own lock file and `cargo deny` allow-list builds the release `raul`, which is the same CLI plus the
GUI. A test fails when the two builds do not expose the same commands.

## Considered options

- One workspace with a `gui` feature: rejected, every GPUI snapshot bump would reach every pull request.
- Two binaries (`raul`, `raul-gui`): rejected by the owner in favour of one thing to install and sign.
