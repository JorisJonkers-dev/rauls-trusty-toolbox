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

The file itself is an on-disk document with `schema_version = 1`. Keys this build does not know survive a `set`.
