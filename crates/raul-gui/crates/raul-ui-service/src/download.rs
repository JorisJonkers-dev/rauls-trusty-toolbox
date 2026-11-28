// SPDX-License-Identifier: GPL-3.0-only
//! Finding the TTW package a player downloaded, for the TTW page's "Find it for me" and its Downloads watch.
//!
//! What counts is decided by the one content rule of `raul ttw` (`raul_drivers::ttw::package::inspect`): a file that
//! starts like a package, or an archive that holds exactly one. The name only narrows what is looked at: any `.mpi`
//! file, and a `.7z` or `.zip` whose name says TTW. An archive that holds no package (the YUPTTW patch archive in the
//! same folder) is skipped, and the search goes on to the next file.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use raul_drivers::ttw::package::{Package, inspect};
use raul_ui_model::pages::ttw::{MIN_PACKAGE_BYTES, PackageInfo};

/// Sizes at which a file was last refused, so a watch does not read the same archive again every second.
pub type Refused = HashMap<PathBuf, u64>;

fn extension_is(path: &Path, wanted: &[&str]) -> bool {
    path.extension()
        .is_some_and(|e| wanted.iter().any(|w| e.eq_ignore_ascii_case(w)))
}

/// Whether a file of this name is worth looking into.
fn worth_a_look(path: &Path) -> bool {
    if extension_is(path, &["mpi"]) {
        return true;
    }
    if !extension_is(path, &["7z", "zip"]) {
        return false;
    }
    let name = path
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().to_lowercase());
    name.contains("ttw") || name.contains("tale of two wastelands")
}

/// Files of `folder` and of the folders directly under it that are worth a look, newest first.
fn candidates(folders: &[PathBuf]) -> Vec<(SystemTime, PathBuf, u64)> {
    let mut found = Vec::new();
    for folder in folders {
        let mut dirs = vec![folder.clone()];
        if let Ok(entries) = std::fs::read_dir(folder) {
            dirs.extend(
                entries
                    .flatten()
                    .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
                    .map(|e| e.path()),
            );
        }
        for dir in dirs {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if !worth_a_look(&path) {
                    continue;
                }
                let Ok(meta) = entry.metadata() else {
                    continue;
                };
                if !meta.is_file() {
                    continue;
                }
                let Ok(modified) = meta.modified() else {
                    continue;
                };
                found.push((modified, path, meta.len()));
            }
        }
    }
    found.sort_by_key(|(age, _, _)| std::cmp::Reverse(*age));
    found
}

/// The newest file in `folders` (and one level of folders under each) that is the TTW package or an archive that
/// holds it, as its path, size and the installer's space figure when it is known. An archive is returned as it is:
/// raul reads the package out of it. A package smaller than [`MIN_PACKAGE_BYTES`] is a download that has only begun
/// and is skipped until it grows.
pub fn newest_accepted(folders: &[PathBuf], refused: &mut Refused) -> Option<PackageInfo> {
    for (_, path, size) in candidates(folders) {
        if refused.get(&path) == Some(&size) {
            continue;
        }
        match inspect(&path) {
            Ok(package @ Package::Mpi { .. }) if size >= MIN_PACKAGE_BYTES => {
                return Some(PackageInfo {
                    path,
                    size,
                    needed_bytes: package.needed_bytes(),
                });
            }
            Ok(package @ Package::Archive { .. }) => {
                return Some(PackageInfo {
                    path,
                    size,
                    needed_bytes: package.needed_bytes(),
                });
            }
            _ => {
                refused.insert(path, size);
            }
        }
    }
    None
}

/// [`newest_accepted`] with nothing remembered: one search.
pub fn find(folders: &[PathBuf]) -> Option<PackageInfo> {
    newest_accepted(folders, &mut Refused::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nvmod_testkit::TempDir;

    fn mpi(len: usize) -> Vec<u8> {
        let mut b = vec![7_u8; len];
        b[..4].copy_from_slice(b"BSA\0");
        b
    }

    fn make_7z(path: &Path, files: &[(&str, &[u8])]) {
        let mut w = sevenz_rust2::ArchiveWriter::create(path).unwrap();
        for (name, data) in files {
            w.push_archive_entry(sevenz_rust2::ArchiveEntry::new_file(name), Some(*data))
                .unwrap();
        }
        w.finish().unwrap();
    }

    fn later(path: &Path, secs: u64) {
        let t = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_800_000_000 + secs);
        std::fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(t)
            .unwrap();
    }

    #[test]
    fn the_real_download_is_taken_and_the_newer_patch_archive_is_not() {
        let dir = TempDir::new("download-real");
        let real = dir
            .path()
            .join("Tale of Two Wastelands 3.4-133-3.4.0-2026.06.11-[mod.pub].7z");
        make_7z(
            &real,
            &[
                ("Tale of Two Wastelands 3.4.mpi", &mpi(2_000)),
                ("TTW_Installer.exe", b"MZ made-up"),
            ],
        );
        let patch = dir.path().join("YUPTTW 13.9.1-1234-[mod.pub].7z");
        make_7z(&patch, &[("YUPTTW.esm", b"TES4"), ("x.bsa", b"BSA\0")]);
        later(&real, 0);
        later(&patch, 100);
        let found = find(&[dir.path().to_path_buf()]).unwrap();
        assert_eq!(found.path, real);
    }

    #[test]
    fn a_package_under_any_name_counts_once_it_is_big_enough() {
        let dir = TempDir::new("download-mpi");
        std::fs::write(dir.path().join("whatever.mpi"), mpi(10)).unwrap();
        assert_eq!(find(&[dir.path().to_path_buf()]), None, "a begun download");
        std::fs::write(dir.path().join("whatever.mpi"), mpi(1_000_000)).unwrap();
        assert_eq!(
            find(&[dir.path().to_path_buf()]).unwrap().path,
            dir.path().join("whatever.mpi")
        );
    }

    #[test]
    fn a_file_that_is_not_a_package_under_a_package_name_is_skipped_and_folders_below_are_read() {
        let dir = TempDir::new("download-sub");
        std::fs::write(dir.path().join("TTW_fake.mpi"), vec![0_u8; 1_000_000]).unwrap();
        std::fs::write(dir.path().join("notes.txt"), b"x").unwrap();
        std::fs::write(dir.path().join("holiday.7z"), b"x").unwrap();
        let sub = dir.path().join("inside");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("p.mpi"), mpi(1_000_000)).unwrap();
        later(&sub.join("p.mpi"), 0);
        later(&dir.path().join("TTW_fake.mpi"), 100);
        let mut refused = Refused::new();
        let found = newest_accepted(&[dir.path().to_path_buf()], &mut refused).unwrap();
        assert_eq!(found.path, sub.join("p.mpi"));
        assert!(refused.contains_key(&dir.path().join("TTW_fake.mpi")));
    }

    #[test]
    fn nothing_found_in_an_empty_or_missing_folder() {
        let dir = TempDir::new("download-none");
        assert_eq!(
            find(&[dir.path().to_path_buf(), dir.path().join("gone")]),
            None
        );
    }
}
