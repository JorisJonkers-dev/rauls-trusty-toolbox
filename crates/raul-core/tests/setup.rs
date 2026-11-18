// SPDX-License-Identifier: GPL-3.0-only
//! The setup job on synthetic fixtures: the testkit's base copy, archives and MO2 folder. No game data is used.

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
            engine_options: raul_core::setup::EngineOptions::default(),
            ttw: None,
            mods: Vec::new(),
            answers: None,
            extract_tool: None,
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

/// The engine step on synthetic fixtures: a tiny nv-rs workspace, a fake local release and a `gh` stand-in.
mod engine {
    use super::*;
    use nvmod_testkit::{
        FAKE_ASSET, FAKE_BUILT_BANNER, FAKE_CHECK_REFUSED, FakeNvRs, FakeRelease, Standin,
        fake_release_binary, nv_viewer_name,
    };
    use raul_core::setup::EngineOptions;

    const CARGO: &str = env!("CARGO");

    struct Rig {
        s: Scratch,
        tools: TempDir,
    }

    impl Rig {
        fn new() -> Self {
            Self {
                s: Scratch::new(),
                tools: TempDir::new("setup-engine-tools"),
            }
        }

        /// A request for a source build of the fake workspace written by `nv_rs`.
        fn source(&self, nv_rs: &FakeNvRs) -> Request {
            let dir = self.tools.path().join("src");
            fs::create_dir_all(&dir).expect("source dir");
            nv_rs.write(&dir);
            let mut request = self.s.request();
            request.engine = true;
            request.engine_options = EngineOptions {
                source: Some(dir),
                cargo: Some(PathBuf::from(CARGO)),
                ..EngineOptions::default()
            };
            request
        }

        /// A request for the fake local `release`; `gh` exits with `gh_code` and logs its calls.
        fn release(&self, release: &FakeRelease, gh_code: i32) -> Request {
            let dir = self.tools.path().join("release");
            fs::create_dir_all(&dir).expect("release dir");
            release.write(&dir, nvmod_install::sha256_hex);
            let gh = Standin::new()
                .logging()
                .exits(gh_code)
                .write(self.tools.path(), "gh");
            let mut request = self.s.request();
            request.engine = true;
            request.engine_options = EngineOptions {
                release: true,
                release_url: format!("file://{}", dir.display()),
                gh,
                retry_delay_ms: 0,
                ..EngineOptions::default()
            };
            request
        }

        fn gh_calls(&self) -> usize {
            fs::read_to_string(self.tools.path().join("gh.log"))
                .unwrap_or_default()
                .lines()
                .count()
        }

        fn nv_rs(&self) -> PathBuf {
            self.s.instance().join("nv-rs")
        }

        fn nothing_installed(&self) {
            assert!(!self.nv_rs().exists(), "nv-rs is not installed");
            assert!(!self.s.instance().join(".nv-rs-staging").exists());
        }
    }

    fn labels(events: &[Event]) -> Vec<&str> {
        events
            .iter()
            .filter_map(|e| match e {
                Event::Detail {
                    step: Step::Engine,
                    label,
                    ..
                } => Some(label.as_str()),
                _ => None,
            })
            .collect()
    }

    fn failure(result: Result<Outcome, Error>) -> Error {
        match result {
            Err(e) => e,
            other => panic!("expected the setup to fail, got {other:?}"),
        }
    }

    #[test]
    fn a_source_build_installs_checks_and_the_target_is_set() {
        let rig = Rig::new();
        let request = rig.source(&FakeNvRs::new());
        let (result, events) = record(&request, &CancelToken::new());
        let out = done(result);
        assert_eq!(
            ids(&out.completed),
            [
                "init", "profile", "engine", "target", "stage", "sort", "verify"
            ]
        );
        assert_eq!(
            labels(&events),
            ["setup.engine.building", "setup.engine.checked"]
        );
        let binary = rig.nv_rs().join("bin").join(nv_viewer_name());
        // The test runs the fake engine it just installed, as the integration tests of the built binaries do.
        #[allow(clippy::disallowed_methods)]
        let run = std::process::Command::new(&binary).output().expect("run");
        assert!(String::from_utf8_lossy(&run.stdout).contains(FAKE_BUILT_BANNER));
        let record = fs::read_to_string(rig.nv_rs().join("install.json")).expect("record");
        assert!(record.contains("\"method\": \"source\""), "{record}");
        let config =
            fs::read_to_string(rig.s.instance().join("nvmod-instance.json")).expect("instance");
        assert!(config.contains("nv-rs"), "{config}");
    }

    #[test]
    fn a_verified_release_installs_and_the_attestation_is_asked_for() {
        let rig = Rig::new();
        let request = rig.release(&FakeRelease::new(), 0);
        let (result, events) = record(&request, &CancelToken::new());
        let out = done(result);
        assert!(ids(&out.completed).contains(&"engine".to_owned()));
        assert_eq!(
            labels(&events),
            ["setup.engine.fetching", "setup.engine.checked"]
        );
        let binary = rig.nv_rs().join("bin").join(nv_viewer_name());
        assert_eq!(fs::read(binary).expect("binary"), fake_release_binary());
        assert_eq!(rig.gh_calls(), 1);
    }

    #[test]
    fn no_toolchain_fetches_the_release_without_being_asked() {
        let rig = Rig::new();
        let mut request = rig.release(&FakeRelease::new(), 0);
        request.engine_options.release = false;
        request.engine_options.cargo = Some(rig.tools.path().join("no-such-cargo"));
        let (result, events) = record(&request, &CancelToken::new());
        done(result);
        assert_eq!(
            labels(&events),
            ["setup.engine.fetching", "setup.engine.checked"]
        );
    }

    #[test]
    fn a_wrong_checksum_is_unverified_and_nothing_is_installed() {
        let rig = Rig::new();
        let request = rig.release(&FakeRelease::new().wrong_checksum(), 0);
        let (result, events) = record(&request, &CancelToken::new());
        let error = failure(result);
        assert_eq!(error.kind, ErrorKind::Unverified);
        assert_eq!(error.step, Step::Engine);
        assert_eq!(error.job, "engine.install");
        assert!(error.summary.contains("checksum"), "{}", error.summary);
        rig.nothing_installed();
        assert_eq!(rig.gh_calls(), 0, "no attestation for a failed checksum");
        assert!(!started(&events).contains(&"target".to_owned()));
    }

    #[test]
    fn a_failed_attestation_is_unverified_and_nothing_is_installed() {
        let rig = Rig::new();
        let request = rig.release(&FakeRelease::new(), 1);
        let error = failure(record(&request, &CancelToken::new()).0);
        assert_eq!(error.kind, ErrorKind::Unverified);
        assert!(error.summary.contains("attestation"), "{}", error.summary);
        rig.nothing_installed();
    }

    #[test]
    fn an_asset_missing_from_the_checksum_list_is_unverified() {
        let rig = Rig::new();
        let request = rig.release(&FakeRelease::new().asset_not_listed(), 0);
        let error = failure(record(&request, &CancelToken::new()).0);
        assert_eq!(error.kind, ErrorKind::Unverified);
        assert!(error.summary.contains(FAKE_ASSET), "{}", error.summary);
        rig.nothing_installed();
    }

    #[test]
    fn a_failing_check_fails_the_step_and_is_rolled_back() {
        let rig = Rig::new();
        let request = rig.source(&FakeNvRs::new().failing_check());
        let (result, events) = record(&request, &CancelToken::new());
        let error = failure(result);
        assert_eq!(error.kind, ErrorKind::Failed);
        assert!(
            error.summary.contains(FAKE_CHECK_REFUSED),
            "{}",
            error.summary
        );
        rig.nothing_installed();
        assert_eq!(labels(&events), ["setup.engine.building"]);
    }

    #[test]
    fn a_failing_release_check_fails_the_step_and_is_rolled_back() {
        let rig = Rig::new();
        let request = rig.release(&FakeRelease::new().failing_check(), 0);
        let error = failure(record(&request, &CancelToken::new()).0);
        assert_eq!(error.kind, ErrorKind::Failed);
        assert!(error.summary.contains(FAKE_CHECK_REFUSED));
        rig.nothing_installed();
    }

    #[test]
    fn a_failing_source_build_fails_the_step_and_installs_nothing() {
        let rig = Rig::new();
        let request = rig.source(&FakeNvRs::new().failing_build());
        let error = failure(record(&request, &CancelToken::new()).0);
        assert_eq!(error.kind, ErrorKind::Failed);
        assert_ne!(error.summary, "");
        rig.nothing_installed();
    }

    #[test]
    fn a_toolchain_and_no_source_is_refused_before_anything_is_written() {
        let rig = Rig::new();
        let mut request = rig.s.request();
        request.engine = true;
        request.engine_options.cargo = Some(PathBuf::from(CARGO));
        let error = failure(record(&request, &CancelToken::new()).0);
        assert_eq!(error.kind, ErrorKind::Refused);
        assert_eq!(error.step, Step::Engine);
        rig.nothing_installed();
    }

    #[test]
    fn an_engine_already_installed_is_refused_and_left_alone() {
        let rig = Rig::new();
        let request = rig.source(&FakeNvRs::new());
        done(record(&request, &CancelToken::new()).0);
        let mut again = request.clone();
        again.done = vec![Step::Init, Step::Profile];
        let error = failure(record(&again, &CancelToken::new()).0);
        assert_eq!(error.kind, ErrorKind::Refused);
        assert!(error.summary.contains("already installed"));
        assert!(rig.nv_rs().join("bin").join(nv_viewer_name()).is_file());
    }

    #[test]
    fn a_stop_asked_for_when_the_step_reports_its_work_installs_nothing() {
        let rig = Rig::new();
        let request = rig.source(&FakeNvRs::new());
        let cancel = CancelToken::new();
        let mut events = Vec::new();
        let result = run(
            &request,
            &mut |e: &Event| {
                if matches!(e, Event::Detail { label, .. } if label == "setup.engine.building") {
                    cancel.cancel();
                }
                events.push(e.clone());
            },
            &cancel,
        );
        let Ok(Outcome::Cancelled(out)) = result else {
            panic!("expected a cancelled setup, got {result:?}");
        };
        assert_eq!(ids(&out.completed), ["init", "profile"]);
        assert_eq!(labels(&events), ["setup.engine.building"]);
        assert!(!events.iter().any(|e| matches!(
            e,
            Event::Finished {
                step: Step::Engine,
                ..
            }
        )));
        rig.nothing_installed();
    }

    #[test]
    fn a_request_that_does_not_ask_for_the_engine_skips_the_step_and_installs_nothing() {
        let rig = Rig::new();
        let mut request = rig.source(&FakeNvRs::new());
        request.engine = false;
        let (result, events) = record(&request, &CancelToken::new());
        let out = done(result);
        assert!(!ids(&out.completed).contains(&"engine".to_owned()));
        assert_eq!(labels(&events), [] as [&str; 0]);
        rig.nothing_installed();
    }
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
