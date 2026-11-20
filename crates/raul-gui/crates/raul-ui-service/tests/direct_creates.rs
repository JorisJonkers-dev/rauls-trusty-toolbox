// SPDX-License-Identifier: GPL-3.0-only
//! The init and target commands as direct jobs on synthetic folders: a made Instance is the same whether the
//! job or the command line makes it, a missing base copy is one line of refusal, and a held Instance lock refuses
//! the nv-rs target. Every folder is a `TempDir` child; no game data is used.

#![cfg(unix)]

mod common;

use common::{Instance, install_plain, s};
use nvmod_testkit::{BaseBuilder, TempDir};
use raul_core::instance_lock::InstanceGuard;
use raul_ui_model::job::JobId;
use raul_ui_service::exec::Exec;
use raul_ui_service::registry::Action;
use raul_ui_service::{CancelHandle, InProcessExec, JobSpec, direct, run_direct};
use std::path::{Path, PathBuf};

/// A base copy of the game made by the testkit, in a folder of its own.
fn base_copy(dir: &TempDir) -> PathBuf {
    BaseBuilder::steam().write(dir.path());
    dir.path().to_owned()
}

/// Every file under `root`, as sorted paths relative to it.
fn file_list(root: &Path) -> Vec<String> {
    fn walk(base: &Path, dir: &Path, out: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(base, &path, out);
            } else {
                out.push(s(path.strip_prefix(base).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

#[test]
fn init_makes_the_same_instance_as_the_command_line() {
    let base_dir = TempDir::new("rc-init-base");
    let base = s(&base_copy(&base_dir));
    let direct_dir = TempDir::new("rc-init-direct");
    let cli_dir = TempDir::new("rc-init-cli");
    let inst_a = s(&direct_dir.path().join("i"));
    let inst_b = s(&cli_dir.path().join("i"));

    let rest: Vec<String> = ["--base", base.as_str(), "--instance", inst_a.as_str()]
        .iter()
        .map(|w| (*w).to_owned())
        .collect();
    let job = direct::for_command("init", &rest).expect("init is a direct job");
    let spec = JobSpec::new(JobId(1), Action::Init, Vec::new());
    let out = run_direct(&spec, job, &mut |_| {}, &CancelHandle::new());
    assert_eq!(out.code, Some(0), "{out:?}");

    let words: Vec<String> = [
        "init",
        "--base",
        base.as_str(),
        "--instance",
        inst_b.as_str(),
    ]
    .iter()
    .map(|w| (*w).to_owned())
    .collect();
    let cli = InProcessExec::new().run(&words).unwrap();
    assert_eq!(cli.code, Some(0), "{cli:?}");

    let direct_root = Path::new(&inst_a);
    let cli_root = Path::new(&inst_b);
    assert!(direct_root.join("nvmod-instance.json").is_file());
    assert!(cli_root.join("nvmod-instance.json").is_file());
    assert_eq!(file_list(direct_root), file_list(cli_root));
}

#[test]
fn init_refuses_a_missing_base_with_one_line_and_no_command_line() {
    let dir = TempDir::new("rc-init-missing");
    let base = s(&dir.path().join("no-such-base"));
    let inst = s(&dir.path().join("i"));

    let rest: Vec<String> = ["--base", base.as_str(), "--instance", inst.as_str()]
        .iter()
        .map(|w| (*w).to_owned())
        .collect();
    let job = direct::for_command("init", &rest).expect("init is a direct job");
    let spec = JobSpec::new(JobId(2), Action::Init, Vec::new());
    let out = run_direct(&spec, job, &mut |_| {}, &CancelHandle::new());
    assert_eq!(out.code, Some(2), "{out:?}");
    let lines: Vec<&str> = out.stderr.lines().collect();
    assert_eq!(lines.len(), 1, "one line of refusal: {:?}", out.stderr);
    assert!(!lines[0].is_empty(), "{:?}", out.stderr);
    assert!(!out.stderr.contains("raul "), "{:?}", out.stderr);
}

#[test]
fn the_nv_rs_target_writes_the_engines_capabilities_as_the_command_line_does() {
    raul_core::capabilities::provide(r#"{"schema_version":1,"extenders":[]}"#);
    let (direct_side, cli_side) = (Instance::new(), Instance::new());
    install_plain(&direct_side);
    install_plain(&cli_side);
    let (root, cli_root) = (s(&direct_side.root()), s(&cli_side.root()));

    let rest: Vec<String> = ["--instance", root.as_str()]
        .iter()
        .map(|w| (*w).to_owned())
        .collect();
    let job = direct::for_command("target-nv-rs", &rest).expect("target-nv-rs is a direct job");
    let spec = JobSpec::new(JobId(3), Action::Target, Vec::new());
    let out = run_direct(&spec, job, &mut |_| {}, &CancelHandle::new());

    let words: Vec<String> = ["target", "nv-rs", "--instance", cli_root.as_str()]
        .iter()
        .map(|w| (*w).to_owned())
        .collect();
    let cli = InProcessExec::new().run(&words).unwrap();
    assert_eq!(out.code, Some(0), "{out:?}");
    assert_eq!(out.code, cli.code, "direct {out:?} / command line {cli:?}");
    for side in [&direct_side, &cli_side] {
        let file = side.root().join("nv-rs/capabilities.json");
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            r#"{"schema_version":1,"extenders":[]}"#,
            "{file:?}"
        );
    }
}

#[test]
fn a_held_instance_lock_refuses_the_nv_rs_target_and_writes_nothing() {
    let inst = Instance::new();
    install_plain(&inst);
    let root = s(&inst.root());
    let held = InstanceGuard::acquire(&inst.root(), "test").unwrap();

    let rest: Vec<String> = ["--instance", root.as_str()]
        .iter()
        .map(|w| (*w).to_owned())
        .collect();
    let job = direct::for_command("target-nv-rs", &rest).expect("target-nv-rs is a direct job");
    let spec = JobSpec::new(JobId(4), Action::Target, Vec::new());
    let out = run_direct(&spec, job, &mut |_| {}, &CancelHandle::new());
    drop(held);

    assert_eq!(out.code, Some(2), "{out:?}");
    assert!(!out.stderr.is_empty(), "{out:?}");
    assert!(!inst.root().join("nv-rs").exists());
}
