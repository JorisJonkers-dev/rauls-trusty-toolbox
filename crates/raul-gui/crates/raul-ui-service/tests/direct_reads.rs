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
    CancelHandle, CliService, ConflictsRequest, InProcessExec, JobSpec, Service, TreeRequest,
    direct, run_direct,
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
fn direct_output(action: Action, rest: &[&str]) -> Output {
    let rest: Vec<String> = rest.iter().map(|w| (*w).to_owned()).collect();
    let job = direct::for_command(action, &rest).expect("the action is converted");
    let spec = JobSpec::new(raul_ui_model::job::JobId(1), action, Vec::new());
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
        &direct_output(Action::Conflicts, &["--instance", &root]),
        &cli_output(&["conflicts", "--instance", &root, "--json"]),
    );
    let report = service()
        .conflicts(&ConflictsRequest {
            instance: root,
            ..ConflictsRequest::default()
        })
        .unwrap();
    assert!(!report.mods.is_empty());
}

#[test]
fn tree_prints_what_the_command_line_prints() {
    let inst = Instance::new();
    install_plain(&inst);
    let root = s(&inst.root());
    assert_same(
        &direct_output(Action::Tree, &["--instance", &root]),
        &cli_output(&["tree", "--instance", &root, "--json"]),
    );
    let tree = service()
        .tree(&TreeRequest {
            instance: root,
            ..TreeRequest::default()
        })
        .unwrap();
    assert!(!tree.entries.is_empty());
}

#[test]
fn a_refusal_is_one_line_with_no_command_line() {
    let inst = Instance::new();
    let root = s(&inst.root());
    let out = direct_output(
        Action::Conflicts,
        &["--instance", &root, "--mod", "No Such Mod"],
    );
    assert_eq!(out.code, Some(2), "{out:?}");
    assert!(!out.stderr.contains("raul "), "{out:?}");
}
