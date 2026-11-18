---
status: accepted, partly superseded by ADR-0016 and ADR-0019
date: 2026-11-16
---
# The GUI lives in the Toolbox, on GPUI, and calls the core in process for reads and as a subprocess for writes

Superseded in part by [ADR-0016](0016-components-are-incorporated-libraries-called-in-process.md): long writes no longer run as a `raul` subprocess; the GUI calls the in-process job interface.

Superseded in part by [ADR-0019](0019-raul-orchestrates-nvmod-manages-mods.md): nvmod keeps no view model and no GUI; `nvmod-ui-model`, `nvmod-ui-service` and nvmod's `gui/` are deleted.

Amended by [ADR-0007](0007-orchestration-tool-supply-and-the-view-model-live-in-the-toolbox.md): the view model and
service seam move here too. Its toolkit line is replaced by [ADR-0012](0012-egui-is-the-gui-toolkit.md): egui, not GPUI.

nvmod-rs's `gui/` (the GPUI binary, the egui canary and the toolkit-free shell) moves to the Toolbox; nvmod-rs keeps
`nvmod-ui-model` and `nvmod-ui-service`. GPUI stays the product toolkit, with the egui canary painting the same view.
Reads (detection, load order view, conflicts) go through `raul-core` in process; long or risky writes (mod installs,
TTW, LOD, applying a sort) run as a `raul` subprocess that streams progress events, so a crash or out-of-memory in
an install never kills the window. This replaces, for the Toolbox's GUI, the "GUI shells out to the nvmod CLI with
`--json`" row in nvmod-rs `docs/DECISIONS.md` (2026-11-16).

## Consequences

- The GUI smoke test runs GPUI under xvfb with Mesa lavapipe on Linux; if lavapipe cannot start it, the smoke test
  runs the egui canary and the GPUI build is compile-only there.
- On-screen text is written in Raul's voice throughout the GUI; every error also carries a plain detail line. The
  CLI, `--json` and logs stay plain.
