---
status: accepted
date: 2026-11-18
---
# raul orchestrates; nvmod manages mods; no standalone nvmod setup or GUI

`raul` orchestrates every flow and is the only UI. Setup, the TTW install, LOD generation, putting the engine in
place, the game launch and updates are driven by `raul`: it decides the steps, their order, their resume state and
what the person sees. nvmod is the mod-management Component: Mods, install and FOMOD, the Load order and its sort,
Plugins, saves, Profiles, staging, backups and import, and the library jobs `raul` calls for those. nvmod has no
standalone setup: no `nvmod setup` command, no setup wizard, no code that runs its jobs one after another, and no GUI.
Its jobs are building blocks; composing them is `raul`'s.

Every Component is linked as crates and called in `raul`'s process; only the game run is a separate process, started
from `raul`'s own binary in a play mode ([ADR-0016](0016-components-are-incorporated-libraries-called-in-process.md)
as amended). TTW is `raul`'s, through ttwright linked in process; nvmod has no TTW step, job or installer adapter.

Decided by the owner on 2026-11-18.

## What this supersedes

- [ADR-0002](0002-raul-links-nvmod-and-loot-rs-and-runs-the-rest-as-processes.md): its reason for linking nvmod,
  "nvmod already holds the orchestration", no longer holds. `raul` links nvmod for mod management only.
- [ADR-0003](0003-the-gui-lives-here-and-calls-the-core-two-ways.md): nvmod does not keep `nvmod-ui-model` or
  `nvmod-ui-service`. Its `gui/` and both view-model crates are frozen and deleted (nvmod-rs#167); `raul-ui-model`
  and `raul-ui-service` are the only copies.
- [ADR-0016](0016-components-are-incorporated-libraries-called-in-process.md), the row "nvmod: its own CLI, for
  standalone use": nvmod's own CLI keeps the mod-management commands only. `setup`, `ttw`, `lod`, `install-nv-rs` and
  the launch of `run` leave it.
- [REORG.md](../REORG.md) section 3 and section 10.2, and section 12 decision 5: these commands and crates are not
  copied here with their history and then removed. `raul` already has its own drivers, view model and GUI, and composes
  the setup from nvmod's typed jobs (#43); nvmod's copies are deleted, with no stub, as soon as `raul`'s replacement
  has landed.

## Consequences

- The setup's plan, its steps and its resume journal (`raul-journal`) are `raul`'s (#272, #273, #274). An unfinished
  nvmod setup is taken over once.
- The engine step is `raul`'s until the play mode removes the separate engine program (#275, #260).
- The GUI calls nvmod's typed jobs (#269), not a command line in process; actions with no job yet get one in nvmod
  (nvmod-rs#170).
- `raul-runner` is the only Runner (#40). nvmod starts no process once TTW, LOD, the engine install and the launch
  are removed and archives are read in process (nvmod-rs#169).
- nvmod loses commands that work on their own today. A person who wants a setup, TTW or LOD without the GUI uses
  `raul` on the command line.

## Considered options

- Finish a setup job in nvmod and have `raul` call it (the earlier plan, nvmod-rs#152 and #153): rejected. It keeps
  two owners of the step list and of the resume state, and the GUI would mirror nvmod's event shapes by hand.
- Keep nvmod's standalone `setup` beside `raul setup`: rejected for the same drift, and because a Component does one
  job.
