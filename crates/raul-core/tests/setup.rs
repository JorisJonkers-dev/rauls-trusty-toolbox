// SPDX-License-Identifier: GPL-3.0-only
//! The setup job on synthetic fixtures: the testkit's base copy, archives and MO2 folder. No game data is used.

use raul_testkit::{covers, ids};
use std::fs;
use std::path::PathBuf;

use nvmod_testkit::{
    BaseBuilder, Mo2Builder, ModSpec, PeBuilder, PluginBuilder, TempDir, ZipBuilder, write_file,
};
use raul_core::setup::{
    CancelToken, Error, ErrorKind, Event, Outcome, Request, SkipReason, Step, Ttw, run,
};

const MASTERLIST: &str = "globals:\n  - {type: say, content: 'Masterlist for tests'}\n";

struct Scratch {
    base: TempDir,
    work: TempDir,
}

impl Scratch {
    fn new() -> Self {
        let base = TempDir::new("setup-base");
        BaseBuilder::steam().write(base.path());
        write_file(base.path(), "nvse_1_4.dll", &PeBuilder::dll().build());
        write_file(
            base.path(),
            "Data/FalloutNV.esm",
            &PluginBuilder::master().build(),
        );
        let work = TempDir::new("setup-work");
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

/// Run and keep every event.
fn record(request: &Request, cancel: &CancelToken) -> (Result<Outcome, Error>, Vec<Event>) {
    let mut events = Vec::new();
    let result = run(request, &mut |e: &Event| events.push(e.clone()), cancel);
    (result, events)
}

fn done(result: Result<Outcome, Error>) -> raul_core::setup::Output {
    match result {
        Ok(Outcome::Done(out)) => out,
        other => panic!("expected a finished setup, got {other:?}"),
    }
}

/// The ids of the steps an event list started, in order.
fn started(events: &[Event]) -> Vec<String> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Started { step, .. } => Some(step.id()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_fresh_setup_runs_every_planned_step_in_order_and_reports_each() {
    covers(&[ids::setup::pipeline::FRESH_ORDER]);
    let s = Scratch::new();
    let mut request = s.request();
    request.mods = vec![s.archive("Better.zip"), s.archive("Extra.zip")];
    let (result, events) = record(&request, &CancelToken::new());
    let out = done(result);
    let expected = [
        "init",
        "profile",
        "mod:Better.zip",
        "mod:Extra.zip",
        "stage",
        "sort",
        "verify",
    ];
    assert_eq!(ids(&out.completed), expected);
    assert_eq!(out.skipped.len(), 0);
    assert!(out.verify.is_some_and(|v| v.verified));
    assert!(s.instance().join("mods/Better").is_dir());
    assert!(s.instance().join("mods/Extra").is_dir());

    assert!(matches!(&events[0], Event::Planned { steps } if steps.len() == expected.len()));
    assert_eq!(started(&events), expected);
    let finished = events
        .iter()
        .filter(|e| matches!(e, Event::Finished { .. }))
        .count();
    assert_eq!(finished, expected.len());
    // The first step's events: started 1 of 7, then the job's own detail, then finished.
    assert!(matches!(
        &events[1],
        Event::Started {
            step: Step::Init,
            index: 1,
            total: 7
        }
    ));
    assert!(events.iter().any(|e| matches!(
        e,
        Event::Detail { step: Step::Stage, label, .. } if label == "stage.stage.building-tree"
    )));
}

/// The engine is linked in, so its capabilities come from it, not from a package (#398): the nv-rs target writes the
/// text the program provided, and the Doctor then has no `capabilities.missing` to report on the fresh Instance.
#[test]
fn the_nv_rs_target_writes_the_linked_engines_capabilities_and_the_doctor_finds_them() {
    const CAPABILITIES: &str = r#"{"schema_version":1,"extenders":[{"id":"synthetic","name":"Synthetic","commands":{"implemented":1,"total":2,"implemented_names":["Command1"]}}]}"#;
    covers(&[ids::setup::pipeline::ENGINE_CAPABILITIES]);
    raul_core::capabilities::provide(CAPABILITIES);
    let s = Scratch::new();
    let mut request = s.request();
    request.engine = true;
    let out = done(record(&request, &CancelToken::new()).0);
    assert_eq!(
        ids(&out.completed),
        ["init", "profile", "target", "stage", "sort", "verify"]
    );
    assert_eq!(
        fs::read_to_string(s.instance().join("nv-rs/capabilities.json")).unwrap(),
        CAPABILITIES
    );
    let doctor = serde_json::to_string(&out.verify.expect("the check ran").doctor).unwrap();
    assert!(!doctor.contains("capabilities.missing"), "{doctor}");
    assert!(!doctor.contains("capabilities.invalid"), "{doctor}");
}

#[test]
fn a_setup_without_the_nv_rs_target_writes_no_capabilities() {
    let s = Scratch::new();
    let request = s.request();
    done(record(&request, &CancelToken::new()).0);
    assert!(!s.instance().join("nv-rs").exists());
}

#[test]
fn steps_marked_done_are_skipped_and_reported_as_skipped() {
    let s = Scratch::new();
    let first = s.request();
    done(record(&first, &CancelToken::new()).0);
    let mut again = s.request();
    again.mods = vec![s.archive("Better.zip")];
    again.done = vec![Step::Init, Step::Profile];
    // The Instance exists and its first Profile is made, as a resume finds it.
    let (result, events) = record(&again, &CancelToken::new());
    let out = done(result);
    assert_eq!(
        out.skipped,
        [
            (Step::Init, SkipReason::Done),
            (Step::Profile, SkipReason::Done)
        ]
    );
    assert_eq!(
        ids(&out.completed),
        ["mod:Better.zip", "stage", "sort", "verify"]
    );
    assert_eq!(started(&events), ids(&out.completed));
    assert!(events.iter().any(|e| matches!(
        e,
        Event::Skipped {
            step: Step::Init,
            index: 1,
            reason: SkipReason::Done,
            ..
        }
    )));
}

#[test]
fn an_imported_instance_keeps_its_order_and_stamps_it() {
    covers(&[ids::setup::pipeline::IMPORTED_KEEPS_ORDER]);
    let s = Scratch::new();
    let mo2 = TempDir::new("setup-mo2");
    Mo2Builder::new(s.base.path())
        .mod_dir(ModSpec::new("Core").file("Core.esp", &PluginBuilder::new().build()))
        .profile("Default", &["+Core"], &["Core.esp"])
        .write(mo2.path());
    let mut request = s.request();
    request.import_mo2 = Some(mo2.path().to_path_buf());
    let (result, events) = record(&request, &CancelToken::new());
    let out = done(result);
    assert_eq!(
        ids(&out.completed),
        ["import-mo2", "stage", "stamp", "verify"]
    );
    assert_eq!(out.skipped, [(Step::Sort, SkipReason::KeepsImportedOrder)]);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::Skipped {
            step: Step::Sort,
            reason: SkipReason::KeepsImportedOrder,
            ..
        }
    )));
}

fn fails(request: &Request) -> (Error, Vec<Event>) {
    let (result, events) = record(request, &CancelToken::new());
    match result {
        Err(e) => (e, events),
        other => panic!("expected a failure, got {other:?}"),
    }
}

#[test]
fn a_failing_step_stops_the_run_with_its_typed_error_and_no_command_line() {
    covers(&[ids::setup::pipeline::TYPED_FAILURE]);
    let s = Scratch::new();
    let mut request = s.request();
    request.base = s.work.path().join("no-such-base");
    let (error, events) = fails(&request);
    assert_eq!(error.kind, ErrorKind::Refused);
    assert_eq!(error.step, Step::Init);
    assert_eq!(error.job, "init.create");
    assert_eq!(started(&events), ["init"], "nothing runs after the failure");
    assert!(!events.iter().any(|e| matches!(e, Event::Finished { .. })));
    let text = error.copy_text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[0].starts_with("raul "), "{text}");
    assert_eq!(
        lines[1..4],
        ["step: init", "job: init.create", "error: refused"]
    );
    assert!(!text.contains("argv"), "no command line: {text}");
}

#[test]
fn a_mod_that_cannot_be_installed_stops_after_the_steps_before_it() {
    let s = Scratch::new();
    let mut request = s.request();
    request.mods = vec![s.work.path().join("missing.zip"), s.archive("Never.zip")];
    let (error, events) = fails(&request);
    assert_eq!(error.step, Step::Mod(s.work.path().join("missing.zip")));
    assert_eq!(error.job, "install.install");
    assert_eq!(started(&events), ["init", "profile", "mod:missing.zip"]);
    assert!(!s.instance().join("mods/Never").exists());
}

#[test]
fn a_mod_with_unanswered_questions_is_refused_with_the_count() {
    let s = Scratch::new();
    let archive = s.work.path().join("Choice.zip");
    let xml = r#"<config><installSteps order="Explicit"><installStep name="S"><optionalFileGroups order="Explicit"><group name="G" type="SelectExactlyOne"><plugins order="Explicit"><plugin name="A"><description/><files><file source="a.txt" destination="a.txt"/></files><typeDescriptor><type name="Optional"/></typeDescriptor></plugin><plugin name="B"><description/><files><file source="b.txt" destination="b.txt"/></files><typeDescriptor><type name="Optional"/></typeDescriptor></plugin></plugins></group></optionalFileGroups></installStep></installSteps></config>"#;
    ZipBuilder::new()
        .file("fomod/ModuleConfig.xml", xml.as_bytes())
        .file("a.txt", b"a")
        .file("b.txt", b"b")
        .write(&archive);
    let mut request = s.request();
    request.mods = vec![archive];
    let (error, _) = fails(&request);
    assert_eq!(error.kind, ErrorKind::Refused);
    assert_eq!(error.job, "install.install");
    assert!(error.summary.contains("Choice.zip"), "{}", error.summary);
    assert!(error.summary.contains("question"), "{}", error.summary);
}

#[test]
fn the_ttw_step_refuses_a_missing_package_in_process() {
    let s = Scratch::new();
    let mut request = s.request();
    request.ttw = Some(Ttw {
        fallout3: s.work.path().join("fo3"),
        newvegas: None,
        package: Some(s.work.path().join("missing.mpi")),
        resume: false,
        timeout: None,
    });
    let (error, events) = fails(&request);
    assert_eq!(error.step, Step::Ttw);
    assert_eq!(error.job, "ttw.install");
    assert_eq!(error.kind, ErrorKind::Refused);
    assert_eq!(started(&events), ["init", "profile", "ttw"]);
}

#[test]
fn a_missing_masterlist_stops_the_sort_step() {
    covers(&[ids::mods::sort::MISSING_MASTERLIST]);
    let s = Scratch::new();
    let mut request = s.request();
    request.masterlist = s.work.path().join("missing.yaml");
    let (error, events) = fails(&request);
    assert_eq!(error.step, Step::Sort);
    assert_eq!(error.kind, ErrorKind::Refused);
    assert_eq!(started(&events), ["init", "profile", "stage", "sort"]);
}

#[test]
fn a_check_that_does_not_pass_is_an_unverified_error() {
    let s = Scratch::new();
    // A Mod plugin that needs a master nothing provides: the Doctor reports an error.
    let archive = s.work.path().join("Orphan.zip");
    ZipBuilder::new()
        .file(
            "Orphan.esp",
            &PluginBuilder::new().requires("Missing.esm").build(),
        )
        .write(&archive);
    let mut request = s.request();
    request.mods = vec![archive];
    let (error, events) = fails(&request);
    assert_eq!(error.kind, ErrorKind::Unverified, "{error}");
    assert_eq!(error.step, Step::Verify);
    assert_eq!(error.job, "verify.verify");
    assert_eq!(started(&events).last().map(String::as_str), Some("verify"));
}

#[test]
fn a_stop_asked_for_before_the_run_ends_it_before_the_first_step() {
    let s = Scratch::new();
    let cancel = CancelToken::new();
    cancel.cancel();
    let (result, events) = record(&s.request(), &cancel);
    let Ok(Outcome::Cancelled(out)) = result else {
        panic!("expected a cancelled setup");
    };
    assert_eq!(out.completed.len(), 0);
    assert_eq!(events.len(), 1, "only the plan is reported");
    assert!(!s.instance().exists());
}

#[test]
fn a_stop_asked_for_during_a_step_ends_the_run_at_the_next_step() {
    covers(&[ids::setup::pipeline::STOP_REQUEST]);
    let s = Scratch::new();
    let cancel = CancelToken::new();
    let mut events = Vec::new();
    let result = run(
        &s.request(),
        &mut |e: &Event| {
            if matches!(
                e,
                Event::Finished {
                    step: Step::Profile,
                    ..
                }
            ) {
                cancel.cancel();
            }
            events.push(e.clone());
        },
        &cancel,
    );
    let Ok(Outcome::Cancelled(out)) = result else {
        panic!("expected a cancelled setup");
    };
    assert_eq!(ids(&out.completed), ["init", "profile"]);
    assert_eq!(started(&events), ["init", "profile"]);
    assert!(
        !s.instance().join("staging/Default").exists(),
        "no step ran after the stop"
    );
}

#[test]
fn a_stop_asked_for_inside_a_job_ends_the_run_with_the_steps_before_it() {
    let s = Scratch::new();
    let cancel = CancelToken::new();
    let result = run(
        &s.request(),
        &mut |e: &Event| {
            if matches!(
                e,
                Event::Detail {
                    step: Step::Stage,
                    ..
                }
            ) {
                cancel.cancel();
            }
        },
        &cancel,
    );
    let Ok(Outcome::Cancelled(out)) = result else {
        panic!("expected a cancelled setup");
    };
    assert_eq!(ids(&out.completed), ["init", "profile"]);
}

#[test]
fn a_rar_archive_is_refused_with_a_plain_message_and_nothing_is_installed() {
    covers(&[ids::setup::pipeline::RAR_REFUSED]);
    let s = Scratch::new();
    let rar = s.work.path().join("Packed.rar");
    // The signature of a RAR 5 archive and nothing else: nvmod reads zip, 7z and tar in process and refuses the rest.
    fs::write(&rar, b"Rar!\x1a\x07\x01\x00").expect("rar");
    let mut request = s.request();
    request.mods = vec![rar.clone()];
    let (error, events) = fails(&request);
    assert_eq!(error.kind, ErrorKind::Refused);
    assert_eq!(error.step, Step::Mod(rar));
    assert_eq!(error.job, "install.install");
    assert_ne!(error.summary, "");
    assert_eq!(started(&events), ["init", "profile", "mod:Packed.rar"]);
    assert!(!s.instance().join("mods/Packed").exists());
    assert!(!error.copy_text().contains("--"), "no command line");
}

#[test]
fn a_request_saved_with_the_removed_extract_tool_still_reads() {
    let s = Scratch::new();
    let mut value = serde_json::to_value(s.request()).expect("request");
    value["extract_tool"] = "/old/tool".into();
    let read: Request = serde_json::from_value(value).expect("an old request");
    assert_eq!(read, s.request());
}

/// A Mod with Street Lights TTW's plugin gets raul's small lamp glow add-on before Staging, and the run says so; a
/// setup without it gets no add-on and no note.
#[test]
fn a_setup_with_street_lights_adds_the_lamp_glow_add_on_and_says_so() {
    covers(&[ids::mods::addons::ADDED_BY_SETUP]);
    let s = Scratch::new();
    let lamps = s.work.path().join("Street Lights.zip");
    let esm = raul_drivers::addons::plugin::synthetic_plugin(
        &["FalloutNV.esm"],
        &[(
            "ACTI",
            0x0100_0801,
            "LampGlow",
            "Effects\\Ambient\\FXGlowSimpFillRnd.nif",
        )],
    );
    ZipBuilder::new()
        .file("Street Lights TTW.esm", &esm)
        .write(&lamps);
    let mut request = s.request();
    request.mods = vec![lamps];
    let (result, events) = record(&request, &CancelToken::new());
    done(result);
    let notes: Vec<&String> = events
        .iter()
        .filter_map(|e| match e {
            Event::Note {
                step: Step::Stage,
                text,
            } => Some(text),
            _ => None,
        })
        .collect();
    assert_eq!(notes.len(), 1, "{events:?}");
    assert!(notes[0].contains("Raul Small Lamp Glow"), "{notes:?}");
    let made = s.instance().join("mods/Raul Small Lamp Glow");
    assert!(made.join("Raul Small Lamp Glow.esp").is_file());
    assert!(
        made.join("meshes/raul/effects/raulsmallglow01.nif")
            .is_file()
    );

    let plain = Scratch::new();
    let mut request = plain.request();
    request.mods = vec![plain.archive("Better.zip")];
    let (result, events) = record(&request, &CancelToken::new());
    done(result);
    assert!(!events.iter().any(|e| matches!(e, Event::Note { .. })));
    assert!(!plain.instance().join("mods/Raul Small Lamp Glow").exists());
}
