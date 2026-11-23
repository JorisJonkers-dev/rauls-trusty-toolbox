// SPDX-License-Identifier: GPL-3.0-only
//! Free space of the storage a folder is on, for the checks before a large install (the TTW output is tens of GB).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use sysinfo::Disks;

/// The free space where a folder is, or would be made.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Space {
    /// The folder that was asked about.
    pub folder: PathBuf,
    /// Bytes a new file can use on that storage.
    pub free_bytes: u64,
    /// Size of that storage in bytes.
    pub total_bytes: u64,
}

/// Why the free space of a folder is not known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpaceError {
    /// No folder on the way up from it exists.
    NoFolder,
    /// The storage the folder is on is not in the system's list.
    Unknown,
}

impl std::fmt::Display for SpaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NoFolder => "no part of that folder exists",
            Self::Unknown => "the free space of that storage is not known",
        })
    }
}

impl std::error::Error for SpaceError {}

/// The free space on the storage of `folder`. A folder that does not exist yet is looked up by its nearest existing
/// parent, because that is the storage it will be made on.
///
/// # Errors
/// [`SpaceError::NoFolder`] when nothing on the way up exists, [`SpaceError::Unknown`] when the system lists no
/// storage that holds it.
pub fn free_space(folder: &Path) -> Result<Space, SpaceError> {
    let existing = folder
        .ancestors()
        .find(|p| !p.as_os_str().is_empty() && p.exists())
        .ok_or(SpaceError::NoFolder)?;
    let real = plain(existing.canonicalize().map_err(|_| SpaceError::NoFolder)?);
    let disks = Disks::new_with_refreshed_list().map_err(|_| SpaceError::Unknown)?;
    let best = disks
        .iter()
        .filter(|d| real.starts_with(d.mount_point()))
        .max_by_key(|d| d.mount_point().components().count())
        .ok_or(SpaceError::Unknown)?;
    Ok(Space {
        folder: folder.to_path_buf(),
        free_bytes: best.available_space(),
        total_bytes: best.total_space(),
    })
}

/// The free bytes on the storage of `folder`, or `None` when they are not known: the form the TTW install takes its
/// check in (`raul_drivers::ttw::FreeSpace`), where an unknown answer does not block.
#[must_use]
pub fn free_bytes(folder: &Path) -> Option<u64> {
    free_space(folder).ok().map(|space| space.free_bytes)
}

/// A Windows path without the verbatim prefix `canonicalize` adds, because the system lists its drives as `C:\`.
fn plain(path: PathBuf) -> PathBuf {
    match path.to_str().and_then(|p| p.strip_prefix(r"\\?\")) {
        Some(rest) if !rest.starts_with("UNC") => PathBuf::from(rest),
        _ => path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_synthetic_folder_reports_the_space_of_its_storage() {
        let dir = std::env::temp_dir().join(format!("raul-space-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let s = free_space(&dir).unwrap();
        assert_eq!(s.folder, dir);
        assert!(s.total_bytes > 0 && s.free_bytes <= s.total_bytes);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_folder_not_made_yet_uses_its_nearest_parent() {
        let dir = std::env::temp_dir().join("raul-space-not-yet/a/b");
        let s = free_space(&dir).unwrap();
        assert_eq!(s.folder, dir);
        assert!(s.total_bytes > 0);
    }

    #[test]
    fn the_verbatim_prefix_of_a_windows_path_is_dropped() {
        assert_eq!(
            plain(PathBuf::from(r"\\?\C:\a\b")),
            PathBuf::from(r"C:\a\b")
        );
        assert_eq!(plain(PathBuf::from("/a/b")), PathBuf::from("/a/b"));
        let unc = PathBuf::from(r"\\?\UNC\host\share");
        assert_eq!(plain(unc.clone()), unc);
    }

    #[test]
    fn a_relative_path_with_no_existing_part_is_refused() {
        assert_eq!(free_space(Path::new("")).unwrap_err(), SpaceError::NoFolder);
    }
}
