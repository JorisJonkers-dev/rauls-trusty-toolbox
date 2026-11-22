// SPDX-License-Identifier: GPL-3.0-only
//! The Toolbox's data folder: where `raul` keeps what it makes for the person to look at, such as the pictures of
//! `raul scene` (the configuration folder holds settings, this one holds results).

use std::path::PathBuf;

/// The variable that names the data folder instead of the platform's own (for tests and portable setups).
pub const FOLDER_ENV: &str = "RAUL_DATA_DIR";

/// The data folder: `$RAUL_DATA_DIR`, else `raul` under the platform's data folder (`$XDG_DATA_HOME` or
/// `~/.local/share`, `%LOCALAPPDATA%`, `~/Library/Application Support`). `None` when the environment names no absolute
/// folder.
#[must_use]
pub fn folder() -> Option<PathBuf> {
    folder_from(|k| {
        std::env::var_os(k)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    })
}

/// [`folder`] reading its variables through `env`.
#[must_use]
pub fn folder_from(env: impl Fn(&str) -> Option<PathBuf>) -> Option<PathBuf> {
    if let Some(dir) = env(FOLDER_ENV) {
        return Some(dir).filter(|d| d.is_absolute());
    }
    let base = if cfg!(windows) {
        env("LOCALAPPDATA")
    } else if cfg!(target_os = "macos") {
        env("HOME").map(|h| h.join("Library/Application Support"))
    } else {
        env("XDG_DATA_HOME")
            .filter(|d| d.is_absolute())
            .or_else(|| env("HOME").map(|h| h.join(".local/share")))
    };
    base.filter(|b| b.is_absolute()).map(|b| b.join("raul"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_named_folder_wins_and_a_relative_one_is_refused() {
        let named = std::env::temp_dir().join("the-data");
        let found = folder_from(|k| (k == FOLDER_ENV).then(|| named.clone()));
        assert_eq!(found, Some(named));
        let relative = folder_from(|k| (k == FOLDER_ENV).then(|| PathBuf::from("data")));
        assert_eq!(relative, None);
    }

    #[test]
    fn without_a_variable_the_platform_folder_is_used() {
        let home = std::env::temp_dir().join("home-of-p");
        let found = folder_from(|k| match k {
            "XDG_DATA_HOME" | "HOME" | "LOCALAPPDATA" => Some(home.clone()),
            _ => None,
        });
        let found = found.unwrap();
        assert!(found.ends_with("raul"));
        assert!(found.starts_with(&home));
        assert_eq!(folder_from(|_| None), None);
    }
}
