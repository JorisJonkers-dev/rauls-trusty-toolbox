// SPDX-License-Identifier: GPL-3.0-only
//! One LOD run per Profile at a time (#343). A run holds an operating-system lock on
//! `<instance>/.raul/lod/<Profile>/run.lock` while it goes; the system releases it when the run ends, also when the
//! process is killed, so a lock is never left behind. A second run of the same Profile is refused while one holds it.
//! Settings stay editable during a run: a run reads them once, when it builds its request, so a change applies to the
//! next run.

use std::fs::{File, OpenOptions, TryLockError};
use std::path::{Path, PathBuf};

use super::WORK_DIR;

/// The lock a LOD run of one Profile holds; dropping it releases it.
#[derive(Debug)]
pub struct RunLock {
    _file: File,
}

impl RunLock {
    /// The lock file of `profile` in the Instance at `root`.
    #[must_use]
    pub fn path(root: &Path, profile: &str) -> PathBuf {
        root.join(WORK_DIR).join(profile).join("run.lock")
    }

    /// Take the lock of `profile`, or `None` when a run already holds it.
    ///
    /// # Errors
    ///
    /// The lock file cannot be made or locked for another reason than another run holding it.
    pub fn try_acquire(root: &Path, profile: &str) -> std::io::Result<Option<RunLock>> {
        let path = RunLock::path(root, profile);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)?;
        match file.try_lock() {
            Ok(()) => Ok(Some(RunLock { _file: file })),
            Err(TryLockError::WouldBlock) => Ok(None),
            Err(TryLockError::Error(e)) => Err(e),
        }
    }

    /// Whether a run of `profile` is going now. Makes no file when none has run.
    #[must_use]
    pub fn in_progress(root: &Path, profile: &str) -> bool {
        RunLock::path(root, profile).is_file()
            && matches!(RunLock::try_acquire(root, profile), Ok(None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nvmod_testkit::TempDir;

    #[test]
    fn a_profile_is_locked_while_its_lock_is_held_and_free_after() {
        let dir = TempDir::new("raul-lod-run-lock");
        assert!(!RunLock::in_progress(dir.path(), "Default"));
        let held = RunLock::try_acquire(dir.path(), "Default")
            .unwrap()
            .unwrap();
        assert!(RunLock::in_progress(dir.path(), "Default"));
        assert!(
            RunLock::try_acquire(dir.path(), "Default")
                .unwrap()
                .is_none()
        );
        assert!(
            !RunLock::in_progress(dir.path(), "Other"),
            "another Profile is free"
        );
        drop(held);
        assert!(!RunLock::in_progress(dir.path(), "Default"));
    }
}
