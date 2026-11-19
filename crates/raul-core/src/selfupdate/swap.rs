// SPDX-License-Identifier: GPL-3.0-only
//! The atomic replacement of an install unit and its rollback (ADR-0020).
//!
//! A unit is a list of files, relative to the install folder (`raul`, `components/nv-rs/...`, the notices). The new
//! files are staged first under `<root>/.raul-update/new/`, on the same filesystem, so no step copies. The swap writes
//! a journal before the first rename, then moves each installed file to `<root>/.raul-update/previous/` and each new
//! file into its place. A crash at any point is repaired by [`recover`], which puts every file back to the old
//! version; a finished swap is undone by [`rollback`]. Renames are atomic per file and work for a running program
//! on Windows too (it can be renamed, not overwritten).

use std::fmt;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The folder in the install folder that holds the staging, the previous version and the journal.
pub const WORK: &str = ".raul-update";
const NEW: &str = "new";
const PREVIOUS: &str = "previous";
const JOURNAL: &str = "journal.json";

/// Where the swap stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum State {
    /// The renames are running or were interrupted: [`recover`] undoes them.
    Swapping,
    /// Every file is new and the previous version is kept; the new version has not yet passed its check.
    Swapped,
    /// The new version passed its check; the previous version is still kept for [`rollback`].
    Confirmed,
    /// The previous version is back.
    RolledBack,
}

/// What the journal records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Journal {
    /// Where the swap stands.
    pub state: State,
    /// The version that was installed before.
    pub from: String,
    /// The version that was installed by the swap.
    pub to: String,
    /// The files of the unit and whether each had an installed copy.
    pub files: Vec<Entry>,
}

/// One file of the unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// The path relative to the install folder, with `/` separators.
    pub path: String,
    /// Whether an installed copy existed before the swap (a file new in this version has none).
    pub had_old: bool,
}

/// Why a swap, a recovery or a rollback did not happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A path of the unit is absolute, leaves the folder or is empty; nothing was touched.
    BadPath(String),
    /// A staged file is missing; nothing was touched.
    NotStaged(String),
    /// A swap is already recorded and has not been settled; nothing was touched.
    Unsettled,
    /// There is no previous version to go back to.
    NothingToRollBack,
    /// A path of the unit is a link or a folder where a file is expected; nothing was touched.
    NotPlain(String),
    /// Another update, rollback or recovery holds the lock of this install; nothing was touched.
    Locked,
    /// A file operation failed; the message names the path and the reason.
    Io(String),
    /// The test hook that stands in for a crash stopped the swap here.
    #[cfg(test)]
    Interrupted,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadPath(p) => {
                write!(f, "the update names a path outside the install folder: {p}")
            }
            Self::NotStaged(p) => write!(f, "the update is missing a file: {p}"),
            Self::Unsettled => {
                f.write_str("an earlier update has not finished; it is settled first")
            }
            Self::NothingToRollBack => f.write_str("there is no earlier version to go back to"),
            Self::NotPlain(p) => write!(f, "{p} is not a plain file"),
            Self::Locked => f.write_str("another update of this install is running"),
            Self::Io(why) => f.write_str(why),
            #[cfg(test)]
            Self::Interrupted => f.write_str("interrupted"),
        }
    }
}

impl std::error::Error for Error {}

fn io(path: &Path, e: &std::io::Error) -> Error {
    Error::Io(format!("{}: {e}", path.display()))
}

/// The folder the new files are staged in; the caller writes them there before [`apply`].
#[must_use]
pub fn staging_dir(root: &Path) -> PathBuf {
    root.join(WORK).join(NEW)
}

fn previous_dir(root: &Path) -> PathBuf {
    root.join(WORK).join(PREVIOUS)
}

fn journal_file(root: &Path) -> PathBuf {
    root.join(WORK).join(JOURNAL)
}

fn checked(path: &str) -> Result<PathBuf, Error> {
    let relative = Path::new(path);
    let inside = !path.is_empty()
        && relative.is_relative()
        && relative
            .components()
            .all(|c| matches!(c, Component::Normal(_)));
    // The work folder is ours: a unit may not name it, in any letter case (Windows and macOS ignore case).
    let reserved = relative
        .components()
        .next()
        .is_some_and(|c| c.as_os_str().to_string_lossy().eq_ignore_ascii_case(WORK));
    if inside && !reserved {
        Ok(relative.to_path_buf())
    } else {
        Err(Error::BadPath(path.to_owned()))
    }
}

/// The journal of the install at `root`, if one is recorded.
///
/// # Errors
/// [`Error::Io`] when the journal exists and cannot be read or understood.
pub fn journal(root: &Path) -> Result<Option<Journal>, Error> {
    let file = journal_file(root);
    match std::fs::read_to_string(&file) {
        Ok(text) => serde_json::from_str(&text)
            .map(Some)
            .map_err(|e| Error::Io(format!("{}: {e}", file.display()))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(io(&file, &e)),
    }
}

fn write_journal(root: &Path, journal: &Journal) -> Result<(), Error> {
    let file = journal_file(root);
    let text = serde_json::to_string_pretty(journal).map_err(|e| Error::Io(e.to_string()))?;
    let temporary = file.with_extension("json.new");
    let written = (|| {
        let mut f = std::fs::File::create(&temporary)?;
        std::io::Write::write_all(&mut f, text.as_bytes())?;
        f.sync_all()?;
        std::fs::rename(&temporary, &file)?;
        file.parent().map(sync_dir);
        Ok::<(), std::io::Error>(())
    })();
    written.map_err(|e| io(&file, &e))
}

/// Whether `path` is a regular file. A link, even to a file, and a folder are not.
fn plain_file(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_file())
}

/// Whether anything stands at `path`, a dangling link included.
fn present(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok()
}

/// Flush a folder's entries to disk so a rename in it survives a power cut. A no-op where a folder cannot be opened
/// for it (Windows); the journal is what repairs a swap there.
fn sync_dir(dir: &Path) {
    #[cfg(unix)]
    if let Ok(f) = std::fs::File::open(dir) {
        let _ = f.sync_all();
    }
    #[cfg(not(unix))]
    let _ = dir;
}

/// The lock of an install while an update, rollback or recovery changes it. It is a file holding the process id,
/// removed on drop; one left by a process that no longer exists (checked where `/proc` shows it) is taken over.
struct Lock(PathBuf);

impl Lock {
    fn take(root: &Path) -> Result<Self, Error> {
        let dir = root.join(WORK);
        std::fs::create_dir_all(&dir).map_err(|e| io(&dir, &e))?;
        let file = dir.join("lock");
        for _ in 0..2 {
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&file)
            {
                Ok(mut f) => {
                    let _ = std::io::Write::write_all(
                        &mut f,
                        std::process::id().to_string().as_bytes(),
                    );
                    return Ok(Self(file));
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    if Self::stale(&file) {
                        let _ = std::fs::remove_file(&file);
                    } else {
                        return Err(Error::Locked);
                    }
                }
                Err(e) => return Err(io(&file, &e)),
            }
        }
        Err(Error::Locked)
    }

    fn stale(file: &Path) -> bool {
        let Some(pid) = std::fs::read_to_string(file)
            .ok()
            .and_then(|t| t.trim().parse::<u32>().ok())
        else {
            return false;
        };
        if Path::new("/proc/self").exists() {
            return pid != std::process::id() && !Path::new("/proc").join(pid.to_string()).exists();
        }
        // Where a process cannot be looked up, a lock that nothing has touched for ten minutes is a dead one's.
        std::fs::metadata(file)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age > std::time::Duration::from_mins(10))
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn make_parent(path: &Path) -> Result<(), Error> {
    match path.parent() {
        Some(dir) => std::fs::create_dir_all(dir).map_err(|e| io(dir, &e)),
        None => Ok(()),
    }
}

fn mv(from: &Path, to: &Path) -> Result<(), Error> {
    make_parent(to)?;
    std::fs::rename(from, to).map_err(|e| io(from, &e))?;
    for dir in [from.parent(), to.parent()].into_iter().flatten() {
        sync_dir(dir);
    }
    Ok(())
}

/// Replace the unit: `files` (relative paths) staged under [`staging_dir`] become the installed files, and what was
/// installed is kept under the previous folder. The journal is written first and ends as [`State::Swapped`].
///
/// # Errors
/// [`Error::BadPath`] or [`Error::NotStaged`] before anything is touched; [`Error::Unsettled`] when an earlier swap is
/// still recorded as running; [`Error::Io`] when a rename fails, in which case the files already moved are put back.
pub fn apply(root: &Path, files: &[String], from: &str, to: &str) -> Result<(), Error> {
    apply_until(root, files, from, to, usize::MAX)
}

/// [`apply`] that stops dead after `renames` renames, as a crash would, leaving the journal as it stands.
fn apply_until(
    root: &Path,
    files: &[String],
    from: &str,
    to: &str,
    renames: usize,
) -> Result<(), Error> {
    let _lock = Lock::take(root)?;
    if journal(root)?.is_some_and(|j| j.state == State::Swapping) {
        return Err(Error::Unsettled);
    }
    let new = staging_dir(root);
    let mut entries = Vec::new();
    for file in files {
        let relative = checked(file)?;
        if !plain_file(&new.join(&relative)) {
            return Err(if present(&new.join(&relative)) {
                Error::NotPlain(file.clone())
            } else {
                Error::NotStaged(file.clone())
            });
        }
        let installed = root.join(&relative);
        if present(&installed) && !plain_file(&installed) {
            return Err(Error::NotPlain(file.clone()));
        }
        entries.push(Entry {
            path: file.clone(),
            had_old: present(&installed),
        });
    }
    let mut journal = Journal {
        state: State::Swapping,
        from: from.to_owned(),
        to: to.to_owned(),
        files: entries,
    };
    // The journal comes first. The version kept from an earlier update is set aside, not deleted, until this swap has
    // finished, so an interrupted swap leaves that rollback target in place.
    write_journal(root, &journal)?;
    let previous = previous_dir(root);
    let aside = aside_dir(root);
    if present(&aside) {
        std::fs::remove_dir_all(&aside).map_err(|e| io(&aside, &e))?;
    }
    if present(&previous) {
        std::fs::rename(&previous, &aside).map_err(|e| io(&previous, &e))?;
    }
    let mut done = 0usize;
    let result = (|| {
        for entry in &journal.files {
            let relative = checked(&entry.path)?;
            if entry.had_old {
                stop_after(&mut done, renames)?;
                mv(&root.join(&relative), &previous.join(&relative))?;
            }
            stop_after(&mut done, renames)?;
            mv(&new.join(&relative), &root.join(&relative))?;
        }
        Ok(())
    })();
    if let Err(e) = result {
        // A real failure puts the files back at once; a stop (a crash in a test) leaves the journal for `recover`.
        if renames == usize::MAX {
            let _ = restore(root, &journal);
            journal.state = State::RolledBack;
            let _ = write_journal(root, &journal);
        }
        return Err(e);
    }
    journal.state = State::Swapped;
    write_journal(root, &journal)?;
    let _ = std::fs::remove_dir_all(&aside);
    Ok(())
}

fn aside_dir(root: &Path) -> PathBuf {
    root.join(WORK).join("previous.old")
}

/// Count one rename; in a test, stop dead when `limit` renames are done.
#[cfg_attr(not(test), allow(clippy::unnecessary_wraps))] // fallible only as the test crash hook
fn stop_after(done: &mut usize, limit: usize) -> Result<(), Error> {
    #[cfg(test)]
    if *done >= limit {
        return Err(Error::Interrupted);
    }
    #[cfg(not(test))]
    let _ = limit;
    *done += 1;
    Ok(())
}

/// Put every file of `journal` back to the version it had before the swap.
fn restore(root: &Path, journal: &Journal) -> Result<(), Error> {
    let previous = previous_dir(root);
    let new = staging_dir(root);
    for entry in journal.files.iter().rev() {
        let relative = checked(&entry.path)?;
        let installed = root.join(&relative);
        let kept = previous.join(&relative);
        if plain_file(&kept) {
            // The old file was moved away: whatever stands in its place is the new one.
            if plain_file(&installed) {
                mv(&installed, &new.join(&relative))?;
            }
            mv(&kept, &installed)?;
        } else if !entry.had_old && plain_file(&installed) {
            // A file the new version added.
            mv(&installed, &new.join(&relative))?;
        }
    }
    // What this swap set aside is the rollback target of the update before it: give it back.
    let aside = aside_dir(root);
    if present(&aside) {
        if present(&previous) {
            std::fs::remove_dir_all(&previous).map_err(|e| io(&previous, &e))?;
        }
        std::fs::rename(&aside, &previous).map_err(|e| io(&aside, &e))?;
    }
    Ok(())
}

/// Settle what an interrupted run left: a swap that was running is undone, so the install is entirely the old version.
/// Call it when raul starts, before anything reads the install. Returns the journal as it ended up, or `None`.
///
/// # Errors
/// [`Error::Io`] when a file cannot be moved back or the journal cannot be written.
pub fn recover(root: &Path) -> Result<Option<Journal>, Error> {
    if !present(&journal_file(root)) {
        return Ok(None);
    }
    let _lock = Lock::take(root)?;
    let Some(mut j) = journal(root)? else {
        return Ok(None);
    };
    if j.state == State::Swapping {
        restore(root, &j)?;
        j.state = State::RolledBack;
        write_journal(root, &j)?;
    }
    Ok(Some(j))
}

/// Record that the new version passed its check.
///
/// # Errors
/// [`Error::NothingToRollBack`] when no finished swap is recorded; [`Error::Io`] when the journal cannot be written.
pub fn confirm(root: &Path) -> Result<(), Error> {
    let _lock = Lock::take(root)?;
    match journal(root)? {
        Some(mut j) if matches!(j.state, State::Swapped | State::Confirmed) => {
            j.state = State::Confirmed;
            write_journal(root, &j)
        }
        _ => Err(Error::NothingToRollBack),
    }
}

/// Go back to the previous version after a finished swap, confirmed or not. The replaced version is left in the
/// staging folder, where a new update replaces it.
///
/// # Errors
/// [`Error::NothingToRollBack`] when no finished swap is recorded; [`Error::Io`] when a file cannot be moved.
pub fn rollback(root: &Path) -> Result<Journal, Error> {
    let _lock = Lock::take(root)?;
    let Some(mut j) = journal(root)? else {
        return Err(Error::NothingToRollBack);
    };
    if !matches!(j.state, State::Swapped | State::Confirmed) {
        return Err(Error::NothingToRollBack);
    }
    // Journal first, so a crash in the middle is repaired by `recover` to the old version.
    j.state = State::Swapping;
    write_journal(root, &j)?;
    restore(root, &j)?;
    j.state = State::RolledBack;
    write_journal(root, &j)?;
    Ok(j)
}

/// Remove the staging area (after a settled swap). The previous version stays.
///
/// # Errors
/// [`Error::Io`] when the folder cannot be removed.
pub fn clear_staging(root: &Path) -> Result<(), Error> {
    let new = staging_dir(root);
    match std::fs::remove_dir_all(&new) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(io(&new, &e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nvmod_testkit::{TempDir, write_file};

    const FILES: &[&str] = &["raul", "components/nv-rs/bin/nv-viewer", "NEW-IN-2"];

    fn names() -> Vec<String> {
        FILES.iter().map(|f| (*f).to_owned()).collect()
    }

    /// An install of version 1 (without `NEW-IN-2`) with version 2 staged.
    fn install() -> TempDir {
        let t = TempDir::new("swap");
        write_file(t.path(), "raul", b"raul 1");
        write_file(t.path(), "components/nv-rs/bin/nv-viewer", b"viewer 1");
        write_file(t.path(), "UNRELATED", b"keep");
        let new = staging_dir(t.path());
        write_file(&new, "raul", b"raul 2");
        write_file(&new, "components/nv-rs/bin/nv-viewer", b"viewer 2");
        write_file(&new, "NEW-IN-2", b"added");
        t
    }

    fn read(t: &TempDir, name: &str) -> Option<String> {
        std::fs::read_to_string(t.path().join(name)).ok()
    }

    fn all_old(t: &TempDir) {
        assert_eq!(read(t, "raul").as_deref(), Some("raul 1"));
        assert_eq!(
            read(t, "components/nv-rs/bin/nv-viewer").as_deref(),
            Some("viewer 1")
        );
        assert_eq!(read(t, "NEW-IN-2"), None);
        assert_eq!(read(t, "UNRELATED").as_deref(), Some("keep"));
    }

    fn all_new(t: &TempDir) {
        assert_eq!(read(t, "raul").as_deref(), Some("raul 2"));
        assert_eq!(
            read(t, "components/nv-rs/bin/nv-viewer").as_deref(),
            Some("viewer 2")
        );
        assert_eq!(read(t, "NEW-IN-2").as_deref(), Some("added"));
        assert_eq!(read(t, "UNRELATED").as_deref(), Some("keep"));
    }

    #[test]
    fn a_swap_replaces_every_file_and_keeps_the_previous_version() {
        let t = install();
        apply(t.path(), &names(), "1.0.0", "2.0.0").expect("swap");
        all_new(&t);
        let j = journal(t.path()).unwrap().unwrap();
        assert_eq!(
            (j.state, j.from.as_str(), j.to.as_str()),
            (State::Swapped, "1.0.0", "2.0.0")
        );
        let kept = t.path().join(WORK).join("previous");
        assert_eq!(
            std::fs::read_to_string(kept.join("raul")).unwrap(),
            "raul 1"
        );
        assert!(!kept.join("NEW-IN-2").exists());
    }

    #[test]
    fn a_rollback_brings_back_every_old_file_and_removes_the_added_one() {
        let t = install();
        apply(t.path(), &names(), "1.0.0", "2.0.0").unwrap();
        let j = rollback(t.path()).expect("rollback");
        assert_eq!(j.state, State::RolledBack);
        all_old(&t);
    }

    #[test]
    fn a_confirmed_update_can_still_be_rolled_back() {
        let t = install();
        apply(t.path(), &names(), "1.0.0", "2.0.0").unwrap();
        confirm(t.path()).unwrap();
        assert_eq!(journal(t.path()).unwrap().unwrap().state, State::Confirmed);
        rollback(t.path()).unwrap();
        all_old(&t);
    }

    #[test]
    fn a_rollback_with_nothing_to_go_back_to_is_refused() {
        let t = install();
        assert_eq!(rollback(t.path()), Err(Error::NothingToRollBack));
        assert_eq!(confirm(t.path()), Err(Error::NothingToRollBack));
        apply(t.path(), &names(), "1.0.0", "2.0.0").unwrap();
        rollback(t.path()).unwrap();
        assert_eq!(rollback(t.path()), Err(Error::NothingToRollBack));
    }

    #[test]
    fn a_crash_after_any_number_of_renames_is_repaired_to_all_old() {
        // Five renames in all: two files with an old copy move twice, the added file once.
        for stop in 0..5 {
            let t = install();
            let r = apply_until(t.path(), &names(), "1.0.0", "2.0.0", stop);
            assert!(r.is_err(), "stop {stop}");
            let j = recover(t.path()).unwrap().unwrap();
            assert_eq!(j.state, State::RolledBack, "stop {stop}");
            all_old(&t);
        }
    }

    #[test]
    fn recovering_twice_changes_nothing() {
        let t = install();
        let _ = apply_until(t.path(), &names(), "1.0.0", "2.0.0", 3);
        recover(t.path()).unwrap();
        recover(t.path()).unwrap();
        all_old(&t);
    }

    #[test]
    fn a_finished_swap_is_left_alone_by_recovery() {
        let t = install();
        apply(t.path(), &names(), "1.0.0", "2.0.0").unwrap();
        let j = recover(t.path()).unwrap().unwrap();
        assert_eq!(j.state, State::Swapped);
        all_new(&t);
        assert_eq!(recover(TempDir::new("swap-none").path()).unwrap(), None);
    }

    #[test]
    fn a_crash_while_rolling_back_is_repaired_to_the_old_version() {
        let t = install();
        apply(t.path(), &names(), "1.0.0", "2.0.0").unwrap();
        // The rollback's own journal write comes first, so an interruption leaves `swapping`: recovery finishes it.
        let mut j = journal(t.path()).unwrap().unwrap();
        j.state = State::Swapping;
        write_journal(t.path(), &j).unwrap();
        recover(t.path()).unwrap();
        all_old(&t);
    }

    #[test]
    fn a_missing_staged_file_or_a_path_outside_the_folder_touches_nothing() {
        let t = install();
        std::fs::remove_file(staging_dir(t.path()).join("NEW-IN-2")).unwrap();
        assert_eq!(
            apply(t.path(), &names(), "1.0.0", "2.0.0"),
            Err(Error::NotStaged("NEW-IN-2".to_owned()))
        );
        assert_eq!(journal(t.path()).unwrap(), None);
        for bad in ["../outside", "/etc/passwd", "", "a/../../b"] {
            let r = apply(t.path(), &[bad.to_owned()], "1.0.0", "2.0.0");
            assert_eq!(r, Err(Error::BadPath(bad.to_owned())), "{bad}");
        }
        assert_eq!(read(&t, "raul").as_deref(), Some("raul 1"));
    }

    #[test]
    fn a_second_swap_while_one_is_running_is_refused() {
        let t = install();
        let _ = apply_until(t.path(), &names(), "1.0.0", "2.0.0", 2);
        assert_eq!(
            apply(t.path(), &names(), "1.0.0", "2.0.0"),
            Err(Error::Unsettled)
        );
    }

    #[test]
    fn a_later_update_replaces_the_kept_previous_version() {
        let t = install();
        apply(t.path(), &names(), "1.0.0", "2.0.0").unwrap();
        let new = staging_dir(t.path());
        write_file(&new, "raul", b"raul 3");
        write_file(&new, "components/nv-rs/bin/nv-viewer", b"viewer 3");
        write_file(&new, "NEW-IN-2", b"added 3");
        apply(t.path(), &names(), "2.0.0", "3.0.0").unwrap();
        rollback(t.path()).unwrap();
        assert_eq!(read(&t, "raul").as_deref(), Some("raul 2"));
        assert_eq!(read(&t, "NEW-IN-2").as_deref(), Some("added"));
    }

    #[test]
    fn a_failing_rename_puts_the_files_back() {
        let t = install();
        // `components` is a file, so the viewer cannot be moved under it: the second file's rename fails after the
        // first was already swapped.
        std::fs::remove_dir_all(t.path().join("components")).unwrap();
        write_file(t.path(), "components", b"in the way");
        let files = vec![
            "raul".to_owned(),
            "components/nv-rs/bin/nv-viewer".to_owned(),
        ];
        let r = apply(t.path(), &files, "1.0.0", "2.0.0");
        assert!(matches!(r, Err(Error::Io(_))), "{r:?}");
        assert_eq!(read(&t, "raul").as_deref(), Some("raul 1"));
        assert_eq!(read(&t, "components").as_deref(), Some("in the way"));
        assert_eq!(journal(t.path()).unwrap().unwrap().state, State::RolledBack);
    }

    #[cfg(unix)]
    #[test]
    fn a_link_or_a_folder_in_the_unit_is_refused_before_anything_moves() {
        let t = install();
        std::os::unix::fs::symlink("/etc/passwd", staging_dir(t.path()).join("LINK")).unwrap();
        assert_eq!(
            apply(t.path(), &["LINK".to_owned()], "1.0.0", "2.0.0"),
            Err(Error::NotPlain("LINK".to_owned()))
        );
        std::fs::remove_file(t.path().join("raul")).unwrap();
        std::os::unix::fs::symlink("/etc/passwd", t.path().join("raul")).unwrap();
        assert_eq!(
            apply(t.path(), &["raul".to_owned()], "1.0.0", "2.0.0"),
            Err(Error::NotPlain("raul".to_owned()))
        );
        std::fs::create_dir_all(staging_dir(t.path()).join("DIR")).unwrap();
        assert_eq!(
            apply(t.path(), &["DIR".to_owned()], "1.0.0", "2.0.0"),
            Err(Error::NotPlain("DIR".to_owned()))
        );
        assert_eq!(journal(t.path()).unwrap(), None);
    }

    #[test]
    fn the_work_folder_cannot_be_part_of_a_unit_in_any_letter_case() {
        let t = install();
        for bad in [".raul-update/journal.json", ".RAUL-Update/x"] {
            assert_eq!(
                apply(t.path(), &[bad.to_owned()], "1.0.0", "2.0.0"),
                Err(Error::BadPath(bad.to_owned()))
            );
        }
    }

    #[test]
    fn a_held_lock_refuses_a_second_swap_and_a_dead_holders_is_taken_over() {
        let t = install();
        let held = Lock::take(t.path()).unwrap();
        assert_eq!(
            apply(t.path(), &names(), "1.0.0", "2.0.0"),
            Err(Error::Locked)
        );
        assert_eq!(rollback(t.path()), Err(Error::Locked));
        drop(held);
        apply(t.path(), &names(), "1.0.0", "2.0.0").unwrap();
        assert!(
            !t.path().join(WORK).join("lock").exists(),
            "the lock is released"
        );
        if Path::new("/proc/self").exists() {
            // A process id that cannot exist (above the kernel's limit) is a dead holder.
            write_file(&t.path().join(WORK), "lock", b"4294967295");
            rollback(t.path()).expect("the stale lock is taken over");
        }
    }

    #[test]
    fn an_interrupted_second_update_keeps_the_rollback_target_of_the_first() {
        let t = install();
        apply(t.path(), &names(), "1.0.0", "2.0.0").unwrap();
        let new = staging_dir(t.path());
        write_file(&new, "raul", b"raul 3");
        write_file(&new, "components/nv-rs/bin/nv-viewer", b"viewer 3");
        write_file(&new, "NEW-IN-2", b"added 3");
        let _ = apply_until(t.path(), &names(), "2.0.0", "3.0.0", 2);
        recover(t.path()).unwrap();
        assert_eq!(read(&t, "raul").as_deref(), Some("raul 2"));
        // The first update's previous version is still there to go back to.
        let kept = t.path().join(WORK).join("previous");
        assert_eq!(
            std::fs::read_to_string(kept.join("raul")).unwrap(),
            "raul 1"
        );
    }
}
