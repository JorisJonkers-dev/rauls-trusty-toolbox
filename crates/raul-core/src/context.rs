// SPDX-License-Identifier: GPL-3.0-only
//! What a run of `raul` reads from its surroundings besides its command line, as a value passed down from the entry
//! point.
//!
//! The command line reads its configuration folder from `RAUL_CONFIG_DIR` or the platform's own, and writes a run log.
//! Code that runs several commands in one process (the window, a test) cannot change the environment of the process
//! for one of them (`unsafe_code` is forbidden, and a variable is shared by every thread), so it gives each run a
//! [`Context`]. The default context is the person's own folders, so the command line behaves as it always has.

use std::path::PathBuf;

/// The folders and switches one run of `raul` uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    config_folder: Option<PathBuf>,
    run_log: bool,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            config_folder: None,
            run_log: true,
        }
    }
}

impl Context {
    /// The person's own folders and a run log for every nvmod command: what the command line uses.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reads and writes the configuration (`config.toml`, `settings.toml`) in `folder` instead of the person's own.
    #[must_use]
    pub fn with_config_folder(mut self, folder: impl Into<PathBuf>) -> Self {
        self.config_folder = Some(folder.into());
        self
    }

    /// Writes no run log for the nvmod commands of this run, so nothing reaches the state folder of the person. A
    /// state folder of its own needs a path in nvmod's run log; until then a run that must not touch the real one
    /// writes none.
    #[must_use]
    pub fn without_run_log(mut self) -> Self {
        self.run_log = false;
        self
    }

    /// The configuration folder: the one given, else [`crate::config::folder`].
    #[must_use]
    pub fn config_folder(&self) -> Option<PathBuf> {
        self.config_folder.clone().or_else(crate::config::folder)
    }

    /// `config.toml` in the configuration folder.
    #[must_use]
    pub fn config_file(&self) -> Option<PathBuf> {
        self.config_folder()
            .map(|f| f.join(crate::config::FILE_NAME))
    }

    /// `settings.toml` in the configuration folder.
    #[must_use]
    pub fn settings_file(&self) -> Option<PathBuf> {
        self.config_folder().map(|f| f.join("settings.toml"))
    }

    /// Whether the nvmod commands of this run write a run log.
    #[must_use]
    pub fn run_log(&self) -> bool {
        self.run_log
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_folder_given_decides_the_files_and_the_default_leaves_the_environment_to_decide() {
        let ctx = Context::new().with_config_folder("/the/config");
        assert_eq!(
            ctx.config_file(),
            Some(PathBuf::from("/the/config/config.toml"))
        );
        assert_eq!(
            ctx.settings_file(),
            Some(PathBuf::from("/the/config/settings.toml"))
        );
        assert_eq!(Context::new().config_file(), crate::config::default_file());
        assert!(Context::new().run_log());
        assert!(!Context::new().without_run_log().run_log());
    }
}
