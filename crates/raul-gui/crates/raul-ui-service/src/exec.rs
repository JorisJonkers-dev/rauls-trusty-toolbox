// SPDX-License-Identifier: GPL-3.0-only
//! Running the `nvmod` command line.
//!
//! The one exception to ADR-0005 outside `nvmod-runner`: this module starts `nvmod` itself with a process group, a
//! streamed line-by-line output and a cooperative stop with a grace period, none of which the runner offers yet.
//! `crates/nvmod-runner/tests/no_command_elsewhere.rs` names this file; moving it onto the runner removes the entry.
#![allow(clippy::disallowed_methods)]

use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// A request to stop a running command, shared between whoever asks and the runner that obeys. Cloning gives
/// another handle to the same request.
#[derive(Debug, Clone, Default)]
pub struct CancelHandle(Arc<AtomicBool>);

impl CancelHandle {
    /// A handle nobody has used.
    pub fn new() -> Self {
        Self::default()
    }

    /// Asks the command to stop. Asking twice is the same as asking once.
    pub fn request(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    /// Whether a stop was asked for.
    pub fn is_requested(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

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

    /// Runs `nvmod <args>` and calls `on_line` with each line of standard output (without its newline) as it
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

/// Starts the `nvmod` program as a subprocess.
#[derive(Debug, Clone)]
pub struct ProcessExec {
    program: PathBuf,
    grace: Duration,
}

impl ProcessExec {
    /// Runs the program at `program` (a path, or a bare name found on `PATH`).
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            grace: Duration::from_secs(5),
        }
    }

    /// How long a command that was asked to stop may take before it is forced to (five seconds by default).
    #[must_use]
    pub fn with_grace(mut self, grace: Duration) -> Self {
        self.grace = grace;
        self
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
        let mut command = Command::new(&self.program);
        command
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // Its own process group, so that stopping the command also stops the tools it started.
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut command, 0);
        let mut child = command.spawn().map_err(|e| self.start_error(&e))?;
        let pid = child.id();
        // Standard error is read on its own thread so a full pipe cannot stall the process.
        let stderr = child.stderr.take();
        let reader = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            if let Some(mut pipe) = stderr {
                let _ = pipe.read_to_end(&mut bytes);
            }
            String::from_utf8_lossy(&bytes).into_owned()
        });
        let stdout_pipe = child.stdout.take();
        let child = Arc::new(Mutex::new(child));
        let over = Arc::new(AtomicBool::new(false));
        let watcher = {
            let (child, over, cancel, grace) =
                (child.clone(), over.clone(), cancel.clone(), self.grace);
            std::thread::spawn(move || stop_when_asked(pid, &child, &over, &cancel, grace))
        };
        let mut stdout = String::new();
        if let Some(pipe) = stdout_pipe {
            for chunk in BufReader::new(pipe).split(b'\n') {
                let Ok(bytes) = chunk else { break };
                let line = String::from_utf8_lossy(&bytes).into_owned();
                on_line(&line);
                stdout.push_str(&line);
                stdout.push('\n');
            }
        }
        let status = loop {
            let polled = child
                .lock()
                .map_err(|_| String::from("the process handle was poisoned"))?
                .try_wait();
            match polled {
                Ok(Some(status)) => break Ok(status),
                Ok(None) => std::thread::sleep(Duration::from_millis(5)),
                Err(e) => break Err(self.start_error(&e)),
            }
        };
        over.store(true, Ordering::SeqCst);
        let _ = watcher.join();
        let status = status?;
        Ok(Output {
            code: status.code().map(i64::from),
            stdout,
            stderr: reader.join().unwrap_or_default(),
        })
    }
}

/// Waits for the run to end or for a stop request. On a request it asks the process group to end, and forces it
/// to once `grace` has passed.
fn stop_when_asked(
    pid: u32,
    child: &Mutex<Child>,
    over: &AtomicBool,
    cancel: &CancelHandle,
    grace: Duration,
) {
    while !over.load(Ordering::SeqCst) {
        if cancel.is_requested() {
            signal_group(pid, false);
            let asked = Instant::now();
            while !over.load(Ordering::SeqCst) {
                if asked.elapsed() >= grace {
                    signal_group(pid, true);
                    if let Ok(mut c) = child.lock() {
                        let _ = c.kill();
                    }
                    return;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Sends the end request (`SIGTERM`) or the force (`SIGKILL`) to the process group of `pid`. The standard library
/// has no signal call and this crate forbids `unsafe`, so the `kill` program does it.
#[cfg(unix)]
fn signal_group(pid: u32, force: bool) {
    let signal = if force { "-KILL" } else { "-TERM" };
    let _ = Command::new("kill")
        .arg(signal)
        .arg("--")
        .arg(format!("-{pid}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(not(unix))]
fn signal_group(pid: u32, force: bool) {
    let mut command = Command::new("taskkill");
    command.args(["/PID", &pid.to_string(), "/T"]);
    if force {
        command.arg("/F");
    }
    let _ = command.stdout(Stdio::null()).stderr(Stdio::null()).status();
}

impl ProcessExec {
    fn start_error(&self, e: &std::io::Error) -> String {
        format!("could not start {}: {e}", self.program.display())
    }
}
