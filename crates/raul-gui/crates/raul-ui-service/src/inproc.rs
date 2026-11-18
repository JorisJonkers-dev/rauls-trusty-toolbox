// SPDX-License-Identifier: GPL-3.0-only
//! Running the `raul` command line in this process.
//!
//! [`InProcessExec`] calls the command line's own entry point instead of starting a program, so a test, or a build
//! with no `raul` binary beside it, can drive exactly the commands the GUI drives. It is the cheap counterpart of
//! [`ProcessExec`](crate::exec::ProcessExec): no process and no process group, so a stop reaches the command only
//! where the command itself looks at the cancel flag.

use crate::exec::{CancelHandle, Exec, Output};
use std::io::Write;
use std::sync::mpsc;
use std::time::Duration;

/// How often a running command is asked whether it was told to stop.
const TICK: Duration = Duration::from_millis(50);

/// What a run that was stopped leaves on standard error.
const STOPPED: &str = "the command was stopped";

/// Runs `raul` in this process, by calling the command line's own entry point.
#[derive(Debug, Clone, Copy, Default)]
pub struct InProcessExec;

impl InProcessExec {
    /// An exec that runs the command line in this process.
    pub fn new() -> Self {
        Self
    }
}

impl Exec for InProcessExec {
    /// Runs `raul <args>` and keeps what it printed.
    ///
    /// # Errors
    /// The reason the command line failed unexpectedly, which is a panic inside it.
    fn run(&self, args: &[String]) -> Result<Output, String> {
        self.run_streaming(args, &mut |_| {})
    }

    /// Runs `raul <args>` and hands each line of standard output over as it is written.
    ///
    /// # Errors
    /// The reason the command line failed unexpectedly, which is a panic inside it.
    fn run_streaming(
        &self,
        args: &[String],
        on_line: &mut dyn FnMut(&str),
    ) -> Result<Output, String> {
        let mut out = Lines::new(on_line);
        let mut err = Vec::new();
        let code = guarded(|| raul_cli::run(args, &mut out, &mut err))?;
        out.finish();
        Ok(Output {
            code: Some(i64::from(code)),
            stdout: out.into_string(),
            stderr: String::from_utf8_lossy(&err).into_owned(),
        })
    }

    /// Runs `raul <args>` on a worker thread and asks it to stop when `cancel` is requested.
    ///
    /// The command runs here, so there is no process to signal: the request is the cancel flag of the command
    /// line itself, which a long command reads at its next safe point. A command that never reads it runs to its
    /// end, and the stop is reported only once it has.
    ///
    /// # Errors
    /// The reason the command line failed unexpectedly, which is a panic inside it.
    fn run_cancellable(
        &self,
        args: &[String],
        on_line: &mut dyn FnMut(&str),
        cancel: &CancelHandle,
    ) -> Result<Output, String> {
        // A flag left set by an earlier run would stop this one before it started.
        nvmod_core::cancel::reset();
        let (tx, rx) = mpsc::channel();
        let mut lines = Lines::new(on_line);
        let mut stderr = Vec::new();
        let joined = std::thread::scope(|scope| {
            let worker = scope.spawn(|| {
                let mut out = Channel {
                    tx: tx.clone(),
                    err: false,
                };
                let mut err = Channel { tx, err: true };
                guarded(|| raul_cli::run(args, &mut out, &mut err))
            });
            loop {
                if cancel.is_requested() {
                    nvmod_core::cancel::request();
                    break;
                }
                match rx.recv_timeout(TICK) {
                    Ok(Chunk::Out(bytes)) => lines.push(&bytes),
                    Ok(Chunk::Err(bytes)) => stderr.extend_from_slice(&bytes),
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
            worker.join()
        });
        let code = match joined {
            Ok(result) => result?,
            Err(payload) => return Err(failed(&*payload)),
        };
        let stopped = nvmod_core::cancel::requested();
        // The flag belongs to one run: left set, it would stop the next command before it started.
        nvmod_core::cancel::reset();
        if stopped {
            return Ok(Output {
                code: None,
                stdout: lines.into_string(),
                stderr: STOPPED.to_owned(),
            });
        }
        lines.finish();
        Ok(Output {
            code: Some(i64::from(code)),
            stdout: lines.into_string(),
            stderr: String::from_utf8_lossy(&stderr).into_owned(),
        })
    }
}

/// A [`Write`] that hands each complete line to a callback as it is written and keeps everything, so one run can
/// stream its lines and still report the whole of its standard output.
struct Lines<'a> {
    /// Everything written so far.
    whole: Vec<u8>,
    /// The line that is still open.
    partial: Vec<u8>,
    /// Where a finished line goes.
    on_line: &'a mut dyn FnMut(&str),
}

impl<'a> Lines<'a> {
    fn new(on_line: &'a mut dyn FnMut(&str)) -> Self {
        Self {
            whole: Vec::new(),
            partial: Vec::new(),
            on_line,
        }
    }

    /// Takes what was written and hands over every line it completes.
    fn push(&mut self, bytes: &[u8]) {
        self.whole.extend_from_slice(bytes);
        for &byte in bytes {
            if byte == b'\n' {
                self.deliver();
            } else {
                self.partial.push(byte);
            }
        }
    }

    /// Hands over the line that is still open, if any: a command may end without a newline.
    fn finish(&mut self) {
        if !self.partial.is_empty() {
            self.deliver();
        }
    }

    /// Hands over the buffered line, without its line ending.
    fn deliver(&mut self) {
        let mut line = std::mem::take(&mut self.partial);
        if line.last() == Some(&b'\r') {
            line.pop();
        }
        let text = String::from_utf8_lossy(&line).into_owned();
        (self.on_line)(&text);
    }

    /// Everything that was written, as text.
    fn into_string(self) -> String {
        String::from_utf8_lossy(&self.whole).into_owned()
    }
}

impl Write for Lines<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.push(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// One piece of what a command wrote, sent from the thread that runs it to the one that hands the lines over.
enum Chunk {
    /// Standard output.
    Out(Vec<u8>),
    /// Standard error.
    Err(Vec<u8>),
}

/// A [`Write`] that sends what is written to the thread that reads it, so a command can run on a worker while its
/// lines are handed over here.
struct Channel {
    /// Where the pieces go.
    tx: mpsc::Sender<Chunk>,
    /// Whether this one carries standard error.
    err: bool,
}

impl Write for Channel {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let chunk = if self.err {
            Chunk::Err(buf.to_vec())
        } else {
            Chunk::Out(buf.to_vec())
        };
        self.tx
            .send(chunk)
            .map_err(|_| std::io::Error::other("the reader has gone"))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Runs `f`, turning a panic into the reason the caller reports: a command line that panics must not take the
/// caller down with it.
fn guarded<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(value) => Ok(value),
        Err(payload) => Err(failed(&*payload)),
    }
}

/// The reason a panic is reported as, with the message it carried.
fn failed(payload: &(dyn std::any::Any + Send)) -> String {
    let why = if let Some(text) = payload.downcast_ref::<&str>() {
        (*text).to_owned()
    } else if let Some(text) = payload.downcast_ref::<String>() {
        text.clone()
    } else {
        "no message".to_owned()
    };
    format!("the action failed unexpectedly: {why}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|w| (*w).to_owned()).collect()
    }

    #[test]
    fn a_version_run_answers_with_json_and_exit_code_zero() {
        let out = InProcessExec::new()
            .run(&args(&["version", "--json"]))
            .unwrap();
        assert_eq!(out.code, Some(0));
        let value: serde_json::Value = serde_json::from_str(&out.stdout).unwrap();
        assert!(value.get("raul").is_some(), "{}", out.stdout);
        assert!(out.stderr.is_empty(), "{}", out.stderr);
    }

    #[test]
    fn a_command_line_that_cannot_be_understood_is_a_nonzero_code_with_a_reason() {
        let out = InProcessExec::new().run(&args(&["nosuchcommand"])).unwrap();
        assert_ne!(out.code, Some(0));
        assert!(out.stderr.contains("unknown command"), "{}", out.stderr);
    }

    #[test]
    fn streaming_hands_over_the_lines_in_order_and_keeps_them_all() {
        let seen = RefCell::new(Vec::new());
        let mut collect = |line: &str| seen.borrow_mut().push(line.to_owned());
        let out = InProcessExec::new()
            .run_streaming(&args(&["help"]), &mut collect)
            .unwrap();
        assert_eq!(out.code, Some(0));
        let seen = seen.take();
        assert!(seen.len() > 5, "{seen:?}");
        assert_eq!(format!("{}\n", seen.join("\n")), out.stdout);
    }

    #[test]
    fn the_line_adapter_holds_a_partial_line_until_it_is_ended() {
        let seen = RefCell::new(Vec::new());
        let mut collect = |line: &str| seen.borrow_mut().push(line.to_owned());
        let mut lines = Lines::new(&mut collect);
        lines.push(b"one\ntw");
        assert_eq!(*seen.borrow(), ["one"]);
        lines.push(b"o\n\nthree");
        assert_eq!(*seen.borrow(), ["one", "two", ""]);
        lines.finish();
        assert_eq!(*seen.borrow(), ["one", "two", "", "three"]);
        assert_eq!(lines.into_string(), "one\ntwo\n\nthree");
    }

    #[test]
    fn a_stop_asked_for_before_the_run_is_reported_as_stopped() {
        let cancel = CancelHandle::new();
        cancel.request();
        let out = InProcessExec::new()
            .run_cancellable(&args(&["version", "--json"]), &mut |_| {}, &cancel)
            .unwrap();
        assert_eq!(out.code, None);
        assert!(out.stderr.contains("stopped"), "{}", out.stderr);
    }

    #[test]
    fn a_panic_in_the_run_is_reported_and_does_not_reach_the_caller() {
        let why = guarded(|| -> u8 { panic!("boom") }).unwrap_err();
        assert_eq!(why, "the action failed unexpectedly: boom");
    }
}
