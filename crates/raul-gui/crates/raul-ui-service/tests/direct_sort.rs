// SPDX-License-Identifier: GPL-3.0-only
//! The Load order sort as a direct job on a synthetic Instance: the proposal and the applied order are the documents
//! `raul sort --json` prints, the service answers from nvmod's job with no command line, and an apply holds the
//! Instance lock and takes the snapshot the command line takes. No game data is used.

#![cfg(unix)]

mod common;

use common::{Instance, s};
use nvmod_testkit::write_file;
use raul_core::instance_lock::InstanceGuard;
use raul_ui_model::error::ExitClass;
use raul_ui_model::job::JobId;
use raul_ui_service::exec::{Exec, Output};
use raul_ui_service::registry::Action;
use raul_ui_service::{
    CancelHandle, CliService, InProcessExec, JobSpec, RollbackRequest, Service, ServiceError,
    SortRequest, direct, run_direct,
};

/// A runner that must never be asked for anything: the sort calls nvmod's job in process.
struct Never;

impl Exec for Never {
    fn run(&self, args: &[String]) -> Result<Output, String> {
        panic!("a converted action must not run the command line: {args:?}");
    }
}

fn service() -> CliService<Never> {
    CliService::new(Never)
}

const MASTERLIST: &str =
    "globals:\n  - {type: say, content: 'Masterlist for tests'}\nplugins: []\n";

/// An Instance with its Staging tree built and a masterlist file.
fn staged() -> (Instance, String) {
    let inst = Instance::new();
    let masterlist = inst.work.path().join("masterlist.yaml");
    std::fs::write(&masterlist, MASTERLIST).unwrap();
    let args: Vec<String> = [
        "stage",
        "--instance",
        &s(&inst.root()),
        "--profile",
        "Default",
    ]
    .iter()
    .map(|w| (*w).to_owned())
    .collect();
    let out = InProcessExec::new().run(&args).unwrap();
    assert_eq!(out.code, Some(0), "{}", out.stderr);
    (inst, s(&masterlist))
}

fn direct_output(id: &str, rest: &[&str]) -> Output {
    let rest: Vec<String> = rest.iter().map(|w| (*w).to_owned()).collect();
    let job = direct::for_command(id, &rest).expect("the command is converted");
    let spec = JobSpec::new(JobId(1), Action::Sort, Vec::new());
    run_direct(&spec, job, &mut |_| {}, &CancelHandle::new())
}

fn cli_output(args: &[&str]) -> Output {
    let args: Vec<String> = args.iter().map(|w| (*w).to_owned()).collect();
    InProcessExec::new().run(&args).unwrap()
}

#[test]
fn a_sort_preview_prints_what_the_command_line_prints() {
    let (inst, masterlist) = staged();
    let root = s(&inst.root());
    let direct = direct_output(
        "sort-preview",
        &["--instance", &root, "--masterlist", &masterlist],
    );
    let cli = cli_output(&[
        "sort",
        "--instance",
        &root,
        "--masterlist",
        &masterlist,
        "--json",
    ]);
    assert_eq!(direct.code, Some(0), "{direct:?}");
    assert_eq!(cli.code, Some(0), "{cli:?}");
    let a: serde_json::Value = serde_json::from_str(&direct.stdout).unwrap();
    let b: serde_json::Value = serde_json::from_str(&cli.stdout).unwrap();
    assert_eq!(a, b);
    assert!(a["applied"].is_null(), "a preview writes nothing");
}

#[test]
fn an_apply_writes_the_order_and_prints_what_the_command_line_prints() {
    let (direct_inst, direct_list) = staged();
    let (cli_inst, cli_list) = staged();
    let direct = direct_output(
        "sort-apply",
        &[
            "--instance",
            &s(&direct_inst.root()),
            "--masterlist",
            &direct_list,
            "--apply",
        ],
    );
    let cli = cli_output(&[
        "sort",
        "--instance",
        &s(&cli_inst.root()),
        "--masterlist",
        &cli_list,
        "--apply",
        "--json",
    ]);
    assert_eq!(direct.code, Some(0), "{direct:?}");
    assert_eq!(cli.code, Some(0), "{cli:?}");
    let mut a: serde_json::Value = serde_json::from_str(&direct.stdout).unwrap();
    let mut b: serde_json::Value = serde_json::from_str(&cli.stdout).unwrap();
    // The journal is named after the Instance it is in.
    a["applied"]["journal"] = serde_json::Value::Null;
    b["applied"]["journal"] = serde_json::Value::Null;
    assert_eq!(a, b);
}

#[test]
fn the_service_sorts_and_applies_with_no_command_line_and_the_apply_can_be_undone() {
    let (inst, masterlist) = staged();
    let mut request = SortRequest {
        instance: s(&inst.root()),
        masterlist,
        ..SortRequest::default()
    };
    let proposal = service().sort(&request).unwrap();
    assert_eq!(proposal.profile, "Default");
    assert!(proposal.applied.is_none());
    assert_eq!(proposal.order[0].name, "FalloutNV.esm");

    request.apply = true;
    let applied = service().sort(&request).unwrap();
    assert!(applied.applied.is_some());
    service()
        .rollback(&RollbackRequest {
            instance: s(&inst.root()),
        })
        .unwrap();
}

#[test]
fn a_missing_masterlist_is_a_refusal_and_a_held_lock_refuses_an_apply() {
    let (inst, masterlist) = staged();
    let mut request = SortRequest {
        instance: s(&inst.root()),
        masterlist: s(&inst.work.path().join("missing.yaml")),
        ..SortRequest::default()
    };
    let ServiceError::Failed(e) = service().sort(&request).unwrap_err() else {
        panic!("expected a refusal");
    };
    assert_eq!(e.exit, ExitClass::Refused);

    request.masterlist = masterlist;
    request.apply = true;
    let _held = InstanceGuard::acquire(&inst.root(), "test").unwrap();
    let ServiceError::Failed(e) = service().sort(&request).unwrap_err() else {
        panic!("expected a refusal");
    };
    assert_eq!(e.exit, ExitClass::Refused);
    assert!(e.what.contains("the Instance is in use by"), "{e:?}");
}

#[test]
fn a_sort_without_an_instance_or_a_masterlist_source_is_a_usage_error() {
    assert_eq!(direct_output("sort-preview", &[]).code, Some(5));
    let (inst, _) = staged();
    let out = direct_output("sort-preview", &["--instance", &s(&inst.root())]);
    assert_eq!(out.code, Some(5), "{out:?}");
}

#[test]
fn a_fetch_from_a_lock_that_pins_no_masterlist_is_refused_before_anything_is_fetched() {
    let (inst, _) = staged();
    let lock = inst.work.path().join("tools.lock.toml");
    write_file(inst.work.path(), "tools.lock.toml", b"schema_version = 1\n");
    let out = direct_output(
        "sort-preview",
        &[
            "--instance",
            &s(&inst.root()),
            "--fetch-masterlist",
            "--lock",
            &s(&lock),
        ],
    );
    assert_eq!(out.code, Some(2), "{out:?}");
    assert!(out.stderr.contains("pins no tool"), "{}", out.stderr);
}
