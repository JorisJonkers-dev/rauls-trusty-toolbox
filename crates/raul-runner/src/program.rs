// SPDX-License-Identifier: GPL-3.0-only
//! Starting a program the user will use, such as the game or a modding tool (`nvmod run`). Unlike [`crate::Runner`]
//! there is no timeout, output cap or scrubbed environment: the program is long lived, needs the user's display,
//! audio and Steam variables, and may outlive nvmod. It is still an argument vector, never a shell line.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Child, Command, Stdio};

/// A started program.
#[derive(Debug)]
pub struct Started(Child);

impl Started {
    /// Its process id.
    #[must_use]
    pub fn pid(&self) -> u32 {
        self.0.id()
    }

    /// Wait for it to end and return its exit code (128 plus the signal number when a signal ended it).
    ///
    /// # Errors
    /// Waiting failed.
    pub fn wait(mut self) -> Result<i32, String> {
        let status = self.0.wait().map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            if let Some(signal) = status.signal() {
                return Ok(128 + signal);
            }
        }
        Ok(status.code().unwrap_or(1))
    }
}

/// Start `program` with `args` in `cwd`, with nvmod's environment plus `env`. With `attached` the program shares
/// the caller's terminal; otherwise its input is closed, its output dropped, and on Unix it runs in its own
/// process group so the caller's Ctrl+C does not reach it. With `log` the program's output and errors are
/// appended to that file instead (in both modes).
///
/// # Errors
/// The program could not be started.
pub fn start_program(
    program: &str,
    args: &[String],
    env: &BTreeMap<String, String>,
    cwd: &Path,
    attached: bool,
    log: Option<&Path>,
) -> Result<Started, String> {
    // The one place the Toolbox starts a long-lived program (ADR-0007).
    #[allow(clippy::disallowed_methods)]
    let mut command = Command::new(program);
    command.args(args).envs(env).current_dir(cwd);
    if let Some(log) = log {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log)
            .map_err(|e| format!("cannot open the log {}: {e}", log.display()))?;
        let again = file
            .try_clone()
            .map_err(|e| format!("cannot open the log {}: {e}", log.display()))?;
        command.stdout(file).stderr(again);
    } else if !attached {
        command.stdout(Stdio::null()).stderr(Stdio::null());
    }
    if !attached {
        command.stdin(Stdio::null());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
    }
    crate::runner::count_started();
    command
        .spawn()
        .map(Started)
        .map_err(|e| format!("could not start {program}: {e}"))
}
