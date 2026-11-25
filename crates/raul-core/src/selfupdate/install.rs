// SPDX-License-Identifier: GPL-3.0-only
//! The two moments an update is judged (ADR-0020): right after the swap, by the old program asking the new one to check
//! itself, and at the first starts of the new program, which confirms itself or rolls back by itself.

use std::path::Path;

use super::swap::{self, State};

/// How many starts of an unconfirmed new version may end without a confirmation. The first start gets to confirm; a
/// start that finds that an earlier one never did (it crashed or was killed first) rolls back instead of trying again.
pub const UNCONFIRMED_STARTS: u32 = 1;

/// Why an install did not stay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The swap itself did not happen; the install is as it was.
    Swap(swap::Error),
    /// The new version failed its check and the old one is back; the reason in plain words.
    RolledBack(String),
    /// The new version failed its check and going back failed too; both reasons.
    Stuck(String, String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Swap(e) => write!(f, "The update was not installed: {e}"),
            Self::RolledBack(why) => write!(
                f,
                "The update did not work, so raul went back to the old version. {why}"
            ),
            Self::Stuck(why, back) => write!(
                f,
                "The update did not work and going back failed too. {why} {back}"
            ),
        }
    }
}

impl std::error::Error for Error {}

/// Swap the staged unit in, then ask `check` (which starts the new program; [`super::selfcheck::run_child`]) whether it
/// works. A failed check rolls back at once, so the running program never has to go on with a broken one. The journal
/// ends `swapped` on success; the new program confirms itself at its first start ([`settle_at_start`]).
///
/// # Errors
/// [`Error::Swap`] when nothing was swapped, [`Error::RolledBack`] when the new version failed its check, [`Error::Stuck`]
/// when it failed and could not be taken out again.
pub fn install(
    root: &Path,
    files: &[String],
    from: &str,
    to: &str,
    check: &dyn Fn() -> Result<(), String>,
) -> Result<(), Error> {
    swap::apply(root, files, from, to).map_err(Error::Swap)?;
    match check() {
        Ok(()) => Ok(()),
        Err(why) => match swap::rollback(root) {
            Ok(_) => Err(Error::RolledBack(why)),
            Err(back) => Err(Error::Stuck(why, back.to_string())),
        },
    }
}

/// What the start of a program found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Startup {
    /// Nothing to settle.
    Nothing,
    /// An interrupted swap was undone; the install is the old version.
    Recovered,
    /// This version is new, passed its check and is confirmed.
    Confirmed,
    /// This version is new and failed its check, or did not get to confirm in time; the old version is back and the
    /// program must be started again. The reason is in plain words.
    RolledBack(String),
}

fn starts_file(root: &Path) -> std::path::PathBuf {
    root.join(swap::WORK).join("unconfirmed-starts")
}

/// Call first thing at start, before anything reads the install. `check` is the program's check of itself
/// ([`super::selfcheck::run_in_process`] made into a pass or a reason).
///
/// An interrupted swap is undone. A finished swap that no start has confirmed is confirmed when `check` passes; when it
/// fails, or when an earlier start of this version never got as far as confirming (a crash on start is counted), the
/// old version is put back.
///
/// # Errors
/// The swap's own error when a file cannot be moved or the journal cannot be written.
pub fn settle_at_start(
    root: &Path,
    check: &dyn Fn() -> Result<(), String>,
) -> Result<Startup, swap::Error> {
    let Some(journal) = swap::journal(root)? else {
        return Ok(Startup::Nothing);
    };
    match journal.state {
        State::Swapping => {
            swap::recover(root)?;
            Ok(Startup::Recovered)
        }
        State::Swapped => {
            let file = starts_file(root);
            let before: u32 = std::fs::read_to_string(&file)
                .ok()
                .and_then(|t| t.trim().parse().ok())
                .unwrap_or(0);
            if before >= UNCONFIRMED_STARTS {
                swap::rollback(root)?;
                let _ = std::fs::remove_file(&file);
                return Ok(Startup::RolledBack(
                    "The new version did not finish starting, so raul went back to the old one."
                        .to_owned(),
                ));
            }
            let _ = std::fs::write(&file, (before + 1).to_string());
            match check() {
                Ok(()) => {
                    swap::confirm(root)?;
                    let _ = std::fs::remove_file(&file);
                    Ok(Startup::Confirmed)
                }
                Err(why) => {
                    swap::rollback(root)?;
                    let _ = std::fs::remove_file(&file);
                    Ok(Startup::RolledBack(why))
                }
            }
        }
        State::Confirmed | State::RolledBack => Ok(Startup::Nothing),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nvmod_testkit::{TempDir, write_file};
    use raul_testkit::{covers, ids};

    fn install_dir() -> (TempDir, Vec<String>) {
        let t = TempDir::new("install-flow");
        write_file(t.path(), "raul", b"raul 1");
        let new = swap::staging_dir(t.path());
        write_file(&new, "raul", b"raul 2");
        (t, vec!["raul".to_owned()])
    }

    fn read(t: &TempDir) -> String {
        std::fs::read_to_string(t.path().join("raul")).unwrap()
    }

    fn state(t: &TempDir) -> State {
        swap::journal(t.path()).unwrap().unwrap().state
    }

    #[test]
    fn a_passed_check_leaves_the_new_version_swapped_and_unconfirmed() {
        let (t, files) = install_dir();
        install(t.path(), &files, "1.0.0", "2.0.0", &|| Ok(())).unwrap();
        assert_eq!((read(&t).as_str(), state(&t)), ("raul 2", State::Swapped));
    }

    #[test]
    fn a_failed_check_or_a_child_that_never_started_rolls_back_at_once() {
        let (t, files) = install_dir();
        let e = install(t.path(), &files, "1.0.0", "2.0.0", &|| {
            Err("The new version did not start.".to_owned())
        })
        .unwrap_err();
        assert_eq!(
            e,
            Error::RolledBack("The new version did not start.".to_owned())
        );
        assert!(e.to_string().contains("went back to the old version"));
        assert_eq!(
            (read(&t).as_str(), state(&t)),
            ("raul 1", State::RolledBack)
        );
    }

    #[test]
    fn a_swap_that_cannot_start_changes_nothing_and_never_asks_the_check() {
        let (t, _) = install_dir();
        let called = std::cell::Cell::new(false);
        let e = install(t.path(), &["nope".to_owned()], "1.0.0", "2.0.0", &|| {
            called.set(true);
            Ok(())
        })
        .unwrap_err();
        assert!(matches!(e, Error::Swap(_)));
        assert!(!called.get());
        assert_eq!(read(&t), "raul 1");
    }

    #[test]
    fn the_first_start_confirms_a_working_new_version() {
        covers(&[ids::update::install::FIRST_START_CONFIRMS]);
        let (t, files) = install_dir();
        install(t.path(), &files, "1.0.0", "2.0.0", &|| Ok(())).unwrap();
        assert_eq!(
            settle_at_start(t.path(), &|| Ok(())).unwrap(),
            Startup::Confirmed
        );
        assert_eq!(state(&t), State::Confirmed);
        assert_eq!(
            settle_at_start(t.path(), &|| Ok(())).unwrap(),
            Startup::Nothing
        );
    }

    #[test]
    fn the_first_start_rolls_back_a_new_version_that_fails_its_check() {
        let (t, files) = install_dir();
        install(t.path(), &files, "1.0.0", "2.0.0", &|| Ok(())).unwrap();
        let r = settle_at_start(t.path(), &|| Err("A file is changed.".to_owned())).unwrap();
        assert_eq!(r, Startup::RolledBack("A file is changed.".to_owned()));
        assert_eq!(
            (read(&t).as_str(), state(&t)),
            ("raul 1", State::RolledBack)
        );
    }

    #[test]
    fn starts_that_never_confirm_are_counted_and_the_next_one_rolls_back() {
        let (t, files) = install_dir();
        install(t.path(), &files, "1.0.0", "2.0.0", &|| Ok(())).unwrap();
        // A start that crashed before it confirmed leaves its count behind; the next start rolls back.
        std::fs::write(starts_file(t.path()), UNCONFIRMED_STARTS.to_string()).unwrap();
        let r = settle_at_start(t.path(), &|| panic!("the check is not even asked")).unwrap();
        assert!(matches!(r, Startup::RolledBack(_)), "{r:?}");
        assert_eq!(read(&t), "raul 1");
    }

    #[test]
    fn a_first_start_that_confirms_leaves_no_count_and_the_second_start_is_not_rolled_back() {
        let (t, files) = install_dir();
        install(t.path(), &files, "1.0.0", "2.0.0", &|| Ok(())).unwrap();
        assert_eq!(
            settle_at_start(t.path(), &|| Ok(())).unwrap(),
            Startup::Confirmed
        );
        assert!(!starts_file(t.path()).exists());
        assert_eq!(
            settle_at_start(t.path(), &|| Ok(())).unwrap(),
            Startup::Nothing
        );
        assert_eq!(read(&t), "raul 2");
    }

    #[test]
    fn an_interrupted_swap_is_undone_at_start() {
        let (t, files) = install_dir();
        swap::apply(t.path(), &files, "1.0.0", "2.0.0").unwrap();
        let mut j = swap::journal(t.path()).unwrap().unwrap();
        j.state = State::Swapping;
        std::fs::write(
            t.path().join(swap::WORK).join("journal.json"),
            serde_json::to_string(&j).unwrap(),
        )
        .unwrap();
        assert_eq!(
            settle_at_start(t.path(), &|| Ok(())).unwrap(),
            Startup::Recovered
        );
        assert_eq!(read(&t), "raul 1");
    }

    #[test]
    fn an_install_with_no_update_has_nothing_to_settle() {
        let t = TempDir::new("install-none");
        assert_eq!(
            settle_at_start(t.path(), &|| Ok(())).unwrap(),
            Startup::Nothing
        );
    }
}
