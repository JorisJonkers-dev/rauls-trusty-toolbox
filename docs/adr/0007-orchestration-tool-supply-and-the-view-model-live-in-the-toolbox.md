---
status: accepted, partly superseded by ADR-0016
date: 2026-11-16
---
# Orchestration, tool supply, the Runner and the view model live in the Toolbox

Superseded in part by [ADR-0016](0016-components-are-incorporated-libraries-called-in-process.md): the Runner no longer starts Components and the GUI no longer writes through a `raul` subprocess. Only the game and Windows-only tools are started as processes.

The Toolbox contains the tools. Everything that takes a player from a game copy to a running modded game, and keeps
it working, moves from nvmod-rs into the Toolbox: the setup pipeline with its journal and resume, the drivers of the
process Components (TTW, LOD, the engine's install, check and launch), the subprocess Runner, tool supply (pins,
fetch, checksum and attestation checks, the content-addressed store, side-by-side install and the Rolling channel),
the updates listing, run settings, the view model and service seam of the GUI, and the GUI. nvmod-rs keeps mod
management: Instance, Profile and Mod model, install and FOMOD, staging, conflicts, plugins and Load order (with
loot-rs), backups, saves, MO2 import, Nexus, the executables registry as data, and the content Doctor.

This restates ADR-0002's reason for linking nvmod: `raul` links nvmod's mod-management libraries, not its
orchestration, which is now the Toolbox's own. It amends ADR-0003: the view model and service seam move with the GUI
instead of staying in nvmod-rs. The crate-by-crate move, the Toolbox's internal crate graph and the order of the
moves are in [REORG.md](../REORG.md) sections 3 and 10.

## Consequences

- A Component's CLI works alone and never fetches, finds or starts another Component once the moves are done.
- nvmod-rs starts no subprocess after the move; its test that forbids `std::process::Command` runs with no exception.
- Only `raul-runner` may use `std::process::Command` in the Toolbox; a dependency-graph test enforces the crate graph.
- The GUI reads in process and writes through a `raul` subprocess; every write is observed only through the
  `raul-events` stream and the journal, and the read model reloads from disk after an event that changes state.
- nvmod's moved commands are removed when they move; only `raul` has them afterwards.

## Considered options

- Keep orchestration in nvmod-rs and keep the Toolbox thin: rejected, it left one Component owning the product.
- A new orchestration repository between nvmod and the Toolbox: rejected, the Toolbox already is that layer.
