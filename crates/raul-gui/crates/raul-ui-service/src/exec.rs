// SPDX-License-Identifier: GPL-3.0-only
//! Running the `raul` command line.
//!
//! [`Exec`] is the seam the jobs run a command line through. The window's [`InProcessExec`](crate::inproc::InProcessExec)
//! calls the command line's own entry point in this process; tests answer from a script. No implementation here starts
//! a process (ADR-0016).

use raul_runner::StopHandle;

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

/// Something that runs `raul` with arguments. The real one calls the command line in this process; tests answer
/// from a script.
pub trait Exec {
    /// Runs `raul <args>`.
    ///
    /// # Errors
    /// The reason the command line could not be started at all.
    fn run(&self, args: &[String]) -> Result<Output, String>;

    /// Runs `raul <args>` and calls `on_line` with each line of standard output (without its newline) as it
    /// is printed. The returned [`Output`] holds the whole of it as well. The default runs the command and
    /// then hands over the lines, which is right for a scripted run; [`InProcessExec`](crate::inproc::InProcessExec)
    /// delivers each line while the command is still running.
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
