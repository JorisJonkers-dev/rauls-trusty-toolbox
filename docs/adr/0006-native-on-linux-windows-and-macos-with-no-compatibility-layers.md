---
status: accepted
date: 2026-11-16
---
# Native on Linux, Windows and macOS, with no compatibility layers

The Toolbox and nv-rs run natively on all three: macOS 13 or later as a universal build (arm64 and x86_64), Windows
10 22H2 or 11 on x64, and Linux with glibc 2.35 or later on Wayland and X11 (Steam Deck included). The Toolbox never
starts the game through Proton, Wine or CrossOver. Until nv-rs runs natively on a platform, Launch is disabled there
with a plain message; setup, mods, sorting and LOD still work. Proton prefixes remain a place to find the player's
game files, nothing more. This supersedes nv-linux ADR-0009's "macOS is not supported".

## Consequences

- Native macOS is work in nv-rs (Metal through wgpu, platform layer, audio, paths), proposed there as fork pull requests.
- Releases ship unsigned until signing secrets exist; the release notes say so. Windows signing is applied for through
  SignPath Foundation; Apple signing is decided later.
