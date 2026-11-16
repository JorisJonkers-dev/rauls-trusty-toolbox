# Architecture

How the Toolbox and its Components fit together. Terms are defined in [GLOSSARY.md](../GLOSSARY.md); the decisions
behind this shape are [ADR-0001](adr/0001-the-toolbox-is-the-product-umbrella.md) to
[ADR-0006](adr/0006-native-on-linux-windows-and-macos-with-no-compatibility-layers.md).

## Pieces

```mermaid
flowchart LR
  subgraph raul["raul (this repository)"]
    gui["raul-gui<br/>(GUI build, nested workspace)"]
    cli["raul-cli<br/>raul commands + every nvmod command"]
    core["raul-core<br/>detection, Known-good sets, update check"]
    bench["raul-bench"]
  end
  subgraph linked["linked in process (GPL-3.0-only)"]
    nvmod["nvmod crates"]
    loot["loot-rs"]
  end
  subgraph procs["separate programs, versioned Contracts"]
    nvrs["nv-rs<br/>(engine)"]
    lod["lodwright-rs<br/>(LOD)"]
    ttw["ttwright-rs<br/>(TTW / MPI)"]
  end
  gui -- reads --> core
  gui -- writes, as a subprocess --> cli
  cli --> core
  cli --> nvmod
  bench --> nvmod
  bench --> loot
  core --> nvmod
  nvmod --> loot
  nvmod -- Runner --> nvrs
  nvmod -- Runner --> lod
  nvmod -- Runner --> ttw
```

- **raul-core** owns what is the Toolbox's alone: finding game copies, the Known-good set (`known-good.toml`)
  and the update check. Everything about Instances, Mods, Profiles and the Load order is nvmod's, used by path from
  `components/nvmod-rs`.
- **raul-cli** is the `raul` program. Its own commands are `version`, `detect`, `known-good`, `bench` and
  `update`; every other command is nvmod's and runs in process through `nvmod_cli`.
- **raul-gui** (after the GUI moves from nvmod-rs, ADR-0003) reads through raul-core in process and runs
  long or risky work as a `raul` subprocess that streams progress events, so a crash in an install never closes
  the window.
- **Components** that run as separate programs are started only by nvmod's Runner (argument vector, no shell,
  scrubbed environment, timeout, output cap) and speak versioned JSON Contracts.

## Data flow: from a game copy to a running game

1. **Detect** (`raul detect`): find Fallout: New Vegas (and Fallout 3 for TTW) in Steam, GOG or Epic folders, or
   a folder the person picks; check the executable version and the DLC. Read only.
2. **Instance** (`raul init`, nvmod): make an Instance from the base copy. The base copy is never written.
3. **TTW** (`raul ttw`, nvmod driving ttwright-rs): optional; installs the Tale of Two Wastelands MPI package from
   the person's own Fallout 3 and New Vegas copies, with progress and resume.
4. **Mods** (`raul install`, `raul mods`, `raul profile`, nvmod): install from archives or folders into the
   Instance, enable and disable per Profile, and show conflicts. Every change keeps a restore point (Rollback).
5. **Load order** (`raul sort`, nvmod with loot-rs): compute the order from the Staging tree and the LOOT
   masterlist, show the Plan, apply it only when asked; locked plugins keep their place.
6. **LOD** (`raul lod`, nvmod driving lodwright-rs): build distant terrain, object and tree LOD into the Instance,
   reusing blocks that are up to date.
7. **Launch** (`raul run`, nvmod driving nv-rs): start the Staging tree on nv-rs, natively. Where nv-rs is not
   native yet, Launch is disabled (ADR-0006).

## Versions

The Components are submodules under `components/`, each pinned to the version of the latest Known-good set in
`known-good.toml`. `raul known-good check` keeps the two equal, and CI also checks that every pointer is a release
tag or on its repository's default branch. A Component release dispatches `component-bump.yml`, which opens a pull
request that moves only that Component. The release (`release.yml`) packages `raul` with the Component programs of
that set.

## Workspaces and lock files

| Workspace | Lock file | Builds |
| --- | --- | --- |
| root `Cargo.toml` | `Cargo.lock` | raul-core, raul-cli (CLI-only `raul`), raul-bench, and the nvmod and loot-rs crates they link |
| `crates/raul-gui` (to come) | its own | the release `raul`: the same CLI plus the GPUI GUI (ADR-0004) |
| each Component | its own | built from `components/<name>` with `--manifest-path`, never as a member here |
