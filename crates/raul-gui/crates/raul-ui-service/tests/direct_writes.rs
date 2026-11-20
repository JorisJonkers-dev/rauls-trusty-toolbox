// SPDX-License-Identifier: GPL-3.0-only
//! The Mods change as a direct job on a synthetic Instance: a dry run prints the document the command line prints,
//! a change writes `modlist.txt` in process, and a held Instance lock refuses it. No game data is used.

#![cfg(unix)]

mod common;

use common::{Instance, install_plain, s};
use nvmod_api::mods::{Change, ChangeRequest};
use nvmod_api::target::SetRequest;
use raul_core::instance_lock::InstanceGuard;
use raul_ui_model::error::ExitClass;
use raul_ui_model::job::JobId;
use raul_ui_model::mods::ModStep;
use raul_ui_service::dto::ModsReport;
use raul_ui_service::exec::{Exec, Output};
use raul_ui_service::registry::Action;
use raul_ui_service::{
    CancelHandle, CliService, InProcessExec, JobSpec, ModsRequest, ProfileRequest, RuntimeTarget,
    Service, ServiceError, TargetRequest, direct, run_direct,
};
use std::path::PathBuf;

/// A runner that must never be asked for anything: the change calls nvmod's job in process.
struct Never;

impl Exec for Never {
    fn run(&self, args: &[String]) -> Result<Output, String> {
        panic!("a converted action must not run the command line: {args:?}");
    }
}

fn service() -> CliService<Never> {
    CliService::new(Never)
}

fn modlist(inst: &Instance) -> PathBuf {
    inst.root().join("profiles/Default/modlist.txt")
}

/// Plans `step` as a dry run through the service, and the same change as the direct job, and checks both against
/// `raul mods <verb> "Plain Mod" --dry-run --json` run in process.
fn assert_dry_run_matches_command_line(verb: &str, step: ModStep, change: Change, root: &str) {
    let answer = service()
        .mods_change(&ModsRequest {
            instance: root.to_owned(),
            profile: None,
            step,
            dry_run: true,
        })
        .unwrap();
    assert!(answer.dry_run);
    assert_eq!(answer.journal, None, "a dry run writes no journal");

    let words: Vec<String> = [
        "mods",
        verb,
        "Plain Mod",
        "--instance",
        root,
        "--dry-run",
        "--json",
    ]
    .iter()
    .map(|w| (*w).to_owned())
    .collect();
    let cli = InProcessExec::new().run(&words).unwrap();
    assert_eq!(cli.code, Some(0), "{cli:?}");
    let cli_report: ModsReport = serde_json::from_str(&cli.stdout).unwrap();
    assert_eq!(answer, cli_report);

    let request = ChangeRequest {
        instance: PathBuf::from(root),
        profile: None,
        change,
        dry_run: true,
    };
    let spec = JobSpec::new(JobId(1), Action::Mods, Vec::new());
    let out = run_direct(
        &spec,
        direct::mods_change(&request),
        &mut |_| {},
        &CancelHandle::new(),
    );
    assert_eq!(out.code, Some(0), "{out:?}");
    let direct_doc: serde_json::Value = serde_json::from_str(&out.stdout).unwrap();
    let cli_doc: serde_json::Value = serde_json::from_str(&cli.stdout).unwrap();
    assert_eq!(direct_doc, cli_doc);
}

#[test]
fn mods_change_prints_what_the_command_line_prints() {
    let inst = Instance::new();
    install_plain(&inst);
    let root = s(&inst.root());
    let before = std::fs::read_to_string(modlist(&inst)).unwrap();
    assert_dry_run_matches_command_line(
        "disable",
        ModStep::Disable(vec!["Plain Mod".into()]),
        Change::Disable {
            mods: vec!["Plain Mod".into()],
        },
        &root,
    );
    assert_dry_run_matches_command_line(
        "enable",
        ModStep::Enable(vec!["Plain Mod".into()]),
        Change::Enable {
            mods: vec!["Plain Mod".into()],
        },
        &root,
    );
    assert_eq!(
        std::fs::read_to_string(modlist(&inst)).unwrap(),
        before,
        "a dry run writes nothing"
    );
}

#[test]
fn a_disable_through_the_service_changes_the_modlist_with_no_command_line() {
    let inst = Instance::new();
    install_plain(&inst);
    let before = std::fs::read_to_string(modlist(&inst)).unwrap();
    let done = service()
        .mods_change(&ModsRequest {
            instance: s(&inst.root()),
            profile: None,
            step: ModStep::Disable(vec!["Plain Mod".into()]),
            dry_run: false,
        })
        .unwrap();
    assert!(!done.dry_run);
    assert_eq!(done.status, "changed");
    assert!(done.journal.is_some(), "a change leaves a journal to undo");
    let after = std::fs::read_to_string(modlist(&inst)).unwrap();
    assert_ne!(before, after, "the change reached modlist.txt");
    assert!(after.contains("-Plain Mod"), "{after}");
}

#[test]
fn a_held_instance_lock_refuses_a_change_and_changes_nothing() {
    let inst = Instance::new();
    install_plain(&inst);
    let before = std::fs::read_to_string(modlist(&inst)).unwrap();
    let held = InstanceGuard::acquire(&inst.root(), "stage").unwrap();
    let ServiceError::Failed(e) = service()
        .mods_change(&ModsRequest {
            instance: s(&inst.root()),
            profile: None,
            step: ModStep::Disable(vec!["Plain Mod".into()]),
            dry_run: false,
        })
        .unwrap_err()
    else {
        panic!("expected a refusal");
    };
    assert_eq!(e.exit, ExitClass::Refused, "{e:?}");
    assert!(e.what.contains("the Instance is in use by"), "{e:?}");
    assert_eq!(std::fs::read_to_string(modlist(&inst)).unwrap(), before);
    drop(held);
}

/// The Instance file without its `base` line: the base copy is a folder of its own in every test.
fn instance_record(inst: &Instance) -> serde_json::Value {
    let text = std::fs::read_to_string(inst.root().join("nvmod-instance.json")).unwrap();
    let mut record: serde_json::Value = serde_json::from_str(&text).unwrap();
    record.as_object_mut().unwrap().remove("base");
    record
}

#[test]
fn target_sets_what_the_command_line_sets() {
    let inst = Instance::new();
    let cli_inst = Instance::new();
    assert_eq!(instance_record(&inst)["target"], "vanilla");

    let done = service()
        .set_target(&TargetRequest {
            instance: s(&inst.root()),
            target: RuntimeTarget::NvRs,
        })
        .unwrap();
    assert_eq!(done.target, RuntimeTarget::NvRs);
    assert_eq!(instance_record(&inst)["target"], "nv-rs");

    let cli_root = s(&cli_inst.root());
    let words: Vec<String> = ["target", "nv-rs", "--instance", cli_root.as_str()]
        .iter()
        .map(|w| (*w).to_owned())
        .collect();
    let cli = InProcessExec::new().run(&words).unwrap();
    assert_eq!(cli.code, Some(0), "{cli:?}");
    assert_eq!(instance_record(&inst), instance_record(&cli_inst));
}

#[test]
fn target_set_as_a_direct_job_writes_the_instance_file() {
    let inst = Instance::new();
    let request = SetRequest {
        instance: inst.root(),
        target: nvmod_api::target::Target::NvRs,
    };
    let spec = JobSpec::new(JobId(2), Action::Target, Vec::new());
    let out = run_direct(
        &spec,
        direct::target(&request),
        &mut |_| {},
        &CancelHandle::new(),
    );
    assert_eq!(out.code, Some(0), "{out:?}");
    assert_eq!(instance_record(&inst)["target"], "nv-rs");
}

#[test]
fn the_setup_wizards_target_command_is_the_direct_target_job() {
    // The words the wizard's last step passes to the registry command `target-nv-rs`: `--instance <dir>`.
    let words = |w: &[&str]| w.iter().map(|x| (*x).to_owned()).collect::<Vec<_>>();
    let inst = Instance::new();
    assert_eq!(instance_record(&inst)["target"], "vanilla");
    let job = direct::for_command("target-nv-rs", &words(&["--instance", &s(&inst.root())]))
        .expect("target is a direct job");
    let spec = JobSpec::new(JobId(5), Action::Target, Vec::new()).writing();
    let out = run_direct(&spec, job, &mut |_| {}, &CancelHandle::new());
    assert_eq!(out.code, Some(0), "{out:?}");
    assert_eq!(instance_record(&inst)["target"], "nv-rs");

    // No Instance is a usage error that writes nothing.
    let before = instance_record(&inst);
    let job = direct::for_command("target-nv-rs", &[]).unwrap();
    let out = run_direct(&spec, job, &mut |_| {}, &CancelHandle::new());
    assert_eq!(out.code, Some(5), "{out:?}");
    assert_eq!(instance_record(&inst), before);
}

#[test]
fn a_held_instance_lock_refuses_a_target_change_and_changes_nothing() {
    let inst = Instance::new();
    let before = instance_record(&inst);
    let held = InstanceGuard::acquire(&inst.root(), "stage").unwrap();
    let ServiceError::Failed(e) = service()
        .set_target(&TargetRequest {
            instance: s(&inst.root()),
            target: RuntimeTarget::NvRs,
        })
        .unwrap_err()
    else {
        panic!("expected a refusal");
    };
    assert_eq!(e.exit, ExitClass::Refused, "{e:?}");
    assert_eq!(instance_record(&inst), before);
    drop(held);
}

#[test]
fn profile_create_makes_the_profile_in_process_and_a_held_lock_refuses_it() {
    let inst = Instance::new();
    let root = s(&inst.root());
    let request = ProfileRequest {
        instance: root.clone(),
        name: "Second".to_owned(),
    };
    service().profile_create(&request).unwrap();
    assert!(inst.root().join("profiles/Second/modlist.txt").is_file());
    let held = InstanceGuard::acquire(&inst.root(), "test").unwrap();
    let again = service().profile_create(&request);
    drop(held);
    match again {
        Err(ServiceError::Failed(e)) => assert_eq!(e.exit, ExitClass::Refused, "{e:?}"),
        other => panic!("a held Instance lock refuses the job: {other:?}"),
    }
}

#[test]
fn a_move_of_a_mod_matches_the_command_line_and_an_unknown_mod_is_a_refusal() {
    let inst = Instance::new();
    install_plain(&inst);
    let root = s(&inst.root());
    let answer = service()
        .mods_change(&ModsRequest {
            instance: root.clone(),
            profile: None,
            step: ModStep::Move {
                name: "Plain Mod".to_owned(),
                position: 1,
            },
            dry_run: true,
        })
        .unwrap();
    let cli = InProcessExec::new()
        .run(&[
            "mods".to_owned(),
            "reorder".to_owned(),
            "Plain Mod".to_owned(),
            "--position".to_owned(),
            "1".to_owned(),
            "--instance".to_owned(),
            root.clone(),
            "--dry-run".to_owned(),
            "--json".to_owned(),
        ])
        .unwrap();
    assert_eq!(cli.code, Some(0), "{cli:?}");
    assert_eq!(
        answer,
        serde_json::from_str::<ModsReport>(&cli.stdout).unwrap()
    );

    let unknown = service().mods_change(&ModsRequest {
        instance: root,
        profile: None,
        step: ModStep::Disable(vec!["No Such Mod".to_owned()]),
        dry_run: true,
    });
    match unknown {
        Err(ServiceError::Failed(e)) => assert_eq!(e.exit, ExitClass::Refused, "{e:?}"),
        other => panic!("an unknown Mod is refused: {other:?}"),
    }
}

#[test]
fn an_instance_without_a_profile_gets_the_default_one_and_the_action_runs_again() {
    let inst = Instance::new();
    install_plain(&inst);
    std::fs::remove_dir_all(inst.root().join("profiles/Default")).unwrap();
    let report = service()
        .plugins_list(&raul_ui_service::PluginsRequest {
            instance: s(&inst.root()),
            ..raul_ui_service::PluginsRequest::default()
        })
        .unwrap();
    assert!(report.plugins.len() <= 1, "{report:?}");
    assert!(inst.root().join("profiles/Default/modlist.txt").is_file());
}

/// The words the Mods page passes for `id` (the registry command), after the subcommand.
fn page_words(name: &str, root: &str, position: Option<&str>) -> Vec<String> {
    let mut rest = vec![name.to_owned()];
    if let Some(position) = position {
        rest.extend(["--position".to_owned(), position.to_owned()]);
    }
    rest.extend(["--instance".to_owned(), root.to_owned()]);
    rest
}

#[test]
fn the_mods_page_commands_are_direct_jobs_that_write_what_the_command_line_writes() {
    for (id, verb, position) in [
        ("mods-disable", "disable", None),
        ("mods-enable", "enable", None),
        ("mods-reorder", "reorder", Some("1")),
    ] {
        let (direct_inst, cli_inst) = (Instance::new(), Instance::new());
        for inst in [&direct_inst, &cli_inst] {
            install_plain(inst);
        }
        // Start from a disabled Mod so every verb has something to do in a way the file shows.
        for inst in [&direct_inst, &cli_inst] {
            let root = s(&inst.root());
            let out = InProcessExec::new()
                .run(&[
                    "mods".into(),
                    "disable".into(),
                    "Plain Mod".into(),
                    "--instance".into(),
                    root,
                ])
                .unwrap();
            assert_eq!(out.code, Some(0), "{out:?}");
        }
        let words = page_words("Plain Mod", &s(&direct_inst.root()), position);
        let job = direct::for_command(id, &words).unwrap_or_else(|| panic!("{id} is not direct"));
        let spec = JobSpec::new(JobId(3), Action::Mods, Vec::new()).writing();
        let out = run_direct(&spec, job, &mut |_| {}, &CancelHandle::new());
        assert_eq!(out.code, Some(0), "{id}: {out:?}");

        let mut cli_words = vec!["mods".to_owned(), verb.to_owned()];
        cli_words.extend(page_words("Plain Mod", &s(&cli_inst.root()), position));
        cli_words.push("--json".to_owned());
        let cli = InProcessExec::new().run(&cli_words).unwrap();
        assert_eq!(cli.code, Some(0), "{id}: {cli:?}");
        assert_eq!(
            std::fs::read_to_string(modlist(&direct_inst)).unwrap(),
            std::fs::read_to_string(modlist(&cli_inst)).unwrap(),
            "{id}: the direct job leaves the modlist the command leaves"
        );
    }
}

#[test]
fn a_mods_page_command_without_its_mod_or_position_ends_as_a_usage_error_and_a_remove_stays_a_command()
 {
    let words = |w: &[&str]| w.iter().map(|x| (*x).to_owned()).collect::<Vec<_>>();
    let spec = JobSpec::new(JobId(4), Action::Mods, Vec::new());
    for (id, rest) in [
        ("mods-enable", words(&["--instance", "/i"])),
        (
            "mods-reorder",
            words(&["Plain Mod", "--position", "x", "--instance", "/i"]),
        ),
    ] {
        let job = direct::for_command(id, &rest).unwrap();
        let out = run_direct(&spec, job, &mut |_| {}, &CancelHandle::new());
        assert_eq!(out.code, Some(5), "{id}: {out:?}");
    }
    assert!(
        direct::for_command("mods-remove", &words(&["Plain Mod", "--instance", "/i"])).is_none()
    );
}
