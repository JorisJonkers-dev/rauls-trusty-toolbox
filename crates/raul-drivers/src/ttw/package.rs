// SPDX-License-Identifier: GPL-3.0-only
//! What a TTW package is, and how raul gets one from the file a player downloaded.
//!
//! The one rule behind `raul ttw`, `raul config set ttw.package` and the TTW page:
//!
//! - A package is the `.mpi` file, recognised by its content: it starts with `BSA\0`, whatever it is called
//!   (`Tale of Two Wastelands 3.4.mpi` for 3.4).
//! - What the player downloads is a `.7z` (or `.zip`) that holds that file next to a Windows installer program and its
//!   libraries. raul accepts the download: [`inspect`] looks at the list of names in the archive (nothing is
//!   decompressed) and takes an archive that holds exactly one `.mpi`; [`unpack`] then extracts it in process with
//!   nvmod's bounded extractor and checks the extracted file's first bytes. No program is started.
//! - An archive that holds no `.mpi` is never taken, whatever it is called: the YUPTTW patch archive that sits in the
//!   same Downloads folder holds `.esm`, `.bsa` and `.override` files and no package.

use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::path::{Component, Path, PathBuf};

use nvmod_modinstall::Limits;

/// The first four bytes of a `.mpi` package: it is a BSA archive (`42 53 41 00`).
pub const MPI_HEADER: [u8; 4] = *b"BSA\0";

/// Where the download comes from, for the messages.
pub const DOWNLOAD_FROM: &str = "taleoftwowastelands.com";

/// The start of the name of the download (`Tale of Two Wastelands 3.4-133-3.4.0-2026.06.11-[mod.pub].7z`), which is
/// only a hint for finding it: a search may look at an archive with this name first, but what decides is what the
/// archive holds.
pub const DOWNLOAD_NAME_START: &str = "Tale of Two Wastelands";

/// Decoder memory allowed while reading the names of a 7z archive, in KiB (the extractor's own bound).
const LIST_MEMORY_KB: usize = 512 * 1024;

/// What a file given as the package is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Package {
    /// The `.mpi` itself.
    Mpi {
        /// The file.
        path: PathBuf,
    },
    /// An archive that holds exactly one `.mpi`; its content is checked when it is unpacked.
    Archive {
        /// The archive.
        path: PathBuf,
        /// The name of the `.mpi` inside it.
        entry: String,
        /// Bytes the archive holds once unpacked, all its files, as its header says.
        unpacked_bytes: u64,
    },
}

/// Why a file is not accepted as the TTW package, in plain words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// The path is not a file that can be read.
    NotAFile {
        /// The path given.
        path: PathBuf,
    },
    /// A file that is neither a `.mpi` nor an archive raul reads.
    Neither {
        /// The file.
        path: PathBuf,
        /// A kind raul recognises and cannot open (`rar`), when the first bytes say so.
        kind: Option<&'static str>,
    },
    /// An archive, but it holds no `.mpi`.
    NoPackageInside {
        /// The archive.
        path: PathBuf,
    },
    /// An archive that holds more than one `.mpi`.
    SeveralInside {
        /// The archive.
        path: PathBuf,
        /// How many.
        count: usize,
    },
    /// An archive whose list of names cannot be read: damaged, cut short (a download that did not finish) or
    /// protected by a password.
    Unreadable {
        /// The archive.
        path: PathBuf,
        /// Why, in the reader's words.
        why: String,
    },
    /// The unpacked file does not start with `BSA\0`, so it is not a package.
    NotAPackage {
        /// The archive it came out of.
        path: PathBuf,
        /// Its name inside the archive.
        entry: String,
    },
    /// Unpacking failed.
    Unpack {
        /// The archive.
        path: PathBuf,
        /// Why.
        why: String,
    },
}

fn name_of(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

impl Problem {
    /// One short sentence for the TTW page: no paths and no program or command names in it.
    #[must_use]
    pub fn why(&self) -> &'static str {
        match self {
            Problem::NotAFile { .. } => "The file cannot be read.",
            Problem::Neither { kind: Some(_), .. } => {
                "That archive kind cannot be opened here: unpack it and choose the TTW package inside."
            }
            Problem::Neither { kind: None, .. } => {
                "The file is not a TTW package, and not an archive that holds one."
            }
            Problem::NoPackageInside { .. } => "The archive holds no TTW package.",
            Problem::SeveralInside { .. } => {
                "The archive holds more than one package: unpack it and choose the right one."
            }
            Problem::Unreadable { .. } => {
                "The archive cannot be read: the download may not be complete."
            }
            Problem::NotAPackage { .. } => "The file in the archive is not a TTW package.",
            Problem::Unpack { .. } => "The archive could not be unpacked.",
        }
    }
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const WANTED: &str = "raul needs the TTW package, `Tale of Two Wastelands <version>.mpi`, or the .7z download that holds it";
        match self {
            Problem::NotAFile { path } => {
                write!(f, "the TTW package {} is not a file", path.display())
            }
            Problem::Neither { path, kind } => {
                write!(f, "{} is not a TTW package", path.display())?;
                match kind {
                    Some(kind) => write!(
                        f,
                        ": it is a {kind} archive, which raul does not open. Unpack it with an archive program and give raul the `.mpi` inside, or download the .7z of the package from {DOWNLOAD_FROM}"
                    ),
                    None => write!(
                        f,
                        ": it is neither a `.mpi` file (which starts with `BSA\\0`) nor an archive that holds one. {WANTED}; it comes from {DOWNLOAD_FROM}"
                    ),
                }
            }
            Problem::NoPackageInside { path } => write!(
                f,
                "{} holds no TTW package (no `.mpi` file in it). {WANTED}; it comes from {DOWNLOAD_FROM}. An archive of patches or other Mods is not the package",
                path.display()
            ),
            Problem::SeveralInside { path, count } => write!(
                f,
                "{} holds {count} `.mpi` files and raul cannot tell which is the package. Unpack it and give raul the one you want",
                path.display()
            ),
            Problem::Unreadable { path, why } => write!(
                f,
                "{} cannot be read as an archive ({why}). A download that did not finish looks like this; download it again. {WANTED}; it comes from {DOWNLOAD_FROM}",
                path.display()
            ),
            Problem::NotAPackage { path, entry } => write!(
                f,
                "`{entry}` in {} is not a TTW package: a package starts with `BSA\\0`. {WANTED}; it comes from {DOWNLOAD_FROM}",
                path.display()
            ),
            Problem::Unpack { path, why } => {
                write!(f, "{} could not be unpacked: {why}", path.display())
            }
        }
    }
}

impl std::error::Error for Problem {}

/// Whether the file at `path` starts with [`MPI_HEADER`]. Only its first four bytes are read; a file that cannot be
/// read is not one.
#[must_use]
pub fn is_mpi(path: &Path) -> bool {
    let mut head = [0_u8; 4];
    File::open(path)
        .and_then(|mut f| f.read_exact(&mut head))
        .is_ok()
        && head == MPI_HEADER
}

/// The kinds of archive that are recognised from the first bytes.
enum Kind {
    SevenZ,
    Zip,
    /// Recognised and not opened.
    Other(&'static str),
}

fn kind_of(head: &[u8]) -> Option<Kind> {
    if head.starts_with(b"7z\xbc\xaf\x27\x1c") {
        Some(Kind::SevenZ)
    } else if head.starts_with(b"PK\x03\x04") || head.starts_with(b"PK\x05\x06") {
        Some(Kind::Zip)
    } else if head.starts_with(b"Rar!\x1a\x07") {
        Some(Kind::Other("rar"))
    } else if head.starts_with(b"\x1f\x8b") {
        Some(Kind::Other("gzip"))
    } else {
        None
    }
}

/// Decide what the file at `path` is, from its content. A `.mpi` is taken at once; an archive is read for its list of
/// names only, which is quick whatever its size.
///
/// # Errors
/// A [`Problem`] when the file is not the package and not an archive that holds exactly one.
pub fn inspect(path: &Path) -> Result<Package, Problem> {
    let not_a_file = || Problem::NotAFile {
        path: path.to_path_buf(),
    };
    if !path.is_file() {
        return Err(not_a_file());
    }
    let mut head = [0_u8; 8];
    let read = File::open(path)
        .and_then(|mut f| {
            let mut n = 0;
            while n < head.len() {
                match f.read(&mut head[n..])? {
                    0 => break,
                    got => n += got,
                }
            }
            Ok(n)
        })
        .map_err(|_| not_a_file())?;
    let head = &head[..read];
    if head.starts_with(&MPI_HEADER) {
        return Ok(Package::Mpi {
            path: path.to_path_buf(),
        });
    }
    let list = match kind_of(head) {
        Some(Kind::SevenZ) => list_7z(path),
        Some(Kind::Zip) => list_zip(path),
        Some(Kind::Other(kind)) => {
            return Err(Problem::Neither {
                path: path.to_path_buf(),
                kind: Some(kind),
            });
        }
        None => {
            return Err(Problem::Neither {
                path: path.to_path_buf(),
                kind: None,
            });
        }
    }
    .map_err(|why| Problem::Unreadable {
        path: path.to_path_buf(),
        why,
    })?;
    let unpacked_bytes = list
        .iter()
        .fold(0_u64, |sum, (_, size)| sum.saturating_add(*size));
    let mut packages: Vec<String> = list
        .into_iter()
        .map(|(name, _)| name)
        .filter(|name| {
            name.rsplit(['/', '\\'])
                .next()
                .is_some_and(has_mpi_extension)
        })
        .collect();
    match packages.len() {
        0 => Err(Problem::NoPackageInside {
            path: path.to_path_buf(),
        }),
        1 => Ok(Package::Archive {
            path: path.to_path_buf(),
            entry: packages.remove(0),
            unpacked_bytes,
        }),
        count => Err(Problem::SeveralInside {
            path: path.to_path_buf(),
            count,
        }),
    }
}

fn has_mpi_extension(file_name: &str) -> bool {
    Path::new(file_name)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("mpi"))
}

/// The files of a 7z archive with their unpacked sizes, read from its header; nothing is decompressed but the header.
fn list_7z(path: &Path) -> Result<Vec<(String, u64)>, String> {
    let mut reader = BufReader::new(File::open(path).map_err(|e| e.to_string())?);
    let archive = sevenz_rust2::Archive::read_with_memory_limit(
        &mut reader,
        &sevenz_rust2::Password::empty(),
        LIST_MEMORY_KB,
    )
    .map_err(|e| e.to_string())?;
    Ok(archive
        .files
        .iter()
        .filter(|e| !e.is_directory)
        .map(|e| (e.name.clone(), e.size))
        .collect())
}

/// The files of a zip archive with their unpacked sizes, read from its central directory.
fn list_zip(path: &Path) -> Result<Vec<(String, u64)>, String> {
    let mut zip =
        zip::ZipArchive::new(BufReader::new(File::open(path).map_err(|e| e.to_string())?))
            .map_err(|e| e.to_string())?;
    let mut files = Vec::new();
    for i in 0..zip.len() {
        let entry = zip.by_index_raw(i).map_err(|e| e.to_string())?;
        if !entry.is_dir() {
            let name = entry.name().map_err(|e| e.to_string())?.into_owned();
            files.push((name, entry.size()));
        }
    }
    Ok(files)
}

/// The package inside the archive `package`, extracted to `<into>/<file name>`. The archive is unpacked in process, in
/// a folder next to `into` that is removed again, under a byte limit of what its header says it holds; the one `.mpi`
/// it holds is checked to start with `BSA\0` and moved to `into`. `into` is made when it is not there.
///
/// # Errors
/// A [`Problem`]: the archive cannot be unpacked, or what it held is not a package. Nothing is left in `into`.
pub fn unpack(package: &Package, into: &Path) -> Result<PathBuf, Problem> {
    let (path, entry, unpacked_bytes) = match package {
        Package::Mpi { path } => return Ok(path.clone()),
        Package::Archive {
            path,
            entry,
            unpacked_bytes,
        } => (path, entry, *unpacked_bytes),
    };
    let fail = |why: String| Problem::Unpack {
        path: path.clone(),
        why,
    };
    let scratch = into.with_file_name(format!(
        "{}.unpacking",
        into.file_name()
            .map_or_else(|| "package".into(), |n| n.to_string_lossy())
    ));
    let cleanup = |dir: &Path| {
        let _ = fs::remove_dir_all(dir);
    };
    cleanup(&scratch);
    fs::create_dir_all(&scratch).map_err(|e| fail(format!("{}: {e}", scratch.display())))?;
    let limits = Limits {
        max_bytes: unpacked_bytes.max(1),
        ..Limits::default()
    };
    if let Err(why) = nvmod_modinstall::extract::extract(path, &scratch, limits) {
        cleanup(&scratch);
        return Err(fail(why));
    }
    let found = (|| {
        let relative = entry.replace('\\', "/");
        let mut file = scratch.clone();
        for part in relative.split('/').filter(|p| !p.is_empty() && *p != ".") {
            if part == ".."
                || Path::new(part)
                    .components()
                    .any(|c| !matches!(c, Component::Normal(_)))
            {
                return None;
            }
            file.push(part);
        }
        file.is_file().then_some(file)
    })();
    let Some(file) = found else {
        cleanup(&scratch);
        return Err(fail(
            "the package is not where the archive says it is".to_owned(),
        ));
    };
    if !is_mpi(&file) {
        cleanup(&scratch);
        return Err(Problem::NotAPackage {
            path: path.clone(),
            entry: entry.clone(),
        });
    }
    let moved = (|| -> io::Result<PathBuf> {
        cleanup(into);
        fs::create_dir_all(into)?;
        let target = into.join(file.file_name().unwrap_or_default());
        fs::rename(&file, &target)?;
        Ok(target)
    })();
    cleanup(&scratch);
    moved.map_err(|e| fail(format!("{}: {e}", into.display())))
}

/// The one `.mpi` in `folder` (as [`unpack`] leaves it) when it starts with `BSA\0`, so a run that is resumed does
/// not unpack again.
#[must_use]
pub fn unpacked_in(folder: &Path) -> Option<PathBuf> {
    let mut found = fs::read_dir(folder)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && has_mpi_extension(&name_of(p)) && is_mpi(p));
    let one = found.next()?;
    found.next().is_none().then_some(one)
}

/// Bytes as `1.9 GB` or `640 MB`, for messages.
#[must_use]
pub fn size_text(bytes: u64) -> String {
    const MB: u64 = 1_000_000;
    const GB: u64 = 1_000 * MB;
    let mut text = String::new();
    if bytes >= GB {
        let _ = write!(text, "{}.{} GB", bytes / GB, bytes % GB / (GB / 10));
    } else {
        let _ = write!(text, "{} MB", bytes.div_ceil(MB));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use nvmod_testkit::TempDir;

    /// The bytes of a package: the header and some made-up body. No game data.
    fn mpi_bytes() -> Vec<u8> {
        let mut bytes = MPI_HEADER.to_vec();
        bytes.extend_from_slice(&[0x69, 0, 0, 0, 0x24, 0, 0, 0]);
        bytes.extend(std::iter::repeat_n(7_u8, 4096));
        bytes
    }

    fn make_7z(path: &Path, files: &[(&str, &[u8])]) {
        let mut writer = sevenz_rust2::ArchiveWriter::create(path).unwrap();
        for (name, data) in files {
            writer
                .push_archive_entry(sevenz_rust2::ArchiveEntry::new_file(name), Some(*data))
                .unwrap();
        }
        writer.finish().unwrap();
    }

    /// The download of the real package, made small: the package next to the installer program and a library.
    fn download(dir: &TempDir) -> PathBuf {
        let path = dir
            .path()
            .join("Tale of Two Wastelands 3.4-133-[mod.pub].7z");
        make_7z(
            &path,
            &[
                ("Tale of Two Wastelands 3.4.mpi", &mpi_bytes()),
                ("TTW_Linux_Installer.exe", b"MZ made-up installer"),
                ("lib/some.dll", b"MZ made-up library"),
            ],
        );
        path
    }

    #[test]
    fn a_file_that_starts_with_bsa_is_the_package_whatever_it_is_called() {
        let dir = TempDir::new("pkg-mpi");
        for name in ["Tale of Two Wastelands 3.4.mpi", "x.bin", "TTW_3.4.0.mpi"] {
            let path = dir.path().join(name);
            std::fs::write(&path, mpi_bytes()).unwrap();
            assert!(is_mpi(&path), "{name}");
            assert_eq!(inspect(&path), Ok(Package::Mpi { path }), "{name}");
        }
    }

    #[test]
    fn the_download_is_an_archive_with_one_package_and_says_how_much_it_unpacks_to() {
        let dir = TempDir::new("pkg-download");
        let path = download(&dir);
        let all = (mpi_bytes().len() + 20 + 18) as u64;
        assert_eq!(
            inspect(&path),
            Ok(Package::Archive {
                path,
                entry: "Tale of Two Wastelands 3.4.mpi".to_owned(),
                unpacked_bytes: all,
            })
        );
    }

    #[test]
    fn a_patch_archive_without_a_package_is_refused_whatever_it_is_called() {
        let dir = TempDir::new("pkg-patch");
        let path = dir.path().join("Tale of Two Wastelands patch.7z");
        make_7z(
            &path,
            &[
                ("YUPTTW.esm", b"TES4 made-up"),
                ("Fallout - Meshes.bsa", b"BSA\0 inside a Mod, not a package"),
                ("Fallout - Textures.bsa", b"BSA\0 another"),
                ("YUPTTW.esm.override", b""),
            ],
        );
        let why = inspect(&path).unwrap_err();
        assert_eq!(why, Problem::NoPackageInside { path: path.clone() });
        let text = why.to_string();
        assert!(
            text.contains("Tale of Two Wastelands <version>.mpi"),
            "{text}"
        );
        assert!(text.contains(DOWNLOAD_FROM), "{text}");
    }

    #[test]
    fn two_packages_in_one_archive_are_not_guessed() {
        let dir = TempDir::new("pkg-two");
        let path = dir.path().join("two.7z");
        make_7z(&path, &[("a.mpi", &mpi_bytes()), ("b/c.MPI", &mpi_bytes())]);
        assert_eq!(
            inspect(&path),
            Err(Problem::SeveralInside { path, count: 2 })
        );
    }

    #[test]
    fn a_zip_is_read_the_same_way() {
        use std::io::Write as _;
        let dir = TempDir::new("pkg-zip");
        let path = dir.path().join("download.zip");
        let mut zip = zip::ZipWriter::new(File::create(&path).unwrap());
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("TTW 3.4.mpi", options).unwrap();
        zip.write_all(&mpi_bytes()).unwrap();
        zip.start_file("installer.exe", options).unwrap();
        zip.write_all(b"MZ").unwrap();
        zip.finish().unwrap();
        let Ok(Package::Archive {
            entry,
            unpacked_bytes,
            ..
        }) = inspect(&path)
        else {
            panic!("a zip with a package is accepted");
        };
        assert_eq!(entry, "TTW 3.4.mpi");
        assert_eq!(unpacked_bytes, mpi_bytes().len() as u64 + 2);
        let into = dir.path().join("package");
        let mpi = unpack(&inspect(&path).unwrap(), &into).unwrap();
        assert!(is_mpi(&mpi) && mpi.starts_with(&into));
    }

    #[test]
    fn what_is_neither_a_package_nor_an_archive_is_refused_with_the_expected_name() {
        let dir = TempDir::new("pkg-neither");
        let text = dir.path().join("notes.txt");
        std::fs::write(&text, "not a package").unwrap();
        let why = inspect(&text).unwrap_err();
        assert_eq!(
            why,
            Problem::Neither {
                path: text,
                kind: None
            }
        );
        let said = why.to_string();
        assert!(
            said.contains("Tale of Two Wastelands <version>.mpi"),
            "{said}"
        );
        assert!(said.contains(".7z"), "{said}");

        let rar = dir.path().join("download.rar");
        std::fs::write(&rar, b"Rar!\x1a\x07\x01\x00 made up").unwrap();
        let why = inspect(&rar).unwrap_err();
        assert_eq!(
            why,
            Problem::Neither {
                path: rar,
                kind: Some("rar")
            }
        );
        assert!(why.to_string().contains("Unpack it"), "{why}");

        assert!(matches!(inspect(dir.path()), Err(Problem::NotAFile { .. })));
        assert!(matches!(
            inspect(&dir.path().join("missing.7z")),
            Err(Problem::NotAFile { .. })
        ));
    }

    #[test]
    fn an_archive_that_was_cut_short_is_unreadable_not_a_package() {
        let dir = TempDir::new("pkg-cut");
        let whole = std::fs::read(download(&dir)).unwrap();
        let cut = dir.path().join("cut.7z");
        std::fs::write(&cut, &whole[..whole.len() / 2]).unwrap();
        let why = inspect(&cut).unwrap_err();
        assert!(matches!(why, Problem::Unreadable { .. }), "{why:?}");
        assert!(why.to_string().contains("did not finish"), "{why}");
    }

    #[test]
    fn unpacking_leaves_only_the_package_checked_by_its_header() {
        let dir = TempDir::new("pkg-unpack");
        let path = download(&dir);
        let into = dir.path().join("work/package");
        let mpi = unpack(&inspect(&path).unwrap(), &into).unwrap();
        assert_eq!(mpi, into.join("Tale of Two Wastelands 3.4.mpi"));
        assert_eq!(std::fs::read(&mpi).unwrap(), mpi_bytes());
        // The installer program and library are not kept, and the scratch folder is gone.
        let left: Vec<_> = std::fs::read_dir(dir.path().join("work"))
            .unwrap()
            .flatten()
            .map(|e| e.file_name())
            .collect();
        assert_eq!(left, ["package"]);
        assert_eq!(std::fs::read_dir(&into).unwrap().count(), 1);
        assert_eq!(unpacked_in(&into), Some(mpi));
        assert_eq!(unpacked_in(&dir.path().join("none")), None);
    }

    #[test]
    fn an_entry_that_is_not_a_package_by_content_is_refused_and_nothing_is_left() {
        let dir = TempDir::new("pkg-fake");
        let path = dir.path().join("fake.7z");
        make_7z(
            &path,
            &[("Tale of Two Wastelands 3.4.mpi", b"PK\x03\x04 not a bsa")],
        );
        let package = inspect(&path).unwrap();
        let into = dir.path().join("work/package");
        let why = unpack(&package, &into).unwrap_err();
        assert!(matches!(why, Problem::NotAPackage { .. }), "{why:?}");
        assert!(why.to_string().contains("BSA"), "{why}");
        assert!(!into.exists());
        assert_eq!(
            std::fs::read_dir(dir.path().join("work")).unwrap().count(),
            0
        );
    }

    #[test]
    fn sizes_are_shown_in_plain_units() {
        assert_eq!(size_text(1_850_000_000), "1.8 GB");
        assert_eq!(size_text(640_000_001), "641 MB");
        assert_eq!(size_text(0), "0 MB");
    }
}
