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

/// What a streaming job is: it runs with a sink for the lines of its stream and a cancel token, and ends with its
/// exit code and, when it did not end well, the reason in one plain line.
type LinesBody = Box<dyn FnOnce(&mut dyn FnMut(&str), &CancelToken) -> (u8, Option<String>) + Send>;

/// One typed job of a Component, ready to run in process.
pub struct DirectJob {
    /// The Instance whose lock is held while the job runs, and the name the lock file records; `None` for a job
    /// that does not write.
    lock: Option<(PathBuf, &'static str)>,
    body: Body,
    /// The job as a stream of lines, when it is one (see [`DirectJob::streaming`]); `body` is not used then.
    lines: Option<LinesBody>,
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
            lines: None,
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
            lines: None,
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
            lines: None,
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
            lines: None,
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
            lines: None,
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

    /// A job that reports a stream of event lines, as a command line with `--json` does: each line is read as the job
    /// table reads a command's output (the setup stream), and the whole stream is the job's standard output. It
    /// runs holding the Instance lock when `lock` names an Instance and a name for it.
    pub fn streaming<F>(lock: Option<(PathBuf, &'static str)>, run: F) -> Self
    where
        F: FnOnce(&mut dyn FnMut(&str), &CancelToken) -> (u8, Option<String>) + Send + 'static,
    {
        Self {
            lock,
            body: Box::new(|_, _| {
                Err(JobError::new(
                    ErrorKind::Internal,
                    "direct.streaming",
                    "a streaming job was run as a typed one",
                ))
            }),
            lines: Some(Box::new(run)),
            usage: false,
            creating: false,
            snapshot: None,
        }
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
            lines: None,
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
            Some(install_nvrs_request(rest).map_or_else(DirectJob::usage, install_nvrs))
        }
        "install" => {
            Some(install_command_request(rest).map_or_else(DirectJob::usage, |r| install_mod(&r)))
        }
        "conflicts" => Some(conflicts(&conflicts_request(rest))),
        "tree" => Some(tree(&tree_request(rest))),
        "doctor" => Some(doctor_request(rest).map_or_else(DirectJob::usage, |r| doctor(&r))),
        "verify" => Some(verify_request(rest).map_or_else(DirectJob::usage, |r| verify(&r))),
        "plugins-list" => Some(plugins_list(&plugins_list_request(rest))),
        "mods-list" => {
            Some(mods_list_request(rest).map_or_else(DirectJob::usage, |r| mods_list(&r)))
        }
        // `--keep-order` is nvmod's other sort job (`sort_apply::keep_order`), which no page starts: it stays a
        // command line.
        "sort-preview" | "sort-apply" if has_flag(rest, "--keep-order") => None,
        "sort-preview" | "sort-apply" => {
            Some(sort_request(rest).map_or_else(DirectJob::usage, |(r, source)| sort(&r, &source)))
        }
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

/// The request of `raul mods list` from the words after it; a category that is not a number is its usage message.
fn mods_list_request(rest: &[String]) -> Result<nvmod_api::mods::ListRequest, String> {
    let category = match option_of(rest, "--category") {
        None => None,
        Some(word) => Some(
            word.parse::<u32>()
                .map_err(|_| "mods list: --category takes a number".to_string())?,
        ),
    };
    Ok(nvmod_api::mods::ListRequest {
        instance: PathBuf::from(value_of(rest, "--instance")),
        profile: option_of(rest, "--profile"),
        category,
    })
}

/// `nvmod_api::mods::list`: reads only. Its document is the list, as `raul mods list --json` prints it.
pub fn mods_list(request: &nvmod_api::mods::ListRequest) -> DirectJob {
    let request = request.clone();
    DirectJob::reading(move |progress, cancel| nvmod_api::mods::list(&request, progress, cancel))
}

/// Where the masterlist of a sort comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MasterlistSource {
    /// The file the request names.
    File,
    /// The masterlist pinned in the lock file, fetched (checksum verified) into the tool store when it is not
    /// there yet, as `raul sort --fetch-masterlist` does.
    Pinned {
        /// The lock file, `None` for the built-in one.
        lock: Option<PathBuf>,
        /// The tool store, `None` for the default one.
        store: Option<PathBuf>,
    },
}

/// The request of `raul sort` from the words after it, and where its masterlist comes from; a request the command
/// line would not take is its usage message.
fn sort_request(
    rest: &[String],
) -> Result<(nvmod_api::loadorder::SortRequest, MasterlistSource), String> {
    let instance = value_of(rest, "--instance");
    let masterlist = option_of(rest, "--masterlist");
    let fetch = has_flag(rest, "--fetch-masterlist");
    if instance.is_empty() || masterlist.is_some() == fetch {
        return Err(
            "sort: give --instance and either --masterlist <file> or --fetch-masterlist".to_owned(),
        );
    }
    let request = nvmod_api::loadorder::SortRequest {
        instance: PathBuf::from(instance),
        profile: option_of(rest, "--profile"),
        masterlist: masterlist.map_or_default(PathBuf::from),
        apply: has_flag(rest, "--apply"),
    };
    let source = if fetch {
        MasterlistSource::Pinned {
            lock: option_of(rest, "--lock").map(PathBuf::from),
            store: option_of(rest, "--store").map(PathBuf::from),
        }
    } else {
        MasterlistSource::File
    };
    Ok((request, source))
}

/// The tool of `tools.lock.toml` that pins the masterlist.
const MASTERLIST_TOOL: &str = "loot-masterlist-falloutnv";

/// The pinned masterlist, fetched into the tool store when it is not there yet. Fetching is not a job of nvmod's
/// API (its sort takes the file, "the caller fetches and verifies it"), so it is done here with the same pieces the
/// command line uses.
fn fetch_masterlist(
    instance: &Path,
    lock: Option<&Path>,
    store: Option<&Path>,
) -> Result<PathBuf, JobError> {
    use nvmod_tools::{Acquire, AcquireError, Curl, Lock, Store, Trust, builtin_lock};
    const WHAT: &str = "loadorder.sort.fetch";
    let refused = |why: String| JobError::new(ErrorKind::Refused, WHAT, why);
    let lock = match lock {
        None => builtin_lock(),
        Some(path) => {
            let text = std::fs::read_to_string(path)
                .map_err(|e| refused(format!("{}: {e}", path.display())))?;
            Lock::parse(&text).map_err(refused)?
        }
    };
    let entry = lock.tool(MASTERLIST_TOOL).ok_or_else(|| {
        refused(format!(
            "the lock file pins no tool called `{MASTERLIST_TOOL}`, so no masterlist source is pinned: \
             give a masterlist file instead"
        ))
    })?;
    let store = Store::new(store.map_or_else(Store::default_location, Path::to_path_buf));
    let curl = Curl {
        program: "curl".into(),
    };
    let trust = Trust::load(instance).map_err(refused)?;
    let acquire = Acquire {
        fetcher: &curl,
        gh: Path::new("gh"),
        retry_delay: Duration::from_millis(1000),
        store: &store,
        trust: Some(&trust),
    };
    acquire
        .fetch(entry)
        .map(|done| done.path)
        .map_err(|e| match e {
            AcquireError::Refused(why) => refused(why),
            AcquireError::Verification(why) => JobError::new(
                ErrorKind::Corrupt,
                WHAT,
                format!("verification failed: {why}"),
            ),
            AcquireError::Failed(why) => JobError::new(
                ErrorKind::Failed,
                WHAT,
                format!("the masterlist could not be fetched: {why}"),
            ),
        })
}

/// The document of `raul sort --json`, from the proposal and what was applied.
fn sort_document(output: &nvmod_api::loadorder::SortOutput) -> serde_json::Value {
    let proposal = &output.proposal;
    let found = &proposal.output;
    let order: Vec<_> = proposal
        .proposed
        .iter()
        .enumerate()
        .map(|(index, slot)| {
            let result = found
                .plugins
                .iter()
                .find(|p| p.name.eq_ignore_ascii_case(&slot.name));
            serde_json::json!({
                "position": index + 1,
                "name": slot.name,
                "active": slot.active,
                "installed": result.is_some(),
                "group": result.map(|r| r.group.clone()),
                "messages": result.map_or_default(|r| r.messages.iter().map(|m| serde_json::json!({ "kind": m.kind.label(), "text": m.text })).collect::<Vec<_>>()),
                "generated_messages": result.map_or_default(|r| r.generated_messages.iter().map(|m| serde_json::json!({ "kind": m.kind.label(), "text": m.text })).collect::<Vec<_>>()),
                "findings": result.map_or_default(|r| r.findings.iter().map(|f| serde_json::json!({
                    "kind": f.kind.label().replace(' ', ""), "text": f.text, "hard": f.hard,
                })).collect::<Vec<_>>()),
            })
        })
        .collect();
    serde_json::json!({
        "profile": proposal.profile,
        "reordered": proposal.reordered(),
        "positions": { "changed": proposal.positions_changed(), "total": proposal.proposed.len() },
        "order": order,
        "general_messages": found.general_messages.iter()
            .chain(&found.generated_general_messages)
            .map(|m| serde_json::json!({ "kind": m.kind.label(), "text": m.text }))
            .collect::<Vec<_>>(),
        "not_installed": found.not_installed,
        "warnings": found.warnings,
        "applied": output.applied.as_ref().map(|a| serde_json::json!({
            "lists_written": a.lists_written,
            "stamped": a.stamped,
            "detached": a.detached,
            "journal": a.journal,
        })),
    })
}

/// `nvmod_api::loadorder::sort`: proposes the Load order of a profile with loot-rs, and with `apply` writes it. Its
/// document is the one `raul sort --json` prints. A proposal only reads and takes no lock; an apply holds the
/// Instance lock and takes the automatic snapshot first, as the command line does.
pub fn sort(request: &nvmod_api::loadorder::SortRequest, source: &MasterlistSource) -> DirectJob {
    let mut request = request.clone();
    let source = source.clone();
    let instance = request.instance.clone();
    let profile = request.profile.clone();
    let apply = request.apply;
    let run = move |progress: &mut dyn Progress, cancel: &CancelToken| {
        if let MasterlistSource::Pinned { lock, store } = &source {
            progress.report(&Event::step(
                "fetch-masterlist",
                "loadorder.sort.fetching-masterlist",
            ));
            request.masterlist =
                fetch_masterlist(&request.instance, lock.as_deref(), store.as_deref())?;
        }
        nvmod_api::loadorder::sort(&request, progress, cancel)
    };
    let render = |done: &nvmod_api::loadorder::SortOutput| Ok(Reply::ok(sort_document(done)));
    if apply {
        DirectJob::writing_as(instance, "sort", run, render).snapshotting(profile)
    } else {
        DirectJob::reading_as(run, render)
    }
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

/// What `raul install-nv-rs` takes from the words after it: the Instance, and for a developer a source workspace and
/// the `cargo` to build it with. The options that used to fetch a release are gone and are refused (#179).
struct InstallNvRs {
    instance: PathBuf,
    source: Option<PathBuf>,
    cargo: Option<PathBuf>,
}

fn install_nvrs_request(rest: &[String]) -> Result<InstallNvRs, String> {
    if value_of(rest, "--instance").is_empty() {
        return Err("install-nv-rs: give --instance".to_owned());
    }
    for gone in [
        "--release",
        "--release-url",
        "--asset",
        "--repo",
        "--gh",
        "--curl",
        "--retry-delay-ms",
    ] {
        if has_flag(rest, gone) {
            return Err(format!(
                "install-nv-rs: {gone} is gone: the engine comes with raul and is never downloaded"
            ));
        }
    }
    if option_of(rest, "--cargo").is_some() && option_of(rest, "--source").is_none() {
        return Err("install-nv-rs: --cargo only goes with --source".to_owned());
    }
    Ok(InstallNvRs {
        instance: PathBuf::from(value_of(rest, "--instance")),
        source: option_of(rest, "--source").map(PathBuf::from),
        cargo: option_of(rest, "--cargo").map(PathBuf::from),
    })
}

/// `raul_core::setup::packaged::job`: puts the engine that ships with raul in the Instance (or builds a developer's
/// source), under the Instance lock. It fetches nothing.
fn install_nvrs(request: InstallNvRs) -> DirectJob {
    let instance = request.instance.clone();
    DirectJob::writing(
        request.instance,
        "install-nv-rs",
        move |progress, cancel| {
            raul_core::setup::packaged::job(
                &instance,
                request.source.as_deref(),
                request.cargo.as_deref(),
                None,
                progress,
                cancel,
            )
        },
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

/// The request of `raul install` from the words after it: the archive first, then the flags; no archive is its usage
/// message.
fn install_command_request(rest: &[String]) -> Result<nvmod_api::install::InstallRequest, String> {
    let archive = rest
        .first()
        .filter(|w| !w.starts_with("--"))
        .ok_or_else(|| "install: give the archive".to_string())?;
    let answers = option_of(rest, "--answers");
    Ok(install_request(
        Path::new(value_of(rest, "--instance")),
        option_of(rest, "--profile").as_deref(),
        Path::new(archive),
        option_of(rest, "--name").as_deref(),
        has_flag(rest, "--dry-run"),
        answers.as_deref().map(Path::new),
    ))
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

/// The setup job of `raul-core` (`raul_core::setup::execute`) as a streaming job: the first run, the dry run, the
/// resume or the restart of the setup `mode` names, each finished step written to the Instance's journal. Its lines
/// are the events the setup wizard reads. A resume runs holding the Instance lock; a first run has no Instance yet,
/// and a restart removes the folder, so neither takes it.
pub fn setup(mode: raul_core::setup::Mode) -> DirectJob {
    let lock = match &mode {
        raul_core::setup::Mode::Resume { instance } => Some((instance.clone(), "setup")),
        _ => None,
    };
    DirectJob::streaming(lock, move |sink, cancel| {
        let ended =
            raul_core::setup::execute(&mode, &mut |line| sink(&line.json.to_string()), cancel);
        (ended.code, ended.note)
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
    /// A streaming job: its exit code, its reason when it did not end well, and the lines it reported.
    Streamed(u8, Option<String>, Vec<String>),
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

/// [`execute`] for a streaming job: its lines go to the feed as they are reported.
fn execute_lines(
    spec: &JobSpec,
    body: LinesBody,
    feed: &mut Feed<'_>,
    cancel: &CancelHandle,
) -> End {
    let token = CancelToken::new();
    let (tx, rx) = mpsc::channel::<String>();
    let joined = std::thread::scope(|scope| {
        let worker_token = token.clone();
        let worker = scope.spawn(move || {
            let mut lines = Vec::new();
            let ended = catch_unwind(AssertUnwindSafe(|| {
                body(
                    &mut |line: &str| {
                        lines.push(line.to_owned());
                        let _ = tx.send(line.to_owned());
                    },
                    &worker_token,
                )
            }));
            ended.map(|(code, note)| (code, note, lines))
        });
        loop {
            if cancel.is_requested() {
                token.cancel();
            }
            match rx.recv_timeout(TICK) {
                Ok(line) => feed.stream_line(&line),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        worker.join()
    });
    match joined {
        Ok(Ok((code, note, lines))) => End::Streamed(code, note, lines),
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
    let end = match job.lines {
        Some(lines) => execute_lines(spec, lines, &mut feed, cancel),
        None => execute(spec, job.body, &mut feed, cancel),
    };
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
        End::Streamed(code, note, lines) => {
            let class = ExitClass::from_code(i64::from(*code));
            let output = Output {
                code: Some(i64::from(*code)),
                stdout: lines.iter().fold(String::new(), |mut all, l| {
                    all.push_str(l);
                    all.push('\n');
                    all
                }),
                stderr: note
                    .as_deref()
                    .map_or_default(|n| format!("{}\n", one_line(n))),
            };
            // The stream told its own end (its `finished` event): the job table has it already.
            if feed.finished {
                return output;
            }
            (
                note.as_deref()
                    .map_or_else(|| format!("{label} finished"), one_line),
                outcome_of(class),
                class,
                output,
            )
        }
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
