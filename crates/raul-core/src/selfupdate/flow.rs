// SPDX-License-Identifier: GPL-3.0-only
//! The steps the person takes (ADR-0020): download and verify and stage an update, install what was staged, go back.
//! Each step is a function over an install folder and an injected [`Fetch`] or check, so the command line and the
//! window call the same code and the tests need no network and no second program.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::extract::{self, Limits};
use super::fetch;
use super::install;
use super::layout::Layout;
use super::swap;
use super::verify::{self, Fetch};

const STAGED: &str = "staged.json";
const DOWNLOAD: &str = "download";

/// What a download left ready to install.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Staged {
    /// The version that is installed now.
    pub from: String,
    /// The version staged.
    pub to: String,
    /// The files of the unit, relative to the install folder.
    pub files: Vec<String>,
    /// The SHA-256 of the one staged file of a single-file install (an `AppImage`); `None` for a folder, whose
    /// files are checked against the inventory that came with them.
    #[serde(default)]
    pub sha256: Option<String>,
}

/// Why a step did not happen. Each message is plain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// This install cannot be updated from inside raul; the reason.
    NotHere(&'static str),
    /// The release was refused or a download did not check out.
    Verify(verify::Error),
    /// The archive was refused.
    Unpack(extract::Error),
    /// The swap or the new version's check failed.
    Install(install::Error),
    /// Going back failed.
    Swap(swap::Error),
    /// Nothing has been downloaded yet.
    NothingStaged,
    /// A file could not be read or written.
    Io(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotHere(why) => f.write_str(why),
            Self::Verify(e) => write!(f, "{e}"),
            Self::Unpack(e) => write!(f, "{e}"),
            Self::Install(e) => write!(f, "{e}"),
            Self::Swap(e) => write!(f, "{e}"),
            Self::NothingStaged => f.write_str("There is no update downloaded yet."),
            Self::Io(_) => f.write_str("The update could not be prepared."),
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    /// The technical text, with paths, for the details a person can copy. [`std::fmt::Display`] holds none.
    #[must_use]
    pub fn detail(&self) -> String {
        match self {
            Self::Io(why) => why.clone(),
            Self::Unpack(e) => e.detail(),
            other => other.to_string(),
        }
    }
}

fn io(path: &Path, e: &std::io::Error) -> Error {
    Error::Io(format!("{}: {e}", path.display()))
}

fn work(root: &Path) -> PathBuf {
    root.join(swap::WORK)
}

fn staged_file(root: &Path) -> PathBuf {
    work(root).join(STAGED)
}

fn updatable(layout: &Layout) -> Result<&Path, Error> {
    match layout {
        Layout::NotHere(reason) => Err(Error::NotHere(reason.plain())),
        other => other.root().ok_or(Error::NotHere(
            "This copy of raul cannot be updated from here.",
        )),
    }
}

/// What a download left, if anything.
///
/// # Errors
/// [`Error::Io`] when the record exists and cannot be read.
pub fn staged(root: &Path) -> Result<Option<Staged>, Error> {
    let file = staged_file(root);
    match std::fs::read_to_string(&file) {
        Ok(text) => serde_json::from_str(&text)
            .map(Some)
            .map_err(|e| Error::Io(format!("{}: {e}", file.display()))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(io(&file, &e)),
    }
}

/// Download the release tagged `tag`, check it, unpack it into the staging folder and record it. Asks the release page
/// and the release's files, and nothing else.
///
/// # Errors
/// [`Error::NotHere`] for an install that cannot be updated, [`Error::Verify`] when the release is refused or a file does
/// not check out, [`Error::Unpack`] when the archive is refused, [`Error::Io`] otherwise. The staging folder is emptied
/// when anything fails.
pub fn download(
    fetch_with: &dyn Fetch,
    layout: &Layout,
    tag: &str,
    running: &str,
    allow_older: bool,
) -> Result<Staged, Error> {
    let root = updatable(layout)?;
    let dir = work(root).join(DOWNLOAD);
    let _ = std::fs::remove_dir_all(&dir);
    let result = (|| {
        let release = fetch::release(fetch_with, tag, &dir).map_err(Error::Verify)?;
        let version = verify::accept(&release, running, allow_older).map_err(Error::Verify)?;
        let asset = layout.asset(&version.to_string()).ok_or(Error::NotHere(
            "This copy of raul cannot be updated from here.",
        ))?;
        let got = verify::download(fetch_with, &release, running, allow_older, &asset, &dir)
            .map_err(Error::Verify)?;
        let new = swap::staging_dir(root);
        let _ = std::fs::remove_dir_all(&new);
        let mut sha256 = None;
        let files = match layout {
            Layout::Folder { os, .. } => {
                let limits = Limits::default();
                if *os == super::layout::Os::Windows {
                    extract::extract_zip(&got.file, &new, 1, limits)
                } else {
                    extract::extract_tar_gz(&got.file, &new, 1, limits)
                }
                .map_err(Error::Unpack)?;
                extract::check_unit(&new).map_err(Error::Unpack)?
            }
            Layout::AppImage { file } => {
                let name = file
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .ok_or(Error::NotHere(
                        "This copy of raul cannot be updated from here.",
                    ))?;
                std::fs::create_dir_all(&new).map_err(|e| io(&new, &e))?;
                std::fs::copy(&got.file, new.join(&name)).map_err(|e| io(&new, &e))?;
                sha256 = Some(got.sha256.clone());
                vec![name]
            }
            Layout::NotHere(_) => {
                return Err(Error::NotHere(
                    "This copy of raul cannot be updated from here.",
                ));
            }
        };
        let staged = Staged {
            from: running.to_owned(),
            to: version.to_string(),
            files,
            sha256,
        };
        let text = serde_json::to_string_pretty(&staged).map_err(|e| Error::Io(e.to_string()))?;
        let record = staged_file(root);
        std::fs::write(&record, text).map_err(|e| io(&record, &e))?;
        Ok(staged)
    })();
    let _ = std::fs::remove_dir_all(&dir);
    if result.is_err() {
        let _ = std::fs::remove_dir_all(swap::staging_dir(root));
        let _ = std::fs::remove_file(staged_file(root));
    }
    result
}

/// Install what [`download`] staged: swap it in and ask `check` (given the new program and the version) whether it works;
/// a version that does not is taken out again at once. The record of the download is removed once it is installed.
///
/// # Errors
/// [`Error::NothingStaged`], [`Error::NotHere`], or [`Error::Install`] with the reason the update did not stay.
pub fn install(
    layout: &Layout,
    check: &dyn Fn(&Path, &str) -> Result<(), String>,
) -> Result<Staged, Error> {
    let root = updatable(layout)?;
    let staged = staged(root)?.ok_or(Error::NothingStaged)?;
    // The record is only a note of what was staged: what is there now must still be what was checked, whatever the
    // record says (it may be stale or edited).
    if let Err(e) = recheck(layout, root, &staged) {
        forget(root);
        return Err(e);
    }
    let program = layout.program().ok_or(Error::NotHere(
        "This copy of raul cannot be updated from here.",
    ))?;
    let done = install::install(root, &staged.files, &staged.from, &staged.to, &|| {
        check(&program, &staged.to)
    });
    // Whatever happened, the staged update is spent: installed, or taken out again.
    forget(root);
    done.map_err(Error::Install)?;
    Ok(staged)
}

/// Go back to the version before the last update. A download that was waiting is dropped with it: it was an update
/// to the version that is now being left.
///
/// # Errors
/// [`Error::NotHere`] or [`Error::Swap`] (including "there is no earlier version to go back to").
pub fn rollback(layout: &Layout) -> Result<swap::Journal, Error> {
    let root = updatable(layout)?;
    let journal = swap::rollback(root).map_err(Error::Swap)?;
    forget(root);
    Ok(journal)
}

/// Drop the record of a download and what it staged.
fn forget(root: &Path) {
    let _ = std::fs::remove_file(staged_file(root));
    let _ = std::fs::remove_dir_all(swap::staging_dir(root));
}

/// Check that what is staged is what the record says and what was verified: a folder's files against the inventory
/// they came with (and the record's list must be exactly that list), an `AppImage` against the hash taken when it was
/// downloaded.
fn recheck(layout: &Layout, root: &Path, staged: &Staged) -> Result<(), Error> {
    let new = swap::staging_dir(root);
    match layout {
        Layout::Folder { .. } => {
            let mut recorded = staged.files.clone();
            recorded.sort();
            let actual = extract::check_unit(&new).map_err(Error::Unpack)?;
            if actual == recorded {
                Ok(())
            } else {
                Err(Error::Unpack(extract::Error::Unlisted(String::new())))
            }
        }
        Layout::AppImage { .. } => {
            let (Some(name), Some(want)) = (staged.files.first(), staged.sha256.as_deref()) else {
                return Err(Error::NothingStaged);
            };
            let ok = staged.files.len() == 1
                && !name.contains(['/', '\\'])
                && verify::hash_file(&new.join(name)).is_ok_and(|h| h == want);
            if ok {
                Ok(())
            } else {
                Err(Error::Unpack(extract::Error::Mismatch(name.clone())))
            }
        }
        Layout::NotHere(_) => Err(Error::NotHere(
            "This copy of raul cannot be updated from here.",
        )),
    }
}

/// What the person may be told about the update state of this install.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Status {
    /// Whether this install can be updated from inside raul.
    pub updatable: bool,
    /// When it cannot: why, in plain words.
    pub reason: Option<String>,
    /// The version downloaded and waiting to be installed.
    pub staged: Option<String>,
    /// The version an earlier update replaced, which going back restores.
    pub previous: Option<String>,
    /// Whether the version now installed has been confirmed by starting it.
    pub confirmed: bool,
}

/// The update state of the install.
///
/// # Errors
/// [`Error::Io`] or [`Error::Swap`] when a record cannot be read.
pub fn status(layout: &Layout) -> Result<Status, Error> {
    let Some(root) = layout.root() else {
        let reason = match layout {
            Layout::NotHere(r) => Some(r.plain().to_owned()),
            _ => None,
        };
        return Ok(Status {
            updatable: false,
            reason,
            staged: None,
            previous: None,
            confirmed: false,
        });
    };
    let journal = swap::journal(root).map_err(Error::Swap)?;
    let finished = journal
        .as_ref()
        .filter(|j| matches!(j.state, swap::State::Swapped | swap::State::Confirmed));
    Ok(Status {
        updatable: true,
        reason: None,
        staged: staged(root)?.map(|s| s.to),
        previous: finished.map(|j| j.from.clone()),
        confirmed: finished.is_some_and(|j| j.state == swap::State::Confirmed),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selfupdate::extract::INVENTORY;
    use crate::selfupdate::layout::{Os, Reason};
    use crate::selfupdate::verify::{Asset, Error as VerifyError, REPO, Release, SUMS};
    use nvmod_testkit::{TarBuilder, TempDir, write_file};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::io::Write;

    struct Fake(RefCell<HashMap<String, Vec<u8>>>);

    impl Fetch for Fake {
        fn get(&self, url: &str, to: &Path, _limit: u64) -> Result<(), VerifyError> {
            let bytes = self
                .0
                .borrow()
                .get(url)
                .cloned()
                .ok_or_else(|| VerifyError::Fetch("404".into()))?;
            std::fs::write(to, bytes).map_err(|e| VerifyError::Fetch(e.to_string()))
        }
    }

    fn hash(b: &[u8]) -> String {
        raul_runner::sha256_hex(b)
    }

    fn gz(tar: &[u8]) -> Vec<u8> {
        let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        e.write_all(tar).unwrap();
        e.finish().unwrap()
    }

    /// A release 2.0.0 whose tarball holds `raul` and its list; `damage` changes `raul` after the list was made.
    fn server(asset: &str, body: &[u8]) -> Fake {
        let tag = "v2.0.0";
        let release = Release {
            id: 9,
            tag_name: tag.to_owned(),
            draft: false,
            prerelease: false,
            assets: [SUMS, asset]
                .iter()
                .map(|n| Asset {
                    name: (*n).to_owned(),
                    browser_download_url: format!(
                        "https://github.com/{REPO}/releases/download/{tag}/{n}"
                    ),
                })
                .collect(),
        };
        let json = serde_json::json!({
            "id": release.id, "tag_name": release.tag_name,
            "assets": release.assets.iter().map(|a| serde_json::json!({"name": a.name, "browser_download_url": a.browser_download_url})).collect::<Vec<_>>(),
        });
        let mut files = HashMap::new();
        files.insert(fetch::release_url(tag), json.to_string().into_bytes());
        files.insert(release.address_of(asset), body.to_vec());
        files.insert(
            release.address_of(SUMS),
            format!("{}  {asset}\n", hash(body)).into_bytes(),
        );
        Fake(RefCell::new(files))
    }

    fn tarball(raul: &[u8], list_for: &[u8]) -> Vec<u8> {
        let inv = format!("{}  raul\n", hash(list_for));
        gz(&TarBuilder::new()
            .dir("raul-2.0.0/")
            .file("raul-2.0.0/raul", raul)
            .file(&format!("raul-2.0.0/{INVENTORY}"), inv.as_bytes())
            .bytes())
    }

    fn folder() -> (TempDir, Layout) {
        let t = TempDir::new("flow");
        write_file(t.path(), "raul", b"raul 1");
        write_file(
            t.path(),
            INVENTORY,
            format!("{}  raul\n", hash(b"raul 1")).as_bytes(),
        );
        let l = Layout::Folder {
            root: t.path().to_path_buf(),
            exe: "raul".to_owned(),
            os: Os::Linux,
        };
        (t, l)
    }

    fn read(t: &TempDir, n: &str) -> String {
        std::fs::read_to_string(t.path().join(n)).unwrap()
    }

    const ASSET: &str = "raul-2.0.0-linux-x64.tar.gz";

    #[test]
    fn download_then_install_replaces_the_files_and_keeps_the_previous_version() {
        let (t, l) = folder();
        let s = server(ASSET, &tarball(b"raul 2", b"raul 2"));
        let staged = download(&s, &l, "v2.0.0", "1.0.0", false).unwrap();
        assert_eq!(
            (staged.from.as_str(), staged.to.as_str()),
            ("1.0.0", "2.0.0")
        );
        assert_eq!(status(&l).unwrap().staged.as_deref(), Some("2.0.0"));
        assert_eq!(
            read(&t, "raul"),
            "raul 1",
            "downloading changes nothing installed"
        );
        let asked = RefCell::new(Vec::new());
        install(&l, &|program, version| {
            asked
                .borrow_mut()
                .push((program.to_path_buf(), version.to_owned()));
            Ok(())
        })
        .unwrap();
        assert_eq!(read(&t, "raul"), "raul 2");
        assert_eq!(
            asked.into_inner(),
            vec![(t.path().join("raul"), "2.0.0".to_owned())]
        );
        let st = status(&l).unwrap();
        assert_eq!(
            (st.previous.as_deref(), st.staged, st.confirmed),
            (Some("1.0.0"), None, false)
        );
        rollback(&l).unwrap();
        assert_eq!(read(&t, "raul"), "raul 1");
    }

    #[test]
    fn a_new_version_that_fails_its_check_is_taken_out_again() {
        let (t, l) = folder();
        let s = server(ASSET, &tarball(b"raul 2", b"raul 2"));
        download(&s, &l, "v2.0.0", "1.0.0", false).unwrap();
        let e = install(&l, &|_, _| Err("The new version did not start.".to_owned())).unwrap_err();
        assert!(
            e.to_string().contains("went back to the old version"),
            "{e}"
        );
        assert_eq!(read(&t, "raul"), "raul 1");
    }

    #[test]
    fn a_damaged_archive_a_list_that_disagrees_and_an_older_version_stage_nothing() {
        let (t, l) = folder();
        // The list was made for other bytes than the file holds.
        let s = server(ASSET, &tarball(b"raul 2", b"something else"));
        let e = download(&s, &l, "v2.0.0", "1.0.0", false).unwrap_err();
        assert!(matches!(e, Error::Unpack(_)), "{e:?}");
        assert_eq!(status(&l).unwrap().staged, None);
        assert!(!swap::staging_dir(t.path()).exists());
        // Not newer.
        let s = server(ASSET, &tarball(b"raul 2", b"raul 2"));
        assert!(matches!(
            download(&s, &l, "v2.0.0", "2.0.0", false),
            Err(Error::Verify(_))
        ));
        assert!(
            download(&s, &l, "v2.0.0", "2.0.0", true).is_ok(),
            "an older version only when asked for"
        );
        // Nothing staged: install says so.
        let (_t2, l2) = folder();
        assert_eq!(
            install(&l2, &|_, _| Ok(())).unwrap_err(),
            Error::NothingStaged
        );
    }

    #[test]
    fn an_appimage_is_staged_as_its_one_file_and_swapped() {
        let t = TempDir::new("flow-appimage");
        write_file(t.path(), "raul.AppImage", b"image 1");
        let l = Layout::AppImage {
            file: t.path().join("raul.AppImage"),
        };
        let asset = "raul-2.0.0-linux-x64.AppImage";
        let s = server(asset, b"image 2");
        let staged = download(&s, &l, "v2.0.0", "1.0.0", false).unwrap();
        assert_eq!(staged.files, ["raul.AppImage"]);
        install(&l, &|_, _| Ok(())).unwrap();
        assert_eq!(read(&t, "raul.AppImage"), "image 2");
        rollback(&l).unwrap();
        assert_eq!(read(&t, "raul.AppImage"), "image 1");
    }

    #[test]
    fn an_install_that_cannot_be_updated_says_why_in_plain_words_and_asks_for_nothing() {
        let l = Layout::NotHere(Reason::Nix);
        let s = server(ASSET, b"x");
        let e = download(&s, &l, "v2.0.0", "1.0.0", false).unwrap_err();
        assert_eq!(e, Error::NotHere(Reason::Nix.plain()));
        assert!(matches!(
            install(&l, &|_, _| Ok(())),
            Err(Error::NotHere(_))
        ));
        assert!(matches!(rollback(&l), Err(Error::NotHere(_))));
        let st = status(&l).unwrap();
        assert!(!st.updatable && st.reason.as_deref() == Some(Reason::Nix.plain()));
    }

    #[test]
    fn a_failed_download_leaves_no_staged_record_and_no_scratch_files() {
        let (t, l) = folder();
        let s = Fake(RefCell::new(HashMap::new()));
        assert!(download(&s, &l, "v2.0.0", "1.0.0", false).is_err());
        let work = t.path().join(swap::WORK);
        assert!(!work.join(STAGED).exists() && !work.join(DOWNLOAD).exists());
    }

    #[test]
    fn a_failed_install_and_a_rollback_leave_no_record_of_a_download() {
        let (t, l) = folder();
        let s = server(ASSET, &tarball(b"raul 2", b"raul 2"));
        download(&s, &l, "v2.0.0", "1.0.0", false).unwrap();
        install(&l, &|_, _| Err("The new version did not start.".to_owned())).unwrap_err();
        assert_eq!(
            status(&l).unwrap().staged,
            None,
            "a rolled back install is not advertised as staged"
        );
        assert!(!swap::staging_dir(t.path()).exists());
        assert_eq!(
            install(&l, &|_, _| Ok(())).unwrap_err(),
            Error::NothingStaged
        );
        // A download waiting when the person goes back is dropped with it.
        download(&s, &l, "v2.0.0", "1.0.0", false).unwrap();
        install(&l, &|_, _| Ok(())).unwrap();
        download(&s, &l, "v2.0.0", "1.0.0", true).unwrap();
        rollback(&l).unwrap();
        assert_eq!(status(&l).unwrap().staged, None);
    }

    #[test]
    fn what_is_staged_is_checked_again_before_it_is_swapped_in() {
        let (t, l) = folder();
        let s = server(ASSET, &tarball(b"raul 2", b"raul 2"));
        download(&s, &l, "v2.0.0", "1.0.0", false).unwrap();
        // A staged file changed after the download.
        write_file(&swap::staging_dir(t.path()), "raul", b"tampered");
        let e = install(&l, &|_, _| Ok(())).unwrap_err();
        assert!(matches!(e, Error::Unpack(_)), "{e:?}");
        assert_eq!(read(&t, "raul"), "raul 1", "nothing was swapped");
        assert_eq!(
            status(&l).unwrap().staged,
            None,
            "the spoiled download is dropped"
        );
        // A record that names files the staging does not hold, or fewer than it holds.
        download(&s, &l, "v2.0.0", "1.0.0", false).unwrap();
        let mut record = staged(t.path()).unwrap().unwrap();
        record.files = vec!["raul".to_owned()];
        std::fs::write(
            staged_file(t.path()),
            serde_json::to_string(&record).unwrap(),
        )
        .unwrap();
        assert!(
            install(&l, &|_, _| Ok(())).is_err(),
            "an edited list is not trusted"
        );
        assert_eq!(read(&t, "raul"), "raul 1");
    }

    #[test]
    fn a_staged_appimage_is_checked_against_the_hash_taken_at_download() {
        let t = TempDir::new("flow-appimage-recheck");
        write_file(t.path(), "raul.AppImage", b"image 1");
        let l = Layout::AppImage {
            file: t.path().join("raul.AppImage"),
        };
        let s = server("raul-2.0.0-linux-x64.AppImage", b"image 2");
        download(&s, &l, "v2.0.0", "1.0.0", false).unwrap();
        write_file(
            &swap::staging_dir(t.path()),
            "raul.AppImage",
            b"swapped for something else",
        );
        assert!(install(&l, &|_, _| Ok(())).is_err());
        assert_eq!(read(&t, "raul.AppImage"), "image 1");
        // A record without the hash (written by hand) is refused too.
        download(&s, &l, "v2.0.0", "1.0.0", false).unwrap();
        let mut record = staged(t.path()).unwrap().unwrap();
        record.sha256 = None;
        std::fs::write(
            staged_file(t.path()),
            serde_json::to_string(&record).unwrap(),
        )
        .unwrap();
        assert!(install(&l, &|_, _| Ok(())).is_err());
    }
}
