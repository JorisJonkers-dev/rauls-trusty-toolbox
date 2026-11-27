<!-- SPDX-License-Identifier: GPL-3.0-only -->
# Plans and roadmap

The full plan is [docs/PLAN.md](../PLAN.md), with the later reorganisation in [docs/REORG.md](../REORG.md). This page is
the short version.

## The idea

raul is the one product. It pins five parts (the [Components](components.md)), owns the window and the command line,
and releases installers for Linux, Windows and macOS. The parts keep their own repositories and releases. A raul
release uses one set of part versions that were tested together.

raul runs the whole flow and nvmod-rs only manages mods. Every part is linked into raul and called directly. Only the
game itself runs as a separate program, started from raul's own binary.

## The plan in phases

1. **Structure:** the five parts as submodules, one workspace, the licence decided.
2. **Window and command line:** setup, mods, load order, LOD and launch.
3. **Build and sync:** checks on all three systems, and a workflow that proposes a version bump when a part releases.
4. **Benchmarks:** `raul bench` with a [history](https://jorisjonkers-dev.github.io/rauls-trusty-toolbox/bench/).
5. **Releases:** installers for Linux, Windows and macOS, checksums, and an update check you ask for. Signing comes when
   the keys exist.
6. **Docs:** architecture, code of conduct, install guide.

## Where it stands

- Releases are out; the latest is on the [releases page](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/releases).
- Windows and macOS packages are built and smoke-tested, but still need a full run on real computers, and are unsigned.
- The window is being rebuilt to match the [demo](https://jorisjonkers-dev.github.io/rauls-trusty-toolbox/demo/), which is the design standard.
- The generated LOD is being improved.
- The nv-rs engine is being ported to the point where it replaces the original game in every case; a chart of that
  progress is on the [README](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox#readme).

## Next

- Profiles, Doctor and Test your setup from the demo, in the real window.
- Signing for Windows and macOS.
- Frame-time benchmarks, when the engine can record them.

Dates are not promised. Follow the [issues](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/issues) and the
[changelog](https://github.com/JorisJonkers-dev/rauls-trusty-toolbox/blob/main/CHANGELOG.md).
