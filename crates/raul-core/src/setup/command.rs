// SPDX-License-Identifier: GPL-3.0-only
//! `raul setup` as a job with no command line in it: a first run, a dry run, a resume or a restart of the setup of
//! one Instance, reported as the stream of events the setup wizard reads (the stream `nvmod setup --json` printed):
//! `plan`, `step`, `resumed`, `skipped` and `finished`, each with an increasing `seq` that starts at 1 in every
//! call. The command line and the wizard both call [`execute`] with a typed [`Mode`]; it runs [`crate::journal::run`],
//! so every finished step is in the Instance's journal and a resume needs the Instance only.
//!
//! What it keeps from `nvmod setup`: a run is all or nothing (a failing step removes what the run made, and a run
//! whose check fails at the end removes the Instance too) unless `keep_partial` is set; a resume always keeps the
//! Instance; `restart` removes only an Instance with an unfinished
//! setup, and only when confirmed. What differs: a resume trusts the journal and does not re-check the files a
//! finished step left behind (nvmod's witnesses), and the `sort` step an imported Instance skips reports no
//! would-change counts.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::{CancelToken, Event, Outcome, Planned, Progress, Request, SkipReason, Step};
use crate::journal::{self, Journal};

/// The exit code of a run that did what it was asked.
pub const OK: u8 = 0;
/// The exit code of a command refused before it changed anything.
pub const REFUSED: u8 = 2;
/// The exit code of a run whose last step found the Instance does not verify.
pub const VERIFICATION: u8 = 3;
/// The exit code of a step that failed.
pub const FAILED: u8 = 4;

/// What to do.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)] // one value per call; boxing the request would only burden the callers
pub enum Mode {
    /// A first run, or with `dry_run` only the plan.
    Fresh {
        /// What to set up. Every path is absolute.
        request: Request,
        /// Plan only: nothing is written.
        dry_run: bool,
        /// A failing step leaves the Instance where it is, to be resumed, instead of removing what the run made.
        keep_partial: bool,
    },
    /// Continue the unfinished setup of the Instance.
    Resume {
        /// The Instance folder.
        instance: PathBuf,
    },
    /// Remove the unfinished Instance.
    Restart {
        /// The Instance folder.
        instance: PathBuf,
        /// The person's yes; nothing is removed without it.
        confirmed: bool,
    },
}

/// One line of the event stream: the JSON object (with its `seq`) and the same thing as a line of text.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    /// The event.
    pub json: Value,
    /// The text for people; empty when the event has none.
    pub text: String,
}

/// How a call ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ended {
    /// The exit code.
    pub code: u8,
    /// The reason, as one plain line, when the call did not end well; it never holds a command line.
    pub note: Option<String>,
}

/// The name of a step in the stream: the id, except that the engine step is called `nv-rs` there, as before.
#[must_use]
pub fn stream_name(step: &Step) -> String {
    match step {
        Step::Engine => "nv-rs".to_owned(),
        other => other.id(),
    }
}

/// What a step does, in words.
#[must_use]
pub fn describe(step: &Step, request: &Request) -> String {
    match step {
        Step::Init => "make the Instance skeleton for the base copy".to_owned(),
        Step::ImportMo2 => format!(
            "import the MO2 profile of {} into a new Instance",
            request
                .import_mo2
                .as_deref()
                .map_or_else(String::new, |p| p.display().to_string())
        ),
        Step::Profile => "make the first profile".to_owned(),
        Step::Engine => "put nv-rs in the Instance and check it".to_owned(),
        Step::Target => "make nv-rs the target of the Instance".to_owned(),
        Step::Ttw => "install Tale of Two Wastelands".to_owned(),
        Step::Mod(_) => format!("install the Mod {}", &step.id()["mod:".len()..]),
        Step::Stage => "build the Staging tree".to_owned(),
        Step::Sort => {
            "compute the Load order with loot-rs, write it and stamp the plugin file times"
                .to_owned()
        }
        Step::Stamp => "keep the Load order as imported and stamp the plugin file times".to_owned(),
        Step::Verify => {
            "run the Doctor on the finished Instance and print the launch plan".to_owned()
        }
    }
}

const KEEPS_ORDER: &str =
    "the Load order of the imported MO2 profile is kept; sorting is opt-in (--sort)";
const FINISHED_EARLIER: &str = "finished earlier and still holds";

/// The events, numbered.
struct Events<'a> {
    emit: &'a mut dyn FnMut(Line),
    seq: u64,
}

impl Events<'_> {
    fn put(&mut self, mut json: Value, text: impl Into<String>) {
        self.seq += 1;
        json["seq"] = self.seq.into();
        (self.emit)(Line {
            json,
            text: text.into(),
        });
    }
}

/// Run `mode`; every event goes to `emit`.
pub fn execute(mode: &Mode, emit: &mut dyn FnMut(Line), cancel: &CancelToken) -> Ended {
    let mut events = Events { emit, seq: 0 };
    match mode {
        Mode::Fresh {
            request,
            dry_run,
            keep_partial,
        } => fresh(request, *dry_run, *keep_partial, &mut events, cancel),
        Mode::Resume { instance } => resume(instance, &mut events, cancel),
        Mode::Restart {
            instance,
            confirmed,
        } => restart(instance, *confirmed, &mut events),
    }
}

/// A call that ends before it did anything: one `finished` event and a reason.
fn stop(events: &mut Events, code: u8, status: &str, why: &str, extra: &Value) -> Ended {
    let mut json = json!({"event": "finished", "status": status, "exit_code": code, "rolled_back": false, "reason": why});
    if let (Some(o), Some(e)) = (json.as_object_mut(), extra.as_object()) {
        o.extend(e.clone());
    }
    events.put(json, "");
    Ended {
        code,
        note: Some(format!("{status}: {why}")),
    }
}

fn refuse(events: &mut Events, why: &str) -> Ended {
    stop(events, REFUSED, "refused", why, &json!({}))
}

/// Everything that can be checked before anything is written; the reason for a refusal. `done` names the steps a
/// resume will skip, whose inputs are not needed again.
fn preflight(request: &Request, done: &[String], fresh: bool) -> Result<(), String> {
    let instance = &request.instance;
    if fresh
        && instance.exists()
        && (!instance.is_dir() || std::fs::read_dir(instance).is_ok_and(|mut d| d.next().is_some()))
    {
        let hint = if Journal::file(instance).is_file() {
            "; it holds an unfinished setup: continue it with --resume or remove it with --restart --yes"
        } else {
            ""
        };
        return Err(format!(
            "{} already exists and is not an empty folder; setup only starts from a path that holds nothing{hint}",
            instance.display()
        ));
    }
    if fresh {
        check_base(&request.base)?;
    }
    let need = |path: &Path, what: &str, dir: bool| {
        if (dir && path.is_dir()) || (!dir && path.is_file()) {
            Ok(())
        } else {
            Err(format!("{what} {} does not exist", path.display()))
        }
    };
    let finished = |name: &str| done.iter().any(|d| d == name);
    need(&request.masterlist, "the masterlist", false)?;
    for archive in &request.mods {
        if !finished(&Step::Mod(archive.clone()).id()) && !archive.exists() {
            return Err(format!("the archive {} does not exist", archive.display()));
        }
    }
    if let Some(ttw) = &request.ttw
        && !finished("ttw")
    {
        need(&ttw.fallout3, "the Fallout 3 folder", true)?;
    }
    if let Some(mo2) = &request.import_mo2
        && !finished("import-mo2")
    {
        need(mo2, "the MO2 instance folder", true)?;
    }
    Ok(())
}

/// The Doctor's check of the base copy: a base copy that is not supported, or does not verify, is refused before
/// anything is planned or written.
fn check_base(base: &Path) -> Result<(), String> {
    use nvmod_api::doctor::{CheckRequest, Severity, Target, check};
    let request = CheckRequest {
        target: Target::Base {
            folder: base.to_owned(),
        },
        fail_on: Severity::Error,
    };
    match check(
        &request,
        &mut |_: &nvmod_api::Event| {},
        &CancelToken::new(),
    ) {
        Ok(nvmod_api::Outcome::Done(out)) if out.found => Err(format!(
            "the base copy does not verify:\n{}",
            out.report.to_text().trim_end()
        )),
        Ok(_) => Ok(()),
        Err(e) => Err(e.summary),
    }
}

fn plan_event(
    events: &mut Events,
    request: &Request,
    planned: &[Planned],
    dry: bool,
    resume: bool,
) {
    let steps: Vec<Value> = planned
        .iter()
        .map(|p| {
            json!({
                "step": stream_name(&p.step),
                "description": describe(&p.step, request),
                "skip": (p.skip == Some(SkipReason::KeepsImportedOrder)).then_some(KEEPS_ORDER),
            })
        })
        .collect();
    let total = planned.len();
    events.put(
        json!({"event": "plan", "dry_run": dry, "resume": resume, "instance": request.instance, "steps": steps}),
        format!(
            "raul setup: {total} step(s){}",
            if dry { " (dry run: nothing is written)" } else { "" }
        ),
    );
}

fn fresh(
    request: &Request,
    dry_run: bool,
    keep_partial: bool,
    events: &mut Events,
    cancel: &CancelToken,
) -> Ended {
    if let Err(why) = preflight(request, &[], true) {
        return refuse(events, &why);
    }
    if dry_run {
        let planned = super::plan(request);
        plan_event(events, request, &planned, true, false);
        let total = planned.len();
        for (i, p) in planned.iter().enumerate() {
            let name = stream_name(&p.step);
            let what = describe(&p.step, request);
            events.put(
                json!({"event": "step", "step": name, "status": "planned", "index": i + 1, "total": total, "description": what}),
                format!("  [{}/{total}] {name}: {what}", i + 1),
            );
        }
        events.put(
            json!({"event": "finished", "status": "dry-run", "exit_code": OK, "rolled_back": false}),
            "nothing was written; run without --dry-run to apply",
        );
        return Ended {
            code: OK,
            note: None,
        };
    }
    run_steps(request, keep_partial, false, events, cancel)
}

fn resume(instance: &Path, events: &mut Events, cancel: &CancelToken) -> Ended {
    let journal = match Journal::open(instance) {
        Ok(j) => j,
        Err(e) => return refuse(events, &e.summary),
    };
    if !Journal::file(instance).is_file() {
        return refuse(
            events,
            &format!(
                "{} holds no unfinished setup (there is no setup journal)",
                instance.display()
            ),
        );
    }
    let request = match journal.request() {
        Ok(Some(r)) => r,
        Ok(None) => {
            return refuse(
                events,
                "the journal holds no setup request to continue from; start again with --restart --yes",
            );
        }
        Err(e) => return refuse(events, &e.summary),
    };
    if !same_path(&request.instance, instance) {
        return refuse(
            events,
            &format!(
                "the setup was started for {}, not {}",
                request.instance.display(),
                instance.display()
            ),
        );
    }
    if let Err(why) = preflight(&request, journal.finished(), false) {
        return refuse(events, &why);
    }
    if let Err(e) = journal.check_plan(&request) {
        return refuse(events, &e.summary);
    }
    run_steps(&request, true, true, events, cancel)
}

fn same_path(a: &Path, b: &Path) -> bool {
    let canon = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_owned());
    a == b || canon(a) == canon(b)
}

fn restart(instance: &Path, confirmed: bool, events: &mut Events) -> Ended {
    let journal = match Journal::open(instance) {
        Ok(j) => j,
        Err(e) => return refuse(events, &e.summary),
    };
    if !Journal::file(instance).is_file() {
        return refuse(
            events,
            &format!(
                "{} holds no unfinished setup (there is no setup journal)",
                instance.display()
            ),
        );
    }
    if journal.doc().complete {
        return refuse(
            events,
            &format!(
                "{} is a finished Instance, not a partial one; setup will not remove it",
                instance.display()
            ),
        );
    }
    if !confirmed {
        return refuse(
            events,
            &format!(
                "--restart removes {} and everything in it; add --yes to confirm",
                instance.display()
            ),
        );
    }
    // Not while a job, a command or the game is using the Instance; the lock is let go before the folder goes (it
    // is a file in it).
    match crate::instance_lock::InstanceGuard::acquire(instance, "setup") {
        Ok(guard) => drop(guard),
        Err(e) => return refuse(events, &e.to_string()),
    }
    if std::fs::remove_dir_all(instance).is_err() && instance.exists() {
        return stop(
            events,
            FAILED,
            "failed",
            &format!(
                "cannot remove {}; remove the folder by hand",
                instance.display()
            ),
            &json!({}),
        );
    }
    events.put(
        json!({"event": "finished", "status": "restarted", "exit_code": OK, "rolled_back": false, "removed": instance}),
        format!(
            "raul setup: removed {}; start again with a new setup",
            instance.display()
        ),
    );
    Ended {
        code: OK,
        note: None,
    }
}

/// The failing step of a run, for its `step` and `finished` events.
struct Failure {
    step: Step,
    index: usize,
    total: usize,
    code: u8,
    detail: String,
}

/// Turns the job's events into the stream, and remembers where the run stands.
struct Reporter<'a, 'b> {
    events: &'a mut Events<'b>,
    request: &'a Request,
    resume: bool,
    total: usize,
    /// Steps that finished in this call or earlier.
    finished: usize,
    /// The highest place in the plan a step event has named (from 1), counting the steps that were skipped.
    at: usize,
    started: Option<(Step, usize)>,
}

impl Progress for Reporter<'_, '_> {
    fn report(&mut self, event: &Event) {
        match event {
            Event::Planned { steps } => {
                self.total = steps.len();
                plan_event(self.events, self.request, steps, false, self.resume);
                if self.resume {
                    let completed: Vec<String> = steps
                        .iter()
                        .filter(|p| p.skip == Some(SkipReason::Done))
                        .map(|p| stream_name(&p.step))
                        .collect();
                    self.finished = completed.len();
                    let next = steps
                        .iter()
                        .find(|p| p.skip.is_none())
                        .map(|p| stream_name(&p.step));
                    let at = self.finished.min(self.total.saturating_sub(1)) + 1;
                    self.events.put(
                        json!({"event": "resumed", "instance": self.request.instance, "total": self.total, "completed": completed, "next": next}),
                        format!("raul setup: resuming at step {at} of {}", self.total),
                    );
                }
            }
            Event::Started { step, index, total } => {
                self.at = self.at.max(*index);
                self.started = Some((step.clone(), *index));
                let (name, what) = (stream_name(step), describe(step, self.request));
                self.events.put(
                    json!({"event": "step", "step": name, "status": "started", "index": index, "total": total, "description": what}),
                    format!("[{index}/{total}] {name}: {what}"),
                );
            }
            Event::Skipped {
                step,
                index,
                total,
                reason,
            } => {
                self.at = self.at.max(*index);
                let (name, what) = (stream_name(step), describe(step, self.request));
                let why = match reason {
                    SkipReason::Done => FINISHED_EARLIER,
                    SkipReason::KeepsImportedOrder => KEEPS_ORDER,
                };
                self.events.put(
                    json!({"event": "skipped", "step": name, "index": index, "total": total, "description": what, "reason": why}),
                    format!("[{index}/{total}] {name}: skipped: {why}"),
                );
            }
            Event::Finished { step, index, total } => {
                self.at = self.at.max(*index);
                self.finished += 1;
                self.started = None;
                let (name, what) = (stream_name(step), describe(step, self.request));
                self.events.put(
                    json!({"event": "step", "step": name, "status": "done", "index": index, "total": total, "description": what, "code": 0, "detail": ""}),
                    "",
                );
            }
            Event::Detail { .. } => {}
        }
    }
}

fn code_of(error: &super::Error) -> u8 {
    match error.kind {
        super::ErrorKind::Refused => REFUSED,
        super::ErrorKind::Unverified => VERIFICATION,
        _ => FAILED,
    }
}

/// Run the setup `request` asks for through the journal and report it. `keep` leaves a failed run's Instance as
/// it is (a resume always does); otherwise a run that fails removes what it made.
fn run_steps(
    request: &Request,
    keep: bool,
    resume: bool,
    events: &mut Events,
    cancel: &CancelToken,
) -> Ended {
    let pre_existing = request.instance.is_dir();
    let mut reporter = Reporter {
        events,
        request,
        resume,
        total: 0,
        finished: 0,
        at: 0,
        started: None,
    };
    let result = journal::run(request, &mut reporter, cancel);
    let (finished, started, total, at) = (
        reporter.finished,
        reporter.started.take(),
        reporter.total,
        reporter.at,
    );
    match result {
        Ok(Outcome::Done(_)) => {
            events.put(
                json!({"event": "finished", "status": "ok", "exit_code": OK, "rolled_back": false, "resumable": false, "instance": request.instance}),
                format!(
                    "raul setup: done; the Instance at {} passes the Doctor",
                    request.instance.display()
                ),
            );
            Ended {
                code: OK,
                note: None,
            }
        }
        Ok(Outcome::Cancelled(_)) => {
            // Stopped between two steps: the next one did not run.
            let planned = super::plan(&Request {
                done: Vec::new(),
                ..request.clone()
            });
            let index = at.min(planned.len().saturating_sub(1));
            let step = planned.get(index).map_or(Step::Verify, |p| p.step.clone());
            fail(
                events,
                request,
                &Failure {
                    step,
                    index: index + 1,
                    total: planned.len(),
                    code: FAILED,
                    detail: "the run was stopped before this step".to_owned(),
                },
                (keep, pre_existing, finished > 0 || resume, true),
            )
        }
        Err(journal::RunError::Setup(e)) => {
            let (step, index) = started.unwrap_or_else(|| (e.step.clone(), finished + 1));
            fail(
                events,
                request,
                &Failure {
                    step,
                    index,
                    total,
                    code: code_of(&e),
                    detail: e.summary,
                },
                (keep, pre_existing, finished > 0 || resume, false),
            )
        }
        Err(journal::RunError::Journal(e)) => match (e.kind, started) {
            (journal::ErrorKind::WriteFailed, Some((step, index))) => fail(
                events,
                request,
                &Failure {
                    step,
                    index,
                    total,
                    code: FAILED,
                    detail: e.summary,
                },
                // The step that failed to be written was counted as finished by its own event.
                (keep, pre_existing, finished > 1 || resume, false),
            ),
            _ => refuse(events, &e.summary),
        },
    }
}

/// Finish a run whose step failed: keep the Instance resumable, or undo the run.
fn fail(
    events: &mut Events,
    request: &Request,
    f: &Failure,
    (keep, pre_existing, any_finished, interrupted): (bool, bool, bool, bool),
) -> Ended {
    let resumable = keep && any_finished;
    let undone = !resumable && roll_back(&request.instance, pre_existing);
    let (name, what) = (stream_name(&f.step), describe(&f.step, request));
    events.put(
        json!({"event": "step", "step": name, "status": "failed", "index": f.index, "total": f.total, "description": what, "code": f.code, "detail": f.detail}),
        format!("step {name} failed (exit {})", f.code),
    );
    let (code, note) = if resumable {
        (
            f.code,
            format!(
                "the Instance at {} is kept: continue it with a resume or remove it with a restart",
                request.instance.display()
            ),
        )
    } else if undone {
        (
            f.code,
            "rolled back: the Instance was removed and nothing is left behind".to_owned(),
        )
    } else {
        (
            FAILED,
            "the rollback could not remove the Instance; remove the folder by hand".to_owned(),
        )
    };
    events.put(
        json!({
            "event": "finished", "status": "failed", "exit_code": code,
            "rolled_back": undone, "resumable": resumable, "failed_step": name,
            "outcome": if resumable { "kept-partial" } else if undone { "rolled-back" } else { "rollback-failed" },
            "interrupted": interrupted,
        }),
        note.clone(),
    );
    Ended {
        code,
        note: Some(format!("step {name} failed: {}", f.detail)),
    }
}

/// Undo the run: remove what it made. A folder that existed (empty) stays, emptied.
fn roll_back(instance: &Path, pre_existing: bool) -> bool {
    if !pre_existing {
        return std::fs::remove_dir_all(instance).is_ok() || !instance.exists();
    }
    std::fs::read_dir(instance).is_ok_and(|entries| {
        entries.flatten().all(|e| {
            let p = e.path();
            if p.is_dir() {
                std::fs::remove_dir_all(&p).is_ok()
            } else {
                std::fs::remove_file(&p).is_ok()
            }
        })
    })
}
