// SPDX-License-Identifier: GPL-3.0-only
//! The one subprocess runner every call goes through. It takes an argument vector (never a shell line), a
//! timeout, an output size cap and a scrubbed environment, re-verifies the tool's SHA-256 before each run when the
//! caller pins one, and maps what happened onto the exit code table: a tool that failed is 4, a tool that cannot
//! be verified is 3, a tool whose contract version is newer than nvmod reads is refused (2).

use crate::sha256_hex;
use std::ffi::OsString;
use std::fmt;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

/// Exit code of a command that was refused before it changed anything (the exit code table).
pub const EXIT_REFUSED: u8 = 2;
/// Exit code of a verification failure, such as a tool that cannot be verified.
pub const EXIT_VERIFICATION: u8 = 3;
/// Exit code of a tool that failed.
pub const EXIT_TOOL_FAILED: u8 = 4;

/// How much of the tool's standard error is kept for a report, unless the runner is told otherwise.
const STDERR_KEEP: usize = 64 * 1024;
/// How long to wait for a pipe to finish after the tool has ended (a child it left behind may hold it open).
pub(crate) const PIPE_GRACE: Duration = Duration::from_secs(2);

/// Why a tool call did not give a usable answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolError {
    /// The tool could not be verified: its checksum is not the pinned one (exit code 3).
    Unverifiable {
        /// The tool's name.
        tool: String,
        /// What was found.
        reason: String,
    },
    /// The tool could not be run, ran too long, wrote too much, or exited with a failure (exit code 4).
    Failed {
        /// The tool's name.
        tool: String,
        /// What happened.
        reason: String,
    },
    /// The tool's output is not a valid document of its contract (exit code 4).
    Malformed {
        /// The tool's name.
        tool: String,
        /// The first validation error.
        reason: String,
    },
    /// The tool speaks a contract version this nvmod does not read (exit code 2).
    Refused {
        /// The tool's name.
        tool: String,
        /// Why, with both versions.
        reason: String,
    },
}

impl ToolError {
    /// The exit code of the command that hit this error.
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        match self {
            ToolError::Unverifiable { .. } => EXIT_VERIFICATION,
            ToolError::Failed { .. } | ToolError::Malformed { .. } => EXIT_TOOL_FAILED,
            ToolError::Refused { .. } => EXIT_REFUSED,
        }
    }
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolError::Unverifiable { tool, reason } => {
                write!(f, "{tool} could not be verified: {reason}")
            }
            ToolError::Failed { tool, reason } => write!(f, "{tool} failed: {reason}"),
            ToolError::Malformed { tool, reason } => {
                write!(f, "{tool} gave a malformed response: {reason}")
            }
            ToolError::Refused { tool, reason } => write!(f, "{tool} is refused: {reason}"),
        }
    }
}

impl std::error::Error for ToolError {}

/// One call of a tool.
#[derive(Debug, Clone)]
pub struct Call<'a> {
    /// The tool's name, for reports.
    pub tool: &'a str,
    /// The program to run.
    pub program: &'a Path,
    /// Its arguments, one word each, passed to the program exactly as given (a path need not be UTF-8). There is
    /// no shell.
    pub args: &'a [OsString],
    /// Bytes written to the tool's standard input, then it is closed.
    pub stdin: Option<&'a [u8]>,
    /// The SHA-256 the program must have, checked right before it is started; `None` runs it unchecked.
    pub expected_sha256: Option<&'a str>,
    /// The exit codes that count as an answer (default: 0). A tool such as loot-rs exits 1 with a valid report.
    pub ok_exit_codes: &'a [i32],
    /// The folder to run in; `None` keeps nvmod's own.
    pub cwd: Option<&'a Path>,
}

impl<'a> Call<'a> {
    /// A call of `program` with `args`, expecting exit code 0, with no pinned checksum and nothing on stdin.
    #[must_use]
    pub fn new(tool: &'a str, program: &'a Path, args: &'a [OsString]) -> Call<'a> {
        Call {
            tool,
            program,
            args,
            stdin: None,
            expected_sha256: None,
            ok_exit_codes: &[0],
            cwd: None,
        }
    }
}

/// What a finished tool left.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    /// Everything on standard output (at most the cap).
    pub stdout: Vec<u8>,
    /// The start of standard error, as text.
    pub stderr: String,
    /// The exit code.
    pub exit_code: i32,
}

/// The limits and the environment of a tool call.
#[derive(Debug, Clone)]
pub struct Runner {
    /// How long a call may run before the tool is killed.
    pub timeout: Duration,
    /// The most bytes of standard output accepted; more kills the tool.
    pub max_output: usize,
    /// The most bytes of standard error kept; the rest is read and dropped, it never stops the tool.
    pub max_stderr: usize,
    /// The `PATH` the tool sees (by default nvmod's own, which only names folders and is needed on systems such as NixOS
    /// where `/usr/bin` holds almost nothing). Nothing else of nvmod's environment is passed on.
    pub path: String,
    /// Further variables to pass, by name and value.
    pub env: Vec<(String, String)>,
}

impl Default for Runner {
    fn default() -> Runner {
        Runner {
            timeout: Duration::from_secs(300),
            max_output: 64 * 1024 * 1024,
            max_stderr: STDERR_KEEP,
            path: std::env::var("PATH").unwrap_or_default(),
            env: Vec::new(),
        }
    }
}

/// Variables a network client needs to reach the network the way the user's shell does: proxies and certificate
/// locations. Named by a caller that opts in with [`Runner::inherit`].
pub const NETWORK_ENV: &[&str] = &[
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "ALL_PROXY",
    "NO_PROXY",
    "http_proxy",
    "https_proxy",
    "all_proxy",
    "no_proxy",
    "SSL_CERT_FILE",
    "SSL_CERT_DIR",
    "CURL_CA_BUNDLE",
];

impl Runner {
    /// Also pass the variables `names` that are set in nvmod's own environment, with their values. The scrub stays
    /// the default; a caller names what its program cannot work without.
    #[must_use]
    pub fn inherit(mut self, names: &[&str]) -> Runner {
        for name in names {
            if let Ok(value) = std::env::var(name) {
                self.env.push(((*name).to_owned(), value));
            }
        }
        self
    }

    /// Also pass every variable of the caller's own environment whose name and value are text. For a program that
    /// is the Toolbox itself, such as the GUI starting its own `raul`, which needs the same home, display and
    /// Steam variables as the person's shell; every other program keeps the scrub. A variable this Runner already
    /// passes keeps its value.
    #[must_use]
    pub fn inherit_all(mut self) -> Runner {
        for (name, value) in std::env::vars_os() {
            if let (Some(name), Some(value)) = (name.to_str(), value.to_str())
                && name != "PATH"
                && !self.env.iter().any(|(set, _)| set == name)
            {
                self.env.push((name.to_owned(), value.to_owned()));
            }
        }
        self
    }

    /// Also pass `name` with `value`.
    #[must_use]
    pub fn with_env(mut self, name: &str, value: &str) -> Runner {
        self.env.push((name.to_owned(), value.to_owned()));
        self
    }
}

pub(crate) fn failed(call: &Call, reason: impl Into<String>) -> ToolError {
    ToolError::Failed {
        tool: call.tool.to_owned(),
        reason: reason.into(),
    }
}

/// Read `pipe` into memory, at most `cap` bytes, and send what was kept when it ends. With `over`, reading stops
/// and the flag is set as soon as more than `cap` bytes are offered; without it the rest is read and dropped, so
/// the tool never blocks on a full pipe.
pub(crate) fn drain(
    mut pipe: impl Read + Send + 'static,
    cap: usize,
    over: Option<Arc<AtomicBool>>,
) -> mpsc::Receiver<Vec<u8>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut kept = Vec::new();
        let mut chunk = [0u8; 8192];
        loop {
            match pipe.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let room = cap.saturating_sub(kept.len());
                    kept.extend_from_slice(&chunk[..n.min(room)]);
                    if n > room
                        && let Some(flag) = &over
                    {
                        flag.store(true, Ordering::SeqCst);
                        break;
                    }
                }
            }
        }
        let _ = tx.send(kept);
    });
    rx
}

/// How many programs this process has started through the Toolbox's runner. A test reads it before and after an
/// action to prove the action started none (ADR-0016).
static STARTED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// How many programs this process has started so far, through [`Runner`] or [`crate::start_program`].
#[must_use]
pub fn programs_started() -> u64 {
    STARTED.load(std::sync::atomic::Ordering::SeqCst)
}

/// Counts one started program.
pub(crate) fn count_started() {
    STARTED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

/// Start the command. A program that was written an instant ago can still be "busy" on Linux while another thread's
/// fork holds the file open for writing; that clears in milliseconds, so it is tried again a few times.
pub(crate) fn spawn(command: &mut Command) -> std::io::Result<std::process::Child> {
    count_started();
    let mut tries = 0;
    loop {
        match command.spawn() {
            Err(e) if e.raw_os_error() == Some(26) && tries < 10 => {
                tries += 1;
                std::thread::sleep(Duration::from_millis(20));
            }
            other => return other,
        }
    }
}

impl Runner {
    /// Check the pinned checksum of the program, if any.
    pub(crate) fn verify(call: &Call) -> Result<(), ToolError> {
        let Some(expected) = call.expected_sha256 else {
            return Ok(());
        };
        let unverifiable = |reason: String| ToolError::Unverifiable {
            tool: call.tool.to_owned(),
            reason,
        };
        let bytes = std::fs::read(call.program)
            .map_err(|e| unverifiable(format!("cannot read {}: {e}", call.program.display())))?;
        let actual = sha256_hex(&bytes);
        if actual.eq_ignore_ascii_case(expected) {
            Ok(())
        } else {
            Err(unverifiable(format!(
                "{} has SHA-256 {actual}, the pinned one is {expected}; it was not run",
                call.program.display()
            )))
        }
    }

    /// Run `call` and return what the tool left.
    ///
    /// # Errors
    /// `Unverifiable` when the pinned checksum does not match (the tool is not started); `Failed` when the tool
    /// cannot be started, runs past the timeout or the output cap (it is killed), or exits with a code that is
    /// not in `ok_exit_codes`.
    pub fn run(&self, call: &Call) -> Result<Output, ToolError> {
        let output = self.run_any(call)?;
        if call.ok_exit_codes.contains(&output.exit_code) {
            Ok(output)
        } else {
            let stderr = &output.stderr;
            let first_line = stderr.lines().next().unwrap_or("").trim();
            Err(failed(
                call,
                format!("exit code {}. {first_line}", output.exit_code),
            ))
        }
    }

    /// Run `call` and return what the program left whatever its exit code was; `ok_exit_codes` is not consulted.
    /// For callers that read a failing program's output themselves (a build log, an extractor's complaint).
    ///
    /// # Errors
    /// As [`Runner::run`], except that no exit code is a failure; a program ended by a signal still is.
    pub fn run_any(&self, call: &Call) -> Result<Output, ToolError> {
        let _span = tracing::info_span!("tool", tool = call.tool).entered();
        let started = Instant::now();
        let result = self.run_inner(call);
        let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        match &result {
            Ok(output) => tracing::info!(
                exit_code = i64::from(output.exit_code),
                duration_ms,
                "tool finished"
            ),
            Err(e) => tracing::info!(duration_ms, "tool failed: {e}"),
        }
        result
    }

    /// The command for `call`: its program and arguments, the scrubbed environment plus what this Runner passes,
    /// piped output, and standard input piped only when the call has some.
    pub(crate) fn command(&self, call: &Call) -> Command {
        // The one place the Toolbox starts a program (ADR-0007).
        #[allow(clippy::disallowed_methods)]
        let mut command = Command::new(call.program);
        command
            .args(call.args)
            .env_clear()
            .env("PATH", &self.path)
            .env("LC_ALL", "C")
            .stdin(if call.stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if cfg!(windows)
            && let Some(root) = std::env::var_os("SystemRoot")
        {
            command.env("SystemRoot", root);
        }
        // The only extra environment a tool gets from nvmod (ADR-0012): which run it belongs to, and that nvmod
        // reads JSON log lines. A tool may ignore both. Set only inside a command that keeps a run log.
        if let Some(id) = nvmod_runlog::run_id() {
            command
                .env("NVMOD_RUN_ID", id)
                .env("NVMOD_LOG_FORMAT", "json");
        }
        command.envs(self.env.iter().map(|(k, v)| (k, v)));
        if let Some(cwd) = call.cwd {
            command.current_dir(cwd);
        }
        command
    }

    fn run_inner(&self, call: &Call) -> Result<Output, ToolError> {
        Self::verify(call)?;
        let mut command = self.command(call);
        let mut child = spawn(&mut command).map_err(|e| {
            failed(
                call,
                format!("cannot start {}: {e}", call.program.display()),
            )
        })?;
        if let (Some(bytes), Some(mut pipe)) = (call.stdin, child.stdin.take()) {
            let bytes = bytes.to_vec();
            std::thread::spawn(move || {
                let _ = pipe.write_all(&bytes);
            });
        }
        let over = Arc::new(AtomicBool::new(false));
        let (Some(stdout_pipe), Some(stderr_pipe)) = (child.stdout.take(), child.stderr.take())
        else {
            let _ = child.kill();
            return Err(failed(call, "its output pipes could not be opened"));
        };
        let out = drain(stdout_pipe, self.max_output, Some(Arc::clone(&over)));
        let err = drain(stderr_pipe, self.max_stderr, None);
        let deadline = Instant::now() + self.timeout;
        let mut stop = None;
        let status = loop {
            if over.load(Ordering::SeqCst) {
                stop = Some(format!(
                    "it wrote more than the cap of {} bytes",
                    self.max_output
                ));
            } else if nvmod_core::cancel::requested() {
                stop = Some(nvmod_core::cancel::INTERRUPTED.to_owned());
            } else if Instant::now() >= deadline {
                stop = Some(format!("it ran longer than {:?}", self.timeout));
            } else {
                match child.try_wait() {
                    Ok(Some(status)) => break status,
                    Ok(None) => {
                        std::thread::sleep(Duration::from_millis(5));
                        continue;
                    }
                    Err(e) => return Err(failed(call, format!("cannot wait for the tool: {e}"))),
                }
            }
            let _ = child.kill();
            break child
                .wait()
                .map_err(|e| failed(call, format!("cannot wait for the tool: {e}")))?;
        };
        let stdout = out.recv_timeout(PIPE_GRACE).unwrap_or_default();
        let stderr =
            String::from_utf8_lossy(&err.recv_timeout(PIPE_GRACE).unwrap_or_default()).into_owned();
        if !stderr.is_empty() {
            // Redacted by the run log's writer like everything else; capped at 64 KiB per call.
            tracing::info!(
                stderr = nvmod_runlog::capped_stderr(&stderr),
                "tool standard error"
            );
        }
        let first_line = stderr.lines().next().unwrap_or("").trim();
        if let Some(reason) = stop {
            return Err(failed(
                call,
                format!("{reason}; it was stopped. {first_line}"),
            ));
        }
        match status.code() {
            Some(exit_code) => Ok(Output {
                stdout,
                stderr,
                exit_code,
            }),
            None => Err(failed(call, "it was ended by a signal".to_owned())),
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use nvmod_testkit::TempDir;
    use std::path::PathBuf;

    fn script(dir: &TempDir, name: &str, body: &str) -> PathBuf {
        nvmod_testkit::write_script(dir.path(), name, body)
    }

    fn args(words: &[&str]) -> Vec<OsString> {
        words.iter().map(OsString::from).collect()
    }

    fn fast() -> Runner {
        Runner {
            timeout: Duration::from_secs(10),
            ..Runner::default()
        }
    }

    #[test]
    fn arguments_are_passed_as_words_with_no_shell() {
        let dir = TempDir::new("run-args");
        let p = script(&dir, "t", r#"printf '%s|' "$@""#);
        let a = args(&["a b", "c;touch pwned", "$HOME"]);
        let out = fast().run(&Call::new("t", &p, &a)).unwrap();
        assert_eq!(out.stdout, b"a b|c;touch pwned|$HOME|");
        assert!(!dir.path().join("pwned").exists());
    }

    /// A file name that is not UTF-8 reaches the program as the same bytes (a mod or a file may be named so on Linux).
    #[cfg(unix)]
    #[test]
    fn an_argument_that_is_not_utf8_reaches_the_program_unchanged() {
        use std::os::unix::ffi::OsStringExt;
        let dir = TempDir::new("run-nonutf8");
        let p = script(&dir, "t", r#"printf '%s' "$1" | od -An -tx1 | tr -d ' \n'"#);
        let odd = OsString::from_vec(vec![b'a', 0xff, 0xfe, b'z']);
        let a = [odd];
        let out = fast().run(&Call::new("t", &p, &a)).unwrap();
        assert_eq!(out.stdout, b"61fffe7a");
    }

    #[test]
    fn the_environment_is_scrubbed_and_extras_are_passed() {
        let dir = TempDir::new("run-env");
        let p = script(&dir, "t", "env");
        // Set in the parent: must not reach the tool.
        let mut runner = fast();
        runner.env.push(("NVMOD_EXTRA".into(), "1".into()));
        let out = runner.run(&Call::new("t", &p, &[])).unwrap();
        let text = String::from_utf8(out.stdout).unwrap();
        let names: Vec<&str> = text.lines().filter_map(|l| l.split('=').next()).collect();
        assert!(
            names.contains(&"PATH") && names.contains(&"NVMOD_EXTRA"),
            "{text}"
        );
        assert!(
            !names.contains(&"HOME") && !names.contains(&"USER"),
            "{text}"
        );
        assert!(text.contains("LC_ALL=C"), "{text}");
    }

    #[test]
    fn standard_input_is_passed_and_closed() {
        let dir = TempDir::new("run-stdin");
        let p = script(&dir, "t", "cat");
        let mut call = Call::new("t", &p, &[]);
        call.stdin = Some(b"{\"request\":1}");
        assert_eq!(fast().run(&call).unwrap().stdout, b"{\"request\":1}");
    }

    #[test]
    fn a_tool_that_runs_too_long_is_killed_and_reported_as_failed() {
        let dir = TempDir::new("run-timeout");
        let p = script(&dir, "t", "exec sleep 30");
        let runner = Runner {
            timeout: Duration::from_millis(300),
            ..Runner::default()
        };
        let started = Instant::now();
        let why = runner.run(&Call::new("slow", &p, &[])).unwrap_err();
        assert!(started.elapsed() < Duration::from_secs(10));
        assert!(
            matches!(&why, ToolError::Failed { tool, reason } if tool == "slow" && reason.contains("longer")),
            "{why}"
        );
        assert_eq!(why.exit_code(), 4);
    }

    #[test]
    fn a_tool_that_writes_past_the_cap_is_killed() {
        let dir = TempDir::new("run-cap");
        let p = script(&dir, "t", "exec yes");
        let runner = Runner {
            max_output: 1000,
            ..fast()
        };
        let why = runner.run(&Call::new("loud", &p, &[])).unwrap_err();
        assert!(
            matches!(&why, ToolError::Failed { reason, .. } if reason.contains("cap of 1000")),
            "{why}"
        );
    }

    #[test]
    fn output_exactly_at_the_cap_is_accepted() {
        let dir = TempDir::new("run-cap-exact");
        let p = script(&dir, "t", "printf 12345");
        let runner = Runner {
            max_output: 5,
            ..fast()
        };
        assert_eq!(
            runner.run(&Call::new("t", &p, &[])).unwrap().stdout,
            b"12345"
        );
    }

    #[test]
    fn a_failing_exit_code_is_a_tool_failure_with_the_first_line_of_its_error() {
        let dir = TempDir::new("run-exit");
        let p = script(&dir, "t", "echo 'it broke' >&2; echo second >&2; exit 7");
        let why = fast().run(&Call::new("t", &p, &[])).unwrap_err();
        let text = why.to_string();
        assert!(
            text.contains("exit code 7") && text.contains("it broke") && !text.contains("second"),
            "{text}"
        );
        assert_eq!(why.exit_code(), 4);
        // An exit code the caller lists is an answer.
        let q = script(&dir, "u", "echo out; exit 1");
        let mut call = Call::new("u", &q, &[]);
        call.ok_exit_codes = &[0, 1];
        let out = fast().run(&call).unwrap();
        assert_eq!(
            (out.exit_code, out.stdout.as_slice()),
            (1, b"out\n".as_slice())
        );
    }

    #[test]
    fn a_program_that_cannot_start_is_a_tool_failure() {
        let dir = TempDir::new("run-missing");
        let why = fast()
            .run(&Call::new("t", &dir.path().join("missing"), &[]))
            .unwrap_err();
        assert!(matches!(why, ToolError::Failed { .. }), "{why}");
    }

    #[test]
    fn a_checksum_mismatch_exits_3_and_the_tool_is_not_started() {
        let dir = TempDir::new("run-sha");
        let marker = dir.path().join("ran");
        let p = script(&dir, "t", &format!("touch {}", marker.display()));
        let good = sha256_hex(&std::fs::read(&p).unwrap());
        let mut call = Call::new("t", &p, &[]);
        let wrong = "0".repeat(64);
        call.expected_sha256 = Some(&wrong);
        let why = fast().run(&call).unwrap_err();
        assert!(matches!(why, ToolError::Unverifiable { .. }), "{why}");
        assert_eq!(why.exit_code(), 3);
        assert!(!marker.exists());
        call.expected_sha256 = Some(&good);
        fast().run(&call).unwrap();
        assert!(marker.exists());
        // Changed after it was pinned: refused again on the next call.
        std::fs::write(&p, "#!/bin/sh\ntrue\n# changed\n").unwrap();
        assert_eq!(fast().run(&call).unwrap_err().exit_code(), 3);
        let upper = good.to_ascii_uppercase();
        call.expected_sha256 = Some(&upper);
        assert_eq!(fast().run(&call).unwrap_err().exit_code(), 3);
    }
}
