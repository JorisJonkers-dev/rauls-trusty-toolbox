---
status: accepted
date: 2026-11-16
---
# The GUI is readable first, with a light Fallout flavour

The GUI is a neutral dark surface with hairline separators, one teal accent for actions, selection, focus and
links, and Raul's amber for the brand only, never for a status. The Fallout flavour comes from hand-drawn line icons
(vault door, crate, holotape, radio mast) and small section ticks. Inter is bundled; the themes are dark and high
contrast. A left sidebar holds seven pages in two groups (Play: Home, Mods, Load order, LOD; System: Versions,
Doctor, Settings), with a command palette over all of them. Every page and dialog works with a gamepad from the
first release, Steam Deck included. The first-run wizard uses the same shell as the app. Raul's voice is light: one
short line in calm places only. The window says "Raul's Toolbox"; About, installers and the README use the full
name. The rules are [`UI-RULES.md`](../UI-RULES.md); the reference is [`design/mockup.html`](../design/mockup.html).

This replaces the earlier "Mojave Dusk" design and the monochrome amber Pip-Boy direction, which was tried in a
mockup and read like a 1990s website.

## Consequences

- Updates are split by owner: engine builds, Components and the masterlist on Versions; mod updates on Mods; Home
  shows both as chips.
- TTW is set up from Mods in a four-step flow and becomes one bundle row that can be switched on and off.
- No light theme is designed or tested.
- Gamepad input is part of the view model's input layer and of the GUI tests from the start.

## Considered options

- Monochrome amber with scanlines: rejected, hard to read and dated.
- Bottom tabs with a System menu: rejected for seven pages; the sidebar shows every page and its attention count.
- A light theme: not needed; high contrast covers the accessibility case.
