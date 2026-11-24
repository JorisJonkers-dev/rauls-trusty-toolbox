# Installing raul

The [README](../README.md) is the short version. This page has which file to take, how to check it, and how to build
raul from source.

## Status

**Early releases (0.x).** The setup wizard, Tale of Two Wastelands, Mods, Load order, LOD and Play work on Linux,
Windows and macOS. The installers are built and smoke-tested for all three systems; Windows and macOS have not yet had
a full run on real hardware, their packages are unsigned, and the icon is a placeholder. The plan is in
[PLAN.md](PLAN.md) and [REORG.md](REORG.md), the design in [ARCHITECTURE.md](ARCHITECTURE.md), decisions in
[adr](adr) and terms in [GLOSSARY.md](../GLOSSARY.md).

**Known issue in v0.6.0:** the distant land that raul generated had no far water; lakes and rivers such as Lake Mead
and the Colorado were not drawn at a distance. The generator that raul builds in now draws them (lodwright 0.15.1). If
you made LOD with v0.6.0, install a newer release and generate the LOD again.

## Which file, and how to check it

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

## From source

```sh
git clone --recursive https://github.com/JorisJonkers-dev/rauls-trusty-toolbox.git
cd rauls-trusty-toolbox
cargo run -p raul-cli -- --version
```

The toolchain (Rust 1.100.0) comes from `rust-toolchain.toml`. Building the bundled ttwright-rs needs `make`, a C
compiler and `pkg-config`.

## Legal

You need your own, legally obtained copy of Fallout: New Vegas (and of Fallout 3 for Tale of Two
Wastelands). This project ships and downloads no game files and no game assets. It is an
independent project, not affiliated with or endorsed by Bethesda Softworks, ZeniMax Media or
Obsidian Entertainment; Fallout and related names are trademarks of their respective owners.
The pictures of the game in `docs/images` are screenshots taken with the engine for the README.

raul is GPL-3.0-only (see [LICENSE](../LICENSE)), because it links nvmod-rs and loot-rs, which are GPL-3.0-only
([ADR-0002](adr/0002-raul-links-nvmod-and-loot-rs-and-runs-the-rest-as-processes.md)).
