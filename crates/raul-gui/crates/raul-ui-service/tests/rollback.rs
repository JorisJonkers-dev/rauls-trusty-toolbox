// SPDX-License-Identifier: GPL-3.0-only
//! The rollback as a direct job on a synthetic Instance: nvmod's job is called in process, under the Instance lock,
//! and no `raul` program runs at all (ADR-0016). Also the runner of direct jobs itself: progress, stop, panic and every
//! error kind. No game data is used.

#![cfg(unix)]

use nvmod_api::{ErrorKind, Event, JobError};
use raul_core::instance_lock::InstanceGuard;
use raul_ui_model::error::ExitClass;
use raul_ui_model::job::{JobEvent, JobId, Outcome as JobOutcome};
use raul_ui_service::exec::{Exec, Output};
use raul_ui_service::registry::Action;
use raul_ui_service::{
    CancelHandle, CliService, DirectJob, JobSpec, RollbackRequest, Service, ServiceError,
    run_direct,
};
mod common;
use common::{Instance, install_plain, s};
use std::path::Path;

/// A runner that must never be asked for anything: the rollback calls nvmod's job in process.
struct Never;

impl Exec for Never {
    fn run(&self, args: &[String]) -> Result<Output, String> {
        panic!("rollback must not run the command line: {args:?}");
    }
}

fn service() -> CliService<Never> {
    CliService::new(Never)
}

#[test]
fn a_rollback_refuses_when_there_is_nothing_to_undo() {
    let inst = Instance::new();
    let ServiceError::Failed(e) = service()
        .rollback(&RollbackRequest {
            instance: s(&inst.root()),
        })
        .unwrap_err()
    else {
        panic!("expected a refusal");
    };
    assert_eq!(e.exit, ExitClass::Refused);
    assert_eq!(e.what, "there is no install to roll back");
    assert_eq!(
        e.repro, None,
        "an in-process job has no command line to name"
    );
}

#[test]
fn a_rollback_undoes_the_last_install() {
    let inst = Instance::new();
    install_plain(&inst);
    let undone = service()
        .rollback(&RollbackRequest {
            instance: s(&inst.root()),
        })
        .unwrap();
    assert_eq!(undone.kind, "install");
    assert_eq!(undone.mod_name, "Plain Mod");
    assert_eq!(undone.profile, "Default");
    assert_ne!(undone.journal, "");
    assert!(!inst.root().join("mods/Plain Mod").exists());
}

fn spec() -> JobSpec {
    JobSpec::new(JobId(1), Action::Rollback, Vec::new()).writing()
}

/// Runs `job`, returning what it left and every event it sent.
fn run(job: DirectJob, cancel: &CancelHandle) -> (Output, Vec<JobEvent>) {
    let mut events = Vec::new();
    let out = run_direct(&spec(), job, &mut |e| events.push(e), cancel);
    (out, events)
}

fn finished(events: &[JobEvent]) -> (JobOutcome, raul_ui_model::error::ExitClass) {
    match events.last() {
        Some(JobEvent::Finished { outcome, exit, .. }) => (*outcome, *exit),
        other => panic!("a job ends with Finished, got {other:?}"),
    }
}

#[test]
fn a_direct_rollback_tells_its_events_and_leaves_the_json_the_page_reads() {
    let inst = Instance::new();
    install_plain(&inst);
    let (out, events) = run(
        raul_ui_service::direct::rollback(&inst.root()),
        &CancelHandle::new(),
    );
    assert_eq!(out.code, Some(0), "{out:?}");
    let report: raul_ui_service::dto::RollbackReport = serde_json::from_str(&out.stdout).unwrap();
    assert_eq!(report.kind, "install");
    assert!(matches!(
        events.first(),
        Some(JobEvent::Started { writes: true, .. })
    ));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, JobEvent::Progress { phase, .. } if phase == "rollback")),
        "nvmod's progress event reaches the table: {events:?}"
    );
    assert_eq!(
        finished(&events),
        (JobOutcome::Completed, raul_ui_model::error::ExitClass::Ok)
    );
}

#[test]
fn a_held_instance_lock_refuses_the_job_and_changes_nothing() {
    let inst = Instance::new();
    install_plain(&inst);
    let held = InstanceGuard::acquire(&inst.root(), "stage").unwrap();
    assert!(held.is_held());
    let (out, events) = run(
        raul_ui_service::direct::rollback(&inst.root()),
        &CancelHandle::new(),
    );
    assert_eq!(out.code, Some(2), "{out:?}");
    assert!(out.stderr.contains("the Instance is in use by"), "{out:?}");
    assert!(!out.stderr.contains("raul "), "no command line: {out:?}");
    assert_eq!(
        finished(&events),
        (
            JobOutcome::Refused,
            raul_ui_model::error::ExitClass::Refused
        )
    );
    assert!(
        inst.root().join("mods/Plain Mod").exists(),
        "nothing was undone"
    );
    drop(held);
    let (out, _) = run(
        raul_ui_service::direct::rollback(&inst.root()),
        &CancelHandle::new(),
    );
    assert_eq!(out.code, Some(0), "the lock was let go: {out:?}");
    assert!(
        !inst.root().join(".nvmod/lock").exists(),
        "the job lets go of the lock when it ends"
    );
}

#[test]
fn every_error_kind_maps_to_its_own_exit_class() {
    use raul_ui_model::error::ExitClass as C;
    let cases = [
        (ErrorKind::Refused, Some(2), C::Refused, JobOutcome::Refused),
        (
            ErrorKind::Corrupt,
            Some(3),
            C::VerificationFailed,
            JobOutcome::Refused,
        ),
        (
            ErrorKind::Failed,
            Some(4),
            C::StepFailedRolledBack,
            JobOutcome::RolledBack,
        ),
        (ErrorKind::Internal, None, C::Internal, JobOutcome::Failed),
    ];
    for (kind, code, class, outcome) in cases {
        let job = DirectJob::reading(move |_, _| -> Result<nvmod_api::Outcome<u8>, JobError> {
            Err(JobError::new(kind, "test.job", "it said\n  why"))
        });
        let (out, events) = run(job, &CancelHandle::new());
        assert_eq!(out.code, code, "{kind:?}");
        assert_eq!(out.stderr, "it said why\n", "one line, no command line");
        assert_eq!(finished(&events), (outcome, class), "{kind:?}");
    }
}

#[test]
fn a_panicking_job_ends_as_an_internal_error_and_lets_go_of_the_lock() {
    let inst = Instance::new();
    let job = DirectJob::writing(
        inst.root(),
        "rollback",
        |_, _| -> Result<nvmod_api::Outcome<u8>, JobError> { panic!("a bug in the job") },
    );
    let (out, events) = run(job, &CancelHandle::new());
    assert_eq!(out.code, None);
    assert!(out.stderr.contains("stopped unexpectedly"), "{out:?}");
    assert_eq!(
        finished(&events),
        (
            JobOutcome::Failed,
            raul_ui_model::error::ExitClass::Internal
        )
    );
    assert!(!inst.root().join(".nvmod/lock").exists());
}

#[test]
fn a_stop_requested_at_a_progress_event_reaches_the_job_and_ends_it_cancelled() {
    let inst = Instance::new();
    let job = DirectJob::writing(inst.root(), "rollback", |progress, cancel| {
        progress.report(&Event::step("wait", "test.wait"));
        // A bounded wait that fails loudly: the stop must arrive through the job's token.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        while !cancel.is_cancelled() {
            if std::time::Instant::now() > deadline {
                return Err(JobError::new(
                    ErrorKind::Internal,
                    "test.wait",
                    "the stop never arrived",
                ));
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        Ok(nvmod_api::Outcome::<u8>::Cancelled)
    });
    let cancel = CancelHandle::new();
    let mut events = Vec::new();
    let out = run_direct(
        &spec(),
        job,
        &mut |e| {
            if matches!(e, JobEvent::Progress { .. }) {
                cancel.request();
            }
            events.push(e);
        },
        &cancel,
    );
    assert_eq!(out.code, None, "{out:?}");
    assert_eq!(
        finished(&events),
        (JobOutcome::Cancelled, raul_ui_model::error::ExitClass::Ok)
    );
    assert!(!inst.root().join(".nvmod/lock").exists());
}

#[test]
fn a_stop_before_the_start_runs_nothing() {
    let cancel = CancelHandle::new();
    cancel.request();
    let job = DirectJob::reading(|_, _| -> Result<nvmod_api::Outcome<u8>, JobError> {
        panic!("must not run")
    });
    let (out, events) = run(job, &cancel);
    assert_eq!(out.code, None);
    assert_eq!(
        finished(&events),
        (JobOutcome::Cancelled, raul_ui_model::error::ExitClass::Ok)
    );
}

#[test]
fn the_service_refuses_while_the_lock_is_held_with_the_same_words() {
    let inst = Instance::new();
    let _held = InstanceGuard::acquire(&inst.root(), "stage").unwrap();
    let ServiceError::Failed(e) = service()
        .rollback(&RollbackRequest {
            instance: s(&inst.root()),
        })
        .unwrap_err()
    else {
        panic!("expected a refusal");
    };
    assert_eq!(e.exit, raul_ui_model::error::ExitClass::Refused);
    assert!(e.what.contains("the Instance is in use by"), "{e:?}");
}

#[test]
fn an_undo_without_an_instance_is_refused_not_run_on_the_working_folder() {
    let job = raul_ui_service::direct::for_command(Action::Rollback, &[]).expect("converted");
    let (out, _) = run(job, &CancelHandle::new());
    assert_eq!(out.code, Some(2), "{out:?}");
    assert!(
        raul_ui_service::direct::for_command(Action::Doctor, &[]).is_none(),
        "an action not converted yet stays a command line"
    );
}

#[test]
fn the_lock_is_held_while_the_job_runs() {
    let inst = Instance::new();
    let lock = inst.root().join(".nvmod/lock");
    let seen = lock.clone();
    let job = DirectJob::writing(inst.root(), "rollback", move |_, _| {
        // While the job runs the file names this process, and a second writer is refused.
        let text = std::fs::read_to_string(&seen).map_err(|why| {
            JobError::new(
                ErrorKind::Internal,
                "test.lock",
                format!("no lock file: {why}"),
            )
        })?;
        assert!(text.contains(&std::process::id().to_string()), "{text}");
        Ok(nvmod_api::Outcome::Done(1_u8))
    });
    let (out, _) = run(job, &CancelHandle::new());
    assert_eq!(out.code, Some(0), "{out:?}");
    assert!(!lock.exists());
}
