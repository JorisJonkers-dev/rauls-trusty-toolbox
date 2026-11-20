// SPDX-License-Identifier: GPL-3.0-only
//! Running the plan: each step is a call in this process, with no argument vector and no command re-entered.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use raul_drivers::events::Event as TtwEvent;
use raul_drivers::events::Status;
use raul_drivers::ttw::{self, TtwRequest};

use super::{
    CancelToken, Error, ErrorKind, Event, Outcome, Output, Progress, Request, Step, engine, plan,
};

/// Run the setup `request` asks for: report the plan, then run each planned step in order, reporting it as a
/// progress step. A step the plan skips is reported as skipped. The run ends at the first failing step with its
/// typed [`Error`]; a stop asked for through `cancel` ends it before the next step, or inside a step that checks
/// the token, as [`Outcome::Cancelled`].
///
/// The engine step installs and checks nv-rs through the engine install library (`engine.rs`). Mod archives are
/// read in process (`zip`, `7z` and compressed `tar`; a `rar` or another format nvmod cannot read is refused with its
/// message). No step starts a process of its own except the engine install, which runs `cargo` for a source build
/// and `curl` and `gh` to fetch and verify the release.
///
/// # Errors
/// An [`Error`] naming the step, the job and the reason; the Instance is as that step left it (each job puts
/// back what it started).
pub fn run(
    request: &Request,
    progress: &mut dyn Progress,
    cancel: &CancelToken,
) -> Result<Outcome, Error> {
    let planned = plan(request);
    let total = planned.len();
    progress.report(&Event::Planned {
        steps: planned.clone(),
    });
    let mut out = Output {
        completed: Vec::new(),
        skipped: Vec::new(),
        verify: None,
    };
    for (i, planned) in planned.into_iter().enumerate() {
        let (step, index) = (planned.step, i + 1);
        if cancel.is_cancelled() {
            return Ok(Outcome::Cancelled(out));
        }
        if let Some(reason) = planned.skip {
            progress.report(&Event::Skipped {
                step: step.clone(),
                index,
                total,
                reason,
            });
            out.skipped.push((step, reason));
            continue;
        }
        progress.report(&Event::Started {
            step: step.clone(),
            index,
            total,
        });
        if !execute(request, &step, progress, cancel, &mut out)? {
            return Ok(Outcome::Cancelled(out));
        }
        progress.report(&Event::Finished {
            step: step.clone(),
            index,
            total,
        });
        out.completed.push(step);
    }
    Ok(Outcome::Done(out))
}

/// The job's progress as the step's detail events.
fn detail<'a>(
    step: &'a Step,
    progress: &'a mut dyn Progress,
) -> impl FnMut(&nvmod_api::Event) + 'a {
    move |e| {
        progress.report(&Event::Detail {
            step: step.clone(),
            label: e.label.to_owned(),
            done: e.done,
            total: e.total,
        });
    }
}

/// The step's result: `None` when the job was cancelled; a job error becomes the step's typed error.
fn finish<T>(
    step: &Step,
    result: Result<nvmod_api::Outcome<T>, nvmod_api::JobError>,
) -> Result<Option<T>, Error> {
    match result {
        Ok(outcome) => Ok(outcome.done()),
        Err(e) => Err(Error {
            kind: match e.kind {
                nvmod_api::ErrorKind::Refused => ErrorKind::Refused,
                nvmod_api::ErrorKind::Internal => ErrorKind::Internal,
                _ => ErrorKind::Failed,
            },
            step: step.clone(),
            job: e.job,
            summary: e.summary,
        }),
    }
}

fn refuse(step: &Step, job: &'static str, summary: String) -> Error {
    Error {
        kind: ErrorKind::Refused,
        step: step.clone(),
        job,
        summary,
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map_or_else(|| path.to_string_lossy(), |n| n.to_string_lossy())
        .into_owned()
}

/// Run one step. `Ok(false)` means the step was cancelled.
#[allow(clippy::too_many_lines)] // one arm per step
fn execute(
    req: &Request,
    step: &Step,
    progress: &mut dyn Progress,
    cancel: &CancelToken,
    out: &mut Output,
) -> Result<bool, Error> {
    use nvmod_api as api;
    let instance = req.instance.clone();
    let profile = req.profile.clone();
    match step {
        Step::Init => {
            let request = api::init::CreateRequest {
                instance,
                base: req.base.clone(),
            };
            let r = api::init::create(&request, &mut detail(step, progress), cancel);
            Ok(finish(step, r)?.is_some())
        }
        Step::ImportMo2 => {
            let Some(mo2) = req.import_mo2.clone() else {
                return Ok(true);
            };
            let request = api::import_mo2::ImportRequest {
                mo2,
                instance,
                base: req.base.clone(),
                profile,
                copy_only: req.copy,
            };
            let r = api::import_mo2::import(&request, &mut detail(step, progress), cancel);
            match finish(step, r)? {
                None => Ok(false),
                Some(api::import_mo2::ImportOutput::Imported(_)) => Ok(true),
                Some(api::import_mo2::ImportOutput::Disagreements(findings)) => Err(refuse(
                    step,
                    "import_mo2.import",
                    format!(
                        "the MO2 profile and the base copy disagree ({} findings); nothing was imported",
                        findings.len()
                    ),
                )),
            }
        }
        Step::Profile => {
            let request = api::profile::CreateRequest {
                instance,
                name: profile,
            };
            let r = api::profile::create(&request, &mut detail(step, progress), cancel);
            Ok(finish(step, r)?.is_some())
        }
        Step::Engine => engine::run(req, step, progress, cancel),
        Step::Target => {
            let request = api::target::SetRequest {
                instance,
                target: api::target::Target::NvRs,
            };
            let r = api::target::set(&request, &mut detail(step, progress), cancel);
            Ok(finish(step, r)?.is_some())
        }
        Step::Ttw => run_ttw(req, step, progress, cancel),
        Step::Mod(archive) => {
            let request = api::install::InstallRequest {
                instance,
                profile,
                archive: archive.clone(),
                name: None,
                dry_run: false,
                answers: req.answers.clone(),
                max_extract_bytes: None,
            };
            let r = api::install::install(&request, &mut detail(step, progress), cancel);
            match finish(step, r)? {
                None => Ok(false),
                Some(api::install::InstallOutput::Installed(_)) => Ok(true),
                Some(api::install::InstallOutput::Questions(questions)) => Err(refuse(
                    step,
                    "install.install",
                    format!(
                        "the installer of {} has {} questions the answers file does not answer",
                        file_name(archive),
                        questions.len()
                    ),
                )),
            }
        }
        Step::Stage => {
            let request = api::stage::StageRequest {
                instance,
                profile,
                copy: req.copy,
            };
            let r = api::stage::stage(&request, &mut detail(step, progress), cancel);
            Ok(finish(step, r)?.is_some())
        }
        Step::Sort => {
            let request = api::loadorder::SortRequest {
                instance,
                profile,
                masterlist: req.masterlist.clone(),
                apply: true,
            };
            let r = api::loadorder::sort(&request, &mut detail(step, progress), cancel);
            Ok(finish(step, r)?.is_some())
        }
        Step::Stamp => {
            let request = api::sort_apply::KeepOrderRequest { instance, profile };
            let r = api::sort_apply::keep_order(&request, &mut detail(step, progress), cancel);
            Ok(finish(step, r)?.is_some())
        }
        Step::Verify => {
            let request = api::verify::VerifyRequest {
                instance,
                profile,
                masterlist: Some(req.masterlist.clone()),
                catalog: req.catalog.clone(),
                proton: None,
                fail_on: api::verify::Severity::Error,
            };
            let r = api::verify::verify(&request, &mut detail(step, progress), cancel);
            let Some(verify) = finish(step, r)? else {
                return Ok(false);
            };
            let passed = verify.verified;
            let why = if verify.nv_rs.verified {
                "the Doctor reports problems at the error level or above".to_owned()
            } else {
                verify.nv_rs.message.clone()
            };
            out.verify = Some(verify);
            if passed {
                Ok(true)
            } else {
                Err(Error {
                    kind: ErrorKind::Unverified,
                    step: step.clone(),
                    job: "verify.verify",
                    summary: format!("the finished Instance does not pass the check: {why}"),
                })
            }
        }
    }
}

/// Sets the flag when it is dropped, so the watcher thread ends even when the install panics.
struct StopWatcher<'a>(&'a AtomicBool);

impl Drop for StopWatcher<'_> {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

/// Run `work` while a watcher thread turns a stop asked for on `cancel` into nvmod's process-wide stop request,
/// which is what the TTW driver polls. The request is cleared again afterwards when this watcher made it and none
/// stood before, so a
/// later job in the same process starts clean.
pub(super) fn with_cancel_bridge<T>(cancel: &CancelToken, work: impl FnOnce() -> T) -> T {
    let (done, bridged) = (AtomicBool::new(false), AtomicBool::new(false));
    // Declared first, so it drops last: the request is cleared even when `work` panics.
    let _clear = ClearBridged {
        already: nvmod_core::cancel::requested(),
        bridged: &bridged,
    };
    std::thread::scope(|scope| {
        let _stop = StopWatcher(&done);
        scope.spawn(|| {
            while !done.load(Ordering::Relaxed) {
                if cancel.is_cancelled() {
                    nvmod_core::cancel::request();
                    bridged.store(true, Ordering::Relaxed);
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        });
        work()
    })
}

/// Clears nvmod's process-wide stop request on drop when the watcher made it and none stood before.
struct ClearBridged<'a> {
    already: bool,
    bridged: &'a AtomicBool,
}

impl Drop for ClearBridged<'_> {
    fn drop(&mut self) {
        if self.bridged.load(Ordering::Relaxed) && !self.already {
            nvmod_core::cancel::reset();
        }
    }
}

/// The TTW step: raul's own in-process ttwright driver, never nvmod's TTW job.
fn run_ttw(
    req: &Request,
    step: &Step,
    progress: &mut dyn Progress,
    cancel: &CancelToken,
) -> Result<bool, Error> {
    let Some(choice) = &req.ttw else {
        return Ok(true);
    };
    let request = TtwRequest {
        instance: req.instance.clone(),
        profile: req.profile.clone(),
        fallout3: Some(choice.fallout3.clone()),
        newvegas: choice.newvegas.clone(),
        package: choice.package.clone(),
        dry_run: false,
        resume: choice.resume,
        timeout: choice.timeout.unwrap_or(ttw::DEFAULT_TIMEOUT),
    };
    // This setup's own cancel token stops the install: no process-wide request is set, so nothing else is stopped.
    let cancelled = || cancel.is_cancelled();
    let outcome = {
        ttw::install_with_stop(
            &request,
            raul_drivers::Stop::Run(&cancelled),
            &mut |event| {
                let (label, done, total) = match event {
                    TtwEvent::Phase { phase } => (format!("ttw.{phase}"), None, None),
                    TtwEvent::Progress {
                        phase, done, of, ..
                    } => (format!("ttw.{phase}"), Some(done), (of > 0).then_some(of)),
                    _ => return,
                };
                progress.report(&Event::Detail {
                    step: step.clone(),
                    label,
                    done,
                    total,
                });
            },
        )
    };
    let kind = match outcome.result.status {
        Status::Success => return Ok(true),
        Status::Cancelled => return Ok(false),
        Status::Failed | Status::Unverifiable => ErrorKind::Failed,
        Status::Refused | Status::Unsupported => ErrorKind::Refused,
    };
    Err(Error {
        kind,
        step: step.clone(),
        job: "ttw.install",
        summary: outcome
            .result
            .error
            .map_or_else(|| "TTW was not installed".to_owned(), |e| e.message),
    })
}
