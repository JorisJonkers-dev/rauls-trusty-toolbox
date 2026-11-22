# Pictures, soaks, sweeps and benches of the game

`raul scene` and `raul bench game` start the game straight at a place, with no main menu, in a frozen world (fixed
hour and weather, no moving creatures), and take pictures, hold places for a number of frames, sweep many places or
measure frame times. The work is the engine's own scene library (`fork-scene`, described in nv-rs `docs/fork/SCENE.md`), linked into
`raul` and run in its process; `raul` resolves what the bare tool wants by hand. Terms: [Place, Place file, Variant](../GLOSSARY.md). The decision
is [ADR-0022](adr/0022-scene-runs-and-the-in-game-bench-are-raul-commands.md).

## Take a picture pair of your modded game with and without generated LOD

You need a Profile that has a generated LOD Mod (`raul lod generate`), and a game folder in `raul config`
(`game.newvegas`). Three commands:

```
raul scene places                      # once: writes mojave-day.toml and mojave-night.toml here, to read or edit
raul scene run --places mojave-day     # one game start with the LOD Mod, one without it, then the difference
raul scene run --places mojave-night --only night-strip-skyline-from-south   # the same pair by night, one place
```

`raul scene run` finds the Profile (`--profile`, else the Instance's selected one), lays its enabled Mods over the game
folder in priority order, loads its active Plugins in Load order, and draws every place once with the generated LOD Mod
(variant `with-lod`) and once without it and without the stub Plugin it carries (`without-lod`). It prints the folder of
the pictures, `<raul data folder>/scene/<Profile>/<UTC time>/<variant>/pictures`, and then, for each place, how much the
two pictures differ (mean and maximum channel difference, share of pixels changed).

**Open both pictures of each name before you say anything about them.** A difference number alone never passes or fails
a view; a picture with a menu, blur or a cursor in it is not evidence, so take it again.

- `--variant <name>=<source>` picks the variants yourself, any number: `profile`, `profile-without-lod`, `game` (no
  Mods) or a folder of data laid over the game, for example a LOD folder a Component is being tried on.
- `--only a,b` takes some places of the file; `--grids 9` loads a square of 9 by 9 cells; `--quick` takes small, fast
  pictures; `--resolution 1920x1080`.
- `--out <dir>` puts the results elsewhere; `--game <dir>` names another game folder.
- `raul scene compare <a> <b> [--noise <a2>]` compares two pictures, or two folders by file name. `--noise` is a second
  run of the first folder; what differs between those two is left out of the difference.

### Place files

A place file is TOML, version 1: a name, a world and coordinates, or a map marker plus an offset, or an interior cell or
a door, and optionally the heading, a free camera, `fov`, `fog`, `far`, `hour` and `weather`. The two files `raul`
carries (four Mojave places, by day and by night) are examples with only coordinates and editor ids in them; the full
list of keys is in nv-rs `docs/fork/SCENE.md`. The hour of a view is a key of its place (`hour = 22.5`), not a flag.

## Soak, sweep and bench

```
raul scene soak  --places mojave-day --frames 300   # hold each place 300 frames; counts stops, errors, warnings
raul scene sweep --places <list>                    # a place that stops or hangs is recorded, the sweep goes on
raul bench game --suite quick --save-baseline mine  # once
raul bench game --suite quick --baseline mine       # after a change; compares with the baseline
raul bench game compare <a/summary.json> <b/summary.json>
```

These draw the Profile's Staging tree over the game folder. A bench measures time, so it takes the machine's game
lock exclusively and waits until no other game runs; it is never run by CI. `raul bench` and `raul bench compare`
(the synthetic suite and its history) are unchanged and never start the game. Frame times of a soak are not a
measurement. Two runs of the same build differ by a few per cent: a change below that is not a finding.

## What raul does for you

- The Profile's Mods, Plugins and Load order are resolved by `raul`; no overlay folders or plugin lists by hand.
- The game folder comes from `raul config`; results go under raul's data folder (`RAUL_DATA_DIR` moves it).
- Every game start carries a label (`raul-scene-<command>-<Profile>-<variant>`) and, where the machine's capping
  program (`gamerun`) is installed, a hard memory limit with no swap; without it raul says so and starts uncapped
  (`--no-cap` asks for that, `--mem 8G` changes the limit).
- The game is a child process started from raul's own binary in the play mode, with the label and the memory cap above;
  a build without the engine refuses unless you pass `--viewer <program>`.

## Limits of this first version

- Linux only: the tool takes locks with `flock` and limits memory with systemd scopes. Other systems get a plain refusal.
- The engine's scene library is linked into `raul` and runs in its process (there is no `nv-rs-scene` program to find and
  no `RAUL_SCENE_TOOL`); a sweep therefore has no supervising child, only the game is a child process.
- No `--json` yet; the window has no Scene page yet.
