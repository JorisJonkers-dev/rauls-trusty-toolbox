// SPDX-License-Identifier: GPL-3.0-only
//! The Instance lock for jobs the Toolbox runs in process (ADR-0016).
//!
//! One writer per Instance is a lock file in the Instance, not a mutex in memory: a job that dies leaves nothing
//! poisoned, and the game launcher takes the same lock. The file is nvmod's own, `.nvmod/lock`, holding the pid,
//! start time and command of the holder as one line of JSON; a lock whose pid is gone is taken over. This module
//! takes that file through `nvmod_run::InstanceLock`, the library type `nvmod`'s command line and `nvmod run` use,
//! so the Toolbox, the command line and the launcher agree on the name, the format and the stale-lock rule by
//! construction. What `nvmod-cli` adds around it (which commands write, one thread never locking twice) is
//! command-line business; a job the Toolbox calls directly says itself that it writes.
//!
//! Like the command line, an Instance that is not one yet (no `nvmod-instance.json`) is not locked: the job
//! refuses it with its own words, and no `.nvmod` folder is made for it.

use nvmod_instance::NvmodInstance;
use nvmod_run::{Holder, InstanceLock, LockError};
use std::fmt;
use std::path::Path;

/// Why the Instance could not be locked. It never holds a command line.
#[derive(Debug, PartialEq, Eq)]
pub enum LockRefused {
    /// A live process holds the lock.
    InUse(Holder),
    /// The lock file could not be handled.
    Io(String),
}

impl fmt::Display for LockRefused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InUse(holder) if holder.pid == std::process::id() => f.write_str(
                "the Instance is in use by another job of this program; wait for it to end",
            ),
            Self::InUse(holder) => write!(
                f,
                "the Instance is in use by another program (process {}); wait for it to end",
                holder.pid
            ),
            Self::Io(why) => write!(f, "cannot take the Instance lock: {why}"),
        }
    }
}

impl std::error::Error for LockRefused {}

/// The Instance lock while a job runs. Dropping it releases the lock.
#[derive(Debug)]
pub struct InstanceGuard {
    lock: Option<InstanceLock>,
}

impl InstanceGuard {
    /// Takes the lock of the Instance at `root` for the job called `job` (a short name the lock file records,
    /// such as `rollback`).
    ///
    /// # Errors
    /// [`LockRefused::InUse`] when a live process holds the lock (a job, `nvmod`, or the running game);
    /// [`LockRefused::Io`] when the lock file cannot be written or something that is not a lock stands in the way.
    pub fn acquire(root: &Path, job: &str) -> Result<Self, LockRefused> {
        if !NvmodInstance::is_instance(root) {
            return Ok(Self { lock: None });
        }
        match InstanceLock::acquire(root, job) {
            Ok(lock) => Ok(Self { lock: Some(lock) }),
            Err(LockError::Held(holder)) => Err(LockRefused::InUse(holder)),
            Err(LockError::Io(why)) => Err(LockRefused::Io(why)),
        }
    }

    #[must_use]
    /// Whether a lock file is held (false for a folder that is not an Instance).
    pub fn is_held(&self) -> bool {
        self.lock.is_some()
    }

    #[must_use]
    /// The dead holder whose lock this one replaced, if any.
    pub fn took_over(&self) -> Option<&Holder> {
        self.lock.as_ref().and_then(|l| l.took_over.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nvmod_instance::INSTANCE_FILE;
    use nvmod_testkit::TempDir;

    /// A folder that is an Instance as far as the lock is concerned.
    fn instance() -> TempDir {
        let dir = TempDir::new("raul-lock");
        std::fs::write(dir.path().join(INSTANCE_FILE), "{}\n").unwrap();
        dir
    }

    fn lock_text(dir: &TempDir) -> String {
        std::fs::read_to_string(dir.path().join(".nvmod/lock")).unwrap()
    }

    #[test]
    fn two_holders_exclude_each_other_until_the_first_lets_go() {
        let dir = instance();
        let first = InstanceGuard::acquire(dir.path(), "rollback").unwrap();
        assert!(first.is_held());
        let Err(LockRefused::InUse(holder)) = InstanceGuard::acquire(dir.path(), "sort") else {
            panic!("a second holder must be refused");
        };
        assert_eq!(holder.pid, std::process::id());
        assert_eq!(holder.command, "rollback");
        drop(first);
        assert!(!dir.path().join(".nvmod").exists(), "the lock is released");
        assert!(InstanceGuard::acquire(dir.path(), "sort").is_ok());
    }

    #[test]
    fn the_lock_is_nvmods_own_file_so_the_command_line_and_the_launcher_see_it() {
        let dir = instance();
        let _held = InstanceGuard::acquire(dir.path(), "rollback").unwrap();
        // The file and its format are nvmod's: a line of JSON naming the pid, the start and the command.
        let value: serde_json::Value = serde_json::from_str(&lock_text(&dir)).unwrap();
        assert_eq!(value["pid"], std::process::id());
        assert_eq!(value["command"], "rollback");
        assert!(value["started"].is_u64());
        // nvmod's own type refuses while the Toolbox holds it.
        assert!(matches!(
            InstanceLock::acquire(dir.path(), "run x"),
            Err(LockError::Held(h)) if h.command == "rollback"
        ));
        // And the other way round: a lock nvmod holds keeps the Toolbox out.
        let other = instance();
        let _nvmod = InstanceLock::acquire(other.path(), "stage").unwrap();
        assert!(matches!(
            InstanceGuard::acquire(other.path(), "rollback"),
            Err(LockRefused::InUse(h)) if h.command == "stage"
        ));
    }

    // A pid can be told dead on Linux and the other Unix systems; elsewhere a lock is never taken over by guesswork.
    #[test]
    #[cfg(unix)]
    fn a_lock_left_by_a_process_that_crashed_is_taken_over() {
        let dir = instance();
        std::fs::create_dir_all(dir.path().join(".nvmod")).unwrap();
        // What a crashed holder leaves behind: its file, naming a pid that no longer runs (none is this high).
        std::fs::write(
            dir.path().join(".nvmod/lock"),
            "{\"pid\":4294967290,\"started\":1,\"command\":\"rollback\"}\n",
        )
        .unwrap();
        let guard = InstanceGuard::acquire(dir.path(), "sort").unwrap();
        assert_eq!(
            guard.took_over().map(|h| h.command.as_str()),
            Some("rollback")
        );
        assert!(lock_text(&dir).contains(&std::process::id().to_string()));
    }

    #[test]
    fn a_file_that_is_not_a_lock_is_refused_and_left_alone() {
        let dir = instance();
        std::fs::create_dir_all(dir.path().join(".nvmod")).unwrap();
        std::fs::write(dir.path().join(".nvmod/lock"), "junk").unwrap();
        assert!(matches!(
            InstanceGuard::acquire(dir.path(), "sort"),
            Err(LockRefused::Io(_))
        ));
        assert_eq!(lock_text(&dir), "junk");
    }

    #[test]
    fn a_folder_that_is_not_an_instance_is_not_locked() {
        let dir = TempDir::new("raul-lock-none");
        let guard = InstanceGuard::acquire(dir.path(), "rollback").unwrap();
        assert!(!guard.is_held());
        assert!(!dir.path().join(".nvmod").exists());
    }

    #[test]
    fn the_refusal_names_no_command_line() {
        let holder = Holder {
            pid: 7,
            started: 1,
            command: "run Some Tool".into(),
        };
        let said = LockRefused::InUse(holder).to_string();
        assert!(
            said.contains("process 7") && !said.contains("run Some Tool"),
            "{said}"
        );
    }
}
