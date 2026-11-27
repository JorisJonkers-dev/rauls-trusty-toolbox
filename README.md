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

## The bundled Beautiful profile

Left is the vanilla game: no mods, the game's own renderer. Right is the bundled Beautiful profile with raul's engine
features (ambient occlusion, sun shadows, reflections, fill light) and a generated far view.

<p align="center">
  <img src="docs/images/beautiful/compare-goodsprings-valley.jpg" alt="Goodsprings main street, vanilla game on the left and Beautiful on the right" width="100%">
  <br><em>Goodsprings main street: new ground and building textures, a street lamp, sun shadows.</em>
</p>

<p align="center">
  <img src="docs/images/beautiful/compare-lake-mead.jpg" alt="Lake Mead from the shore, vanilla game on the left and Beautiful on the right" width="100%">
  <br><em>Lake Mead from the shore: water reflections, sharper rocks and sand.</em>
</p>

<p align="center">
  <img src="docs/images/beautiful/compare-nv-ultra-luxe.jpg" alt="The Ultra-Luxe lobby, vanilla game on the left and Beautiful on the right" width="100%">
  <br><em>Ultra-Luxe lobby: retextured floor, ambient occlusion, coloured light.</em>
</p>

<p align="center">
  <img src="docs/images/beautiful/compare-night-primm-road-lamps.jpg" alt="Primm at night, vanilla game on the left and Beautiful on the right" width="100%">
  <br><em>Primm at night: the street lamps light the ground.</em>
</p>

There are eight views in all, outside and inside, by day and by night. [Slide between them on the site](https://jorisjonkers-dev.github.io/rauls-trusty-toolbox/compare/).

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
