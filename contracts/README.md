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

`snapshots/` holds the schemas of the Component Contracts the Toolbox reads, as their releases published them:
`snapshots/ttwright-rs/` is ttwright-rs v0.3.0 (`ttwright-result`, `ttwright-events`, `ttwright-contracts`, all v1).

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

`raul ttw --json` prints one `raul-events` line per event on standard output while it runs. Each line has
`schema_version`, `contract`, `seq` (from 1), `command` and `event`:

| `event` | Fields | When |
| --- | --- | --- |
| `started` | `dry_run`, `resume`, `instance`, `profile`, `tool` (`name`, `version` (`null` until the result names it), `path`, `sha256`) | the checks passed and the program starts |
| `phase` | `phase` | a phase began: ttwright's `package`, `preflight`, `checks`, `assets`, `post_install`, then `add_mod` |
| `progress` | `phase`, `done`, `of` (0 when unknown), `message` | progress within the phase |
| `warning` | `message` | something went wrong, but the run goes on |
| `result` | `result` | the command's result document (`raul-ttw-result`) |
| `finished` | `status`, `exit_code`, `code`, `message`, `resumable`, `changed` | always the last line |

A command refused before it starts prints only `result` and `finished`. The process exits with `finished.exit_code`.
Standard error carries the same story as text for people.

## `raul-ttw-result`

`raul ttw` installs TTW with ttwright-rs and adds its output to the Instance as the Mod `Tale of Two Wastelands`,
with its plugins enabled in the Profile. The result names the Mod, the plugins it enabled, the ttwright program
that ran (with its SHA-256), the `ttwright-result` document ttwright printed, and the staging folder ttwright
wrote to. A cancelled or failed install keeps the staging folder, so `raul ttw --resume` continues there.

| `status` | Exit code | `error.code` | ttwright |
| --- | --- | --- | --- |
| `success` | 0 | | exit code 0, and the Mod was added (or a dry run) |
| `refused` | 2 | `instance_invalid`, `profile_missing`, `missing_input`, `mod_exists`, `tool_missing` | not started |
| `refused` | 2 | `contract_refused` | printed a `ttwright-result` of a newer version |
| `unsupported` | 2 | `unsupported` | exit code 3 (`unsupported_feature`, `unsupported_version`); nothing was written |
| `unverifiable` | 3 | `tool_unverifiable` | changed after its checksum was taken; not started |
| `failed` | 4 | `ttwright_failed` | exit code 1, or 2 (a usage error) |
| `failed` | 4 | `tool_failed` | could not start, was ended by a signal, or printed no valid result |
| `failed` | 4 | `add_mod_failed` | exit code 0, but adding the Mod failed and was rolled back |
| `cancelled` | 4 | `cancelled` | exit code 4, or stopped by `raul` on SIGINT or SIGTERM; `--resume` continues |

`error.ttwright_code` is ttwright's own `error.code` when ttwright gave one. In every case but `success` the
Instance is as it was before the command. A command line `raul` cannot read exits 5 and prints no events.
