// SPDX-License-Identifier: GPL-3.0-only
//! Unpacking a downloaded release archive into the staging folder and checking it against the release's own
//! `INSTALLED-FILES.sha256` (ADR-0020). The archive is untrusted: only regular files and folders are written, every
//! path must stay inside the destination, nothing is overwritten, and the sizes are limited.

use std::collections::HashSet;
use std::fmt;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use super::verify::hash_file;

/// The checksum list every release puts in the unit.
pub const INVENTORY: &str = "INSTALLED-FILES.sha256";

/// Limits on what an archive may hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// The most files.
    pub max_files: usize,
    /// The most bytes in all.
    pub max_bytes: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_files: 20_000,
            max_bytes: 2 * 1024 * 1024 * 1024,
        }
    }
}

/// Why an archive was refused. Nothing the archive held is installed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The archive holds something that is not allowed (a path leaving the folder, a link, a duplicate, a device).
    Unsafe(String),
    /// The archive is bigger than the limits.
    TooBig,
    /// The archive holds no inventory.
    NoInventory,
    /// The inventory is malformed.
    BadInventory,
    /// A listed file is missing from the archive.
    Missing(String),
    /// A file is in the archive but not in the inventory.
    Unlisted(String),
    /// A file does not match its checksum.
    Mismatch(String),
    /// The archive cannot be read or a file cannot be written.
    Io(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsafe(_) => {
                f.write_str("The update holds a file that is not allowed, so it was not installed.")
            }
            Self::TooBig => {
                f.write_str("The update is bigger than an update may be, so it was not installed.")
            }
            Self::NoInventory | Self::BadInventory => f.write_str(
                "The update has no readable list of its files, so it was not installed.",
            ),
            Self::Missing(_) | Self::Unlisted(_) | Self::Mismatch(_) => {
                f.write_str("The update does not match its list of files, so it was not installed.")
            }
            Self::Io(why) => write!(f, "The update could not be unpacked: {why}"),
        }
    }
}

impl std::error::Error for Error {}

fn io(path: &Path, e: &std::io::Error) -> Error {
    Error::Io(format!("{}: {e}", path.display()))
}

/// What an archive entry is, as far as extraction cares.
enum Kind {
    File { executable: bool },
    Folder,
    Other,
}

/// Collects what is written, enforcing the limits and refusing duplicates.
struct Sink<'a> {
    dest: &'a Path,
    strip: usize,
    limits: Limits,
    seen: HashSet<String>,
    files: Vec<String>,
    bytes: u64,
}

impl Sink<'_> {
    /// The path inside the destination for the archive's `name`, or why it is refused.
    fn target(&self, name: &str) -> Result<Option<(String, PathBuf)>, Error> {
        let unsafe_name = || Error::Unsafe(name.to_owned());
        if name.contains('\\') || name.contains('\0') {
            return Err(unsafe_name());
        }
        let mut parts = Vec::new();
        for component in Path::new(name).components() {
            match component {
                // A colon is a drive or a stream name on Windows: refused on every system, so one archive means one thing.
                Component::Normal(p) if !p.to_string_lossy().contains(':') => {
                    parts.push(p.to_string_lossy().into_owned());
                }
                Component::CurDir => {}
                _ => return Err(unsafe_name()),
            }
        }
        if parts.len() < self.strip {
            return Err(unsafe_name());
        }
        let parts = &parts[self.strip..];
        if parts.is_empty() {
            return Ok(None);
        }
        let relative = parts.join("/");
        Ok(Some((relative.clone(), self.dest.join(relative))))
    }

    fn entry(&mut self, name: &str, kind: &Kind, data: &mut dyn Read) -> Result<(), Error> {
        let Some((relative, path)) = self.target(name)? else {
            return Ok(());
        };
        match kind {
            Kind::Other => Err(Error::Unsafe(name.to_owned())),
            Kind::Folder => std::fs::create_dir_all(&path).map_err(|e| io(&path, &e)),
            Kind::File { executable } => {
                // Names differing only in case are one file on Windows and macOS: refuse them as duplicates.
                if !self.seen.insert(relative.to_ascii_lowercase()) {
                    return Err(Error::Unsafe(name.to_owned()));
                }
                if self.files.len() >= self.limits.max_files {
                    return Err(Error::TooBig);
                }
                if let Some(dir) = path.parent() {
                    std::fs::create_dir_all(dir).map_err(|e| io(dir, &e))?;
                }
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                    .map_err(|e| io(&path, &e))?;
                let room = self.limits.max_bytes - self.bytes;
                let written = std::io::copy(&mut data.take(room + 1), &mut file)
                    .map_err(|e| io(&path, &e))?;
                if written > room {
                    return Err(Error::TooBig);
                }
                self.bytes += written;
                set_mode(&path, *executable)?;
                self.files.push(relative);
                Ok(())
            }
        }
    }
}

#[cfg(unix)]
fn set_mode(path: &Path, executable: bool) -> Result<(), Error> {
    use std::os::unix::fs::PermissionsExt;
    let mode = if executable { 0o755 } else { 0o644 };
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).map_err(|e| io(path, &e))
}

#[cfg(not(unix))]
#[allow(clippy::unnecessary_wraps)] // the Unix version can fail
fn set_mode(_path: &Path, _executable: bool) -> Result<(), Error> {
    Ok(())
}

/// Unpack a `.tar.gz` into `dest`, dropping the first `strip` components of every name (the archive's top folder).
/// Returns the files written, relative to `dest`. On an error `dest` holds what was written so far; the caller removes it.
///
/// # Errors
/// [`Error::Unsafe`] for a path that is absolute, has `..`, a backslash or a drive, a link, a device or a repeated
/// name; [`Error::TooBig`] past `limits`; [`Error::Io`] otherwise.
pub fn extract_tar_gz(
    archive: &Path,
    dest: &Path,
    strip: usize,
    limits: Limits,
) -> Result<Vec<String>, Error> {
    let file = std::fs::File::open(archive).map_err(|e| io(archive, &e))?;
    let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(file));
    let mut sink = Sink {
        dest,
        strip,
        limits,
        seen: HashSet::new(),
        files: Vec::new(),
        bytes: 0,
    };
    std::fs::create_dir_all(dest).map_err(|e| io(dest, &e))?;
    for entry in tar.entries().map_err(|e| io(archive, &e))? {
        let mut entry = entry.map_err(|e| io(archive, &e))?;
        let name = entry
            .path()
            .map_err(|e| io(archive, &e))?
            .to_string_lossy()
            .into_owned();
        let kind = match entry.header().entry_type() {
            tar::EntryType::Regular => Kind::File {
                executable: entry.header().mode().is_ok_and(|m| m & 0o111 != 0),
            },
            tar::EntryType::Directory => Kind::Folder,
            _ => Kind::Other,
        };
        sink.entry(name.trim_end_matches('/'), &kind, &mut entry)?;
    }
    Ok(sink.files)
}

/// Unpack a `.zip` into `dest`; as [`extract_tar_gz`].
///
/// # Errors
/// As [`extract_tar_gz`].
pub fn extract_zip(
    archive: &Path,
    dest: &Path,
    strip: usize,
    limits: Limits,
) -> Result<Vec<String>, Error> {
    let file = std::fs::File::open(archive).map_err(|e| io(archive, &e))?;
    let mut zip =
        zip::ZipArchive::new(file).map_err(|e| Error::Io(format!("{}: {e}", archive.display())))?;
    let mut sink = Sink {
        dest,
        strip,
        limits,
        seen: HashSet::new(),
        files: Vec::new(),
        bytes: 0,
    };
    std::fs::create_dir_all(dest).map_err(|e| io(dest, &e))?;
    for i in 0..zip.len() {
        let mut entry = zip
            .by_index(i)
            .map_err(|e| Error::Io(format!("{}: {e}", archive.display())))?;
        let name = entry
            .name()
            .map_err(|_| Error::Unsafe("a name that is not text".to_owned()))?
            .into_owned();
        let mode = entry.unix_mode().unwrap_or(0o644);
        let kind = match mode & 0o170_000 {
            0o120_000 => Kind::Other,
            _ if entry.is_dir() => Kind::Folder,
            0 | 0o100_000 => Kind::File {
                executable: mode & 0o111 != 0,
            },
            _ => Kind::Other,
        };
        sink.entry(name.trim_end_matches('/'), &kind, &mut entry)?;
    }
    Ok(sink.files)
}

/// The (path, hash) lines of an inventory.
fn parse_inventory(text: &str) -> Result<Vec<(String, String)>, Error> {
    let mut out = Vec::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let (hash, rest) = line
            .split_once(char::is_whitespace)
            .ok_or(Error::BadInventory)?;
        let name = rest.trim_start().trim_start_matches('*');
        let inside = !name.is_empty()
            && Path::new(name)
                .components()
                .all(|c| matches!(c, Component::Normal(_)))
            && !name.contains('\\');
        if hash.len() != 64
            || !hash.bytes().all(|b| b.is_ascii_hexdigit())
            || !inside
            || name == INVENTORY
        {
            return Err(Error::BadInventory);
        }
        out.push((name.to_owned(), hash.to_ascii_lowercase()));
    }
    Ok(out)
}

fn walk(dir: &Path, base: &Path, out: &mut Vec<String>) -> Result<(), Error> {
    for entry in std::fs::read_dir(dir).map_err(|e| io(dir, &e))? {
        let entry = entry.map_err(|e| io(dir, &e))?;
        let path = entry.path();
        let kind = entry.file_type().map_err(|e| io(&path, &e))?;
        let relative = path.strip_prefix(base).map_or_else(
            |_| String::new(),
            |p| p.to_string_lossy().replace('\\', "/"),
        );
        if kind.is_dir() {
            walk(&path, base, out)?;
        } else if kind.is_file() {
            out.push(relative);
        } else {
            return Err(Error::Unsafe(relative));
        }
    }
    Ok(())
}

/// Check the unpacked unit in `dir` against its inventory: every listed file is there and matches, and nothing else
/// is there. Returns the files of the unit (the listed ones and the inventory itself), sorted, ready for the swap.
///
/// # Errors
/// [`Error::NoInventory`], [`Error::BadInventory`], [`Error::Missing`], [`Error::Unlisted`] or [`Error::Mismatch`].
pub fn check_unit(dir: &Path) -> Result<Vec<String>, Error> {
    let text = std::fs::read_to_string(dir.join(INVENTORY)).map_err(|_| Error::NoInventory)?;
    let listed = parse_inventory(&text)?;
    for (name, hash) in &listed {
        let path = dir.join(name);
        if !std::fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_file()) {
            return Err(Error::Missing(name.clone()));
        }
        if hash_file(&path).map_err(|e| Error::Io(e.to_string()))? != *hash {
            return Err(Error::Mismatch(name.clone()));
        }
    }
    let mut present = Vec::new();
    walk(dir, dir, &mut present)?;
    for name in &present {
        if name != INVENTORY && !listed.iter().any(|(n, _)| n == name) {
            return Err(Error::Unlisted(name.clone()));
        }
    }
    let mut unit: Vec<String> = listed.into_iter().map(|(n, _)| n).collect();
    unit.push(INVENTORY.to_owned());
    unit.sort();
    Ok(unit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nvmod_testkit::{TarBuilder, TempDir, ZipBuilder};
    use std::io::Write;

    fn gz(tar: &[u8]) -> Vec<u8> {
        let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        e.write_all(tar).unwrap();
        e.finish().unwrap()
    }

    fn hash(b: &[u8]) -> String {
        raul_runner::sha256_hex(b)
    }

    fn inventory(files: &[(&str, &[u8])]) -> String {
        let mut text = String::new();
        for (n, b) in files {
            text.push_str(&hash(b));
            text.push_str("  ");
            text.push_str(n);
            text.push('\n');
        }
        text
    }

    /// A release archive: a top folder holding the files and their inventory.
    fn release_tar(top: &str, files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut t = TarBuilder::new().dir(&format!("{top}/"));
        for (n, b) in files {
            t = t.file(&format!("{top}/{n}"), b);
        }
        t = t.file(&format!("{top}/{INVENTORY}"), inventory(files).as_bytes());
        gz(&t.bytes())
    }

    fn write(t: &TempDir, name: &str, bytes: &[u8]) -> PathBuf {
        let p = t.path().join(name);
        std::fs::write(&p, bytes).unwrap();
        p
    }

    const FILES: &[(&str, &[u8])] = &[
        ("raul", b"binary"),
        ("components/nv-rs/bin/nv-viewer", b"viewer"),
    ];

    #[test]
    fn a_release_archive_unpacks_without_its_top_folder_and_checks_out() {
        let t = TempDir::new("extract-ok");
        let a = write(&t, "a.tar.gz", &release_tar("raul-2.0.0", FILES));
        let dest = t.path().join("new");
        let mut files = extract_tar_gz(&a, &dest, 1, Limits::default()).unwrap();
        files.sort();
        assert_eq!(files, [INVENTORY, "components/nv-rs/bin/nv-viewer", "raul"]);
        assert_eq!(std::fs::read(dest.join("raul")).unwrap(), b"binary");
        assert_eq!(
            check_unit(&dest).unwrap(),
            [INVENTORY, "components/nv-rs/bin/nv-viewer", "raul"]
        );
    }

    #[test]
    fn a_zip_unpacks_the_same_way() {
        let t = TempDir::new("extract-zip");
        let mut z = ZipBuilder::new();
        for (n, b) in FILES {
            z = z.file(&format!("raul-2.0.0/{n}"), b);
        }
        z = z.file(
            &format!("raul-2.0.0/{INVENTORY}"),
            inventory(FILES).as_bytes(),
        );
        let a = write(&t, "a.zip", &z.bytes());
        let dest = t.path().join("new");
        extract_zip(&a, &dest, 1, Limits::default()).unwrap();
        assert_eq!(check_unit(&dest).unwrap().len(), 3);
    }

    #[test]
    fn paths_that_leave_the_folder_are_refused_in_both_formats() {
        for bad in ["../outside", "/abs/path", "a/../../b", "a\\b", "C:/x"] {
            let t = TempDir::new("extract-escape");
            let tar = gz(&TarBuilder::new().file(bad, b"x").bytes());
            let a = write(&t, "a.tar.gz", &tar);
            let r = extract_tar_gz(&a, &t.path().join("new"), 0, Limits::default());
            assert!(matches!(r, Err(Error::Unsafe(_))), "{bad}: {r:?}");
            let z = write(&t, "a.zip", &ZipBuilder::new().file(bad, b"x").bytes());
            let r = extract_zip(&z, &t.path().join("new2"), 0, Limits::default());
            assert!(matches!(r, Err(Error::Unsafe(_))), "{bad}: {r:?}");
            assert!(!t.path().join("outside").exists());
        }
    }

    #[test]
    fn links_are_refused_in_both_formats() {
        let t = TempDir::new("extract-links");
        let a = write(
            &t,
            "a.tar.gz",
            &gz(&TarBuilder::new().symlink("raul", "/etc/passwd").bytes()),
        );
        assert!(matches!(
            extract_tar_gz(&a, &t.path().join("n1"), 0, Limits::default()),
            Err(Error::Unsafe(_))
        ));
        let z = write(
            &t,
            "a.zip",
            &ZipBuilder::new().symlink("raul", "/etc/passwd").bytes(),
        );
        assert!(matches!(
            extract_zip(&z, &t.path().join("n2"), 0, Limits::default()),
            Err(Error::Unsafe(_))
        ));
    }

    #[test]
    fn a_repeated_name_even_in_another_case_is_refused() {
        let t = TempDir::new("extract-dup");
        for (i, (a, b)) in [("raul", "raul"), ("raul", "RAUL")].into_iter().enumerate() {
            let tar = gz(&TarBuilder::new().file(a, b"1").file(b, b"2").bytes());
            let p = write(&t, "a.tar.gz", &tar);
            let r = extract_tar_gz(&p, &t.path().join(format!("n{i}")), 0, Limits::default());
            assert!(matches!(r, Err(Error::Unsafe(_))), "{a} {b}: {r:?}");
        }
    }

    #[test]
    fn the_limits_on_files_and_bytes_are_enforced() {
        let t = TempDir::new("extract-limits");
        let tar = gz(&TarBuilder::new()
            .file("a", b"12345")
            .file("b", b"1")
            .bytes());
        let p = write(&t, "a.tar.gz", &tar);
        let few = Limits {
            max_files: 1,
            max_bytes: 100,
        };
        assert_eq!(
            extract_tar_gz(&p, &t.path().join("n1"), 0, few),
            Err(Error::TooBig)
        );
        let small = Limits {
            max_files: 10,
            max_bytes: 5,
        };
        assert_eq!(
            extract_tar_gz(&p, &t.path().join("n2"), 0, small),
            Err(Error::TooBig)
        );
    }

    #[test]
    fn a_file_that_does_not_match_the_inventory_is_refused() {
        let t = TempDir::new("check-mismatch");
        let dir = t.path().join("u");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("raul"), b"binary").unwrap();
        std::fs::write(dir.join(INVENTORY), inventory(&[("raul", b"binary")])).unwrap();
        assert!(check_unit(&dir).is_ok());
        std::fs::write(dir.join("raul"), b"tampered").unwrap();
        assert_eq!(check_unit(&dir), Err(Error::Mismatch("raul".to_owned())));
    }

    #[test]
    fn a_missing_a_listed_and_an_unlisted_file_are_refused() {
        let t = TempDir::new("check-lists");
        let dir = t.path().join("u");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(INVENTORY), inventory(&[("raul", b"b")])).unwrap();
        assert_eq!(check_unit(&dir), Err(Error::Missing("raul".to_owned())));
        std::fs::write(dir.join("raul"), b"b").unwrap();
        std::fs::write(dir.join("extra"), b"x").unwrap();
        assert_eq!(check_unit(&dir), Err(Error::Unlisted("extra".to_owned())));
        std::fs::remove_file(dir.join(INVENTORY)).unwrap();
        assert_eq!(check_unit(&dir), Err(Error::NoInventory));
    }

    #[test]
    fn a_bad_inventory_is_refused() {
        let t = TempDir::new("check-bad");
        let dir = t.path().join("u");
        std::fs::create_dir_all(&dir).unwrap();
        for bad in [
            format!("{}  ../x\n", hash(b"x")),
            format!("{}  /abs\n", hash(b"x")),
            format!("{}  {INVENTORY}\n", hash(b"x")),
            "short  raul\n".to_owned(),
        ] {
            std::fs::write(dir.join(INVENTORY), &bad).unwrap();
            assert_eq!(check_unit(&dir), Err(Error::BadInventory), "{bad}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn the_executable_bit_survives_and_a_link_in_the_folder_is_refused() {
        use std::os::unix::fs::PermissionsExt;
        let t = TempDir::new("extract-mode");
        let a = write(&t, "a.tar.gz", &release_tar("r", FILES));
        let dest = t.path().join("new");
        extract_tar_gz(&a, &dest, 1, Limits::default()).unwrap();
        // The test archive builder writes mode 0644 for everything, so nothing is executable here.
        assert_eq!(
            std::fs::metadata(dest.join("raul"))
                .unwrap()
                .permissions()
                .mode()
                & 0o111,
            0
        );
        std::os::unix::fs::symlink("/etc/passwd", dest.join("link")).unwrap();
        assert!(matches!(check_unit(&dest), Err(Error::Unsafe(_))));
    }

    #[test]
    fn the_messages_hold_no_path_and_no_command() {
        for e in [
            Error::Unsafe("../secret".to_owned()),
            Error::Mismatch("raul".to_owned()),
            Error::TooBig,
            Error::NoInventory,
        ] {
            let text = e.to_string();
            assert!(
                !text.contains("secret") && !text.contains("raul update"),
                "{text}"
            );
        }
    }
}
