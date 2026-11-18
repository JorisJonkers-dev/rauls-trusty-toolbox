// SPDX-License-Identifier: GPL-3.0-only
//! The read-only actions as direct jobs, checked against the command line they replace: on one synthetic Instance
//! the direct job prints the same document as `raul <command> --json` run in process, and the service answers from
//! it without asking the runner for anything. No game data is used.

#![cfg(unix)]

mod common;

use common::{Instance, install_plain, s};
use raul_ui_service::exec::{Exec, Output};
use raul_ui_service::registry::Action;
use raul_ui_service::{
    CancelHandle, CliService, ConflictsRequest, DoctorRequest, InProcessExec, JobSpec,
    PluginsRequest, Service, TreeRequest, VerifyRequest, direct, run_direct,
};

/// A runner that must never be asked for anything.
struct Never;

impl Exec for Never {
    fn run(&self, args: &[String]) -> Result<Output, String> {
        panic!("a converted action must not run the command line: {args:?}");
    }
}

fn service() -> CliService<Never> {
    CliService::new(Never)
}

/// What the direct job for `action` prints, from the words a page passes after the command.
fn direct_output(id: &str, rest: &[&str]) -> Output {
    let rest: Vec<String> = rest.iter().map(|w| (*w).to_owned()).collect();
    let job = direct::for_command(id, &rest).expect("the command is converted");
    let spec = JobSpec::new(raul_ui_model::job::JobId(1), Action::Doctor, Vec::new());
    run_direct(&spec, job, &mut |_| {}, &CancelHandle::new())
}

/// What the command line prints for the same words, run in process.
fn cli_output(args: &[&str]) -> Output {
    let args: Vec<String> = args.iter().map(|w| (*w).to_owned()).collect();
    InProcessExec::new().run(&args).unwrap()
}

/// Both ended alike and printed the same JSON document.
fn assert_same(direct: &Output, cli: &Output) {
    assert_eq!(direct.code, cli.code, "{direct:?} / {cli:?}");
    let a: serde_json::Value = serde_json::from_str(&direct.stdout).unwrap();
    let b: serde_json::Value = serde_json::from_str(&cli.stdout).unwrap();
    assert_eq!(a, b);
}

#[test]
fn conflicts_prints_what_the_command_line_prints() {
    let inst = Instance::new();
    install_plain(&inst);
    let root = s(&inst.root());
    assert_same(
        &direct_output("conflicts", &["--instance", &root]),
        &cli_output(&["conflicts", "--instance", &root, "--json"]),
    );
    let report = service()
        .conflicts(&ConflictsRequest {
            instance: root,
            ..ConflictsRequest::default()
        })
        .unwrap();
    assert_ne!(report.mods.len(), 0);
}

#[test]
fn doctor_prints_what_the_command_line_prints() {
    let inst = Instance::new();
    install_plain(&inst);
    let root = s(&inst.root());
    assert_same(
        &direct_output("doctor", &["--instance", &root]),
        &cli_output(&["doctor", "--instance", &root, "--json"]),
    );
    service()
        .doctor(&DoctorRequest {
            instance: root,
            ..DoctorRequest::default()
        })
        .unwrap();
}

#[test]
fn verify_prints_what_the_command_line_prints() {
    let inst = Instance::new();
    install_plain(&inst);
    let root = s(&inst.root());
    assert_same(
        &direct_output("verify", &["--instance", &root]),
        &cli_output(&["verify", "--instance", &root, "--json"]),
    );
    // A verification that does not pass is still a report, not an error.
    service()
        .verify(&VerifyRequest {
            instance: root,
            ..VerifyRequest::default()
        })
        .unwrap();
}

#[test]
fn plugins_list_prints_what_the_command_line_prints() {
    let inst = Instance::new();
    install_plain(&inst);
    let root = s(&inst.root());
    assert_same(
        &direct_output("plugins-list", &["--instance", &root]),
        &cli_output(&["plugins", "list", "--instance", &root, "--json"]),
    );
    service()
        .plugins_list(&PluginsRequest {
            instance: root,
            ..PluginsRequest::default()
        })
        .unwrap();
}

#[test]
fn tree_prints_what_the_command_line_prints() {
    let inst = Instance::new();
    install_plain(&inst);
    let root = s(&inst.root());
    assert_same(
        &direct_output("tree", &["--instance", &root]),
        &cli_output(&["tree", "--instance", &root, "--json"]),
    );
    let tree = service()
        .tree(&TreeRequest {
            instance: root,
            ..TreeRequest::default()
        })
        .unwrap();
    assert_ne!(tree.entries.len(), 0);
}

#[test]
fn a_refusal_is_one_line_with_no_command_line() {
    let inst = Instance::new();
    let root = s(&inst.root());
    let out = direct_output("conflicts", &["--instance", &root, "--mod", "No Such Mod"]);
    assert_eq!(out.code, Some(2), "{out:?}");
    assert!(!out.stderr.contains("raul "), "{out:?}");
}

#[test]
fn flags_a_page_passes_reach_the_job_as_they_reach_the_command_line() {
    let inst = Instance::new();
    install_plain(&inst);
    let root = s(&inst.root());
    for fail_on in ["error", "warning", "info"] {
        assert_same(
            &direct_output("doctor", &["--instance", &root, "--fail-on", fail_on]),
            &cli_output(&[
                "doctor",
                "--instance",
                &root,
                "--fail-on",
                fail_on,
                "--json",
            ]),
        );
        assert_same(
            &direct_output("verify", &["--instance", &root, "--fail-on", fail_on]),
            &cli_output(&[
                "verify",
                "--instance",
                &root,
                "--fail-on",
                fail_on,
                "--json",
            ]),
        );
    }
    assert_same(
        &direct_output(
            "plugins-list",
            &["--instance", &root, "--profile", "Default"],
        ),
        &cli_output(&[
            "plugins",
            "list",
            "--instance",
            &root,
            "--profile",
            "Default",
            "--json",
        ]),
    );
    assert_same(
        &direct_output(
            "conflicts",
            &["--instance", &root, "--profile", "Default", "--archives"],
        ),
        &cli_output(&[
            "conflicts",
            "--instance",
            &root,
            "--profile",
            "Default",
            "--archives",
            "--json",
        ]),
    );
    assert_same(
        &direct_output("tree", &["--instance", &root, "--path", "meshes"]),
        &cli_output(&["tree", "--instance", &root, "--path", "meshes", "--json"]),
    );
}

#[test]
fn an_unknown_severity_is_a_usage_error_like_on_the_command_line() {
    let inst = Instance::new();
    let root = s(&inst.root());
    for id in ["doctor", "verify"] {
        let out = direct_output(id, &["--instance", &root, "--fail-on", "nonsense"]);
        assert_eq!(out.code, Some(5), "{id}: {out:?}");
        assert!(out.stderr.contains("--fail-on"), "{out:?}");
    }
}

#[test]
fn a_failure_of_a_read_has_no_repro_command_line() {
    let inst = Instance::new();
    let none = s(&inst.work.path().join("none"));
    let e = service()
        .plugins_list(&PluginsRequest {
            instance: none,
            ..PluginsRequest::default()
        })
        .unwrap_err()
        .view_error();
    assert_eq!(e.repro, None);
}
