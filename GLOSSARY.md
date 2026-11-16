# Raul's tRusty Toolbox

One product that takes a player's own copy of Fallout: New Vegas (and, for Tale of Two Wastelands, Fallout 3) to a
modded game running natively on nv-rs, built from separately developed Components. This glossary is the canonical
one for the product; the `nv-linux` workshop repository points here.

## Language

### Product

**Toolbox**:
The whole product, Raul's tRusty Toolbox: the `raul` program, its installers and the Known-good set it ships.
_Avoid_: Umbrella (for the product; the umbrella is the repository), suite, launcher

**Component**:
One of the separately developed projects the Toolbox pins and ships: nv-rs, nvmod, loot-rs, lodwright-rs and ttwright-rs.
_Avoid_: Tool (for the project), module, submodule (that is how it is pinned, not what it is)

**CI unit**:
One independently built and tested part of the nv-rs workspace in its CI split.
_Avoid_: Component, shard, group

**Raul's voice**:
The tone of the Toolbox's on-screen text: dry, an old ghoul mechanic who has seen worse. Written words only; every error under it still carries a plain detail line.
_Avoid_: Persona (too broad), narration, voice-over

### Game and files

**Instance**:
One MO2-style installation of the game with its mods, profiles and downloads.
_Avoid_: Install, setup, environment

**Profile**:
A named load order together with the list of enabled mods, inside an Instance.
_Avoid_: Preset, loadout

**Mod**:
A staged folder of files that came from one archive.
_Avoid_: Package, addon

**Plugin**:
An `.esm` or `.esp` data file that the game loads in order. Nothing else is called a Plugin.
_Avoid_: Mod (for the file), ESP (for both kinds)

**NVSE plugin**:
A DLL that xNVSE loads into the game process.
_Avoid_: Plugin (alone), script extender mod

**Load order**:
The order in which Plugins are applied; later Plugins win conflicts.
_Avoid_: Priority, sort order

**Staging tree**:
The merged view of the game's files and the active Mods that the game is started against.
_Avoid_: Virtual file system, merged folder, overlay (overlay is one way to build it)

### Mod managing

**FOMOD**:
A mod's own installer: a short list of questions whose answers decide which of the mod's files are installed.
_Avoid_: Wizard (that word is for the Toolbox's own step-by-step flows)

**Master**:
A Plugin that another Plugin needs in order to work; it must come earlier in the Load order.
_Avoid_: Dependency (for plugins)

**MO2**:
Mod Organizer 2, the mod manager many players used before; the Toolbox can read its setup and never writes to it.
_Avoid_: The other manager

**TTW**:
Tale of Two Wastelands, which merges Fallout 3's content into New Vegas; it needs the player's own Fallout 3 copy.
_Avoid_: The merge

**MPI package**:
The archive a publisher distributes for an install such as TTW, which ttwright-rs applies to the player's own game files.
_Avoid_: TTW file, installer (for the package)

**Plan**:
The list of changes an operation would make, shown before anything is changed; applying it is a separate, explicit step.
_Avoid_: Preview (for a plan that can be applied), dry run (for the plan shown to a person)

**Rollback**:
Going back to how an Instance was before an earlier change, using the restore point that change kept. The GUI calls it "Undo".
_Avoid_: Revert

### Components

**nv-rs**:
The Rust port of the game's engine, kept in the organisation's fork of the upstream project.
_Avoid_: The engine port, the rewrite

**Fork line**:
The `fork` branch of nv-rs, which holds upstream's commits replayed unchanged plus the fork's own changes.
_Avoid_: Main (that name belongs to the untouched upstream copy)

**nvmod**:
The Component that manages Instances: installs, profiles, sorting through loot-rs, Doctor, and driving the other Components.
_Avoid_: The launcher, the mod manager (when meaning the whole product: that is the Toolbox)

**ttwright-rs**:
The Component that installs MPI packages such as TTW; the organisation's rewrite of an upstream installer.
_Avoid_: TTW installer (for ours), the installer

**Engine extension**:
Native code added to nv-rs, compiled in or loaded through its C ABI, described by its own manifest.
_Avoid_: Plugin, native plugin, mod (for the code)

**Doctor**:
The read-only check that inspects an Instance and reports what is wrong without changing anything.
_Avoid_: Linter, validator

### Integration

**Contract**:
A versioned JSON Schema, owned by the Component that emits it, for what is exchanged with that Component.
_Avoid_: API, interface (too broad)

**Tool pin**:
An entry in nvmod's `tools.lock.toml` naming one Component's or third-party program's exact version, checksum and attestation identity.
_Avoid_: Dependency (for external programs)

**Known-good set**:
A combination of Component versions recorded in `known-good.toml` after the contract and end-to-end tests passed for exactly that combination.
_Avoid_: Compatible versions, supported matrix, compat set

**Rolling channel**:
The opt-in way an nv-rs build newer than the Known-good set is installed beside the current one, after a compatibility check, with one-click return to the previous build.
_Avoid_: Nightly, beta (for nv-rs builds)

**Runner**:
The one part of nvmod that starts a process: argument vector, no shell, scrubbed environment, timeout, output cap and whole-tree kill.
_Avoid_: Launcher (that word is for starting the game), executor

### Interface

**Install skin**:
The opt-in setting that reads interface icons and fonts from the player's own game install at run time, never shipped or uploaded (ADR-0014).
_Avoid_: Theme (themes are the Toolbox's own: dark and high contrast), mod skin

**TTW bundle**:
The one row in Mods that stands for an installed TTW, which can be switched on and off as a whole.
_Avoid_: TTW mod (it is many files from the player's own games, not one downloaded mod)

**Attention chip**:
A small count with an icon that says a page needs a look; the sidebar, Home and Doctor show the same ones.
_Avoid_: Badge, notification (notifications are transient messages)

### Observability

**Run log**:
The structured record of one command, written locally and never sent anywhere.
_Avoid_: Telemetry, trace, audit log

**Support bundle**:
A zip a person chooses to make, after seeing what it would contain, to attach to a bug report by hand; it is never uploaded for them.
_Avoid_: Diagnostics upload, crash report

### Testing

**Oracle**:
A source of ground truth that a Component's output is compared with, such as real LOOT, a real LOD generator or the real game.
_Avoid_: Reference implementation, golden source

**Dataset**:
A pinned, hashed set of private inputs or oracle outputs kept outside the public repositories.
_Avoid_: Fixtures (those are the public synthetic ones), test data

**Release test**:
The scripted local checklist a project must pass on the owner's machine before its release is cut.
_Avoid_: Smoke test (that is the automated start check of a built artefact), QA pass

**Fixture**:
A hand-made or synthetic input that is safe to keep in a public repository.
_Avoid_: Sample, asset

**Clean room**:
The rule that an implementation is written only from public specifications and own measurements, never from the original tool's source.
_Avoid_: Reimplementation, port (when no source was read)
