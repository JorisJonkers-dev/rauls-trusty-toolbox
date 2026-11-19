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
    /// Whether the job writes though it holds no lock, because the Instance it makes does not exist yet.
    creating: bool,
    /// The profile of the Instance that gets an automatic snapshot before the job, as the command line takes one
    /// before a change of the lists: `Some(None)` is the profile the Instance has.
    snapshot: Option<SnapshotPlan>,
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
            creating: false,
            snapshot: None,
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
            creating: false,
            snapshot: None,
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
            creating: false,
            snapshot: None,
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
            creating: false,
            snapshot: None,
        }
    }

    /// [`DirectJob::writing_as`] for a job that makes an Instance: it writes, but there is no Instance to lock yet.
    pub fn creating_as<T, F>(run: F, render: fn(&T) -> Result<Reply, serde_json::Error>) -> Self
    where
        F: FnOnce(&mut dyn Progress, &CancelToken) -> Result<Outcome<T>, JobError> + Send + 'static,
        T: 'static,
    {
        Self {
            lock: None,
            body: Self::boxed_as(run, render),
            usage: false,
            creating: true,
            snapshot: None,
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
            creating: false,
            snapshot: None,
        }
    }

    /// Takes an automatic snapshot of `profile` (`None` for the Instance's own) before the job and puts it away again
    /// when the job did not go through, as `nvmod` does around a change of the lists.
    #[must_use]
    pub fn snapshotting(mut self, profile: Option<String>) -> Self {
        self.snapshot = Some(SnapshotPlan { profile });
        self
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
        "profile-create" => Some(profile_create(&nvmod_api::profile::CreateRequest {
            instance: PathBuf::from(value_of(rest, "--instance")),
            name: option_of(rest, "--name"),
        })),
        "init" => Some(
            if value_of(rest, "--instance").is_empty() || value_of(rest, "--base").is_empty() {
                DirectJob::usage("init: give --base and --instance")
            } else {
                init(&nvmod_api::init::CreateRequest {
                    instance: PathBuf::from(value_of(rest, "--instance")),
                    base: PathBuf::from(value_of(rest, "--base")),
                })
            },
        ),
        "install-nv-rs" => {
            Some(install_nvrs_request(rest).map_or_else(DirectJob::usage, |r| install_nvrs(&r)))
        }
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

/// `nvmod_api::mods::change`: enables, disables or moves Mods of a profile. Its document is the report, as
/// `raul mods enable|disable|reorder --json` prints it. A dry run only plans and takes no lock; a change holds the
/// Instance lock while it runs.
pub fn mods_change(request: &nvmod_api::mods::ChangeRequest) -> DirectJob {
    let instance = request.instance.clone();
    let dry_run = request.dry_run;
    let profile = request.profile.clone();
    let request = request.clone();
    let run = move |progress: &mut dyn Progress, cancel: &CancelToken| {
        nvmod_api::mods::change(&request, progress, cancel)
    };
    if dry_run {
        DirectJob::reading(run)
    } else {
        DirectJob::writing(instance, "mods", run).snapshotting(profile)
    }
}

/// `nvmod_api::init::create`: makes an Instance. It writes, and holds no lock because the Instance is not there yet.
/// The command prints a line of text, so the document here is the Instance that was made.
pub fn init(request: &nvmod_api::init::CreateRequest) -> DirectJob {
    let request = request.clone();
    DirectJob::creating_as(
        move |progress, cancel| nvmod_api::init::create(&request, progress, cancel),
        |made: &nvmod_api::init::CreateOutput| serde_json::to_value(made).map(Reply::ok),
    )
}

/// The request of `raul install-nv-rs` from the words after it, with the defaults the command line has.
fn install_nvrs_request(
    rest: &[String],
) -> Result<nvmod_api::install_nvrs::InstallRequest, String> {
    if value_of(rest, "--instance").is_empty() {
        return Err("install-nv-rs: give --instance".to_owned());
    }
    let path = |flag: &str| option_of(rest, flag).map(PathBuf::from);
    let mut request: nvmod_api::install_nvrs::InstallRequest =
        serde_json::from_value(serde_json::json!({ "instance": value_of(rest, "--instance") }))
            .expect("an install request needs only its Instance");
    request.source = path("--source");
    request.release = has_flag(rest, "--release");
    request.cargo = path("--cargo").or_else(|| std::env::var_os("NVMOD_CARGO").map(PathBuf::from));
    if let Some(url) = option_of(rest, "--release-url") {
        request.release_url = url;
    }
    if let Some(ms) = option_of(rest, "--retry-delay-ms") {
        request.retry_delay_ms = ms
            .parse()
            .map_err(|_| "install-nv-rs: --retry-delay-ms needs a number".to_owned())?;
    }
    Ok(request)
}

/// `nvmod_api::install_nvrs::install`: builds or fetches nv-rs into the Instance, under the Instance lock.
pub fn install_nvrs(request: &nvmod_api::install_nvrs::InstallRequest) -> DirectJob {
    let request = request.clone();
    DirectJob::writing(
        request.instance.clone(),
        "install-nv-rs",
        move |progress, cancel| nvmod_api::install_nvrs::install(&request, progress, cancel),
    )
}

/// The one place an install request is made, from what the page has: the day the request grows a field, it is
/// added here.
pub fn install_request(
    instance: &Path,
    profile: Option<&str>,
    archive: &Path,
    name: Option<&str>,
    dry_run: bool,
    answers: Option<&Path>,
) -> nvmod_api::install::InstallRequest {
    nvmod_api::install::InstallRequest {
        instance: instance.to_path_buf(),
        profile: profile.map(str::to_owned),
        archive: archive.to_path_buf(),
        name: name.map(str::to_owned),
        dry_run,
        answers: answers.map(Path::to_path_buf),
        max_extract_bytes: None,
    }
}

/// `nvmod_api::install::install`: installs a Mod, or plans to. Its document is the report, as `raul install --json`
/// prints it; a FOMOD that needs answers is a result with exit 1 and the questions, as the command ends. A plan
/// takes no lock; an install holds the Instance lock.
pub fn install_mod(request: &nvmod_api::install::InstallRequest) -> DirectJob {
    let instance = request.instance.clone();
    let dry_run = request.dry_run;
    let profile = request.profile.clone();
    let request = request.clone();
    let run = move |progress: &mut dyn Progress, cancel: &CancelToken| {
        nvmod_api::install::install(&request, progress, cancel)
    };
    let render: fn(&nvmod_api::install::InstallOutput) -> Result<Reply, serde_json::Error> =
        |done| match done {
            nvmod_api::install::InstallOutput::Installed(report) => {
                serde_json::to_value(report).map(Reply::ok)
            }
            nvmod_api::install::InstallOutput::Questions(questions) => Ok(Reply {
                json: serde_json::json!({ "status": "questions", "questions": questions }),
                exit: 1,
            }),
        };
    if dry_run {
        DirectJob::reading_as(run, render)
    } else {
        DirectJob::writing_as(instance, "install", run, render).snapshotting(profile)
    }
}

/// `nvmod_api::import_mo2::import`: builds a new Instance from an MO2 profile. It writes and holds no lock, because
/// the Instance is not there yet. Profile files that disagree are a result with exit 1, as the command ends.
pub fn import_mo2(request: &nvmod_api::import_mo2::ImportRequest) -> DirectJob {
    let request = request.clone();
    DirectJob::creating_as(
        move |progress, cancel| nvmod_api::import_mo2::import(&request, progress, cancel),
        |done: &nvmod_api::import_mo2::ImportOutput| match done {
            nvmod_api::import_mo2::ImportOutput::Imported(report) => {
                serde_json::to_value(report).map(Reply::ok)
            }
            nvmod_api::import_mo2::ImportOutput::Disagreements(findings) => Ok(Reply {
                json: serde_json::json!({ "disagreements": findings }),
                exit: 1,
            }),
        },
    )
}

/// `nvmod_api::profile::create`: makes a profile of the Instance, under the Instance lock.
pub fn profile_create(request: &nvmod_api::profile::CreateRequest) -> DirectJob {
    let request = request.clone();
    DirectJob::writing(
        request.instance.clone(),
        "profile",
        move |progress, cancel| nvmod_api::profile::create(&request, progress, cancel),
    )
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

/// `nvmod_api::target::set`: sets what the Instance at `request.instance` runs on. The command has no `--json`, so
/// the document is the outcome only.
pub fn target(request: &nvmod_api::target::SetRequest) -> DirectJob {
    let request = request.clone();
    DirectJob::writing(
        request.instance.clone(),
        "target",
        move |progress, cancel| nvmod_api::target::set(&request, progress, cancel),
    )
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

/// The profile whose automatic snapshot a job takes first.
struct SnapshotPlan {
    profile: Option<String>,
}

/// The automatic snapshot taken before a change of the lists, kept only when the change went through.
struct AutoSnapshot {
    instance: nvmod_instance::NvmodInstance,
    profile: String,
    id: String,
    before: nvmod_backup::Presence,
}

impl AutoSnapshot {
    /// Takes the snapshot when the newest one is older than the configured interval; a snapshot that cannot be taken
    /// is no reason to refuse the change.
    fn take(root: &Path, profile: Option<&str>) -> Option<Self> {
        let instance = nvmod_instance::NvmodInstance::open_read_only(root).ok()?;
        let profile = instance.resolve_profile(profile).ok()?;
        let config = nvmod_backup::load_config(&instance);
        let before = nvmod_backup::presence(&instance);
        let taken =
            nvmod_backup::auto(&instance, &profile, nvmod_backup::clock::now(), &config).ok()??;
        Some(Self {
            instance,
            profile,
            id: taken.id,
            before,
        })
    }

    fn discard(self) {
        nvmod_backup::discard(&self.instance, &self.profile, &self.id, self.before);
    }
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
    let reads = job.lock.is_none() && !job.creating;
    let mut feed = Feed::new(spec, sink);
    if cancel.is_requested() {
        return finish(
            spec,
            &mut feed,
            &End::Stopped,
            "cancelled before it started: nothing was run",
            false,
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
            return finish(spec, &mut feed, &End::Failed(error), "", false, false);
        }
        Some(Ok(guard)) => Some(guard),
        None => None,
    };
    let snapshot = job
        .snapshot
        .as_ref()
        .zip(job.instance())
        .and_then(|(profile, root)| AutoSnapshot::take(root, profile.profile.as_deref()));
    feed.start();
    let end = execute(spec, job.body, &mut feed, cancel);
    if let Some(snapshot) = snapshot
        && !matches!(&end, End::Done(reply) if reply.exit == 0)
    {
        snapshot.discard();
    }
    // Released before the end is told: the next writing job may start the moment this one is seen to be over.
    drop(guard);
    let stopped = format!(
        "cancelled: {label} was stopped at a safe point; the step in flight was stopped at its safe point"
    );
    finish(spec, &mut feed, &end, &stopped, usage, reads)
}

/// Tells the end of the job as events and as the [`Output`].
#[allow(clippy::fn_params_excessive_bools)]
fn finish(
    spec: &JobSpec,
    feed: &mut Feed<'_>,
    end: &End,
    stopped: &str,
    usage: bool,
    reads: bool,
) -> Output {
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
            } else if reads && error.kind == ErrorKind::Failed {
                // A read that failed changed nothing, so there is no step to have put back: it is a refusal.
                ExitClass::Refused
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
