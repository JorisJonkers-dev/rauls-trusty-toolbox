---
status: accepted, amended by ADR-0019
date: 2026-11-17
---
# Components are incorporated: raul links and calls them in process

nvmod, lodwright-rs, ttwright-rs, loot-rs (through nvmod) and nv-rs are libraries that `raul` links and calls in
process. `raul` does not start them as commands, and nothing the person sees (a screen, a message, a log line) shows
a command being run. There is one exception: running the game is a child process of `raul`, started from `raul`'s
own binary in a play mode that calls the engine's entry point, so there is no separate engine program to ship, place
or find. Everything else of nv-rs (reading saves and config, shader precompile, the viewer's non-play parts, the
checks) is linked as crates and called in process. A process is otherwise started only for a tool that only exists as
a Windows program. One `raul` version carries one set of Component versions, and the person sees only `raul`'s.

The program packages on ghcr.io (ADR-0017) are standalone builds for people who want a Component without `raul`;
`raul` does not use them.

This supersedes the "Runner starts the tools" parts of earlier ADRs (the note in each says where).

## Per Component

| Component | In `raul` | Stays a program | Reason |
|---|---|---|---|
| nvmod | linked: mods, load order, saves, run plan | its own CLI, for standalone use, with the mod-management commands only (ADR-0019) | already linked (ADR-0002) |
| loot-rs | linked through nvmod (ADR-0015) | its own CLI | as above |
| lodwright-rs | linked: LOD generation | its own CLI | no reason to isolate it any more |
| ttwright-rs | linked: the TTW install | its own CLI | as above |
| nv-rs | linked: everything but the game run | its own CLI and the standalone package | the game run is a child process of `raul` itself, started in a play mode (above) |

## The in-process job interface

Each linked Component exposes a documented library API (nvmod-rs#115 is the first). Every long action has the same
shape, defined in the Component and wrapped by one adapter in the Toolbox:

- **Request and result**: plain typed Rust structs, `serde`-serialisable so a Plan can be shown and a journal entry
  written. A call is `run(request, &mut dyn Progress, &CancelToken) -> Result<Output, Error>`. `Progress` and
  `CancelToken` are small types each Component defines for itself, so Components stay independent of the Toolbox.
- **Progress**: events with a step id, a stable label key (the GUI words it in Raul's voice), and optional done and
  total counts. The adapter forwards them to the existing `raul-events` stream and the journal (ADR-0007), now over an
  in-process channel with the same event schema. The GUI reads only that stream.
- **Cancel**: cooperative. The call checks the token at least once per file or per second and stops at the next safe
  point, leaving the journal resumable. A cancelled job returns `Cancelled`, which is a result, not an error.
- **Where it runs**: a worker thread per job. One writer per Instance is enforced by a lock file in the Instance, not
  by a mutex shared in memory, so a dead job leaves nothing poisoned; the game launcher takes the same lock and
  refuses to start while a job holds it. The release profile keeps `panic = "unwind"`, so a panic in a job is caught
  at the thread boundary and becomes an `Internal` error. A job that never checks the cancel token cannot be
  stopped; the adapter shows it as not responding and keeps the window usable, and the lock file records who holds it.
- **Errors**: one typed `JobError` with a stable `kind`, a plain `summary`, a technical `detail` (source chain, paths
  with the home directory shown as `~`), the Component, `raul`'s version and the job id. `copy_text()` renders it as
  one block for pasting into a bug report. It never contains a command line (#170).

The CLI subcommands of `raul` and of each Component are thin wrappers over the same calls. A test fails when a GUI
action starts a process other than the game.

## The play mode

The game runs in a child of `raul`'s own binary: `raul` starts `current_exe()` with the reserved first argument
`--play-mode <DATA> [arguments]`, and that child calls the engine's `fork_play::play_main`. No environment variable
selects the mode, so a build that does not link the engine (`engine` feature off, and the CLI-only `raul`) refuses
the argument with one plain line and exit code 2. The game is a process of its own so a crash of the engine stays out
of the window.

- **Stop**: the launcher names a stop file in `NV_STOP_FILE` and creates it to ask for the stop. The game ends between
  two frames with status 143 after its exit hooks and the pipeline-cache flush; a signal or a kill skips both, so
  it is only the last resort after a grace period.
- **Exit codes** are the engine's (`fork_play::ExitClass`): 0 finished, 1 start-up error, 2 arguments, 3 the game's
  code stopped, 69 the system refused the start, 143 stopped by the stop file. The launcher trusts a code only
  together with the log and with whether it asked for the stop.
- **Shader precompile** runs in this child (`--play-mode <DATA> --precompile-shaders`), not in the window's process,
  because it opens a GPU device and a driver may abort or hang; the result is the child's exit code and log lines.
- A test forbids any other process start in the GUI; the play mode is the one allowed.
- The engine program that installers carried, and the place where its pin lived, are removed (amended 2026-11-20).
  Installers hold no `components/nv-rs` folder and no `FILES.sha256`, and the release smoke test fails if one appears.
  ADR-0017's standalone nv-rs package stays for people who run the engine without `raul`.

## Folders as a value, not the environment

Several runs share one process (the window's jobs, a test), so a run cannot get its own configuration folder from an
environment variable: the variable is shared by every thread and `unsafe_code` is forbidden. `raul_core::context::Context`
carries the configuration folder (and whether nvmod's run log is written) from the entry point down: `raul_cli::run_in`
takes it, `InProcessExec` and the window's `Jobs` hold one, and the typed jobs that read `raul config` take it. The
default is the person's own folders, so the command line is unchanged. A state folder of its own needs a path in nvmod's
run log, so a context that must not touch the real one writes no run log. Tests give each fixture a context instead of
starting the built `raul` with `RAUL_CONFIG_DIR` and `NVMOD_STATE_DIR`; the service has no implementation that starts
a process any more.

## What changes

- `raul-runner` and the rule "only `raul-runner` may start a process" (ADR-0007) narrow to the game launcher and any
  Windows-only tool. The GUI no longer runs writes as a `raul` subprocess (ADR-0003); it calls the job interface.
- Wire Contracts of linked Components (`contracts --json`, ADR-0008, ADR-0009) become Rust types; Cargo.lock and the
  submodule pin decide what is compatible, checked at compile time. On-disk Contracts stay: they define what is kept
  and its migration.
- Precedence of compatibility checks: the compiler (Rust types) first, then the on-disk Contract migrations, then
  `known-good.toml` and `raul e2e` for sets. A Component's API version is part of its release and the lockfile.
- `known-good.toml` stays as the record of which Component versions passed the end-to-end tests together. A Known-good
  set is no longer shown to the person and no longer lists per-platform fetch rows for linked Components.
  `supply.toml` keeps rows only for packages `raul` installs: the engine.
- ADR-0011 (Rolling channel) stands. A newer engine build is still opt-in; its build id shows only on that page.

## Consequences

- An out-of-memory kill, a stack overflow or an abort in a linked job takes the window down; a panic does not. The
  lock file may then be stale, so the next start checks that its holder is gone before it takes the lock. Mitigations: jobs bound
  their memory over untrusted input (archive limits), every step is journaled, and the next start resumes. This
  gives up the crash isolation ADR-0002 chose; the epic (#166) prefers one self-contained app over it.
- Installers no longer contain the lodwright-rs and ttwright-rs programs, no engine program either: the engine is linked.
- A Component release reaches the person only in the next `raul` release.
- Licences are unchanged: `raul` is GPL-3.0-only, and MPL-2.0 lodwright-rs may be linked into it (ADR-0002).

## What is not bundled, and why

The installer holds everything we build. A few things cannot ship in it for a licence or ownership reason, so each has
its own guided step and nothing is fetched silently:

| Item | Why it is not bundled | Guided step |
|---|---|---|
| Fallout 3 and Fallout: New Vegas game files | The person's own copy; never redistributed | The setup asks for the game folder and only reads it |
| The TTW package (`.mpi`) | TTW data, not redistributable (docs/PLAN.md section 3) | The TTW step asks for the file and says where to get it |
| xNVSE and JIP LN DLLs | Third-party binaries, not ours to redistribute | Installed as Mods through the Mods page, by choice |
| The LOOT masterlist | Data, fetched at run time (CC0) | Fetched when a load order is sorted |
| Mod archives | Owned by their authors and the Nexus terms | The Mods page, one archive at a time |

## Considered options

- Keep the Runner and start Components as commands: rejected, it shows commands, parses text for every screen, and
  gives each Component its own version, fetch and failure to explain.
- A shared `progress` crate used by every Component: rejected for now, a fifth repository for about forty lines.
