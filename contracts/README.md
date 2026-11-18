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

`raul lod plan [--instance <dir>] [--profile <name>] [--world <name>]... [--json]` prints the `lodwright-request` v1
document `raul lod generate` will pass to `lodwright generate --request <file>`. It is lodwright's Contract; `raul`
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

`contracts/fixtures/lodwright-request/profile-settings.json` is the request of a Profile with the settings in its
test, with the two folders written as `<data>` and `<output>`.

## `raul-lod-result`

`raul lod generate` builds the request (above), writes it to `<instance>/.raul/lod/request.json`, runs
`lodwright generate --request <file> --json --events <file>`, reads the `lodwright-result` and the manifest, and keeps
the output as the Mod `LOD (lodwright)`, enabled in the Profile, so the next run loads it. A second run over an
unchanged Load order: lodwright answers `up_to_date`, the Mod is already there, and nothing is written
(`up_to_date: true`, `changed: false`). lodwright's block progress becomes `progress` events of the phase `terrain`.

| `status` | Exit code | `error.code` | lodwright |
| --- | --- | --- | --- |
| `success` | 0 | | exit code 0; the Mod holds the output (or a dry run, or `up_to_date`) |
| `refused` | 2 | `instance_invalid`, `profile_missing`, `plan_refused`, `tool_missing` | not started |
| `refused` | 2 | `contract_refused` | printed a `lodwright-result`, or left a manifest, of a newer version |
| `unsupported` | 2 | `unsupported` | exit code 3; nothing was written |
| `unverifiable` | 3 | `tool_unverifiable` | changed after its checksum was taken; not started |
| `failed` | 4 | `lodwright_failed` | exit code 1, or 2 (a usage error) |
| `failed` | 4 | `tool_failed` | could not start, was ended by a signal, or printed no valid result |
| `failed` | 4 | `add_mod_failed` | exit code 0, but replacing the Mod failed and was rolled back |
| `cancelled` | 4 | `cancelled` | stopped by `raul` on SIGINT or SIGTERM |

In every case but `success` the Instance is as it was before the command. lodwright keeps its own output folder
between runs (that is how it knows a later run is `up_to_date`); the Mod is a copy of it.

The tests run `raul lod generate` against a fake lodwright for every outcome, and CI runs it once more against the
real lodwright program of the shipped Component set on a synthetic world (`crates/raul-cli/tests/lod_generate_real.rs`,
behind the `real-lodwright` feature).
