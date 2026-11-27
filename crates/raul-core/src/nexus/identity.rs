// SPDX-License-Identifier: GPL-3.0-only
//! What `settings.toml` remembers about the Nexus account: the name, whether it is premium and when the key was last
//! validated. Never the key (docs/adr/0024).

use std::path::Path;

use serde::{Deserialize, Serialize};

/// The `[nexus]` table of `settings.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    /// The account name.
    pub user_name: String,
    /// Whether the account is premium.
    pub is_premium: bool,
    /// When the key was last validated, in seconds since the Unix epoch.
    pub validated_at: u64,
}

const TABLE: &str = "nexus";

impl Identity {
    /// The identity in `path`, or `None` when the file or the table is missing or unreadable.
    #[must_use]
    pub fn load(path: &Path) -> Option<Identity> {
        let table: toml::Table = std::fs::read_to_string(path).ok()?.parse().ok()?;
        table.get(TABLE)?.clone().try_into().ok()
    }

    /// Writes this identity into `path`, keeping every other key, and creating the folder.
    ///
    /// # Errors
    /// When the file does not parse (it is left as it is) or cannot be written.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        edit(path, |table| {
            let value = toml::Value::try_from(self).map_err(std::io::Error::other)?;
            table.insert(TABLE.into(), value);
            Ok(())
        })
    }

    /// Removes the `[nexus]` table from `path`, keeping every other key; a missing file is fine.
    ///
    /// # Errors
    /// When the file does not parse or cannot be written.
    pub fn forget(path: &Path) -> std::io::Result<()> {
        if !path.exists() {
            return Ok(());
        }
        edit(path, |table| {
            table.remove(TABLE);
            Ok(())
        })
    }
}

fn edit(
    path: &Path,
    change: impl FnOnce(&mut toml::Table) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let mut table = match std::fs::read_to_string(path) {
        Ok(text) => text
            .parse::<toml::Table>()
            .map_err(|e| std::io::Error::other(format!("{}: {e}", path.display())))?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => toml::Table::new(),
        Err(e) => return Err(e),
    };
    change(&mut table)?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(
        path,
        toml::to_string(&table).map_err(std::io::Error::other)?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> Identity {
        Identity {
            user_name: "Courier".into(),
            is_premium: true,
            validated_at: 1_700_000_000,
        }
    }

    #[test]
    fn the_identity_is_saved_beside_the_other_settings_and_read_back() {
        let dir = std::env::temp_dir().join(format!("raul-identity-{}", std::process::id()));
        let file = dir.join("settings.toml");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&file, "check_for_updates = true\n").unwrap();
        identity().save(&file).unwrap();
        assert_eq!(Identity::load(&file), Some(identity()));
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(text.contains("check_for_updates = true"), "{text}");
        Identity::forget(&file).unwrap();
        assert_eq!(Identity::load(&file), None);
        assert!(
            std::fs::read_to_string(&file)
                .unwrap()
                .contains("check_for_updates")
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
