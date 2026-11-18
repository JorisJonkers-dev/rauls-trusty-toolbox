// SPDX-License-Identifier: GPL-3.0-only
//! Jobs that call a Component's typed job in this process instead of running a command line (ADR-0016).
//!
//! A [`DirectJob`] is the in-process counterpart of an `Exec` argument vector: a typed request and a closure that
//! calls `nvmod_api`'s `run(request, &mut dyn Progress, &CancelToken) -> Result<Outcome<T>, JobError>`. [`run_direct`]
//! runs it as one job of the job table and answers with the [`Output`] a command line would have left, so
//! everything above it stays as it is: the pages read an exit code, a JSON document on standard output and the
//! last line of standard error, and the job table reads `Started`, `Progress`, `Log` and `Finished` events.
//!
//! * **Progress**: each `nvmod_api::Event` becomes a `JobEvent::Progress` (the step is the phase, the counts are
//!   the counts). A page that wants it worded in the Toolbox's voice maps the stable label key.
//! * **Cancel**: the job's [`CancelHandle`] is watched while the job runs on a worker thread, and a request
//!   cancels the job's `CancelToken`, which the job reads at its next safe point. A job that never reads it runs
//!   to its end, and the stop is reported once it has.
//! * **Writing**: a job that writes names its Instance, and [`run_direct`] holds the Instance lock around it
//!   (`raul_core::instance_lock`, the file `nvmod` and the launcher use), so a direct job and a command line, or
//!   the running game, exclude each other. A lock that is held ends the job as a refusal.
//! * **Panic**: a panic inside the job is caught at the worker thread and ends the job as an internal error, the
//!   way [`InProcessExec`](crate::inproc::InProcessExec) does, so the window and the writer slot are never lost.
//! * **Result**: `Done(T)` is exit 0 with `T` as JSON; `Cancelled` is a stop (no code); a [`JobError`] is the exit
//!   class of its kind (`refused` 2, `corrupt` 3, `failed` 4, `internal` none) with its summary, one line, on
//!   standard error. The summary never holds a command line.

use crate::exec::{CancelHandle, Output};
use crate::jobs::{Feed, JobSpec};
use crate::registry::Action;
use crate::wiring::outcome_of;
use nvmod_api::{CancelToken, ErrorKind, Event, JobError, Outcome, Progress};
use raul_core::instance_lock::InstanceGuard;
use raul_ui_model::error::ExitClass;
use raul_ui_model::job::{JobEvent, Outcome as JobOutcome};
use serde::Serialize;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

/// How often a running job is asked whether it was told to stop.
const TICK: Duration = Duration::from_millis(50);

/// What the summary of a job that panicked says.
const PANICKED: &str = "the job stopped unexpectedly; the state is not known, run the Doctor";

/// What a typed job is: a closure that runs it with progress and a cancel token, and returns its result as JSON.
type Body = Box<
    dyn FnOnce(&mut dyn Progress, &CancelToken) -> Result<Outcome<serde_json::Value>, JobError>
        + Send,
>;

/// One typed job of a Component, ready to run in process.
pub struct DirectJob {
    /// The Instance whose lock is held while the job runs, and the name the lock file records; `None` for a job
    /// that does not write.
    lock: Option<(PathBuf, &'static str)>,
    body: Body,
}

impl std::fmt::Debug for DirectJob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirectJob")
            .field("lock", &self.lock)
            .finish_non_exhaustive()
    }
}

impl DirectJob {
    /// A job that writes to the Instance at `instance`: it runs holding the Instance lock, recorded as `name`.
    pub fn writing<T, F>(instance: impl Into<PathBuf>, name: &'static str, run: F) -> Self
    where
        T: Serialize,
        F: FnOnce(&mut dyn Progress, &CancelToken) -> Result<Outcome<T>, JobError> + Send + 'static,
    {
        Self {
            lock: Some((instance.into(), name)),
            body: Self::boxed(run),
        }
    }

    /// A job that only reads: no lock is taken.
    pub fn reading<T, F>(run: F) -> Self
    where
        T: Serialize,
        F: FnOnce(&mut dyn Progress, &CancelToken) -> Result<Outcome<T>, JobError> + Send + 'static,
    {
        Self {
            lock: None,
            body: Self::boxed(run),
        }
    }

    fn boxed<T, F>(run: F) -> Body
    where
        T: Serialize,
        F: FnOnce(&mut dyn Progress, &CancelToken) -> Result<Outcome<T>, JobError> + Send + 'static,
    {
        Box::new(move |progress, cancel| match run(progress, cancel)? {
            Outcome::Done(value) => serde_json::to_value(value)
                .map(Outcome::Done)
                .map_err(|why| {
                    JobError::new(
                        ErrorKind::Internal,
                        "direct.result",
                        format!("the result of the job could not be written: {why}"),
                    )
                }),
            Outcome::Cancelled => Ok(Outcome::Cancelled),
        })
    }

    /// The Instance the job writes to, if it does.
    pub fn instance(&self) -> Option<&Path> {
        self.lock.as_ref().map(|(root, _)| root.as_path())
    }
}

/// The direct job that does what the page's command of `action` does, from the values and flags after its words;
/// `None` for an action that still runs as a command line. Each converted action adds one arm here.
pub fn for_command(action: Action, rest: &[String]) -> Option<DirectJob> {
    match action {
        Action::Rollback => Some(rollback(&PathBuf::from(value_of(rest, "--instance")))),
        _ => None,
    }
}

/// The value after `flag` in `rest`, empty when there is none.
fn value_of<'a>(rest: &'a [String], flag: &str) -> &'a str {
    rest.iter()
        .position(|w| w == flag)
        .and_then(|i| rest.get(i + 1))
        .map_or("", String::as_str)
}

/// `nvmod_api::install::rollback` of the Instance at `instance`: undoes the last install, sort or Mods change.
/// Its report is the document `raul rollback --json` prints.
pub fn rollback(instance: &Path) -> DirectJob {
    let request = nvmod_api::install::RollbackRequest {
        instance: instance.to_path_buf(),
    };
    DirectJob::writing(instance, "rollback", move |progress, cancel| {
        nvmod_api::install::rollback(&request, progress, cancel)
    })
}

/// The exit class of a job's error kind: the code the command line would have ended with.
fn class_of(kind: ErrorKind) -> ExitClass {
    match kind {
        ErrorKind::Refused => ExitClass::Refused,
        ErrorKind::Corrupt => ExitClass::VerificationFailed,
        ErrorKind::Internal => ExitClass::Internal,
        // `failed` now, and any kind a newer nvmod adds: a step failed and was put back.
        _ => ExitClass::StepFailedRolledBack,
    }
}

/// A summary as one line, since a page shows the last line of standard error.
fn one_line(summary: &str) -> String {
    summary.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// How a direct job ended, before it is told as events and an [`Output`].
enum End {
    Done(serde_json::Value),
    Stopped,
    Failed(JobError),
}

/// Runs `body` on a worker thread, turning its progress into job events and passing a stop request on to its
/// token. A panic ends it as an internal error.
fn execute(spec: &JobSpec, body: Body, feed: &mut Feed<'_>, cancel: &CancelHandle) -> End {
    let token = CancelToken::new();
    let (tx, rx) = mpsc::channel::<Event>();
    let joined = std::thread::scope(|scope| {
        let worker_token = token.clone();
        let worker = scope.spawn(move || {
            let mut progress = move |event: &Event| {
                let _ = tx.send(event.clone());
            };
            catch_unwind(AssertUnwindSafe(|| body(&mut progress, &worker_token)))
        });
        loop {
            if cancel.is_requested() {
                token.cancel();
            }
            match rx.recv_timeout(TICK) {
                Ok(event) => progress_event(spec, feed, &event),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        worker.join()
    });
    match joined {
        Ok(Ok(Ok(Outcome::Done(value)))) => End::Done(value),
        Ok(Ok(Ok(Outcome::Cancelled))) => End::Stopped,
        Ok(Ok(Err(error))) => End::Failed(error),
        Ok(Err(_)) | Err(_) => End::Failed(JobError::new(
            ErrorKind::Internal,
            spec.action.command(),
            PANICKED,
        )),
    }
}

/// One progress event of the job, as the job table reads it.
fn progress_event(spec: &JobSpec, feed: &mut Feed<'_>, event: &Event) {
    let seq = feed.next_seq();
    feed.emit(JobEvent::Progress {
        id: spec.id,
        seq,
        phase: event.step.to_owned(),
        done: event.done.unwrap_or(0),
        total: event.total,
        message: None,
    });
}

/// Runs `job` as the job `spec` describes, handing each [`JobEvent`] to `sink` as it happens, and answers with what
/// a command line would have left: the JSON of the result on standard output with exit 0, or the one-line summary
/// of the error on standard error with the exit code of its class. `spec.args` is not used; `spec.writes` and
/// `spec.action` are what the table shows.
///
/// A writing job holds the Instance lock from before it starts until after it ended. When `cancel` is requested
/// before the job starts, it is not started.
pub fn run_direct(
    spec: &JobSpec,
    job: DirectJob,
    sink: &mut dyn FnMut(JobEvent),
    cancel: &CancelHandle,
) -> Output {
    let label = spec.action.command();
    let mut feed = Feed::new(spec, sink);
    if cancel.is_requested() {
        return finish(
            spec,
            &mut feed,
            &End::Stopped,
            "cancelled before it started: nothing was run",
        );
    }
    // Held while the job runs.
    let guard = match job
        .lock
        .as_ref()
        .map(|(root, name)| InstanceGuard::acquire(root, name))
    {
        Some(Err(refused)) => {
            let error = JobError::new(ErrorKind::Refused, label, refused.to_string());
            return finish(spec, &mut feed, &End::Failed(error), "");
        }
        Some(Ok(guard)) => Some(guard),
        None => None,
    };
    feed.start();
    let end = execute(spec, job.body, &mut feed, cancel);
    // Released before the end is told: the next writing job may start the moment this one is seen to be over.
    drop(guard);
    let stopped =
        format!("cancelled: {label} was stopped at a safe point; nothing was written after it");
    finish(spec, &mut feed, &end, &stopped)
}

/// Tells the end of the job as events and as the [`Output`].
fn finish(spec: &JobSpec, feed: &mut Feed<'_>, end: &End, stopped: &str) -> Output {
    let label = spec.action.command();
    feed.start();
    let (line, outcome, class, output) = match end {
        End::Done(value) => (
            format!("{label} finished"),
            JobOutcome::Completed,
            ExitClass::Ok,
            Output {
                code: Some(0),
                stdout: format!("{value}\n"),
                stderr: String::new(),
            },
        ),
        End::Stopped => (
            stopped.to_owned(),
            JobOutcome::Cancelled,
            ExitClass::Ok,
            Output {
                code: None,
                stdout: String::new(),
                stderr: format!("{}\n", one_line(stopped)),
            },
        ),
        End::Failed(error) => {
            let class = class_of(error.kind);
            let said = one_line(&error.summary);
            (
                said.clone(),
                outcome_of(class),
                class,
                Output {
                    code: class.code().map(i64::from),
                    stdout: String::new(),
                    stderr: format!("{said}\n"),
                },
            )
        }
    };
    let seq = feed.next_seq();
    feed.emit(JobEvent::Log {
        id: spec.id,
        seq,
        line,
    });
    let seq = feed.next_seq();
    feed.emit(JobEvent::Finished {
        id: spec.id,
        seq,
        outcome,
        exit: class,
    });
    output
}
