// SPDX-License-Identifier: GPL-3.0-only
//! The streaming mode of the [`Runner`]: the same argument vector, checksum, scrubbed environment, timeout and output
//! cap as [`Runner::run`], plus what a long job in the GUI needs. The program runs in its own process group (a new
//! process group on Windows), each line of its standard output is handed over while it runs, and a stop request asks
//! the whole group to end (`SIGTERM`, `taskkill /T` on Windows) and forces it (`SIGKILL`, `taskkill /T /F`) once the
//! grace period has passed.

use crate::runner::{PIPE_GRACE, drain, failed, spawn};
use crate::{Call, Runner, ToolError};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

/// How long a stopped program may take to end before it is forced to, unless the caller says otherwise.
pub const STOP_GRACE: Duration = Duration::from_secs(5);

/// How often the runner looks at the stop request, the timeout and the program while no line arrives.
const TICK: Duration = Duration::from_millis(5);

/// A request to stop a streamed call, shared between whoever asks and the runner that obeys. Cloning gives another
/// handle to the same request.
#[derive(Debug, Clone, Default)]
pub struct StopHandle(Arc<AtomicBool>);

impl StopHandle {
    /// A handle nobody has used.
    #[must_use]
    pub fn new() -> StopHandle {
        StopHandle::default()
    }

    /// Asks the program to stop. Asking twice is the same as asking once.
    pub fn request(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    /// Whether a stop was asked for.
    #[must_use]
    pub fn is_requested(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

/// How a streamed program ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Streamed {
    /// The exit code; `None` when a signal ended it.
    pub exit_code: Option<i32>,
    /// Every line of standard output, each ended by a newline (at most the cap).
    pub stdout: String,
    /// The start of standard error, as text.
    pub stderr: String,
    /// Whether it ended because a stop was asked for.
    pub stopped: bool,
}

/// One message of the standard output reader.
enum Piped {
    /// A line, without its line ending.
    Line(String),
    /// More than the cap was offered; reading stopped.
    Over,
}

/// Read `pipe` line by line and send each line, until it ends or more than `cap` bytes have been offered (line endings
/// count, as they do for [`Runner::run`]).
fn lines(pipe: impl Read + Send + 'static, cap: usize) -> mpsc::Receiver<Piped> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(pipe);
        let mut total = 0usize;
        loop {
            let mut bytes = Vec::new();
            match reader.read_until(b'\n', &mut bytes) {
                Ok(0) | Err(_) => break,
                Ok(n) => total = total.saturating_add(n),
            }
            if total > cap {
                let _ = tx.send(Piped::Over);
                break;
            }
            if bytes.last() == Some(&b'\n') {
                bytes.pop();
            }
            if bytes.last() == Some(&b'\r') {
                bytes.pop();
            }
            if tx
                .send(Piped::Line(String::from_utf8_lossy(&bytes).into_owned()))
                .is_err()
            {
                break;
            }
        }
    });
    rx
}

/// Asks the process group led by `pid` to end, or forces it to. The standard library has no signal call and the
/// Toolbox forbids `unsafe`, so the system's own program does it.
#[cfg(unix)]
pub(crate) fn signal_group(pid: u32, force: bool) {
    let signal = if force { "-KILL" } else { "-TERM" };
    // The group is the Runner's own child; signalling it is part of running it (ADR-0007).
    crate::runner::count_started();
    #[allow(clippy::disallowed_methods)]
    let _ = Command::new("kill")
        .args([signal, "--", &format!("-{pid}")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

/// Asks the process tree of `pid` to end, or forces it to.
#[cfg(windows)]
pub(crate) fn signal_group(pid: u32, force: bool) {
    let pid = pid.to_string();
    let mut args = vec!["/PID", pid.as_str(), "/T"];
    if force {
        args.push("/F");
    }
    // The tree is the Runner's own child; ending it is part of running it (ADR-0007).
    crate::runner::count_started();
    #[allow(clippy::disallowed_methods)]
    let _ = Command::new("taskkill")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

/// Puts the program in a process group of its own, so a stop reaches the programs it starts too.
pub(crate) fn own_group(command: &mut Command) {
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(command, 0);
    #[cfg(windows)]
    {
        /// `CREATE_NEW_PROCESS_GROUP` of the Windows API.
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        std::os::windows::process::CommandExt::creation_flags(command, CREATE_NEW_PROCESS_GROUP);
    }
}

/// Where a stop stands.
enum Stop {
    /// Nobody asked.
    None,
    /// The group was asked to end at this instant.
    Asked(Instant),
    /// The group was forced to end, for this reason (`None` for a stop request whose grace ran out).
    Forced(Option<String>),
}

fn force(pid: u32, child: &mut Child) {
    signal_group(pid, true);
    let _ = child.kill();
}

impl Runner {
    /// Run `call` and hand each line of its standard output to `on_line` while it runs (without its line ending;
    /// the last line may lack one). `stop` asks the program and every program it started to end; after `grace` they
    /// are forced to. The global cancel flag of `nvmod_core::cancel` counts as a stop request too. No exit code is
    /// a failure: the caller reads the [`Streamed`] result.
    ///
    /// # Errors
    /// `Unverifiable` when the pinned checksum does not match (the program is not started); `Failed` when it cannot
    /// be started, or runs past the timeout or the output cap (the group is forced to end).
    pub fn run_streaming(
        &self,
        call: &Call,
        stop: &StopHandle,
        grace: Duration,
        on_line: &mut dyn FnMut(&str),
    ) -> Result<Streamed, ToolError> {
        let _span = tracing::info_span!("tool", tool = call.tool, streaming = true).entered();
        let started = Instant::now();
        let result = self.streaming_inner(call, stop, grace, on_line);
        let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        match &result {
            Ok(done) => tracing::info!(
                exit_code = done.exit_code.map(i64::from),
                stopped = done.stopped,
                duration_ms,
                "tool finished"
            ),
            Err(e) => tracing::info!(duration_ms, "tool failed: {e}"),
        }
        result
    }

    fn streaming_inner(
        &self,
        call: &Call,
        stop: &StopHandle,
        grace: Duration,
        on_line: &mut dyn FnMut(&str),
    ) -> Result<Streamed, ToolError> {
        Self::verify(call)?;
        let mut command = self.command(call);
        own_group(&mut command);
        let mut child = spawn(&mut command).map_err(|e| {
            failed(
                call,
                format!("cannot start {}: {e}", call.program.display()),
            )
        })?;
        let pid = child.id();
        if let (Some(bytes), Some(mut pipe)) = (call.stdin, child.stdin.take()) {
            let bytes = bytes.to_vec();
            std::thread::spawn(move || {
                let _ = pipe.write_all(&bytes);
            });
        }
        let (Some(stdout_pipe), Some(stderr_pipe)) = (child.stdout.take(), child.stderr.take())
        else {
            force(pid, &mut child);
            let _ = child.wait();
            return Err(failed(call, "its output pipes could not be opened"));
        };
        let out = lines(stdout_pipe, self.max_output);
        let err = drain(stderr_pipe, self.max_stderr, None);
        let watch = Watch {
            pid,
            deadline: started_plus(self.timeout),
            grace,
        };
        let mut stdout = String::new();
        let mut state = Stop::None;
        let mut over = false;
        let mut reader_done = false;
        let status = loop {
            // One line per turn at most, so a program that never stops printing is still stopped on time.
            if reader_done {
                std::thread::sleep(TICK);
            } else {
                match out.recv_timeout(TICK) {
                    Ok(Piped::Line(line)) => deliver(&line, on_line, &mut stdout),
                    Ok(Piped::Over) => over = true,
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => reader_done = true,
                }
            }
            state = self.advance(state, over, &watch, stop, &mut child);
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => {}
                Err(e) => {
                    force(pid, &mut child);
                    let _ = child.wait();
                    return Err(failed(call, format!("cannot wait for the tool: {e}")));
                }
            }
        };
        // The rest of the output; a program it left behind may hold the pipe open, so this is bounded.
        let rest = Instant::now() + PIPE_GRACE;
        if !reader_done && !over {
            over = rest_of(&out, rest, on_line, &mut stdout);
        }
        // Standard error shares the same bound, so a left-behind program costs at most one grace period.
        let left = rest.saturating_duration_since(Instant::now());
        let stderr =
            String::from_utf8_lossy(&err.recv_timeout(left).unwrap_or_default()).into_owned();
        if !stderr.is_empty() {
            tracing::info!(
                stderr = nvmod_runlog::capped_stderr(&stderr),
                "tool standard error"
            );
        }
        if over && !matches!(state, Stop::Forced(Some(_))) {
            state = Stop::Forced(Some(self.over_cap()));
        }
        let stopped = match state {
            Stop::Forced(Some(reason)) => {
                let first_line = stderr.lines().next().unwrap_or("").trim();
                return Err(failed(
                    call,
                    format!("{reason}; it was stopped. {first_line}"),
                ));
            }
            Stop::Asked(_) | Stop::Forced(None) => true,
            Stop::None => false,
        };
        Ok(Streamed {
            exit_code: status.code(),
            stdout,
            stderr,
            stopped,
        })
    }

    fn over_cap(&self) -> String {
        format!("it wrote more than the cap of {} bytes", self.max_output)
    }

    /// The next stop state: the output cap and the timeout force the group at once, a stop request asks it and
    /// forces it once the grace period has passed.
    fn advance(
        &self,
        state: Stop,
        over: bool,
        watch: &Watch,
        stop: &StopHandle,
        child: &mut Child,
    ) -> Stop {
        match state {
            Stop::None if over => {
                force(watch.pid, child);
                Stop::Forced(Some(self.over_cap()))
            }
            Stop::None if watch.deadline.is_some_and(|d| Instant::now() >= d) => {
                force(watch.pid, child);
                Stop::Forced(Some(format!("it ran longer than {:?}", self.timeout)))
            }
            Stop::None if stop.is_requested() || nvmod_core::cancel::requested() => {
                signal_group(watch.pid, false);
                Stop::Asked(Instant::now())
            }
            Stop::Asked(at) if at.elapsed() >= watch.grace => {
                force(watch.pid, child);
                Stop::Forced(None)
            }
            other => other,
        }
    }
}

/// What the stop logic of one run needs to know.
struct Watch {
    /// The program's process id, which leads its group.
    pid: u32,
    /// When the timeout runs out; `None` for no deadline.
    deadline: Option<Instant>,
    /// How long a stop request waits before it forces.
    grace: Duration,
}

/// Hands one line to the caller and keeps it.
fn deliver(line: &str, on_line: &mut dyn FnMut(&str), stdout: &mut String) {
    on_line(line);
    stdout.push_str(line);
    stdout.push('\n');
}

/// Reads what is left of standard output once the program has ended, until the pipe closes or `until`; returns
/// whether the cap was passed.
fn rest_of(
    out: &mpsc::Receiver<Piped>,
    until: Instant,
    on_line: &mut dyn FnMut(&str),
    stdout: &mut String,
) -> bool {
    loop {
        match out.recv_timeout(until.saturating_duration_since(Instant::now())) {
            Ok(Piped::Line(line)) => deliver(&line, on_line, stdout),
            Ok(Piped::Over) => return true,
            Err(_) => return false,
        }
    }
}

/// The instant `timeout` from now, or `None` when it is too far away to represent (no deadline).
fn started_plus(timeout: Duration) -> Option<Instant> {
    Instant::now().checked_add(timeout)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use nvmod_testkit::TempDir;
    use std::path::PathBuf;

    fn script(dir: &TempDir, name: &str, body: &str) -> PathBuf {
        nvmod_testkit::write_script(dir.path(), name, body)
    }

    fn fast() -> Runner {
        Runner {
            timeout: Duration::from_secs(20),
            ..Runner::default()
        }
    }

    fn collect(
        runner: &Runner,
        call: &Call,
        stop: &StopHandle,
        grace: Duration,
    ) -> (Result<Streamed, ToolError>, Vec<String>) {
        let mut seen = Vec::new();
        let result = runner.run_streaming(call, stop, grace, &mut |l| seen.push(l.to_owned()));
        (result, seen)
    }

    #[test]
    fn each_line_is_handed_over_while_the_program_runs() {
        let dir = TempDir::new("stream-lines");
        let gate = dir.path().join("gate");
        // The second line is printed only once the first was seen, so a runner that waited for the end would hang
        // until the timeout.
        let p = script(
            &dir,
            "t",
            &format!(
                "echo one; while [ ! -e {g} ]; do sleep 0.01; done; printf 'two\\r\\nthree'",
                g = gate.display()
            ),
        );
        let runner = Runner {
            timeout: Duration::from_secs(10),
            ..Runner::default()
        };
        let mut seen = Vec::new();
        let done = runner
            .run_streaming(
                &Call::new("t", &p, &[]),
                &StopHandle::new(),
                STOP_GRACE,
                &mut |line| {
                    if line == "one" {
                        std::fs::write(&gate, "").unwrap();
                    }
                    seen.push(line.to_owned());
                },
            )
            .unwrap();
        assert_eq!(seen, ["one", "two", "three"]);
        assert_eq!(done.stdout, "one\ntwo\nthree\n");
        assert_eq!((done.exit_code, done.stopped), (Some(0), false));
    }

    #[test]
    fn a_failing_exit_code_is_reported_not_an_error() {
        let dir = TempDir::new("stream-exit");
        let p = script(&dir, "t", "echo out; echo 'it broke' >&2; exit 7");
        let (done, seen) = collect(
            &fast(),
            &Call::new("t", &p, &[]),
            &StopHandle::new(),
            STOP_GRACE,
        );
        let done = done.unwrap();
        assert_eq!(seen, ["out"]);
        assert_eq!(done.exit_code, Some(7));
        assert!(done.stderr.contains("it broke"), "{}", done.stderr);
    }

    #[test]
    fn the_environment_is_scrubbed_and_standard_input_is_passed() {
        let dir = TempDir::new("stream-env");
        let p = script(&dir, "t", "cat; echo; env");
        let mut call = Call::new("t", &p, &[]);
        call.stdin = Some(b"in");
        let runner = fast().with_env("RAUL_EXTRA", "1");
        let (done, _) = collect(&runner, &call, &StopHandle::new(), STOP_GRACE);
        let text = done.unwrap().stdout;
        assert!(text.starts_with("in\n"), "{text}");
        assert!(
            text.contains("RAUL_EXTRA=1") && text.contains("LC_ALL=C"),
            "{text}"
        );
        assert!(!text.lines().any(|l| l.starts_with("HOME=")), "{text}");
    }

    #[test]
    fn inherit_all_passes_the_callers_environment() {
        let dir = TempDir::new("stream-inherit");
        let p = script(&dir, "t", "env");
        let (done, _) = collect(
            &fast().inherit_all(),
            &Call::new("t", &p, &[]),
            &StopHandle::new(),
            STOP_GRACE,
        );
        let text = done.unwrap().stdout;
        if let Some(home) = std::env::var_os("HOME").and_then(|h| h.into_string().ok()) {
            assert!(text.contains(&format!("HOME={home}")), "{text}");
        }
        assert_eq!(
            text.lines().filter(|l| l.starts_with("PATH=")).count(),
            1,
            "{text}"
        );
        // A variable the caller set keeps its value.
        let (done, _) = collect(
            &fast().with_env("HOME", "/set/by/caller").inherit_all(),
            &Call::new("t", &p, &[]),
            &StopHandle::new(),
            STOP_GRACE,
        );
        let text = done.unwrap().stdout;
        let homes: Vec<&str> = text.lines().filter(|l| l.starts_with("HOME=")).collect();
        assert_eq!(homes, ["HOME=/set/by/caller"], "{text}");
    }

    #[test]
    fn a_stop_ends_the_whole_group_politely() {
        let dir = TempDir::new("stream-term");
        let marker = dir.path().join("child-alive");
        // A child of the program that would write the marker if it outlived the stop.
        let p = script(
            &dir,
            "t",
            &format!(
                "(sleep 2; touch {m}) & echo started; wait",
                m = marker.display()
            ),
        );
        let stop = StopHandle::new();
        let asker = stop.clone();
        let started = Instant::now();
        let done = fast()
            .run_streaming(&Call::new("t", &p, &[]), &stop, STOP_GRACE, &mut |line| {
                if line == "started" {
                    asker.request();
                }
            })
            .unwrap();
        assert!(done.stopped);
        assert_eq!(done.exit_code, None, "ended by SIGTERM");
        assert!(started.elapsed() < STOP_GRACE, "it was not forced");
        std::thread::sleep(Duration::from_millis(2500));
        assert!(
            !marker.exists(),
            "the program's own child outlived the stop"
        );
    }

    #[test]
    fn a_program_that_ignores_the_stop_is_forced_after_the_grace_period() {
        let dir = TempDir::new("stream-kill");
        let p = script(
            &dir,
            "t",
            "trap '' TERM; echo started; while :; do sleep 0.05; done",
        );
        let stop = StopHandle::new();
        let asker = stop.clone();
        let started = Instant::now();
        let done = fast()
            .run_streaming(
                &Call::new("t", &p, &[]),
                &stop,
                Duration::from_millis(300),
                &mut |line| {
                    if line == "started" {
                        asker.request();
                    }
                },
            )
            .unwrap();
        let took = started.elapsed();
        assert!(done.stopped && done.exit_code.is_none(), "{done:?}");
        assert!(took >= Duration::from_millis(300), "{took:?}");
        assert!(took < Duration::from_secs(10), "{took:?}");
    }

    #[test]
    fn a_program_that_runs_too_long_is_forced_and_reported_as_failed() {
        let dir = TempDir::new("stream-timeout");
        let p = script(&dir, "t", "echo first; exec sleep 30");
        let runner = Runner {
            timeout: Duration::from_millis(300),
            ..Runner::default()
        };
        let started = Instant::now();
        let (why, seen) = collect(
            &runner,
            &Call::new("slow", &p, &[]),
            &StopHandle::new(),
            STOP_GRACE,
        );
        let why = why.unwrap_err();
        assert!(started.elapsed() < Duration::from_secs(10));
        assert_eq!(seen, ["first"]);
        assert!(
            matches!(&why, ToolError::Failed { tool, reason } if tool == "slow" && reason.contains("longer")),
            "{why}"
        );
    }

    #[test]
    fn a_program_that_writes_past_the_cap_is_forced() {
        let dir = TempDir::new("stream-cap");
        let p = script(&dir, "t", "exec yes");
        let runner = Runner {
            max_output: 1000,
            ..fast()
        };
        let (why, seen) = collect(
            &runner,
            &Call::new("loud", &p, &[]),
            &StopHandle::new(),
            STOP_GRACE,
        );
        let why = why.unwrap_err();
        assert!(seen.len() <= 500, "{}", seen.len());
        assert!(
            matches!(&why, ToolError::Failed { reason, .. } if reason.contains("cap of 1000")),
            "{why}"
        );
    }

    #[test]
    fn output_exactly_at_the_cap_is_accepted_and_one_byte_more_is_not() {
        let dir = TempDir::new("stream-cap-exact");
        let runner = Runner {
            max_output: 5,
            ..fast()
        };
        for (name, body, ok) in [
            ("a", "printf '1234\\n'", true),
            ("b", "printf '12345'", true),
            ("c", "printf '12345\\n'", false),
        ] {
            let p = script(&dir, name, body);
            let call = Call::new("t", &p, &[]);
            let (done, _) = collect(&runner, &call, &StopHandle::new(), STOP_GRACE);
            assert_eq!(done.is_ok(), ok, "{body}: {done:?}");
        }
    }

    #[test]
    fn a_program_that_cannot_start_or_be_verified_is_an_error() {
        let dir = TempDir::new("stream-missing");
        let missing = dir.path().join("missing");
        let (why, _) = collect(
            &fast(),
            &Call::new("t", &missing, &[]),
            &StopHandle::new(),
            STOP_GRACE,
        );
        assert!(matches!(why, Err(ToolError::Failed { .. })), "{why:?}");
        let marker = dir.path().join("ran");
        let p = script(&dir, "t", &format!("touch {}", marker.display()));
        let mut call = Call::new("t", &p, &[]);
        let wrong = "0".repeat(64);
        call.expected_sha256 = Some(&wrong);
        let (why, _) = collect(&fast(), &call, &StopHandle::new(), STOP_GRACE);
        assert_eq!(why.unwrap_err().exit_code(), crate::EXIT_VERIFICATION);
        assert!(!marker.exists());
    }

    #[test]
    fn a_stop_handle_is_shared_by_its_clones() {
        let a = StopHandle::new();
        let b = a.clone();
        assert!(!b.is_requested());
        a.request();
        a.request();
        assert!(b.is_requested());
    }
}
