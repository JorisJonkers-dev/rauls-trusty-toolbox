# Raul's tRusty Toolbox

> *Modding New Vegas without the fRUSTration.*

Raul's tRusty Toolbox (`raul`) is one all-Rust installer, setup wizard and mod manager for
Fallout: New Vegas, running natively on Linux, Windows and macOS. It finds your game, sets up
Tale of Two Wastelands if you want it, installs and sorts your mods, builds distant LOD and
launches the game through the nv-rs engine port, all from one window or one command line.

**Status: early releases (0.x).** One window and the same commands on the command line: the setup wizard, Tale of
Two Wastelands, Mods, Load order, distant LOD and Play, on Linux, Windows and macOS. The installers are built and
smoke-tested for all three systems; Windows and macOS have not yet had a full run on real hardware. The Windows and macOS
packages are unsigned, the icon is a placeholder, and the generated LOD is packed into one archive by default (the
LOD settings can turn that off). The plan is in [`docs/PLAN.md`](docs/PLAN.md) and the reorganisation that refines it in [`docs/REORG.md`](docs/REORG.md), the design in [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md), decisions in
[`docs/adr`](docs/adr) and terms in [`GLOSSARY.md`](GLOSSARY.md).

<!-- Screenshots go here. -->

**Known issue in v0.6.0:** the distant land that raul generated had no far water; lakes and rivers such as Lake Mead
and the Colorado were not drawn at a distance. The generator that raul builds in now draws them (lodwright 0.15.1). If
you made LOD with v0.6.0, install a newer release and generate the LOD again.

## Install

Download the package for your system from the
[latest release](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/releases/latest) and check it against
`SHA256SUMS`: `sha256sum -c SHA256SUMS --ignore-missing` in the download folder (Linux), or compare
`shasum -a 256 <file>` (macOS) or `Get-FileHash <file>` (Windows PowerShell) with the matching line of that file.
That needs nothing else installed.

To also check where the file was built, use `gh attestation verify <file> --repo JorisJonkers-dev/rauls-trusty-toolbox`.
It needs GitHub CLI 2.49 or newer (the `gh` package of many distributions is older and has no `attestation` command) and
a GitHub login (`gh auth login`), because GitHub asks for one even for a public repository.

| System | Package | Notes |
| --- | --- | --- |
| Linux (x64, glibc 2.35+, Steam Deck) | `.AppImage`, `.deb` or `.tar.gz` | `chmod +x` the AppImage and run it; or `sudo apt install ./raul-*.deb` (package `raul`, its libraries come with it) |
| Windows 10 22H2 / 11 (x64) | `-setup.exe` or `-portable.zip` | installs for your user only; unsigned for now, so SmartScreen may warn |
| macOS 13+ (Apple silicon and Intel) | `.dmg` | unsigned until signing is set up: allow it under System Settings, Privacy & Security |

The `.deb` pulls in the libraries the window needs. The AppImage and the `.tar.gz` do not carry them: on a plain
Debian or Ubuntu install `libx11-6 libx11-xcb1 libxcb1 libxcursor1 libxi6 libxrandr2 libxrender1 libxkbcommon0
libxkbcommon-x11-0 libwayland-client0 libwayland-egl1 libegl1 libgl1 libvulkan1 libpulse0 mesa-vulkan-drivers` first
(other distributions: the same libraries under their own names). To play, the engine's shader compiler `libvkd3d-shader1`
is also needed where your distribution has it.

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
