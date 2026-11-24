# A tour of raul

The [README](../README.md) is the short version. This page walks through raul's window, page by page.

The pictures below are raul's own pages, rendered by its test suite from made-up example data
(`crates/raul-gui/tests/snapshots`), so they always show the current window.

## Setup

The first start walks through five steps: find New Vegas (Steam, GOG and Epic libraries are searched), choose whether
you want Tale of Two Wastelands, make the Instance (raul's own folder for the game's mods, profiles and downloads), run a quick check, done.

<p align="center"><img src="../crates/raul-gui/tests/snapshots/setup/setup-new-vegas-found.png" alt="Setup, step 1 of 5: the game folder found and a storage folder chosen" width="800"></p>

## Mods

Install a mod from its archive; a mod with an installer (FOMOD) asks its questions in the window. Mods can be turned
on and off, moved up and down, and removed, and the last change can be undone. This page is
[nvmod-rs](https://github.com/JorisJonkers-dev/nvmod-rs) at work.

<p align="center">
  <img src="../crates/raul-gui/tests/snapshots/mods/mods-listed.png" alt="The Mods page: a list of mods with their priority, and the selected mod's actions" width="49%">
  <img src="../crates/raul-gui/tests/snapshots/mods/mods-install-questions.png" alt="Installing a mod with an installer: raul asks the installer's questions" width="49%">
</p>

## Load order

Sort the plugins with the LOOT masterlist. raul shows every move and the masterlist's messages first, and changes
nothing until you press Apply. Sorting is [loot-rs](https://github.com/JorisJonkers-dev/loot-rs).

<p align="center"><img src="../crates/raul-gui/tests/snapshots/load-order/load-order-preview.png" alt="The Load order page: a preview of three moves with the masterlist's messages, and an Apply button" width="800"></p>

## LOD

Generate the distant land, objects and trees for exactly your load order. The result is one Mod in your mod list,
`Generated LOD (<profile>)`; a new run replaces it. The settings page changes what is built. The generator is
[lodwright-rs](https://github.com/JorisJonkers-dev/lodwright-rs).

<p align="center">
  <img src="../crates/raul-gui/tests/snapshots/lod/lod-generated.png" alt="The LOD page after a run: what was read and written, and every block generated" width="49%">
  <img src="../crates/raul-gui/tests/snapshots/lod-settings/lod-settings-defaults.png" alt="The LOD settings page: worlds, terrain, land textures, water, objects and trees" width="49%">
</p>

Why it matters: with mods, the game's own far view no longer matches the world you walk through. On a 312-mod Tale
of Two Wastelands profile, a far view generated for that load order brings back the dishes on Black Mountain and the
casinos and buildings of the Strip (right):

<p align="center">
  <img src="images/lod-modded-black-mountain.jpg" alt="Black Mountain from above: without a generated far view the dishes are missing; with lodwright's they are there" width="100%">
  <img src="images/lod-modded-strip.jpg" alt="The Strip from above: without a generated far view casinos and buildings are missing; with lodwright's they are there" width="100%">
</p>

## Tale of Two Wastelands

TTW is built from your own Fallout 3 and New Vegas copies and the installer package you download from the TTW site;
both games stay untouched. raul checks the sources, shows where the output goes and how much space it needs, installs,
and adds TTW to the Instance as one Mod. The installer is [ttwright-rs](https://github.com/JorisJonkers-dev/ttwright-rs).

<p align="center">
  <img src="../crates/raul-gui/tests/snapshots/ttw/ttw-sources-ready.png" alt="TTW, step 1 of 4: the Fallout 3 folder, New Vegas and the TTW package are set" width="49%">
  <img src="../crates/raul-gui/tests/snapshots/ttw/ttw-installing.png" alt="TTW installing at 25 percent" width="49%">
</p>

## Play

**Play** starts the game on the nv-rs engine, which is built into raul. The engine's graphics presets and its F10
overlay are described in the [nv-rs README](https://github.com/JorisJonkers-dev/nv-rs#readme). Settings holds the game
folders, the TTW package and raul's storage folder; About and updates checks for a new release only when you ask, and
an update keeps the version before it so you can go back.

The same steps as commands are in [USAGE.md](USAGE.md).
