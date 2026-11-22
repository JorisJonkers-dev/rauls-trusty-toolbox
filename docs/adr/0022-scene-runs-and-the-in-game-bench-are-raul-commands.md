---
status: accepted
date: 2026-11-22
---
# Scene runs and the in-game bench are `raul` commands; `raul bench` stays synthetic

The engine has a tool that starts the game at a place and takes pictures, holds places (soak), sweeps many places and
benchmarks (nv-rs `docs/fork/SCENE.md`). `raul` is the one tool people and workers use, so the commands are `raul`'s:
`raul scene run|compare|soak|sweep|places` and `raul bench game`. `raul` resolves the Profile (Mods in priority order,
active Plugins in Load order, the game folder from `raul config`), the output folder under its data folder, the label
and the memory cap of the game start; the game is the child process of ADR-0016.

`raul bench` and `raul bench compare` keep their meaning: the synthetic suite, its reports and the history CI keeps.
ADR-0010 said `raul bench` never starts the game; that stays true of `raul bench`. Only `raul bench game` starts the
game, takes the machine's game lock exclusively, and is never run by CI. Its summaries and baselines stay on the person's
machine and do not enter the CI history.

## Consequences

- The engine's scene library (`fork-scene`) is linked into `raul` and runs in its process, as ADR-0016 asks of every
  Component. The only child process is the game, started from `raul`'s own binary in the play mode through
  `raul-runner`, with the run label and the memory cap. The first version ran the engine's tool as a program behind a
  seam; that exception to ADR-0016 is gone.
- The tool is Linux only for now (`flock`, systemd scopes).
