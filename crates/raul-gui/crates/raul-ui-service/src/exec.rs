// SPDX-License-Identifier: GPL-3.0-only
//! Running the `nvmod` command line.

use std::path::PathBuf;
use std::process::Command;

/// What one run of the command line printed and how it ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    /// The exit code, `None` when the process did not exit normally (killed by a signal).
    pub code: Option<i64>,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
}

/// Something that runs `nvmod` with arguments. The real one starts a process; tests run the command line in
/// process or answer from a script.
pub trait Exec {
    /// Runs `nvmod <args>`.
    ///
    /// # Errors
    /// The reason the command line could not be started at all.
    fn run(&self, args: &[String]) -> Result<Output, String>;
}

/// Starts the `nvmod` program as a subprocess.
#[derive(Debug, Clone)]
pub struct ProcessExec {
    program: PathBuf,
}

impl ProcessExec {
    /// Runs the program at `program` (a path, or a bare name found on `PATH`).
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
        }
    }
}

impl Default for ProcessExec {
    fn default() -> Self {
        Self::new("nvmod")
    }
}

impl Exec for ProcessExec {
    fn run(&self, args: &[String]) -> Result<Output, String> {
        let out = Command::new(&self.program)
            .args(args)
            .output()
            .map_err(|e| format!("could not start {}: {e}", self.program.display()))?;
        Ok(Output {
            code: out.status.code().map(i64::from),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        })
    }
}
