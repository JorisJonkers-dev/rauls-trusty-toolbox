// SPDX-License-Identifier: GPL-3.0-only
//! Running the `raul` command line.
//!
//! [`ProcessExec`] starts `raul` (by default the GUI's own executable) through the streaming mode of
//! `raul_runner::Runner`: its own process group, each line of standard output as it is printed, and a stop that
//! asks first and forces after a grace period. Nothing here starts a process itself (ADR-0007).

use raul_runner::{Call, Runner, StopHandle, ToolError};
use std::path::PathBuf;
use std::time::Duration;

/// A request to stop a running command, shared between whoever asks and the runner that obeys. Cloning gives
/// another handle to the same request.
pub type CancelHandle = StopHandle;

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

/// Something that runs `raul` with arguments. The real one starts a process; tests run the command line in
/// process or answer from a script.
pub trait Exec {
    /// Runs `raul <args>`.
    ///
    /// # Errors
    /// The reason the command line could not be started at all.
    fn run(&self, args: &[String]) -> Result<Output, String>;

    /// Runs `raul <args>` and calls `on_line` with each line of standard output (without its newline) as it
    /// is printed. The returned [`Output`] holds the whole of it as well. The default runs the command and
    /// then hands over the lines, which is right for an in-process or scripted run; [`ProcessExec`] delivers
    /// each line while the process is still running.
    ///
    /// # Errors
    /// The reason the command line could not be started at all.
    fn run_streaming(
        &self,
        args: &[String],
        on_line: &mut dyn FnMut(&str),
    ) -> Result<Output, String> {
        let out = self.run(args)?;
        for line in out.stdout.lines() {
            on_line(line);
        }
        Ok(out)
    }

    /// [`Exec::run_streaming`] that stops the command when `cancel` is requested: it is asked to end (and
    /// forced to after a grace period), and the [`Output`] says how it ended (`code` is `None` when a signal
    /// ended it). The default cannot stop a command in the middle and runs it to its end.
    ///
    /// # Errors
    /// The reason the command line could not be started at all.
    fn run_cancellable(
        &self,
        args: &[String],
        on_line: &mut dyn FnMut(&str),
        cancel: &CancelHandle,
    ) -> Result<Output, String> {
        let _ = cancel;
        self.run_streaming(args, on_line)
    }
}

/// How long one command may run before it is stopped: long enough for a full setup with TTW and LOD.
pub const COMMAND_TIMEOUT: Duration = Duration::from_hours(12);

/// Starts the `raul` program as a subprocess, through the streaming Runner.
#[derive(Debug, Clone)]
pub struct ProcessExec {
    program: PathBuf,
    grace: Duration,
    timeout: Duration,
}

impl ProcessExec {
    /// Runs the program at `program` (a path, or a bare name found on `PATH`).
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            grace: raul_runner::STOP_GRACE,
            timeout: COMMAND_TIMEOUT,
        }
    }

    /// Runs the executable this process was started from: the release `raul` is the GUI and the command line in
    /// one binary (ADR-0004). Falls back to `raul` on `PATH` when the executable cannot be found.
    pub fn current() -> Self {
        Self::new(std::env::current_exe().unwrap_or_else(|_| PathBuf::from("raul")))
    }

    /// How long a command that was asked to stop may take before it is forced to (five seconds by default).
    #[must_use]
    pub fn with_grace(mut self, grace: Duration) -> Self {
        self.grace = grace;
        self
    }

    /// How long a command may run before it is stopped ([`COMMAND_TIMEOUT`] by default).
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// The program it runs.
    pub fn program(&self) -> &std::path::Path {
        &self.program
    }

    /// The Runner: no output cap beyond the default, the timeout above, and the person's own environment, since
    /// `raul` is the Toolbox itself and needs the same home, display and Steam variables as their shell.
    fn runner(&self) -> Runner {
        Runner {
            timeout: self.timeout,
            ..Runner::default()
        }
        .inherit_all()
    }
}

impl Default for ProcessExec {
    fn default() -> Self {
        Self::current()
    }
}

impl Exec for ProcessExec {
    fn run(&self, args: &[String]) -> Result<Output, String> {
        self.run_cancellable(args, &mut |_| {}, &CancelHandle::new())
    }

    fn run_streaming(
        &self,
        args: &[String],
        on_line: &mut dyn FnMut(&str),
    ) -> Result<Output, String> {
        self.run_cancellable(args, on_line, &CancelHandle::new())
    }

    fn run_cancellable(
        &self,
        args: &[String],
        on_line: &mut dyn FnMut(&str),
        cancel: &CancelHandle,
    ) -> Result<Output, String> {
        let call = Call::new("raul", &self.program, args);
        match self
            .runner()
            .run_streaming(&call, cancel, self.grace, on_line)
        {
            Ok(done) => Ok(Output {
                code: done.exit_code.map(i64::from),
                stdout: done.stdout,
                stderr: done.stderr,
            }),
            Err(ToolError::Failed { reason, .. }) if reason.starts_with("cannot start") => Err(
                format!("could not start {}: {reason}", self.program.display()),
            ),
            // Stopped for its timeout or output cap: it ended without an exit code, and the reason is what it left.
            Err(e @ ToolError::Failed { .. }) => Ok(Output {
                code: None,
                stdout: String::new(),
                stderr: e.to_string(),
            }),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|w| (*w).to_owned()).collect()
    }

    #[test]
    fn the_default_program_is_the_running_executable() {
        assert_eq!(
            ProcessExec::default().program(),
            std::env::current_exe().unwrap()
        );
    }

    #[test]
    fn a_command_streams_its_lines_and_keeps_the_exit_code() {
        let dir = nvmod_testkit::TempDir::new("exec-stream");
        let p = nvmod_testkit::write_script(
            dir.path(),
            "raul",
            "echo \"$1\"; echo two; echo bad >&2; exit 3",
        );
        let mut seen = Vec::new();
        let out = ProcessExec::new(p)
            .run_streaming(&args(&["one"]), &mut |l| seen.push(l.to_owned()))
            .unwrap();
        assert_eq!(seen, ["one", "two"]);
        assert_eq!(out.code, Some(3));
        assert_eq!(out.stdout, "one\ntwo\n");
        assert!(out.stderr.contains("bad"));
    }

    #[test]
    fn a_program_that_cannot_start_is_an_error() {
        let why = ProcessExec::new("/no/such/raul").run(&[]).unwrap_err();
        assert!(why.starts_with("could not start /no/such/raul"), "{why}");
    }

    #[test]
    fn a_command_past_its_timeout_ends_without_an_exit_code() {
        let dir = nvmod_testkit::TempDir::new("exec-timeout");
        let p = nvmod_testkit::write_script(dir.path(), "raul", "exec sleep 30");
        let out = ProcessExec::new(p)
            .with_timeout(Duration::from_millis(200))
            .run(&[])
            .unwrap();
        assert_eq!(out.code, None);
        assert!(out.stderr.contains("longer"), "{}", out.stderr);
    }

    #[test]
    fn a_cancelled_command_is_stopped_after_the_grace_period() {
        let dir = nvmod_testkit::TempDir::new("exec-cancel");
        let p = nvmod_testkit::write_script(
            dir.path(),
            "raul",
            "trap '' TERM; echo started; while :; do sleep 0.05; done",
        );
        let cancel = CancelHandle::new();
        let asker = cancel.clone();
        let out = ProcessExec::new(p)
            .with_grace(Duration::from_millis(100))
            .run_cancellable(&[], &mut |_| asker.request(), &cancel)
            .unwrap();
        assert_eq!(out.code, None);
    }
}
