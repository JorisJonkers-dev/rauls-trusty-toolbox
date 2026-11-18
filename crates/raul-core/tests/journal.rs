// SPDX-License-Identifier: GPL-3.0-only
//! The setup journal on synthetic fixtures: the testkit's base copy and archives, and a hand-built `nvmod setup`
//! state in the documented shape. No game data is used.

use std::fs;
use std::path::{Path, PathBuf};

use nvmod_testkit::{
    BaseBuilder, Mo2Builder, ModSpec, PeBuilder, PluginBuilder, TempDir, ZipBuilder, write_file,
};
use raul_core::journal::{self, ErrorKind, FILE_NAME, FOLDER, Journal, NVMOD_COPY, RunError};
use raul_core::setup::{CancelToken, Event, Outcome, Request, Step};
use serde_json::{Value, json};

const MASTERLIST: &str = "globals:\n  - {type: say, content: 'Masterlist for tests'}\n";

struct Scratch {
    base: TempDir,
    work: TempDir,
}

impl Scratch {
    fn new() -> Self {
        let base = TempDir::new("journal-base");
        BaseBuilder::steam().write(base.path());
        write_file(base.path(), "nvse_1_4.dll", &PeBuilder::dll().build());
        write_file(
            base.path(),
            "Data/FalloutNV.esm",
            &PluginBuilder::master().build(),
        );
        let work = TempDir::new("journal-work");
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

    fn request(&self) -> Request {
        Request {
            base: self.base.path().to_path_buf(),
            instance: self.instance(),
            masterlist: self.work.path().join("masterlist.yaml"),
            profile: None,
            import_mo2: None,
            copy: false,
            engine: false,
            engine_options: raul_core::setup::EngineOptions::default(),
            ttw: None,
            mods: Vec::new(),
            answers: None,
            sort: false,
            catalog: None,
            done: Vec::new(),
        }
    }
}

fn ids(steps: &[Step]) -> Vec<String> {
    steps.iter().map(Step::id).collect()
}

fn started(events: &[Event]) -> Vec<String> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Started { step, .. } => Some(step.id()),
            _ => None,
        })
        .collect()
}

fn run_journaled(
    request: &Request,
    cancel: &CancelToken,
) -> (Result<Outcome, RunError>, Vec<Event>) {
    let mut events = Vec::new();
    let result = journal::run(request, &mut |e: &Event| events.push(e.clone()), cancel);
    (result, events)
}

/// Cancels once `step` has finished.
fn cancel_after<'a>(step: &str, cancel: &'a CancelToken) -> impl FnMut(&Event) + 'a {
    let step = step.to_owned();
    move |e| {
        if matches!(e, Event::Finished { step: s, .. } if s.id() == step) {
            cancel.cancel();
        }
    }
}

fn read(instance: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(Journal::file(instance)).unwrap()).unwrap()
}

fn nvmod_state(complete: bool, schema_version: u64) -> Value {
    json!({
        "schema_version": schema_version,
        "nvmod_version": "0.9.0",
        "args": ["--base", "/base", "--instance", "/inst"],
        "plan": [
            {"step": "init", "description": "make the Instance skeleton"},
            {"step": "profile", "description": "make the first profile"},
            {"step": "nv-rs", "description": "the engine"},
            {"step": "target", "description": "the target"},
            {"step": "mod:Better.zip", "description": "a Mod"},
            {"step": "stage", "description": "Staging"},
        ],
        "completed": [
            {"step": "init", "witness": {}},
            {"step": "profile", "witness": {}},
            {"step": "nv-rs", "witness": {"files": [{"path": "nv-rs/install.json", "size": 2}]}},
        ],
        "failure": {"step": "target", "code": 4, "detail": "it broke"},
        "complete": complete
    })
}

fn write_nvmod_state(instance: &Path, state: &Value) {
    let dir = instance.join(".nvmod");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("setup.json"), state.to_string()).unwrap();
}

#[test]
fn a_run_writes_the_journal_after_each_step_and_marks_it_complete() {
    let s = Scratch::new();
    let mut request = s.request();
    request.mods = vec![s.archive("Better.zip")];
    // Nothing is made in the folder before the first step has.
    assert!(!s.instance().exists());
    let journal = Journal::open(&s.instance()).unwrap();
    assert!(journal.finished().is_empty() && !s.instance().exists());

    let (result, _) = run_journaled(&request, &CancelToken::new());
    assert!(matches!(result, Ok(Outcome::Done(_))));
    let doc = read(&s.instance());
    assert_eq!(doc["schema_version"], 1);
    assert_eq!(doc["contract"], "raul-journal");
    assert_eq!(doc["complete"], true);
    let expected = [
        "init",
        "profile",
        "mod:Better.zip",
        "stage",
        "sort",
        "verify",
    ];
    assert_eq!(doc["finished"], json!(expected));
    assert_eq!(doc["plan"], json!(expected));
}

#[test]
fn an_interrupted_setup_resumes_at_the_first_unfinished_step() {
    let s = Scratch::new();
    let mut request = s.request();
    request.mods = vec![s.archive("Better.zip"), s.archive("Extra.zip")];
    let cancel = CancelToken::new();
    let mut stop = cancel_after("mod:Better.zip", &cancel);
    let result = journal::run(&request, &mut stop, &cancel);
    assert!(matches!(result, Ok(Outcome::Cancelled(_))));

    let journal = Journal::open(&s.instance()).unwrap();
    assert_eq!(journal.finished(), ["init", "profile", "mod:Better.zip"]);
    assert!(!journal.doc().complete);
    assert_eq!(
        journal.next_step(&request),
        Some(Step::Mod(request.mods[1].clone()))
    );
    assert_eq!(
        ids(&journal.resume(&request)),
        ["init", "profile", "mod:Better.zip"]
    );
    let listed: Vec<(String, bool)> = journal
        .list()
        .into_iter()
        .map(|l| (l.id, l.finished))
        .collect();
    assert_eq!(listed[2], ("mod:Better.zip".to_owned(), true));
    assert_eq!(listed[3], ("mod:Extra.zip".to_owned(), false));

    let (result, events) = run_journaled(&request, &CancelToken::new());
    assert!(matches!(result, Ok(Outcome::Done(_))));
    assert_eq!(
        started(&events),
        ["mod:Extra.zip", "stage", "sort", "verify"]
    );
    assert_eq!(read(&s.instance())["complete"], true);
}

#[test]
fn restart_forgets_the_finished_steps_and_keeps_the_file() {
    let s = Scratch::new();
    let request = s.request();
    let cancel = CancelToken::new();
    let mut stop = cancel_after("profile", &cancel);
    journal::run(&request, &mut stop, &cancel).unwrap();

    let mut journal = Journal::open(&s.instance()).unwrap();
    assert_eq!(journal.finished().len(), 2);
    journal.restart().unwrap();
    assert_eq!(journal.finished(), [] as [String; 0]);
    let reopened = Journal::open(&s.instance()).unwrap();
    assert!(reopened.finished().is_empty() && reopened.doc().plan.is_empty());
    assert_eq!(reopened.next_step(&request), Some(Step::Init));
}

#[test]
fn remove_deletes_the_journal_and_a_new_run_starts_from_the_first_step() {
    let s = Scratch::new();
    let cancel = CancelToken::new();
    let mut stop = cancel_after("init", &cancel);
    journal::run(&s.request(), &mut stop, &cancel).unwrap();

    let journal = Journal::open(&s.instance()).unwrap();
    journal.remove().unwrap();
    assert!(!Journal::file(&s.instance()).exists());
    assert_eq!(
        Journal::open(&s.instance()).unwrap().finished(),
        [] as [String; 0]
    );
    // Removing a journal that is not there is not an error.
    Journal::open(&s.instance()).unwrap().remove().unwrap();
}

#[test]
fn an_unfinished_nvmod_setup_is_taken_over_once_and_resumes_at_the_same_step() {
    let s = Scratch::new();
    let instance = s.instance();
    fs::create_dir_all(&instance).unwrap();
    let state = nvmod_state(false, 1);
    write_nvmod_state(&instance, &state);
    let before = fs::read(instance.join(".nvmod/setup.json")).unwrap();

    let mut request = s.request();
    request.engine = true;
    request.mods = vec![PathBuf::from("/archives/Better.zip")];
    let journal = Journal::open(&instance).unwrap();
    // nvmod's `nv-rs` is raul's `engine`; the next step is the one nvmod would run, the target.
    assert_eq!(journal.finished(), ["init", "profile", "engine"]);
    assert_eq!(journal.next_step(&request), Some(Step::Target));
    let taken = journal.doc().taken_over.clone().unwrap();
    assert_eq!(
        (taken.from.as_str(), taken.version.as_str()),
        ("nvmod-setup", "0.9.0")
    );
    assert_eq!(journal.doc().plan[2], "engine");

    // The old file is left as it was, and a copy of it is kept in raul's folder.
    assert_eq!(
        fs::read(instance.join(".nvmod/setup.json")).unwrap(),
        before
    );
    assert_eq!(
        fs::read(instance.join(FOLDER).join(NVMOD_COPY)).unwrap(),
        before
    );

    // Once: later opens read the journal, and the nvmod file changing does not matter.
    write_nvmod_state(&instance, &json!({"schema_version": 1, "completed": []}));
    assert_eq!(Journal::open(&instance).unwrap().finished().len(), 3);
    // Removing the journal does not take the nvmod state over again.
    Journal::open(&instance).unwrap().remove().unwrap();
    write_nvmod_state(&instance, &state);
    assert_eq!(
        Journal::open(&instance).unwrap().finished(),
        [] as [String; 0]
    );
}

#[test]
fn a_finished_nvmod_setup_is_not_taken_over() {
    let s = Scratch::new();
    fs::create_dir_all(s.instance()).unwrap();
    write_nvmod_state(&s.instance(), &nvmod_state(true, 1));
    let journal = Journal::open(&s.instance()).unwrap();
    assert!(journal.finished().is_empty() && journal.doc().taken_over.is_none());
    assert!(!s.instance().join(FOLDER).exists());
}

#[test]
fn an_nvmod_state_that_cannot_be_read_refuses_the_takeover_and_is_left_alone() {
    let s = Scratch::new();
    let instance = s.instance();
    fs::create_dir_all(&instance).unwrap();
    write_nvmod_state(&instance, &nvmod_state(false, 2));
    let e = Journal::open(&instance).unwrap_err();
    assert_eq!(e.kind, ErrorKind::TakeoverRefused);
    assert!(e.summary.contains("newer"), "{e}");
    fs::write(instance.join(".nvmod/setup.json"), "{ not json").unwrap();
    assert_eq!(
        Journal::open(&instance).unwrap_err().kind,
        ErrorKind::TakeoverRefused
    );
    assert!(!instance.join(FOLDER).exists());
}

#[test]
fn a_journal_of_an_older_version_is_migrated_after_a_copy_is_kept() {
    let s = Scratch::new();
    let instance = s.instance();
    let file = Journal::file(&instance);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    let v0 = include_str!("../../../contracts/fixtures/raul-journal-v0/setup-journal.json");
    fs::write(&file, v0).unwrap();

    let journal = Journal::open(&instance).unwrap();
    assert_eq!(journal.finished(), ["init", "profile", "mod:Example.zip"]);
    let doc = read(&instance);
    assert_eq!(doc["schema_version"], 1);
    assert_eq!(doc["contract"], "raul-journal");
    assert_eq!(
        doc["finished"],
        json!(["init", "profile", "mod:Example.zip"])
    );
    let copy = file.with_file_name(format!("{FILE_NAME}.v0.bak"));
    assert_eq!(fs::read_to_string(&copy).unwrap(), v0);
}

#[test]
fn a_migration_never_replaces_an_earlier_copy() {
    let s = Scratch::new();
    let instance = s.instance();
    let file = Journal::file(&instance);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(
        file.with_file_name(format!("{FILE_NAME}.v0.bak")),
        "older copy",
    )
    .unwrap();
    fs::write(&file, r#"{"schema_version": 0, "steps": ["init"]}"#).unwrap();
    Journal::open(&instance).unwrap();
    assert_eq!(
        fs::read_to_string(file.with_file_name(format!("{FILE_NAME}.v0.bak"))).unwrap(),
        "older copy"
    );
    assert!(
        file.with_file_name(format!("{FILE_NAME}.v0.bak.1"))
            .exists()
    );

    // Names taken by folders are passed over too.
    let t = Scratch::new();
    let broken = Journal::file(&t.instance());
    fs::create_dir_all(broken.parent().unwrap()).unwrap();
    let text = r#"{"schema_version": 0, "steps": ["init"]}"#;
    fs::write(&broken, text).unwrap();
    for n in 0..=1 {
        let name = if n == 0 {
            format!("{FILE_NAME}.v0.bak")
        } else {
            format!("{FILE_NAME}.v0.bak.{n}")
        };
        fs::create_dir(broken.with_file_name(name)).unwrap();
    }
    // Both names are taken by folders, so the counter moves on and still finds a free name: it succeeds.
    assert!(Journal::open(&t.instance()).is_ok());
    assert!(
        broken
            .with_file_name(format!("{FILE_NAME}.v0.bak.2"))
            .is_file()
    );
}

#[test]
fn a_journal_of_a_newer_version_is_refused_and_left_as_it_is() {
    let s = Scratch::new();
    let file = Journal::file(&s.instance());
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    let text = r#"{"schema_version": 2, "contract": "raul-journal", "finished": ["init"]}"#;
    fs::write(&file, text).unwrap();
    let e = Journal::open(&s.instance()).unwrap_err();
    assert_eq!(e.kind, ErrorKind::NewerSchema);
    assert!(e.summary.contains("schema_version 2") && e.summary.contains("Update raul"));
    assert_eq!(fs::read_to_string(&file).unwrap(), text);
    // The run refuses too, before any step.
    let (result, events) = run_journaled(&s.request(), &CancelToken::new());
    assert!(matches!(result, Err(RunError::Journal(e)) if e.kind == ErrorKind::NewerSchema));
    assert_eq!(started(&events), [] as [String; 0]);
}

#[test]
fn a_file_that_is_not_a_journal_is_refused_and_left_as_it_is() {
    let s = Scratch::new();
    let file = Journal::file(&s.instance());
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    for text in [
        "{ not json",
        "{}",
        r#"{"schema_version": 1, "contract": "other", "raul_version": "1", "finished": []}"#,
        r#"{"schema_version": 0, "steps": [1]}"#,
    ] {
        fs::write(&file, text).unwrap();
        let e = Journal::open(&s.instance()).unwrap_err();
        assert_eq!(e.kind, ErrorKind::Unreadable, "{text}");
        assert_eq!(fs::read_to_string(&file).unwrap(), text);
    }
}

#[test]
fn a_partial_file_from_an_interrupted_write_does_not_replace_the_journal() {
    let s = Scratch::new();
    let cancel = CancelToken::new();
    let mut stop = cancel_after("init", &cancel);
    journal::run(&s.request(), &mut stop, &cancel).unwrap();
    let file = Journal::file(&s.instance());
    // A write that died halfway leaves a temporary file next to the journal; the journal itself is whole.
    fs::write(
        file.with_file_name(format!("{FILE_NAME}.1-2.new")),
        r#"{"schema_ver"#,
    )
    .unwrap();
    let journal = Journal::open(&s.instance()).unwrap();
    assert_eq!(journal.finished(), ["init"]);
}

#[cfg(unix)]
#[test]
fn a_failed_write_leaves_the_old_file_and_stops_the_run() {
    use std::os::unix::fs::PermissionsExt;
    let s = Scratch::new();
    let request = s.request();
    let cancel = CancelToken::new();
    let mut stop = cancel_after("init", &cancel);
    journal::run(&request, &mut stop, &cancel).unwrap();
    let folder = s.instance().join(FOLDER);
    let before = fs::read_to_string(Journal::file(&s.instance())).unwrap();
    fs::set_permissions(&folder, fs::Permissions::from_mode(0o500)).unwrap();
    let (result, events) = run_journaled(&request, &CancelToken::new());
    fs::set_permissions(&folder, fs::Permissions::from_mode(0o700)).unwrap();
    if fs::write(folder.join("probe"), "x").is_ok() {
        // Running with rights that ignore the mode (for example as root): the case cannot be made.
        return;
    }
    let Err(RunError::Journal(e)) = result else {
        panic!("expected a journal error, got {result:?}");
    };
    assert_eq!(e.kind, ErrorKind::WriteFailed);
    // The step that finished is the only one that ran before the run stopped.
    assert_eq!(started(&events), ["profile"]);
    assert_eq!(
        fs::read_to_string(Journal::file(&s.instance())).unwrap(),
        before
    );
}

#[test]
fn errors_carry_a_code_and_copy_text_without_a_command_line() {
    let s = Scratch::new();
    let file = Journal::file(&s.instance());
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, r#"{"schema_version": 9}"#).unwrap();
    let e = Journal::open(&s.instance()).unwrap_err();
    let text = e.copy_text();
    assert!(text.starts_with(&format!("raul {}\n", env!("CARGO_PKG_VERSION"))));
    assert!(text.contains("error: newer_schema"));
    assert!(!text.contains("nvmod ") && !text.contains("--"));
    for (kind, code) in [
        (ErrorKind::NewerSchema, "newer_schema"),
        (ErrorKind::Unreadable, "unreadable"),
        (ErrorKind::WriteFailed, "write_failed"),
        (ErrorKind::TakeoverRefused, "takeover_refused"),
        (ErrorKind::PlanMismatch, "plan_mismatch"),
    ] {
        assert_eq!(kind.code(), code);
    }
    assert_eq!(RunError::Journal(e.clone()).copy_text(), text);
}

#[test]
fn a_request_that_is_not_the_journals_setup_is_refused_before_any_step() {
    let s = Scratch::new();
    let mut request = s.request();
    request.mods = vec![s.archive("Better.zip")];
    let cancel = CancelToken::new();
    let mut stop = cancel_after("mod:Better.zip", &cancel);
    journal::run(&request, &mut stop, &cancel).unwrap();

    // The same setup without the Mod: a finished step is missing from it.
    request.mods.clear();
    let (result, events) = run_journaled(&request, &CancelToken::new());
    assert!(matches!(&result, Err(RunError::Journal(e)) if e.kind == ErrorKind::PlanMismatch));
    assert_eq!(started(&events), [] as [String; 0]);

    // Two archives of one file name cannot be told apart.
    let t = Scratch::new();
    let mut twice = t.request();
    let one = t.archive("Same.zip");
    let other = t.work.path().join("other");
    fs::create_dir_all(&other).unwrap();
    fs::copy(&one, other.join("Same.zip")).unwrap();
    twice.mods = vec![one, other.join("Same.zip")];
    let (result, _) = run_journaled(&twice, &CancelToken::new());
    let Err(RunError::Journal(e)) = result else {
        panic!("expected a refusal")
    };
    assert!(e.summary.contains("mod:Same.zip"), "{e}");
    assert!(!t.instance().exists());
}

#[test]
fn an_imported_setup_is_complete_with_the_sort_it_keeps_out_of_the_plan() {
    let s = Scratch::new();
    let mo2 = TempDir::new("journal-mo2");
    Mo2Builder::new(s.base.path())
        .mod_dir(ModSpec::new("Core").file("Core.esp", &PluginBuilder::new().build()))
        .profile("Default", &["+Core"], &["Core.esp"])
        .write(mo2.path());
    let mut request = s.request();
    request.import_mo2 = Some(mo2.path().to_path_buf());
    let (result, _) = run_journaled(&request, &CancelToken::new());
    assert!(matches!(result, Ok(Outcome::Done(_))));
    let doc = read(&s.instance());
    assert_eq!(doc["complete"], true);
    assert_eq!(doc["plan"], doc["finished"]);
    assert_eq!(
        doc["finished"],
        json!(["import-mo2", "stage", "stamp", "verify"])
    );
}

#[test]
fn an_object_that_is_not_an_nvmod_state_is_not_taken_over() {
    let s = Scratch::new();
    fs::create_dir_all(s.instance()).unwrap();
    write_nvmod_state(&s.instance(), &json!({"schema_version": 1}));
    assert_eq!(
        Journal::open(&s.instance()).unwrap_err().kind,
        ErrorKind::TakeoverRefused
    );
}
