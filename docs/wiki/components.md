<!-- SPDX-License-Identifier: GPL-3.0-only -->
# Components

raul is made of five parts. Each has its own repository, its own tests and its own releases, and each works alone with
its own command line.

| Part | What it does |
| --- | --- |
| [nv-rs](https://github.com/JorisJonkers-dev/nv-rs) | The game engine. A Rust port of Fallout: New Vegas, from slaterain. This is the project's fork of [slaterain/nv-rs](https://github.com/slaterain/nv-rs). raul starts the game on it. |
| [nvmod-rs](https://github.com/JorisJonkers-dev/nvmod-rs) | Mods. Instances, profiles, installing mods, conflicts and the load order list. |
| [loot-rs](https://github.com/JorisJonkers-dev/loot-rs) | Sorts the load order with the [LOOT](https://loot.github.io/) masterlist. It reads the list when it runs and does not copy it. |
| [lodwright-rs](https://github.com/JorisJonkers-dev/lodwright-rs) | Builds the distant terrain, objects and trees for your mods. |
| [ttwright-rs](https://github.com/JorisJonkers-dev/ttwright-rs) | Installs Tale of Two Wastelands from your own game copies. A rewrite of [TTW_Linux_Installer](https://github.com/SulfurNitride/TTW_Linux_Installer). |

## Why separate

Each part does one job and can be tested alone. A part can be released when it is ready. raul links them into one
program, and a raul release uses one set of versions that were tested together.

## More

[docs/COMPONENTS.md](../COMPONENTS.md) has the reasons and the origins in full, and
[docs/ARCHITECTURE.md](../ARCHITECTURE.md) shows how they fit together in the code.
