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
type Body =
    Box<dyn FnOnce(&mut dyn Progress, &CancelToken) -> Result<Outcome<Reply>, JobError> + Send>;

/// What a finished job answers: the JSON document the command prints with `--json`, and the exit code the command
/// ends with when that is not 0 (the Doctor's findings end with 1, a failed verification with 3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply {
    /// The document on standard output.
    pub json: serde_json::Value,
    /// The exit code: 0 for success, the command's own code otherwise.
    pub exit: u8,
}

impl Reply {
    /// A successful answer.
    pub fn ok(json: serde_json::Value) -> Self {
        Self { json, exit: 0 }
    }
}

/// One typed job of a Component, ready to run in process.
pub struct DirectJob {
    /// The Instance whose lock is held while the job runs, and the name the lock file records; `None` for a job
    /// that does not write.
    lock: Option<(PathBuf, &'static str)>,
    body: Body,
    /// Whether the job ends as a usage error (a request the command line would not take), exit class 5.
    usage: bool,
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
        T: Serialize + 'static,
        F: FnOnce(&mut dyn Progress, &CancelToken) -> Result<Outcome<T>, JobError> + Send + 'static,
    {
        Self {
            lock: Some((instance.into(), name)),
            body: Self::boxed(run),
            usage: false,
        }
    }

    /// A job that only reads: no lock is taken.
    pub fn reading<T, F>(run: F) -> Self
    where
        T: Serialize + 'static,
        F: FnOnce(&mut dyn Progress, &CancelToken) -> Result<Outcome<T>, JobError> + Send + 'static,
    {
        Self {
            lock: None,
            body: Self::boxed(run),
            usage: false,
        }
    }

    /// [`DirectJob::writing`] for a command whose document and exit code are not simply the serialised result:
    /// `render` makes them from it (mirror the command's own `print_*` function).
    pub fn writing_as<T, F>(
        instance: impl Into<PathBuf>,
        name: &'static str,
        run: F,
        render: fn(&T) -> Result<Reply, serde_json::Error>,
    ) -> Self
    where
        F: FnOnce(&mut dyn Progress, &CancelToken) -> Result<Outcome<T>, JobError> + Send + 'static,
        T: 'static,
    {
        Self {
            lock: Some((instance.into(), name)),
            body: Self::boxed_as(run, render),
            usage: false,
        }
    }

    /// [`DirectJob::reading`] with a `render`, as [`DirectJob::writing_as`].
    pub fn reading_as<T, F>(run: F, render: fn(&T) -> Result<Reply, serde_json::Error>) -> Self
    where
        F: FnOnce(&mut dyn Progress, &CancelToken) -> Result<Outcome<T>, JobError> + Send + 'static,
        T: 'static,
    {
        Self {
            lock: None,
            body: Self::boxed_as(run, render),
            usage: false,
        }
    }

    fn boxed<T, F>(run: F) -> Body
    where
        T: Serialize + 'static,
        F: FnOnce(&mut dyn Progress, &CancelToken) -> Result<Outcome<T>, JobError> + Send + 'static,
    {
        Self::boxed_as(run, |value: &T| serde_json::to_value(value).map(Reply::ok))
    }

    fn boxed_as<T: 'static, F>(run: F, render: fn(&T) -> Result<Reply, serde_json::Error>) -> Body
    where
        F: FnOnce(&mut dyn Progress, &CancelToken) -> Result<Outcome<T>, JobError> + Send + 'static,
    {
        Box::new(move |progress, cancel| match run(progress, cancel)? {
            Outcome::Done(value) => render(&value).map(Outcome::Done).map_err(|why| {
                JobError::new(
                    ErrorKind::Internal,
                    "direct.result",
                    format!("the result of the job could not be written: {why}"),
                )
            }),
            Outcome::Cancelled => Ok(Outcome::Cancelled),
        })
    }

    /// A job that does nothing but end as a usage error with `summary`, for a request the command line would refuse
    /// before running (an unknown severity, say).
    pub fn usage(summary: impl Into<String>) -> Self {
        let summary = summary.into();
        Self {
            lock: None,
            body: Box::new(move |_, _| {
                Err(JobError::new(ErrorKind::Refused, "direct.usage", summary))
            }),
            usage: true,
        }
    }

    /// The Instance the job writes to, if it does.
    pub fn instance(&self) -> Option<&Path> {
        self.lock.as_ref().map(|(root, _)| root.as_path())
    }
}

/// The direct job that does what the page's command with the registry id `id` does, from the values and flags after
/// its words; `None` for a command that still runs as a command line. Each converted action adds one arm here.
pub fn for_command(id: &str, rest: &[String]) -> Option<DirectJob> {
    match id {
        "rollback" => Some(match value_of(rest, "--instance") {
            "" => DirectJob::reading(|_, _| -> Result<Outcome<()>, JobError> {
                Err(JobError::new(
                    ErrorKind::Refused,
                    "install.rollback",
                    "no Instance was given to roll back",
                ))
            }),
            instance => rollback(Path::new(instance)),
        }),
        "conflicts" => Some(conflicts(&conflicts_request(rest))),
        "tree" => Some(tree(&tree_request(rest))),
        "doctor" => Some(doctor_request(rest).map_or_else(DirectJob::usage, |r| doctor(&r))),
        "verify" => Some(verify_request(rest).map_or_else(DirectJob::usage, |r| verify(&r))),
        "plugins-list" => Some(plugins_list(&plugins_list_request(rest))),
        _ => None,
    }
}

/// The value after `flag` in `rest` as an option: `None` when the flag is absent.
fn option_of(rest: &[String], flag: &str) -> Option<String> {
    rest.iter()
        .position(|w| w == flag)
        .and_then(|i| rest.get(i + 1))
        .cloned()
}

/// Whether the switch `flag` is in `rest`.
fn has_flag(rest: &[String], flag: &str) -> bool {
    rest.iter().any(|w| w == flag)
}

/// The request of `raul conflicts` from the words after it.
fn conflicts_request(rest: &[String]) -> nvmod_api::conflicts::ReportRequest {
    nvmod_api::conflicts::ReportRequest {
        instance: PathBuf::from(value_of(rest, "--instance")),
        profile: option_of(rest, "--profile"),
        mod_name: option_of(rest, "--mod"),
        archives: has_flag(rest, "--archives"),
    }
}

/// `nvmod_api::conflicts::report`: reads only. Its document is the report, as `raul conflicts --json` prints it.
pub fn conflicts(request: &nvmod_api::conflicts::ReportRequest) -> DirectJob {
    let request = request.clone();
    DirectJob::reading_as(
        move |progress, cancel| nvmod_api::conflicts::report(&request, progress, cancel),
        |done: &nvmod_api::conflicts::ReportOutput| {
            serde_json::to_value(&done.report).map(Reply::ok)
        },
    )
}

/// The request of `raul tree` from the words after it.
fn tree_request(rest: &[String]) -> nvmod_api::tree::TreeRequest {
    nvmod_api::tree::TreeRequest {
        instance: PathBuf::from(value_of(rest, "--instance")),
        profile: option_of(rest, "--profile"),
        path: option_of(rest, "--path"),
    }
}

/// `nvmod_api::tree::show`: reads only. Its document is the tree, as `raul tree --json` prints it.
pub fn tree(request: &nvmod_api::tree::TreeRequest) -> DirectJob {
    let request = request.clone();
    DirectJob::reading_as(
        move |progress, cancel| nvmod_api::tree::show(&request, progress, cancel),
        |done: &nvmod_api::tree::TreeOutput| serde_json::to_value(&done.tree).map(Reply::ok),
    )
}

/// The request of `raul doctor --instance` from the words after it.
fn doctor_request(rest: &[String]) -> Result<nvmod_api::doctor::CheckRequest, String> {
    let target = nvmod_api::doctor::Target::Instance {
        folder: PathBuf::from(value_of(rest, "--instance")),
        profile: option_of(rest, "--profile"),
        game: option_of(rest, "--game").map(PathBuf::from),
        masterlist: option_of(rest, "--masterlist").map(PathBuf::from),
        catalog: option_of(rest, "--catalog").map(PathBuf::from),
    };
    let fail_on = fail_on_of(rest)?;
    Ok(nvmod_api::doctor::CheckRequest { target, fail_on })
}

/// `nvmod_api::doctor::check`: reads only. Its document is the report, as `raul doctor --json` prints it, and its exit
/// code is 1 when the report has a finding at or above the request's `fail_on`, as the command ends.
pub fn doctor(request: &nvmod_api::doctor::CheckRequest) -> DirectJob {
    let request = request.clone();
    DirectJob::reading_as(
        move |progress, cancel| nvmod_api::doctor::check(&request, progress, cancel),
        |done: &nvmod_api::doctor::CheckOutput| {
            serde_json::to_value(&done.report).map(|json| Reply {
                json,
                exit: u8::from(done.found),
            })
        },
    )
}

/// The request of `raul verify --instance` from the words after it.
fn verify_request(rest: &[String]) -> Result<nvmod_api::verify::VerifyRequest, String> {
    let fail_on = fail_on_of(rest)?;
    Ok(nvmod_api::verify::VerifyRequest {
        instance: PathBuf::from(value_of(rest, "--instance")),
        profile: option_of(rest, "--profile"),
        masterlist: option_of(rest, "--masterlist").map(PathBuf::from),
        catalog: option_of(rest, "--catalog").map(PathBuf::from),
        proton: option_of(rest, "--proton"),
        fail_on,
    })
}

/// The `--fail-on` severity of `rest`, `Error` when absent; an unknown word is the usage error the command line gives.
fn fail_on_of(rest: &[String]) -> Result<nvmod_api::doctor::Severity, String> {
    option_of(rest, "--fail-on").map_or(Ok(nvmod_api::doctor::Severity::Error), |word| {
        word.parse().map_err(|why| format!("--fail-on: {why}"))
    })
}

/// `nvmod_api::verify::verify`: reads only. Its document is the verification, as `raul verify --json` prints it; its
/// exit code is 3 when the install is not verified, as the command ends.
pub fn verify(request: &nvmod_api::verify::VerifyRequest) -> DirectJob {
    let request = request.clone();
    DirectJob::reading_as(
        move |progress, cancel| nvmod_api::verify::verify(&request, progress, cancel),
        |done: &nvmod_api::verify::VerifyOutput| -> Result<Reply, serde_json::Error> {
            Ok(Reply {
                json: serde_json::json!({
                    "schema_version": 1,
                    "verified": done.verified,
                    "doctor": done.doctor,
                    "nv_rs": done.nv_rs,
                    "launch_plan": done.launch_plan,
                }),
                exit: if done.verified { 0 } else { 3 },
            })
        },
    )
}

/// The request of `raul plugins list --instance` from the words after `list`.
fn plugins_list_request(rest: &[String]) -> nvmod_api::plugins::ListRequest {
    nvmod_api::plugins::ListRequest {
        instance: PathBuf::from(value_of(rest, "--instance")),
        profile: option_of(rest, "--profile"),
    }
}

/// `nvmod_api::plugins::list`: reads only. Its document is the list, as `raul plugins list --json` prints it.
pub fn plugins_list(request: &nvmod_api::plugins::ListRequest) -> DirectJob {
    let request = request.clone();
    DirectJob::reading(move |progress, cancel| nvmod_api::plugins::list(&request, progress, cancel))
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

/// The exit class of a job's error kind: the code the command line would have ended with, except that an
/// `internal` error (a bug, never a property of the input) stays [`ExitClass::Internal`] instead of being folded into
/// "a step failed and was put back", so the page does not offer a retry that cannot help.
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
    Done(Reply),
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
    let usage = job.usage;
    let mut feed = Feed::new(spec, sink);
    if cancel.is_requested() {
        return finish(
            spec,
            &mut feed,
            &End::Stopped,
            "cancelled before it started: nothing was run",
            false,
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
            return finish(spec, &mut feed, &End::Failed(error), "", false);
        }
        Some(Ok(guard)) => Some(guard),
        None => None,
    };
    feed.start();
    let end = execute(spec, job.body, &mut feed, cancel);
    // Released before the end is told: the next writing job may start the moment this one is seen to be over.
    drop(guard);
    let stopped = format!(
        "cancelled: {label} was stopped at a safe point; the step in flight was stopped at its safe point"
    );
    finish(spec, &mut feed, &end, &stopped, usage)
}

/// Tells the end of the job as events and as the [`Output`].
fn finish(spec: &JobSpec, feed: &mut Feed<'_>, end: &End, stopped: &str, usage: bool) -> Output {
    let label = spec.action.command();
    feed.start();
    let (line, outcome, class, output) = match end {
        End::Done(reply) => {
            let class = ExitClass::from_code(i64::from(reply.exit));
            (
                format!("{label} finished"),
                outcome_of(class),
                class,
                Output {
                    code: Some(i64::from(reply.exit)),
                    stdout: format!("{}\n", reply.json),
                    stderr: String::new(),
                },
            )
        }
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
            let class = if usage {
                ExitClass::Usage
            } else {
                class_of(error.kind)
            };
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
