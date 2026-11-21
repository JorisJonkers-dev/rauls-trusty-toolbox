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
fn mods_list_prints_what_the_command_line_prints() {
    let inst = Instance::new();
    install_plain(&inst);
    let root = s(&inst.root());
    assert_same(
        &direct_output("mods-list", &["--instance", &root]),
        &cli_output(&["mods", "list", "--instance", &root, "--json"]),
    );
    assert_same(
        &direct_output(
            "mods-list",
            &[
                "--instance",
                &root,
                "--profile",
                "Default",
                "--category",
                "1",
            ],
        ),
        &cli_output(&[
            "mods",
            "list",
            "--instance",
            &root,
            "--profile",
            "Default",
            "--category",
            "1",
            "--json",
        ]),
    );
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

#[test]
fn version_prints_what_the_command_line_prints() {
    assert_same(
        &direct_output("version", &[]),
        &cli_output(&["version", "--json"]),
    );
}

#[test]
fn detect_in_a_folder_prints_what_the_command_line_prints_and_ends_alike_when_nothing_is_found() {
    // An Instance folder holds no game: the document is an empty list and the code is the command's "nothing found".
    let inst = Instance::new();
    let dir = s(&inst.root());
    let direct = direct_output("detect", &["--folder", &dir]);
    let cli = cli_output(&["detect", "--folder", &dir, "--json"]);
    assert_same(&direct, &cli);
    assert_eq!(direct.stdout.trim(), "[]");
    assert_eq!(direct.code, Some(i64::from(raul_cli::FOUND)));
}

/// What the direct job for a `config` action prints with `ctx`'s folder, and what the command prints for the same.
fn config_pair(
    ctx: &raul_core::context::Context,
    action: &str,
    words: &[&str],
) -> (Output, Output) {
    let rest: Vec<String> = words.iter().map(|w| (*w).to_owned()).collect();
    let job = direct::for_command_in(&format!("config-{action}"), &rest, ctx)
        .expect("the command is converted");
    let spec = JobSpec::new(raul_ui_model::job::JobId(1), Action::Doctor, Vec::new());
    let direct = run_direct(&spec, job, &mut |_| {}, &CancelHandle::new());
    let mut args = vec!["config".to_owned(), action.to_owned()];
    args.extend(rest);
    args.push("--json".to_owned());
    let cli = InProcessExec::with_context(ctx.clone()).run(&args).unwrap();
    (direct, cli)
}

#[test]
fn config_actions_print_what_the_command_line_prints_and_write_the_same_file() {
    let inst = Instance::new();
    let folder = inst.root().join("cfg");
    std::fs::create_dir_all(&folder).unwrap();
    let ctx = raul_core::context::Context::new()
        .with_config_folder(&folder)
        .without_run_log();
    let game = s(&inst.root());
    for (action, words) in [
        ("list", vec![]),
        ("set", vec!["storage.root", game.as_str()]),
        ("get", vec!["storage.root"]),
        ("unset", vec!["storage.root"]),
        ("get", vec!["storage.root"]),
        // A key that does not exist is a refusal document with the command's own exit code.
        ("get", vec!["no.such.key"]),
    ] {
        let (direct, cli) = config_pair(&ctx, action, &words);
        assert_same(&direct, &cli);
    }
    // A set through the direct job is on disk for the command to read.
    let (direct, _) = config_pair(&ctx, "set", &["storage.root", &game]);
    assert_eq!(direct.code, Some(0));
    let (_, cli) = config_pair(&ctx, "get", &["storage.root"]);
    assert!(cli.stdout.contains(&game), "{}", cli.stdout);
}

#[test]
fn a_config_action_with_the_wrong_words_is_a_usage_error() {
    let ctx = raul_core::context::Context::new().without_run_log();
    let job = direct::for_command_in("config-set", &["storage.root".to_owned()], &ctx).unwrap();
    let spec = JobSpec::new(raul_ui_model::job::JobId(1), Action::Doctor, Vec::new());
    let out = run_direct(&spec, job, &mut |_| {}, &CancelHandle::new());
    assert_ne!(out.code, Some(0));
}

#[test]
fn lod_mods_and_lod_plan_print_what_the_command_line_prints() {
    let inst = Instance::new();
    install_plain(&inst);
    let root = s(&inst.root());
    assert_same(
        &direct_output("lod-mods", &["--instance", &root]),
        &cli_output(&["lod", "mods", "--instance", &root, "--json"]),
    );
    // A plan of an Instance with no plugin that carries a world is the command's refusal, word for word and with its
    // code, or the same request document.
    let direct = direct_output("lod-plan", &["--instance", &root]);
    let cli = cli_output(&["lod", "plan", "--instance", &root, "--json"]);
    assert_eq!(direct.code, cli.code, "{direct:?} / {cli:?}");
    if cli.code == Some(0) {
        assert_same(&direct, &cli);
    } else {
        assert!(
            direct.stderr.contains("refused"),
            "the refusal is shown: {direct:?}"
        );
    }
}

#[test]
fn lod_settings_actions_print_what_the_command_line_prints() {
    let inst = Instance::new();
    install_plain(&inst);
    let root = s(&inst.root());
    for (verb, extra) in [
        ("show", vec![]),
        ("reset", vec!["--dry-run"]),
        ("unset", vec!["no.such.option"]),
    ] {
        let mut rest = vec!["--instance", root.as_str()];
        rest.extend(extra.iter().copied());
        let id = format!("lod-settings-{verb}");
        let mut cli = vec!["lod", "settings", verb];
        cli.extend(rest.iter().copied());
        cli.push("--json");
        assert_same(&direct_output(&id, &rest), &cli_output(&cli));
    }
}
