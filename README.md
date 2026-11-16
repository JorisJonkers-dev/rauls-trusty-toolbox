# Raul's tRusty Toolbox

> *Modding New Vegas without the fRUSTration.*

Raul's tRusty Toolbox (`raul`) is one all-Rust installer, setup wizard and mod manager for
Fallout: New Vegas, running natively on Linux, Windows and macOS. It finds your game, sets up
Tale of Two Wastelands if you want it, installs and sorts your mods, builds distant LOD and
launches the game through the nv-rs engine port, all from one window or one command line.

**Status: planning.** Nothing here builds yet. The findings, the proposed architecture, the
open questions and the migration plan are in [`docs/PLAN.md`](docs/PLAN.md), which is under
review before any code lands.

## Components

The toolbox ties together four projects that keep their own history, issues, releases and CI:

| Component | What it does |
| --- | --- |
| [nv-rs](https://github.com/JorisJonkers-dev/nv-rs) | The engine port (this organisation's fork of [slaterain/nv-rs](https://github.com/slaterain/nv-rs)) |
| [nvmod-rs](https://github.com/JorisJonkers-dev/nvmod-rs) | Mod management: instances, profiles, installs, archives, TTW setup, Doctor |
| [loot-rs](https://github.com/JorisJonkers-dev/loot-rs) | Load-order sorting against the LOOT masterlist |
| [lodwright-rs](https://github.com/JorisJonkers-dev/lodwright-rs) | Distant terrain, object and tree LOD generation |

## Origins

The engine is [nv-rs](https://github.com/slaterain/nv-rs) by slaterain, the Rust port of
Fallout: New Vegas; this project uses the JorisJonkers-dev fork of it and keeps its licence,
attribution and legal notice. Load-order sorting follows the rules and masterlist of the
[LOOT](https://loot.github.io/) project, which loot-rs reads at run time and does not
redistribute.

## Legal

You need your own, legally obtained copy of Fallout: New Vegas (and of Fallout 3 for Tale of Two
Wastelands). This project ships and downloads no game files and no game assets. It is an
independent project, not affiliated with or endorsed by Bethesda Softworks, ZeniMax Media or
Obsidian Entertainment; Fallout and related names are trademarks of their respective owners.

## Licence

Not chosen yet; see the licence section of [`docs/PLAN.md`](docs/PLAN.md). Until a `LICENSE`
file is added, no licence is granted.
