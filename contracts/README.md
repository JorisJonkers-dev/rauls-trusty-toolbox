# The Toolbox's own Contracts

The JSON documents `raul` prints for the GUI and for scripts (ADR-0009). Each one has a schema here,
`<name>.v<N>.json`, generated from its Rust type in `raul-core`, and fixtures in `fixtures/<name>/`. The test
`crates/raul-core/tests/contracts.rs` keeps the schema equal to the type and checks every fixture against it;
`RAUL_BLESS_CONTRACTS=1 cargo test -p raul-core --test contracts` rewrites the schemas after a change to a type.

Every document carries an integer `schema_version` and a `contract` name. A reader accepts the version it knows
and the one before it, and refuses a newer one with a plain reason. Adding an optional field is not a new version;
removing, renaming or retyping one is.

| Contract | Version | Printed by |
| --- | --- | --- |
| `raul-config` | 1 | `raul config list\|get\|set\|unset --json` |
| `raul-events` | 1 | the JSON lines of a driver command: `raul ttw --json` |
| `raul-ttw-result` | 1 | the `result` event of `raul ttw --json` |
| `raul-lod-result` | 1 | the `result` event of `raul lod generate --json` |
| `raul-lod-mods` | 1 | `raul lod mods --json` |
| `raul-journal` | 1 | the file `.raul/setup-journal.json` in an Instance (on disk) |

`snapshots/` holds the schemas of the Component Contracts the Toolbox reads, as their releases published them:
`snapshots/ttwright-rs/` is ttwright-rs v0.3.0 (`ttwright-result`, `ttwright-events`, `ttwright-contracts`, all v1).
`snapshots/lodwright-rs/` is lodwright-rs v0.3.0 (`lodwright-request` and `lodwright-result`, both v1), byte for byte
the schemas attached to that release.

## `raul-config`

The paths the GUI needs and nothing else. The file is `config.toml` in the Toolbox's configuration folder
(`$RAUL_CONFIG_DIR`, else `$XDG_CONFIG_HOME/raul` or `~/.config/raul` on Linux, `%APPDATA%\raul` on Windows,
`~/Library/Application Support/raul` on macOS). The GUI reads it in process through `raul-core` and writes it only
through `raul config set` and `raul config unset`.

| Key | Value |
| --- | --- |
| `game.newvegas` | the Fallout: New Vegas folder; `set` refuses a folder `raul detect --folder` does not accept as New Vegas |
| `game.fallout3` | the Fallout 3 folder; the same check for Fallout 3 |
| `ttw.package` | the TTW installer package the player downloaded (the `.mpi` file); `set` refuses a path that is not a file |
| `storage.root` | the Toolbox storage folder, which holds the Instance; `set` refuses a path that is a file |
| `instance` | `<storage.root>/instance`; derived and read only |

Values are absolute paths; `set` makes a relative path absolute against the working folder. A key that is not set
is `null`.

`list` prints every key in the order above; `get <key>`, `set <key> <path>` and `unset <key>` print the one key.
When a command fails, `entries` is empty and `error` holds a stable `code` and a `message` for people:

| `error.code` | Exit code | Meaning |
| --- | --- | --- |
| `unknown_key` | 5 | not one of the keys above |
| `read_only` | 2 | `instance` cannot be set |
| `invalid_value` | 2 | the path is not acceptable for the key; nothing was written |
| `newer_schema` | 2 | the file's `schema_version` is newer than this `raul` reads; nothing was written |
| `unreadable` | 2 | the file cannot be read or is not a configuration file; nothing was written |
| `no_config_folder` | 2 | no configuration folder (`HOME`, `XDG_CONFIG_HOME` or `APPDATA` is not set) |
| `write_failed` | 4 | the file could not be written; the old file is left as it was |

A command line `raul config` cannot read exits 5 with a usage line on standard error and prints no document.

The file itself is an on-disk document with `schema_version = 1` (the first version: a file without one is read as
version 1). Keys this build does not know survive a `set`. `raul config` is the file's only writer: the GUI and
scripts run one `set` at a time.

## `raul-journal`

The setup job's resume journal, an on-disk document: `<instance>/.raul/setup-journal.json`. The setup writes it after each
step that finishes, so a setup that was interrupted continues at the first unfinished step, starts again, or is
removed from `raul`. Nothing is created in the Instance folder before the first step has made it.

| Field | Meaning |
| --- | --- |
| `schema_version`, `contract`, `raul_version` | the version (1), `raul-journal`, and the `raul` that last wrote the file |
| `plan` | the ids of the steps of the run that are not left out (an imported Instance keeps its order, so it has no `sort`), in order (`init`, `import-mo2`, `profile`, `engine`, `target`, `ttw`, `mod:<file name>`, `stage`, `sort`, `stamp`, `verify`) |
| `finished` | the ids of the steps that finished, in order; a Mod step is matched to the request by its file name |
| `complete` | every step of the plan finished |
| `request` | the setup request the run was started with (the request of `raul-core`'s setup job, without its list of finished steps), so `raul setup --resume --instance <dir>` needs the Instance and nothing else; `null` before a run started, or for an `nvmod` state whose command line could not be read |
| `taken_over` | `null`, or where the journal came from: `from` is `nvmod-setup` and `version` the version that wrote that state |

Opening a journal of an older version migrates it after a copy was kept next to it (`setup-journal.json.v0.bak`,
never replacing an earlier copy); version 0 was the draft shape `{"schema_version": 0, "steps": [...]}`. A newer
version is refused with a plain reason and the file is left as it is. Every write goes to a temporary file that is
renamed over the journal, so an interrupted write leaves the old file whole. The setup job is the only writer; the
Toolbox has no Instance lock yet.

An Instance that holds an unfinished `nvmod setup` state (`.nvmod/setup.json`) and no journal is taken over once: its
finished steps become the journal's (its `nv-rs` step is `engine`), a copy of the file is kept as
`.raul/nvmod-setup.json.bak`, and the old file is not touched. The copy stops a later takeover after the journal was removed.

| Error `code` | Meaning |
| --- | --- |
| `newer_schema` | the file's `schema_version` is newer than this `raul` reads; nothing was written |
| `unreadable` | the file cannot be read or is not a journal; nothing was written |
| `write_failed` | the file or a copy could not be written; the old file is as it was |
| `takeover_refused` | the `nvmod` state cannot be read or is newer than this `raul` reads; it was left as it is |
| `plan_mismatch` | the request names a step twice (two Mod archives of one file name), or leaves out a step the journal holds as finished; nothing was run |

## `raul-events`

`raul ttw --json` and `raul lod generate --json` print one `raul-events` line per event on standard output while they run. Each line has
`schema_version`, `contract`, `seq` (from 1), `command` and `event`:

| `event` | Fields | When |
| --- | --- | --- |
| `started` | `dry_run`, `resume`, `instance`, `profile`, `tool` (`name`, `version` (`null` until the result names it), `path`, `sha256`) | the checks passed and the program starts |
| `phase` | `phase` | a phase began: for `ttw` ttwright's `package`, `preflight`, `checks`, `assets`, `post_install`; for `lod` `terrain`; then `add_mod` |
| `progress` | `phase`, `done`, `of` (0 when unknown), `message` | progress within the phase |
| `warning` | `message` | something went wrong, but the run goes on |
| `result` | `result` | the command's result document (`raul-ttw-result`, `raul-lod-result`) |
| `finished` | `status`, `exit_code`, `code`, `message`, `resumable`, `changed` | always the last line |

A command refused before it starts prints only `result` and `finished`. The process exits with `finished.exit_code`.
Standard error carries the same story as text for people.

## `raul-ttw-result`

`raul ttw` installs TTW with ttwright-rs, which runs inside raul's own process (no program is started), and adds
its output to the Instance as the Mod `Tale of Two Wastelands`, with its plugins enabled in the Profile. The result
names the Mod, the plugins it enabled, the `tool` block (ttwright, as the in-process driver: the raul executable,
with an empty SHA-256), the `ttwright-result` document ttwright returned, and the staging folder ttwright
wrote to. A cancelled or failed install keeps the staging folder, so `raul ttw --resume` continues there.

| `status` | Exit code | `error.code` | ttwright |
| --- | --- | --- | --- |
| `success` | 0 | | exit code 0, and the Mod was added (or a dry run) |
| `refused` | 2 | `instance_invalid`, `profile_missing`, `missing_input`, `mod_exists` | not started |
| `refused` | 2 | `tool_missing`, `contract_refused` | kept for older result documents; raul no longer produces them |
| `unsupported` | 2 | `unsupported` | exit code 3 (`unsupported_feature`, `unsupported_version`); nothing was written |
| `unverifiable` | 3 | `tool_unverifiable` | kept for older result documents; raul no longer produces it |
| `failed` | 4 | `ttwright_failed` | exit code 1, or 2 (a usage error) |
| `failed` | 4 | `tool_failed` | the work folder could not be prepared, ttwright ended with an undocumented exit code, or its result was not a success |
| `failed` | 4 | `add_mod_failed` | exit code 0, but adding the Mod failed and was rolled back |
| `cancelled` | 4 | `cancelled` | exit code 4, or stopped by `raul` on SIGINT or SIGTERM, or by the install timeout (`--timeout-secs`, which also cancels and is resumable); `--resume` continues |

`error.ttwright_code` is ttwright's own `error.code` when ttwright gave one. In every case but `success` the
Instance is as it was before the command. A command line `raul` cannot read exits 5 and prints no events.

## `lodwright-request`, as `raul lod plan` builds it

`raul lod plan [--instance <dir>] [--profile <name>] [--world <name>]... [--allow-in-repo] [--json]` prints the `lodwright-request` v1
document `raul lod generate` passes to lodwright, linked into `raul` and called in process. It is lodwright's Contract; `raul`
checks every request against the snapshot before it prints or passes it. The request holds:

- `data_folder`: the Profile's Staging tree `Data` folder;
- `plugins`: the Profile's active plugins, in Load order;
- `worlds`: the `--world` flags, else the Profile's settings, else empty (every world with terrain);
- `output_dir`: `<instance>/.raul/lod/out`, where lodwright keeps its manifest between runs;
- the switches, inclusion rules and limits of the Profile's settings.

The settings are `raul-lod.toml` in the Profile's folder; every key is optional:

```toml
worlds = ["SyntheticWorld"]
threads = 2            # unset: lodwright decides
memory_mib = 4096      # advisory
allow_in_repo = false  # true: the output folder may lie inside a git working tree

[inclusion]
exclude_plugins = []
only_plugins = []
exclude_records = []   # FormIDs, eight hex digits
exclude_models = []    # patterns relative to meshes/, `*` matches any run

[output]
terrain = true
land_textures = true
water = true
objects = true
trees = true
```

The output folder (`<instance>/.raul/lod/out`) may not lie inside a git working tree (a `.git` in it or any folder
above it): `raul lod plan` and `raul lod generate` refuse it with `plan_refused`, and lodwright refuses it too. Either
`allow_in_repo = true` in the Profile's `raul-lod.toml` or `--allow-in-repo` on the command allows it; `raul` then
skips its own check and tells lodwright it is allowed. The flag applies to that run only and writes nothing to the
settings. `allow_in_repo` is not part of the request, so it does not change the request digest, and it is not a LOD
generation setting. An output folder inside the data folder stays refused either way.

`raul-lod.toml` refuses keys it does not know. A file that uses a key added in a later `raul` (such as
`allow_in_repo`) cannot be read by an earlier one, which refuses to plan until the key is removed.

`contracts/fixtures/lodwright-request/profile-settings.json` is the request of a Profile with the settings in its
test, with the two folders written as `<data>` and `<output>`.

## `raul-lod-result`

`raul lod generate` builds the request (above) and calls lodwright's library API (`lodwright-api`) in process with it,
mapping lodwright's progress to `raul-events` lines; it reads the manifest in lodwright's output folder and keeps the
output as the Profile's LOD Mod, `Generated LOD (<Profile>)`: one Mod for every world of that Profile. Its `meta.ini`
carries the marker section `[raulGeneratedLod]` (`generated=true`, `profile`, `requestDigest`, `lodwrightVersion`);
every other key of the file is kept, so a category the user gave the Mod survives the next run. A new Mod goes on top
of the Profile, enabled; a run that replaces the Mod keeps its place and its enabled state, so a disabled LOD Mod stays
disabled. A Mod named `LOD (lodwright)`, the name every Profile shared before, is generated LOD too: the next run of a
Profile that lists it gives the Profile's own Mod its place and state, then removes it, or only disables it in that
Profile while another Profile lists it. A Mod of the new name that is not generated LOD is refused (`plan_refused`)
and nothing changes. A second run over an unchanged Load order: lodwright answers `up_to_date`, the Mod is already
there, and nothing is written (`up_to_date: true`, `changed: false`); a removed Mod is made again. lodwright's block progress becomes `progress` events of the phase `terrain`.

| `status` | Exit code | `error.code` | lodwright |
| --- | --- | --- | --- |
| `success` | 0 | | exit code 0; the Mod holds the output (or a dry run, or `up_to_date`) |
| `refused` | 2 | `instance_invalid`, `profile_missing`, `plan_refused` | not started |
| `refused` | 2 | `contract_refused` | the output folder holds a manifest of a newer version |
| `failed` | 4 | `lodwright_failed` | lodwright's generation returned an error; `lodwright_code` is its code |
| `failed` | 4 | `tool_failed` | stopped unexpectedly, ran past the timeout, or reported a status other than `ok` |
| `failed` | 4 | `add_mod_failed` | the output was written, but replacing the Mod failed and was rolled back |
| `cancelled` | 4 | `cancelled` | stopped by `raul` on SIGINT or SIGTERM |

In every case but `success` the Instance is as it was before the command. lodwright keeps its own output folder
between runs (that is how it knows a later run is `up_to_date`); the Mod is a copy of it.

The tests run `raul lod generate` with the linked lodwright on a synthetic world (`crates/raul-cli/tests/lod_generate.rs`),
and the cancel and resume cases call the same generation in process with a cancel token.

## `raul-lod-mods`

`raul lod mods --json` lists the generated LOD Mods a Profile has in its mod list, highest priority first, and writes
nothing. A Mod is generated LOD when its `meta.ini` carries the `[raulGeneratedLod]` marker, or when it has the name
every Profile shared before, `LOD (lodwright)` (`legacy: true`, and `profile`, `request_digest` and
`lodwright_version` are `null`). `own` marks the Profile's own Mod, `own_name`, which `raul lod generate` replaces; a
Profile may also list another Profile's LOD Mod. `position` counts separators, as `raul mods list` does. An Instance or
Profile that cannot be read is refused (exit code 2) with one line on standard error and no document.
