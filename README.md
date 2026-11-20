# Raul's tRusty Toolbox

> *Modding New Vegas without the fRUSTration.*

Raul's tRusty Toolbox (`raul`) is one all-Rust installer, setup wizard and mod manager for
Fallout: New Vegas, running natively on Linux, Windows and macOS. It finds your game, sets up
Tale of Two Wastelands if you want it, installs and sorts your mods, builds distant LOD and
launches the game through the nv-rs engine port, all from one window or one command line.

**Status: early.** `raul` runs from the command line today: it finds your game, checks the pinned
Components and runs every nvmod command. The GUI arrives when it moves here from nvmod-rs. The plan is in
[`docs/PLAN.md`](docs/PLAN.md) and the reorganisation that refines it in [`docs/REORG.md`](docs/REORG.md), the design in [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md), decisions in
[`docs/adr`](docs/adr) and terms in [`GLOSSARY.md`](GLOSSARY.md).

<!-- Screenshots go here once the GUI ships. -->

## Install

Download the package for your system from the
[latest release](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/releases/latest) and check it against
`SHA256SUMS` (or `gh attestation verify <file> --repo JorisJonkers-dev/rauls-trusty-toolbox`).

| System | Package | Notes |
| --- | --- | --- |
| Linux (x64, glibc 2.35+, Steam Deck) | `.AppImage`, `.deb` or `.tar.gz` | `chmod +x` the AppImage and run it |
| Windows 10 22H2 / 11 (x64) | `-setup.exe` or `-portable.zip` | installs for your user only; unsigned for now, so SmartScreen may warn |
| macOS 13+ (Apple silicon and Intel) | `.dmg` | unsigned until signing is set up: allow it under System Settings, Privacy & Security |

### From source

```sh
git clone --recursive https://github.com/JorisJonkers-dev/rauls-trusty-toolbox.git
cd rauls-trusty-toolbox
cargo run -p raul-cli -- --version
```

The toolchain (Rust 1.100.0) comes from `rust-toolchain.toml`. Building the bundled ttwright-rs needs `make`, a C
compiler and `pkg-config`.

## Quickstart

```sh
raul detect                                   # find your game copies
raul init --base "<game folder>" --instance ~/raul/fnv
raul profile create --instance ~/raul/fnv
raul install <mod archive> --instance ~/raul/fnv
raul sort --instance ~/raul/fnv --fetch-masterlist --apply
raul target nv-rs --instance ~/raul/fnv        # run on the engine built into raul; nothing to install
raul lod generate --instance ~/raul/fnv
raul run nv-rs --instance ~/raul/fnv
```

`raul help` lists the Toolbox's own commands, `raul nvmod-help` every nvmod command; add `--json` for scripts.

## Components

The toolbox ties together four projects that keep their own history, issues, releases and CI:

| Component | What it does |
| --- | --- |
| [nv-rs](https://github.com/JorisJonkers-dev/nv-rs) | The engine port (this organisation's fork of [slaterain/nv-rs](https://github.com/slaterain/nv-rs)) |
| [nvmod-rs](https://github.com/JorisJonkers-dev/nvmod-rs) | Mod management: instances, profiles, mod installs, staging, conflicts, load order, content Doctor |
| [loot-rs](https://github.com/JorisJonkers-dev/loot-rs) | Load-order sorting against the LOOT masterlist |
| [lodwright-rs](https://github.com/JorisJonkers-dev/lodwright-rs) | Distant terrain, object and tree LOD generation |
| [ttwright-rs](https://github.com/JorisJonkers-dev/ttwright-rs) | Tale of Two Wastelands installation from the player's own game copies |

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

GPL-3.0-only (see [`LICENSE`](LICENSE)), because `raul` links nvmod-rs and loot-rs, which are
GPL-3.0-only ([ADR-0002](docs/adr/0002-raul-links-nvmod-and-loot-rs-and-runs-the-rest-as-processes.md)).
