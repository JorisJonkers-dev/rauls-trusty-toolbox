// SPDX-License-Identifier: GPL-3.0-only
//! The Toolbox's configuration: the paths the GUI needs and nothing else (the game folders, the TTW package and the
//! storage folder). This module holds the `raul-config` v1 Contract, the document `raul config get|set|list --json`
//! prints; its schema is `contracts/raul-config.v1.json` at the repository root.

use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The Contract name of the document `raul config --json` prints.
pub const CONTRACT: &str = "raul-config";

/// The version of that document this build prints.
pub const SCHEMA_VERSION: u32 = 1;

/// One configuration key.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
pub enum Key {
    /// The Fallout: New Vegas folder (the one holding `FalloutNV.exe`).
    #[serde(rename = "game.newvegas")]
    GameNewVegas,
    /// The Fallout 3 folder (the one holding `Fallout3.exe`); TTW needs it.
    #[serde(rename = "game.fallout3")]
    GameFallout3,
    /// The TTW installer package the player downloaded (the `.mpi` file).
    #[serde(rename = "ttw.package")]
    TtwPackage,
    /// The Toolbox storage folder, which holds the Instance.
    #[serde(rename = "storage.root")]
    StorageRoot,
    /// The Instance folder: `<storage.root>/instance`. Derived, read only.
    #[serde(rename = "instance")]
    Instance,
}

impl Key {
    /// Every key, in the order `raul config list` prints them.
    pub const ALL: [Key; 5] = [
        Key::GameNewVegas,
        Key::GameFallout3,
        Key::TtwPackage,
        Key::StorageRoot,
        Key::Instance,
    ];

    /// The key as written on the command line, for example `game.newvegas`.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Key::GameNewVegas => "game.newvegas",
            Key::GameFallout3 => "game.fallout3",
            Key::TtwPackage => "ttw.package",
            Key::StorageRoot => "storage.root",
            Key::Instance => "instance",
        }
    }

    /// The key named `name`, if there is one.
    #[must_use]
    pub fn parse(name: &str) -> Option<Key> {
        Key::ALL.into_iter().find(|k| k.name() == name)
    }

    /// Whether the key is derived from others and cannot be set.
    #[must_use]
    pub fn read_only(self) -> bool {
        self == Key::Instance
    }
}

/// One key and its value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Entry {
    /// The key.
    pub key: Key,
    /// The value: an absolute path, or `null` when it is not set.
    pub value: Option<PathBuf>,
    /// Whether the key is derived and `raul config set` refuses it.
    pub read_only: bool,
}

/// Why a `raul config` command did not do what it was asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// The key is not one of the keys this build knows (exit code 5).
    UnknownKey,
    /// The key is derived and cannot be set (exit code 2).
    ReadOnly,
    /// The value is not an acceptable path for the key: not a game copy of that game, not a file, or not a folder
    /// that can hold the storage (exit code 2). Nothing was written.
    InvalidValue,
    /// The file has a `schema_version` newer than this build reads (exit code 2). Nothing was written.
    NewerSchema,
    /// The file cannot be read or is not a valid configuration file (exit code 2). Nothing was written.
    Unreadable,
    /// No configuration folder could be found: `HOME`, `XDG_CONFIG_HOME` or `APPDATA` is not set (exit code 2).
    NoConfigFolder,
    /// The file could not be written (exit code 4); the old file is left as it was.
    WriteFailed,
}

/// The error of a `raul config` command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ErrorInfo {
    /// The stable code callers match on.
    pub code: ErrorCode,
    /// The key the command was about, when there was one.
    pub key: Option<String>,
    /// Prose for people; never matched on.
    pub message: String,
}

/// The document `raul config list|get|set --json` prints (Contract `raul-config`, version 1).
///
/// `list` prints every key, in the order `game.newvegas`, `game.fallout3`, `ttw.package`, `storage.root`, `instance`;
/// `get` and `set` print the one key they were about. When the command fails, `entries` is empty and `error` says
/// why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigDoc {
    /// The document version; 1.
    #[schemars(extend("const" = 1))]
    pub schema_version: u32,
    /// Always `raul-config`.
    #[schemars(extend("const" = "raul-config"))]
    pub contract: String,
    /// The configuration file (`config.toml` in the Toolbox's configuration folder), or `null` when there is no
    /// configuration folder. The file need not exist: a missing file means every key is unset.
    pub file: Option<PathBuf>,
    /// The keys, with their values.
    pub entries: Vec<Entry>,
    /// Set when the command failed.
    pub error: Option<ErrorInfo>,
}

impl ConfigDoc {
    /// A document that carries `entries`.
    #[must_use]
    pub fn new(file: Option<PathBuf>, entries: Vec<Entry>) -> ConfigDoc {
        ConfigDoc {
            schema_version: SCHEMA_VERSION,
            contract: CONTRACT.to_owned(),
            file,
            entries,
            error: None,
        }
    }

    /// A document that says why the command failed.
    #[must_use]
    pub fn failed(file: Option<PathBuf>, error: ErrorInfo) -> ConfigDoc {
        ConfigDoc {
            error: Some(error),
            ..ConfigDoc::new(file, Vec::new())
        }
    }
}

/// The JSON Schema of [`ConfigDoc`], as `contracts/raul-config.v1.json` holds it.
#[must_use]
pub fn schema() -> String {
    crate::contracts::schema_text(&schemars::schema_for!(ConfigDoc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_reads_back_from_its_name_and_only_the_instance_is_read_only() {
        for key in Key::ALL {
            assert_eq!(Key::parse(key.name()), Some(key));
            assert_eq!(
                serde_json::to_value(key).unwrap(),
                serde_json::Value::from(key.name())
            );
            assert_eq!(key.read_only(), key == Key::Instance);
        }
        assert_eq!(Key::parse("game.oblivion"), None);
    }

    #[test]
    fn a_failed_document_has_no_entries_and_names_the_error() {
        let ok = ConfigDoc::new(None, Vec::new());
        assert_eq!((ok.schema_version, ok.contract.as_str()), (1, CONTRACT));
        let doc = ConfigDoc::failed(
            Some(PathBuf::from("/c/config.toml")),
            ErrorInfo {
                code: ErrorCode::ReadOnly,
                key: Some("instance".into()),
                message: "derived".into(),
            },
        );
        assert_eq!(doc.entries.len(), 0);
        assert_eq!(doc.error.unwrap().code, ErrorCode::ReadOnly);
    }
}
