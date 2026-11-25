# Raul's tRusty Toolbox

> *Modding New Vegas without the fRUSTration.*

Raul's tRusty Toolbox, or **raul**, is one app that sets up Fallout: New Vegas with your mods and starts the game.
It works on Linux, Windows and macOS, and it never changes your game folder.

<p align="center">
  <img src="crates/raul-gui/tests/snapshots/home/home-ready.png" alt="raul's Home page: a check list for the game folder, storage, Tale of Two Wastelands, load order and distant view, and a Play now button" width="800">
</p>

## From game to playing, in six steps

<p align="center">
  <img src="docs/images/raul-journey.svg" alt="raul's six steps: find your game, Tale of Two Wastelands (optional), add mods, sort the load order, build the distant view, play on the nv-rs engine" width="100%">
</p>

raul does each step for you in one window. Tale of Two Wastelands, a mod that joins Fallout 3 and New Vegas into one
game, is optional.

## A distant view that matches your mods

Mods change the world, but the game's own far view stays the old one. raul builds a new far view for exactly your mods.

<p align="center">
  <img src="docs/images/lod-overpasses-freeside.jpg" alt="Side by side: on the left the I-15 overpasses north of Freeside vanish at mid distance; on the right they stay in view" width="100%">
  <br><em>Large far objects stay in view: the I-15 overpasses north of Freeside no longer vanish at mid distance.</em>
</p>

<p align="center">
  <img src="docs/images/lod-modded-strip.jpg" alt="The Strip from above, side by side: without a generated far view casinos and buildings are missing (left); with it they are there (right)" width="100%">
  <br><em>The Strip with 312 mods: the game's far view (left) and the one raul built (right).</em>
</p>

## Small fixes for the mods you have

Some mods look better with a small touch. raul makes these add-ons on your computer, for the mods you installed, and
says so when it adds one; turn one off on the Mods page and it stays off.

<p align="center">
  <img src="docs/images/lamp-glow-capital.jpg" alt="Street lamps along the Potomac at night, side by side: on the left one wide white glow covers the far end of the row; on the right each bulb has a small soft halo" width="100%">
  <br><em>Street Lights TTW at night: one wide glow over the whole row (left), a small halo per bulb with raul's add-on (right).</em>
</p>

## How far along it is

- ✅ Setup, mods, load order, distant view, Tale of Two Wastelands and Play work on Linux, Windows and macOS.
- 🔨 These are early versions. Windows and macOS still need a full test on real computers, and your computer may
  warn that the download comes from an unknown maker, because it is not signed yet.

How much of the game engine inside raul is rebuilt; the chart updates itself with every change to the engine:

<p align="center">
  <img src="https://raw.githubusercontent.com/JorisJonkers-dev/nv-rs/fork/docs/media/fork-progress-engine.svg" alt="Progress of the nv-rs engine port" width="100%">
</p>

How much of what raul promises is tested, per area; the chart updates itself with every change
([how it works](docs/COVERAGE.md#feature-catalogue)):

<p align="center">
  <img src="docs/coverage/tally.svg" alt="Feature catalogue: items, tagged, passing and covered per area" width="100%">
</p>

## How it fits together

<p align="center">
  <img src="docs/images/raul-parts.svg" alt="raul and its five parts: nv-rs the game engine, nvmod-rs mods, loot-rs load order, lodwright-rs distant view, ttwright-rs Tale of Two Wastelands" width="100%">
</p>

Splitting the work this way lets each part be tested on its own. [More about the parts](docs/COMPONENTS.md).

## Install

**[Download raul](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/releases/latest)** and pick the file for
your computer: Linux, Windows or macOS. The [install guide](docs/INSTALL.md) names the exact file, shows how to check
the download, and how to build raul yourself.

You need your own copy of Fallout: New Vegas (and of Fallout 3 for Tale of Two Wastelands).

## Learn more

- [A tour of the window, page by page](docs/TOUR.md)
- [Using raul from the command line](docs/USAGE.md)
- [The parts and where they come from](docs/COMPONENTS.md)
- [How it is built](docs/ARCHITECTURE.md) and [the plan](docs/PLAN.md)

raul ships no game files and is not affiliated with Bethesda Softworks, ZeniMax Media or Obsidian Entertainment.
The game pictures in `docs/images` are screenshots taken with the engine for this page. Licence: GPL-3.0-only, a free licence
([LICENSE](LICENSE), [legal notes](docs/INSTALL.md#legal)).
