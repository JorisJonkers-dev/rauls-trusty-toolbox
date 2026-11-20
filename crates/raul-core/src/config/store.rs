// SPDX-License-Identifier: GPL-3.0-only
//! The configuration file, `config.toml` in the Toolbox's configuration folder: an on-disk document with an integer
//! `schema_version` (ADR-0009). A file with a newer version is refused and left as it is; keys this build does not
//! know survive a write. `raul config set` is its only writer (the one-writer rule); the GUI reads it with
//! [`Config::load`].

use std::fmt;
use std::path::{Path, PathBuf};

use super::{Entry, ErrorCode, ErrorInfo, Key, SCHEMA_VERSION};
use crate::detect::{Game, inspect};

/// The file's name inside the configuration folder.
pub const FILE_NAME: &str = "config.toml";

/// The variable that names the configuration folder instead of the platform's own (for tests and portable setups).
pub const FOLDER_ENV: &str = "RAUL_CONFIG_DIR";

/// The Toolbox's configuration folder: `$RAUL_CONFIG_DIR`, else `raul` under the platform's configuration folder
/// (`$XDG_CONFIG_HOME` or `~/.config`, `%APPDATA%`, `~/Library/Application Support`). `None` when the environment
/// names no absolute folder (a relative fallback would scatter files across working folders).
#[must_use]
pub fn folder() -> Option<PathBuf> {
    let env = |k| {
        std::env::var_os(k)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    if let Some(dir) = env(FOLDER_ENV) {
        return Some(dir).filter(|d| d.is_absolute());
    }
    let base = if cfg!(windows) {
        env("APPDATA")
    } else if cfg!(target_os = "macos") {
        env("HOME").map(|h| h.join("Library/Application Support"))
    } else {
        env("XDG_CONFIG_HOME")
            .filter(|d| d.is_absolute())
            .or_else(|| env("HOME").map(|h| h.join(".config")))
    };
    base.filter(|b| b.is_absolute()).map(|b| b.join("raul"))
}

/// The configuration file under [`folder`].
#[must_use]
pub fn default_file() -> Option<PathBuf> {
    folder().map(|f| f.join(FILE_NAME))
}

/// Why the configuration could not be read, changed or written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// The file's `schema_version` is newer than this build reads.
    NewerSchema {
        /// The file.
        file: PathBuf,
        /// Its version.
        found: i64,
    },
    /// The file cannot be read or is not a configuration file.
    Unreadable(String),
    /// The key is derived and cannot be set.
    ReadOnly(Key),
    /// The value is not acceptable for the key.
    InvalidValue {
        /// The key.
        key: Key,
        /// Why, for people.
        reason: String,
    },
    /// The file could not be written.
    WriteFailed(String),
}

impl ConfigError {
    /// The stable code of the error in the `raul-config` Contract.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            ConfigError::NewerSchema { .. } => ErrorCode::NewerSchema,
            ConfigError::Unreadable(_) => ErrorCode::Unreadable,
            ConfigError::ReadOnly(_) => ErrorCode::ReadOnly,
            ConfigError::InvalidValue { .. } => ErrorCode::InvalidValue,
            ConfigError::WriteFailed(_) => ErrorCode::WriteFailed,
        }
    }

    /// The error as the Contract carries it.
    #[must_use]
    pub fn info(&self) -> ErrorInfo {
        let key = match self {
            ConfigError::ReadOnly(key) | ConfigError::InvalidValue { key, .. } => {
                Some(key.name().to_owned())
            }
            _ => None,
        };
        ErrorInfo {
            code: self.code(),
            key,
            message: self.to_string(),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::NewerSchema { file, found } => write!(
                f,
                "{} has schema_version {found}; this raul reads {SCHEMA_VERSION}. Update raul; the file was left as it is",
                file.display()
            ),
            ConfigError::Unreadable(why) | ConfigError::WriteFailed(why) => f.write_str(why),
            ConfigError::ReadOnly(key) => write!(
                f,
                "{} is derived from storage.root and cannot be set",
                key.name()
            ),
            ConfigError::InvalidValue { reason, .. } => f.write_str(reason),
        }
    }
}

impl std::error::Error for ConfigError {}

/// The configuration as read from its file.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    file: PathBuf,
    table: toml::Table,
}

/// The table and field a settable key is stored under: `game.newvegas` is `newvegas` in `[game]`.
fn location(key: Key) -> Option<(&'static str, &'static str)> {
    match key {
        Key::GameNewVegas => Some(("game", "newvegas")),
        Key::GameFallout3 => Some(("game", "fallout3")),
        Key::TtwPackage => Some(("ttw", "package")),
        Key::StorageRoot => Some(("storage", "root")),
        Key::Instance => None,
    }
}

/// The folder under `storage.root` that holds the Instance.
pub const INSTANCE_FOLDER: &str = "instance";

impl Config {
    /// Read `file`. A missing file is an empty configuration (every key unset).
    ///
    /// # Errors
    ///
    /// [`ConfigError::NewerSchema`] when the file's `schema_version` is newer than this build reads, and
    /// [`ConfigError::Unreadable`] when it cannot be read, is not TOML, or holds a known key that is not a path.
    pub fn load(file: &Path) -> Result<Config, ConfigError> {
        let text = match std::fs::read_to_string(file) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => {
                return Err(ConfigError::Unreadable(format!("{}: {e}", file.display())));
            }
        };
        let table: toml::Table = text
            .parse()
            .map_err(|e| ConfigError::Unreadable(format!("{}: {e}", file.display())))?;
        // Version 1 is the first: a file without a version is read as version 1, a newer one is refused, and
        // anything else (0, a negative number, a string) is not a file this raul wrote.
        match table.get("schema_version") {
            None => {}
            Some(toml::Value::Integer(v)) if *v == i64::from(SCHEMA_VERSION) => {}
            Some(toml::Value::Integer(v)) if *v > i64::from(SCHEMA_VERSION) => {
                return Err(ConfigError::NewerSchema {
                    file: file.to_path_buf(),
                    found: *v,
                });
            }
            Some(other) => {
                return Err(ConfigError::Unreadable(format!(
                    "{}: schema_version {other} is not a version this raul knows",
                    file.display()
                )));
            }
        }
        let config = Config {
            file: file.to_path_buf(),
            table,
        };
        for key in Key::ALL {
            config.stored(key)?;
        }
        Ok(config)
    }

    /// The file this configuration was read from and is written to.
    #[must_use]
    pub fn file(&self) -> &Path {
        &self.file
    }

    fn stored(&self, key: Key) -> Result<Option<PathBuf>, ConfigError> {
        let Some((table, field)) = location(key) else {
            return Ok(None);
        };
        let Some(section) = self.table.get(table) else {
            return Ok(None);
        };
        let not_a_path = || {
            ConfigError::Unreadable(format!(
                "{}: {} is not a path",
                self.file.display(),
                key.name()
            ))
        };
        let section = section.as_table().ok_or_else(|| {
            ConfigError::Unreadable(format!("{}: [{table}] is not a table", self.file.display()))
        })?;
        match section.get(field) {
            None => Ok(None),
            Some(toml::Value::String(s)) => Ok(Some(PathBuf::from(s))),
            Some(_) => Err(not_a_path()),
        }
    }

    /// The value of `key`, or `None` when it is not set. `instance` is `<storage.root>/instance`.
    #[must_use]
    pub fn get(&self, key: Key) -> Option<PathBuf> {
        match key {
            Key::Instance => self
                .get(Key::StorageRoot)
                .map(|root| root.join(INSTANCE_FOLDER)),
            _ => self.stored(key).ok().flatten(),
        }
    }

    /// `key` with its value, as the Contract carries it.
    #[must_use]
    pub fn entry(&self, key: Key) -> Entry {
        Entry {
            key,
            value: self.get(key),
            read_only: key.read_only(),
        }
    }

    /// Every key with its value, in the order of [`Key::ALL`].
    #[must_use]
    pub fn entries(&self) -> Vec<Entry> {
        Key::ALL.into_iter().map(|k| self.entry(k)).collect()
    }

    /// Set `key` to `value` after checking it (see [`check`]); the file is not written until [`Config::save`].
    ///
    /// # Errors
    ///
    /// [`ConfigError::ReadOnly`] for `instance`, [`ConfigError::InvalidValue`] when [`check`] refuses the value.
    pub fn set(&mut self, key: Key, value: &Path) -> Result<PathBuf, ConfigError> {
        let (table, field) = location(key).ok_or(ConfigError::ReadOnly(key))?;
        let value =
            check(key, value).map_err(|reason| ConfigError::InvalidValue { key, reason })?;
        let text = value
            .to_str()
            .ok_or_else(|| ConfigError::InvalidValue {
                key,
                reason: format!("{} is not valid Unicode", value.display()),
            })?
            .to_owned();
        let section = self
            .table
            .entry(table)
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        let toml::Value::Table(section) = section else {
            return Err(ConfigError::Unreadable(format!(
                "{}: [{table}] is not a table",
                self.file.display()
            )));
        };
        section.insert(field.to_owned(), toml::Value::String(text));
        Ok(value)
    }

    /// Clear `key`; the file is not written until [`Config::save`].
    ///
    /// # Errors
    ///
    /// [`ConfigError::ReadOnly`] for `instance`.
    pub fn unset(&mut self, key: Key) -> Result<(), ConfigError> {
        let (table, field) = location(key).ok_or(ConfigError::ReadOnly(key))?;
        if let Some(toml::Value::Table(section)) = self.table.get_mut(table) {
            section.remove(field);
            if section.is_empty() {
                self.table.remove(table);
            }
        }
        Ok(())
    }

    /// Write the configuration to its file with `schema_version = 1`, keeping every key this build does not
    /// know. The file is replaced in one step (a temporary file next to it, then a rename), so a failed write
    /// leaves the old file as it was.
    ///
    /// # Errors
    ///
    /// [`ConfigError::WriteFailed`] when the folder or the file cannot be written.
    pub fn save(&self) -> Result<(), ConfigError> {
        let failed = |e: &dyn fmt::Display, path: &Path| {
            ConfigError::WriteFailed(format!("{}: {e}", path.display()))
        };
        let mut table = toml::Table::new();
        table.insert(
            "schema_version".into(),
            toml::Value::Integer(i64::from(SCHEMA_VERSION)),
        );
        for (k, v) in &self.table {
            if k != "schema_version" {
                table.insert(k.clone(), v.clone());
            }
        }
        let text = toml::to_string(&table).map_err(|e| failed(&e, &self.file))?;
        if let Some(dir) = self.file.parent() {
            std::fs::create_dir_all(dir).map_err(|e| failed(&e, dir))?;
        }
        // A temporary file of this process next to the real one, created new (never through an existing file
        // or link), synced, then renamed over the real one.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos());
        let mut temporary = self.file.clone().into_os_string();
        temporary.push(format!(".{}-{nanos}.new", std::process::id()));
        let temporary = PathBuf::from(temporary);
        let written = (|| {
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            std::io::Write::write_all(&mut f, text.as_bytes())?;
            f.sync_all()?;
            std::fs::rename(&temporary, &self.file)
        })();
        written.map_err(|e| {
            let _ = std::fs::remove_file(&temporary);
            failed(&e, &self.file)
        })
    }
}

/// Check `value` for `key` and return it as an absolute path (a relative one is taken against the working folder).
///
/// - `game.newvegas` and `game.fallout3`: a folder that `raul detect --folder` accepts as that game.
/// - `ttw.package`: the TTW package by content: a `.mpi` file, or an archive that holds exactly one.
/// - `storage.root`: not an existing file (the folder is created when something is stored in it).
///
/// # Errors
///
/// Why the value is refused, for people.
pub fn check(key: Key, value: &Path) -> Result<PathBuf, String> {
    if value.as_os_str().is_empty() {
        return Err(format!("{} needs a path", key.name()));
    }
    let path = std::path::absolute(value).map_err(|e| format!("{}: {e}", value.display()))?;
    match key {
        Key::GameNewVegas | Key::GameFallout3 => {
            let (wanted, title) = if key == Key::GameNewVegas {
                (Game::NewVegas, "Fallout: New Vegas")
            } else {
                (Game::Fallout3, "Fallout 3")
            };
            match inspect(&path) {
                Some(copy) if copy.game == wanted => Ok(path),
                Some(_) => Err(format!(
                    "{} holds the other game, not {title}",
                    path.display()
                )),
                None => Err(format!(
                    "{} is not a {title} folder: `raul detect --folder` finds no game there",
                    path.display()
                )),
            }
        }
        // The one rule of `raul ttw`: the `.mpi` itself (it starts with `BSA\0`, whatever it is called), or the archive
        // that was downloaded, which holds it.
        Key::TtwPackage => raul_drivers::ttw::package::inspect(&path)
            .map(|_| path)
            .map_err(|why| why.to_string()),
        Key::StorageRoot if path.exists() && !path.is_dir() => Err(format!(
            "{} is a file; storage.root must be a folder",
            path.display()
        )),
        Key::StorageRoot => Ok(path),
        Key::Instance => Err(ConfigError::ReadOnly(key).to_string()),
    }
}
