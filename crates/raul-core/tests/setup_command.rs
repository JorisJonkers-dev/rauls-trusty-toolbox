// SPDX-License-Identifier: GPL-3.0-only
//! `setup::execute`: the event stream the setup wizard reads, a dry run, a failed run that rolls back or is kept, a
//! resume from the journal alone, a restart, and the takeover of an `nvmod setup` state. Synthetic fixtures only.

use std::fs;
use std::path::PathBuf;

use nvmod_testkit::{BaseBuilder, PeBuilder, PluginBuilder, TempDir, ZipBuilder, write_file};
use raul_core::journal::Journal;
use raul_core::setup::{CancelToken, Ended, Mode, Request, execute, parse_words};
use serde_json::{Value, json};

const MASTERLIST: &str = "globals:\n  - {type: say, content: 'Masterlist for tests'}\n";

struct Scratch {
    base: TempDir,
    work: TempDir,
}

impl Scratch {
    fn new() -> Self {
        let base = TempDir::new("command-base");
        BaseBuilder::steam().write(base.path());
        write_file(base.path(), "nvse_1_4.dll", &PeBuilder::dll().build());
        write_file(
            base.path(),
            "Data/FalloutNV.esm",
            &PluginBuilder::master().build(),
        );
        let work = TempDir::new("command-work");
        fs::write(work.path().join("masterlist.yaml"), MASTERLIST).expect("masterlist");
        Self { base, work }
    }

    fn instance(&self) -> PathBuf {
        self.work.path().join("fnv")
    }

    fn archive(&self, name: &str) -> PathBuf {
        let path = self.work.path().join(name);
        ZipBuilder::new()
            .file(&format!("{name}.esp"), &PluginBuilder::new().build())
            .write(&path);
        path
    }

    fn bad_archive(&self, name: &str) -> PathBuf {
        let path = self.work.path().join(name);
        fs::write(&path, b"this is not an archive").expect("bad archive");
        path
    }

    fn request(&self) -> Request {
        let words = vec![
            "--base".to_owned(),
            self.base.path().display().to_string(),
            "--instance".to_owned(),
            self.instance().display().to_string(),
            "--masterlist".to_owned(),
            self.work
                .path()
                .join("masterlist.yaml")
                .display()
                .to_string(),
            "--skip-nv-rs".to_owned(),
        ];
        parse_words(&words).expect("words").request
    }
}

fn run(mode: &Mode) -> (Ended, Vec<Value>) {
    let mut lines = Vec::new();
    let ended = execute(mode, &mut |l| lines.push(l.json), &CancelToken::new());
    (ended, lines)
}

fn fresh(request: Request, dry_run: bool, keep_partial: bool) -> Mode {
    Mode::Fresh {
        request,
        dry_run,
        keep_partial,
    }
}

fn events<'a>(lines: &'a [Value], kind: &str) -> Vec<&'a Value> {
    lines.iter().filter(|l| l["event"] == kind).collect()
}

fn steps_with(lines: &[Value], status: &str) -> Vec<String> {
    events(lines, "step")
        .into_iter()
        .filter(|l| l["status"] == status)
        .map(|l| l["step"].as_str().unwrap().to_owned())
        .collect()
}

fn last(lines: &[Value]) -> &Value {
    lines.last().expect("an event")
}

#[test]
fn a_run_is_the_stream_the_wizard_reads_and_ends_ok() {
    let s = Scratch::new();
    let mut request = s.request();
    request.mods = vec![s.archive("Better.zip")];
    let (ended, lines) = run(&fresh(request, false, false));
    assert_eq!(ended.code, 0, "{ended:?}");
    let seqs: Vec<u64> = lines.iter().map(|l| l["seq"].as_u64().unwrap()).collect();
    assert_eq!(seqs, (1..=lines.len() as u64).collect::<Vec<_>>());
    assert_eq!(lines[0]["event"], "plan");
    assert_eq!(lines[0]["dry_run"], false);
    assert_eq!(lines[0]["resume"], false);
    let planned: Vec<&str> = lines[0]["steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["step"].as_str().unwrap())
        .collect();
    assert_eq!(
        planned,
        [
            "init",
            "profile",
            "mod:Better.zip",
            "stage",
            "sort",
            "verify"
        ]
    );
    assert_eq!(steps_with(&lines, "started"), planned);
    assert_eq!(steps_with(&lines, "done"), planned);
    let done = last(&lines);
    assert_eq!(done["event"], "finished");
    assert_eq!(done["status"], "ok");
    assert_eq!(done["exit_code"], 0);
    assert_eq!(done["resumable"], false);
    assert!(Journal::file(&s.instance()).is_file());
}

#[test]
fn a_dry_run_plans_and_writes_nothing() {
    let s = Scratch::new();
    let (ended, lines) = run(&fresh(s.request(), true, false));
    assert_eq!(ended.code, 0);
    assert_eq!(lines[0]["dry_run"], true);
    assert_eq!(
        steps_with(&lines, "planned"),
        ["init", "profile", "stage", "sort", "verify"]
    );
    assert_eq!(last(&lines)["status"], "dry-run");
    assert!(!s.instance().exists());
}

#[test]
fn the_nv_rs_target_is_a_step_of_the_stream_when_asked_for() {
    let s = Scratch::new();
    let mut request = s.request();
    request.engine = true;
    let (_, lines) = run(&fresh(request, true, false));
    assert!(steps_with(&lines, "planned").contains(&"target".to_owned()));
}

#[test]
fn a_refused_run_is_one_finished_event_and_writes_nothing() {
    let s = Scratch::new();
    let mut request = s.request();
    request.masterlist = s.work.path().join("no-such.yaml");
    let (ended, lines) = run(&fresh(request, false, false));
    assert_eq!(ended.code, 2);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["status"], "refused");
    assert_eq!(lines[0]["seq"], 1);
    assert!(lines[0]["reason"].as_str().unwrap().contains("masterlist"));
    assert!(!s.instance().exists());
}

#[test]
fn a_failing_step_removes_what_the_run_made() {
    let s = Scratch::new();
    let mut request = s.request();
    request.mods = vec![s.bad_archive("Bad.zip")];
    let (ended, lines) = run(&fresh(request, false, false));
    assert_ne!(ended.code, 0);
    let end = last(&lines);
    assert_eq!(end["status"], "failed");
    assert_eq!(end["failed_step"], "mod:Bad.zip");
    assert_eq!(end["rolled_back"], true);
    assert_eq!(end["resumable"], false);
    assert_eq!(steps_with(&lines, "failed"), ["mod:Bad.zip"]);
    assert!(!s.instance().exists());
    assert!(!ended.note.unwrap().contains("--"), "no command line");
}

#[test]
fn a_kept_partial_run_resumes_from_the_journal_alone_and_a_restart_removes_it() {
    let s = Scratch::new();
    let mut request = s.request();
    let bad = s.bad_archive("Bad.zip");
    request.mods = vec![bad.clone()];
    let (ended, lines) = run(&fresh(request.clone(), false, true));
    assert_ne!(ended.code, 0);
    assert_eq!(last(&lines)["resumable"], true);
    assert!(s.instance().join("nvmod-instance.json").is_file());

    // Mend the archive, then resume with the Instance and nothing else.
    fs::remove_file(&bad).unwrap();
    ZipBuilder::new()
        .file("Bad.esp", &PluginBuilder::new().build())
        .write(&bad);
    let (ended, lines) = run(&Mode::Resume {
        instance: s.instance(),
    });
    assert_eq!(ended.code, 0, "{ended:?} {lines:?}");
    assert_eq!(lines[0]["event"], "plan");
    assert_eq!(lines[0]["resume"], true);
    let resumed = events(&lines, "resumed");
    assert_eq!(resumed[0]["completed"], json!(["init", "profile"]));
    assert_eq!(resumed[0]["next"], "mod:Bad.zip");
    let skipped: Vec<&str> = events(&lines, "skipped")
        .iter()
        .map(|l| l["step"].as_str().unwrap())
        .collect();
    assert_eq!(skipped, ["init", "profile"]);
    assert_eq!(
        steps_with(&lines, "done"),
        ["mod:Bad.zip", "stage", "sort", "verify"]
    );
    assert_eq!(last(&lines)["status"], "ok");

    // A finished Instance is not removed by a restart.
    let (ended, lines) = run(&Mode::Restart {
        instance: s.instance(),
        confirmed: true,
    });
    assert_eq!(ended.code, 2);
    assert!(
        last(&lines)["reason"]
            .as_str()
            .unwrap()
            .contains("finished")
    );
    assert!(s.instance().is_dir());
}

#[test]
fn a_restart_needs_the_yes_and_only_removes_an_unfinished_instance() {
    let s = Scratch::new();
    let mut request = s.request();
    request.mods = vec![s.bad_archive("Bad.zip")];
    run(&fresh(request, false, true));
    let (ended, lines) = run(&Mode::Restart {
        instance: s.instance(),
        confirmed: false,
    });
    assert_eq!(ended.code, 2);
    assert!(last(&lines)["reason"].as_str().unwrap().contains("--yes"));
    assert!(s.instance().is_dir());
    let (ended, lines) = run(&Mode::Restart {
        instance: s.instance(),
        confirmed: true,
    });
    assert_eq!(ended.code, 0);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["status"], "restarted");
    assert_eq!(lines[0]["seq"], 1);
    assert!(!s.instance().exists());
}

#[test]
fn an_instance_with_no_setup_cannot_be_resumed_or_restarted() {
    let s = Scratch::new();
    fs::create_dir_all(s.instance()).unwrap();
    for mode in [
        Mode::Resume {
            instance: s.instance(),
        },
        Mode::Restart {
            instance: s.instance(),
            confirmed: true,
        },
    ] {
        let (ended, lines) = run(&mode);
        assert_eq!(ended.code, 2);
        assert!(
            last(&lines)["reason"]
                .as_str()
                .unwrap()
                .contains("no unfinished setup")
        );
    }
    assert!(s.instance().is_dir());
}

#[test]
fn a_new_run_over_an_unfinished_setup_points_to_resume_and_restart() {
    let s = Scratch::new();
    let mut request = s.request();
    request.mods = vec![s.bad_archive("Bad.zip")];
    run(&fresh(request.clone(), false, true));
    let (ended, lines) = run(&fresh(request, false, false));
    assert_eq!(ended.code, 2);
    let why = last(&lines)["reason"].as_str().unwrap();
    assert!(
        why.contains("--resume") && why.contains("--restart"),
        "{why}"
    );
}

#[test]
fn an_unfinished_nvmod_setup_resumes_from_raul_at_the_same_next_step() {
    let s = Scratch::new();
    // Make the first steps with raul's own job, then write the state nvmod would have left for them.
    let mut request = s.request();
    request.mods = vec![s.archive("Better.zip")];
    let cancel = CancelToken::new();
    let mut stop_after_profile = |l: raul_core::setup::Line| {
        if l.json["event"] == "step" && l.json["status"] == "done" && l.json["step"] == "profile" {
            cancel.cancel();
        }
    };
    execute(
        &fresh(request.clone(), false, true),
        &mut stop_after_profile,
        &cancel,
    );
    // Forget raul's journal, as an Instance that nvmod set up has none.
    fs::remove_dir_all(s.instance().join(".raul")).unwrap();
    let args: Vec<String> = [
        "--base",
        &s.base.path().display().to_string(),
        "--instance",
        &s.instance().display().to_string(),
        "--masterlist",
        &s.work.path().join("masterlist.yaml").display().to_string(),
        "--mod",
        &s.work.path().join("Better.zip").display().to_string(),
        "--skip-nv-rs",
        "--keep-partial",
    ]
    .iter()
    .map(|w| (*w).to_owned())
    .collect();
    let state = json!({
        "schema_version": 1,
        "nvmod_version": "0.9.0",
        "args": args,
        "plan": [
            {"step": "init", "description": "d"}, {"step": "profile", "description": "d"},
            {"step": "mod:Better.zip", "description": "d"}, {"step": "stage", "description": "d"},
            {"step": "sort", "description": "d"}, {"step": "verify", "description": "d"},
        ],
        "completed": [{"step": "init", "witness": {}}, {"step": "profile", "witness": {}}],
        "complete": false
    });
    fs::create_dir_all(s.instance().join(".nvmod")).unwrap();
    fs::write(s.instance().join(".nvmod/setup.json"), state.to_string()).unwrap();

    let (ended, lines) = run(&Mode::Resume {
        instance: s.instance(),
    });
    assert_eq!(ended.code, 0, "{ended:?} {lines:?}");
    let resumed = events(&lines, "resumed");
    assert_eq!(resumed[0]["completed"], json!(["init", "profile"]));
    assert_eq!(resumed[0]["next"], "mod:Better.zip");
    assert_eq!(
        steps_with(&lines, "done"),
        ["mod:Better.zip", "stage", "sort", "verify"]
    );
}

#[test]
fn a_restart_is_refused_while_the_instance_is_in_use() {
    let s = Scratch::new();
    let mut request = s.request();
    request.mods = vec![s.bad_archive("Bad.zip")];
    run(&fresh(request, false, true));
    let guard = raul_core::instance_lock::InstanceGuard::acquire(&s.instance(), "test").unwrap();
    assert!(guard.is_held());
    let (ended, lines) = run(&Mode::Restart {
        instance: s.instance(),
        confirmed: true,
    });
    assert_eq!(ended.code, 2);
    assert!(last(&lines)["reason"].as_str().unwrap().contains("in use"));
    assert!(s.instance().is_dir());
    drop(guard);
    assert_eq!(
        run(&Mode::Restart {
            instance: s.instance(),
            confirmed: true
        })
        .0
        .code,
        0
    );
}
