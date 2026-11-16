---
status: accepted
date: 2026-11-16
---
# Only our own and openly licensed assets ship; an opt-in skin may read the player's own interface art

The repository and every release contain only our own work and openly licensed assets (CC0, CC-BY, CC-BY-SA, OFL,
MIT, ISC, Apache-2.0), each listed with source, licence and attribution in `packaging/NOTICE-assets.toml` and shown
in Settings, About and notices. No game file and no mod file ships, in the repository, installers, fixtures or
support bundles.

A survey of the interface mods in common use found none whose files may be redistributed, and most replace the
game's own art, which stays its publisher's even where a mod's text sounds permissive. So no mod's interface art
ships and mod authors are not asked.

An install skin, "Game interface art" in Settings, is built after the base GUI (M5). It is opt-in and off by
default. It reads icons and fonts from the player's own install at run time through a Component's archive reader,
keeps a cache in the player's data folder, redistributes and uploads nothing, and falls back to the bundled set when
anything is missing, changed or unreadable. It changes icons and fonts only, never colours, layout or contrast.

## Consequences

- CI fails on an asset that the notice file does not list, and on game-style paths and formats.
- Support-bundle screenshots always use the bundled set.
