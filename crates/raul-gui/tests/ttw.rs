// SPDX-License-Identifier: GPL-3.0-only
//! The TTW request end to end: `raul_ui_service::CliService` runs the release `raul ttw --json` with ttwright installed
//! in process, on synthetic fixtures from `ttwright-testkit`. It reads its `raul-events` stream and its
//! `raul-ttw-result`, and feeds the job table. The fixtures are made up; no game data is involved.
//!
//! The TTW flow page is driven the same way: through the window, with the sources set by `raul config`, Next to the
//! review (a dry run that writes nothing) and Install, which adds the Mod to the Instance.
#![cfg(unix)]

use eframe::egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use nvmod_testkit::{BaseBuilder, TempDir};
use raul_gui::jobs::Jobs;
use raul_gui::pages::ttw::{
    CANCEL, INSTALL, INSTALL_CANCELLED, INSTALL_FAILED, INSTALL_UNKNOWN, INSTALLED_STATUS, NEXT,
    RESUME, TRY_AGAIN,
};
use raul_gui::shell::App;
use raul_ui_model::events::Status;
use raul_ui_model::job::{JobEvent, JobId, JobState, JobTable, Outcome};
use raul_ui_model::pages::{Flow, View};
use raul_ui_service::registry::Action;
use raul_ui_service::{CancelHandle, CliService, JobSpec, ProcessExec, Service, TtwRequest};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use ttwright_testkit::{Manifest, loc, mpi_package};

/// The package's own folder name inside the Instance's `mods`.
const MOD: &str = "Tale of Two Wastelands";

/// The fixture manifest plus two plugins copied from the game copy, so the Mod has plugins to enable.
fn manifest() -> Manifest {
    let mut m = ttwright_testkit::Fixture::manifest();
    m.asset(
        0,
        "",
        loc::GAME_DATA,
        loc::DEST,
        "loose.txt",
        Some("Example.esm"),
    )
    .asset(
        0,
        "",
        loc::GAME_DATA,
        loc::DEST,
        "loose.txt",
        Some("Example - Patch.esp"),
    );
    m
}

/// The same package with many small copies, so an install runs long enough to be stopped while it works.
fn slow_manifest() -> Manifest {
    let mut m = ttwright_testkit::Fixture::manifest();
    for i in 0..4000 {
        m.asset(
            0,
            "",
            loc::GAME_DATA,
            loc::DEST,
            "loose.txt",
            Some(&format!("slow/{i:04}.txt")),
        );
    }
    m
}

/// The package with a version this Toolbox does not support.
fn unsupported_manifest() -> Manifest {
    let mut m = manifest();
    m.set(
        "Package",
        json!({"Title": MOD, "Version": "9.9.9", "Author": "synthetic"}),
    );
    m
}

struct Fixture {
    root: TempDir,
    /// The ttwright fixture: a Fallout 3 copy, the `.mpi` package and a destination folder.
    ttw: ttwright_testkit::Fixture,
}

impl Fixture {
    fn new(label: &str) -> Fixture {
        Self::with_manifest(label, &manifest())
    }

    fn with_manifest(label: &str, manifest: &Manifest) -> Fixture {
        let f = Fixture {
            root: TempDir::new(label),
            ttw: ttwright_testkit::Fixture::with_manifest(label, manifest),
        };
        BaseBuilder::steam()
            .exe_version([1, 4, 0, 525])
            .write(&f.path("FalloutNV"));
        f.raul_ok(&[
            "init",
            "--base",
            &f.s("FalloutNV"),
            "--instance",
            &f.s("instance"),
        ]);
        f.raul_ok(&["profile", "create", "--instance", &f.s("instance")]);
        f
    }

    /// `raul` with this fixture's own configuration and state, so nothing reaches the real home.
    fn exec(&self) -> ProcessExec {
        ProcessExec::new(env!("CARGO_BIN_EXE_raul"))
            .with_env("RAUL_CONFIG_DIR", &self.s("config"))
            .with_env("NVMOD_STATE_DIR", &self.s("state"))
    }

    fn path(&self, p: &str) -> PathBuf {
        self.root.path().join(p)
    }

    fn s(&self, p: &str) -> String {
        self.path(p).to_string_lossy().into_owned()
    }

    fn raul_ok(&self, args: &[&str]) {
        use raul_ui_service::Exec;
        let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
        let out = self.exec().run(&args).unwrap();
        assert_eq!(out.code, Some(0), "{args:?}: {}", out.stderr);
    }

    fn service(&self) -> CliService<ProcessExec> {
        CliService::new(self.exec().with_grace(Duration::from_secs(5)))
    }

    /// A request with every input given.
    fn request(&self) -> TtwRequest {
        TtwRequest {
            instance: Some(self.s("instance")),
            fallout3: Some(self.ttw.fallout3.to_string_lossy().into_owned()),
            package: Some(self.ttw.mpi.to_string_lossy().into_owned()),
            ..TtwRequest::default()
        }
    }

    /// The folder ttwright writes into, kept between a cancelled run and `--resume`.
    fn work(&self) -> PathBuf {
        self.path("instance/.raul/ttw")
    }

    /// A game file the check does not expect: ttwright's check fails before anything is written.
    fn break_check(&self) {
        std::fs::write(
            self.ttw.fallout3.join("Data/loose.txt"),
            b"not the expected file",
        )
        .unwrap();
    }

    /// Replaces the package with one whose version is not supported.
    fn unsupported_package(&self) {
        std::fs::write(
            &self.ttw.mpi,
            mpi_package(&unsupported_manifest().build(), &[]),
        )
        .unwrap();
    }

    fn installed(&self) -> bool {
        self.path("instance/mods").join(MOD).exists()
    }
}

#[test]
fn a_successful_install_is_a_report_with_the_mod() {
    let f = Fixture::new("gui-ttw-ok");
    let report = f.service().ttw(&f.request()).unwrap();
    assert_eq!(report.status, Status::Success);
    assert_eq!(report.mod_name.as_deref(), Some(MOD));
    assert!(
        report.plugins_enabled.iter().any(|p| p == "Example.esm"),
        "{report:?}"
    );
    assert!(report.error.is_none() && !report.resumable && !report.dry_run);
    assert_eq!(
        report.tool.as_ref().map(|t| t.name.as_str()),
        Some("ttwright")
    );
    assert!(
        f.path("instance/mods")
            .join(MOD)
            .join("Example.esm")
            .is_file()
    );
}

#[test]
fn a_dry_run_writes_nothing() {
    let f = Fixture::new("gui-ttw-dry");
    let report = f
        .service()
        .ttw(&TtwRequest {
            dry_run: true,
            ..f.request()
        })
        .unwrap();
    assert!(report.dry_run, "{report:?}");
    assert_eq!(report.status, Status::Success);
    assert!(report.mod_name.is_none());
    assert!(!f.installed());
    assert!(!f.work().exists());
}

#[test]
fn a_failure_or_a_refusal_is_a_report_that_says_why() {
    // A game copy the check rejects: ttwright fails before it writes anything.
    let f = Fixture::new("gui-ttw-fail");
    f.break_check();
    let report = f.service().ttw(&f.request()).unwrap();
    assert_eq!(report.status, Status::Failed, "{report:?}");
    assert_eq!(
        report.error.as_ref().map(|e| e.code.as_str()),
        Some("ttwright_failed")
    );
    assert!(report.mod_name.is_none());
    assert!(!f.installed());

    // A package whose version is not supported: refused before anything is written.
    let f = Fixture::with_manifest("gui-ttw-unsupported", &unsupported_manifest());
    let report = f.service().ttw(&f.request()).unwrap();
    assert_eq!(report.status, Status::Unsupported, "{report:?}");
    assert_eq!(
        report.error.as_ref().map(|e| e.code.as_str()),
        Some("unsupported")
    );
    assert!(report.mod_name.is_none());
    assert!(!f.installed());

    // Nothing given and nothing configured: refused before ttwright starts.
    let f = Fixture::new("gui-ttw-missing");
    let report = f
        .service()
        .ttw(&TtwRequest {
            instance: Some(f.s("instance")),
            ..TtwRequest::default()
        })
        .unwrap();
    assert_eq!(report.status, Status::Refused);
    assert_eq!(
        report.error.map(|e| e.code),
        Some("missing_input".to_owned())
    );
    assert!(report.tool.is_none());
}

#[test]
fn a_ttw_job_streams_its_phases_and_ends_as_the_stream_says() {
    let f = Fixture::new("gui-ttw-job");
    let mut table = JobTable::default();
    let mut events = Vec::new();
    let spec = JobSpec::new(JobId(1), Action::Ttw, f.request().args()).writing();
    f.service()
        .run_job(&spec, &mut |e| {
            events.push(e.clone());
            table.apply(e);
        })
        .unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, JobEvent::Progress { phase, .. } if phase == "assets")),
        "{events:?}"
    );
    assert_eq!(
        table.get(JobId(1)).map(|j| j.state),
        Some(JobState::Finished(Outcome::Completed))
    );
}

#[test]
fn a_cancelled_ttw_job_stops_the_install() {
    let f = Fixture::with_manifest("gui-ttw-cancel", &slow_manifest());
    let mut table = JobTable::default();
    let cancel = CancelHandle::new();
    let asker = cancel.clone();
    let spec = JobSpec::new(JobId(1), Action::Ttw, f.request().args()).writing();
    let started = Instant::now();
    f.service()
        .run_job_cancellable(
            &spec,
            &mut |e| {
                if matches!(&e, JobEvent::Progress { phase, .. } if phase == "assets") {
                    asker.request();
                }
                table.apply(e);
            },
            &cancel,
        )
        .unwrap();
    assert!(
        started.elapsed() < Duration::from_secs(25),
        "{:?}",
        started.elapsed()
    );
    let state = table.get(JobId(1)).map(|j| j.state);
    // A cancel ends as `cancelled` on every path in the driver (`cancelled before`, `cancelled while adding`), never
    // as a rollback or a failure.
    assert_eq!(state, Some(JobState::Finished(Outcome::Cancelled)));
    assert!(!f.installed(), "the cancelled install added the Mod");
}

impl Fixture {
    /// The fixture as the window sees it: the storage folder holds the Instance, the sources are set with
    /// `raul config`, and the window gets the same program with the same folders.
    fn for_window(label: &str, manifest: &Manifest) -> Fixture {
        let f = Fixture::with_manifest(label, manifest);
        let storage = f.root.path().to_string_lossy().into_owned();
        f.raul_ok(&["config", "set", "storage.root", &storage]);
        f.raul_ok(&[
            "config",
            "set",
            "game.fallout3",
            &f.ttw.fallout3.to_string_lossy(),
        ]);
        f.raul_ok(&["config", "set", "ttw.package", &f.ttw.mpi.to_string_lossy()]);
        f
    }

    fn window(&self) -> Harness<'static, App> {
        let exec = self.exec().with_grace(Duration::from_secs(5));
        let app = App::new(Jobs::new(Arc::new(exec)).with_config_folder(&self.path("config")));
        let mut h = Harness::builder()
            .with_size([1280.0, 800.0])
            .build_eframe(|cc| {
                raul_gui::set_up(&cc.egui_ctx);
                app
            });
        settle(&mut h);
        h.state_mut().open(View::Flow(Flow::Ttw));
        settle(&mut h);
        h
    }
}

/// Runs frames until the jobs the page starts have ended and it has taken their answers.
fn settle(h: &mut Harness<'static, App>) {
    for _ in 0..6 {
        h.run_steps(8);
        assert!(h.state_mut().wait_for_jobs(Duration::from_secs(60)));
    }
    h.run_steps(8);
}

/// Next on Sources and Output, which starts the review's dry run.
fn to_review(h: &mut Harness<'static, App>) {
    for _ in 0..2 {
        h.get_by_role_and_label(Role::Button, NEXT).click();
        settle(h);
    }
}

/// Runs frames until ttwright has written its first asset (not just the journal that comes before it). The slow
/// manifest has thousands more, so the install is still running when the caller clicks.
fn until_writing(h: &mut Harness<'static, App>, f: &Fixture) {
    let out = f.work().join("out");
    let started = Instant::now();
    while written_assets(&out) < 1 {
        assert!(
            started.elapsed() < Duration::from_secs(20),
            "ttwright never started writing"
        );
        h.run_steps(1);
        std::thread::sleep(Duration::from_millis(5));
    }
    h.run_steps(2);
}

/// The files ttwright has written under `dir`: all of them but its journal, which comes before the first asset.
fn written_assets(dir: &Path) -> usize {
    count_files(dir).saturating_sub(usize::from(dir.join(".ttwright-journal.json").is_file()))
}

/// Files under `dir`, counting links as one file.
fn count_files(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| match e.file_type() {
            Ok(t) if t.is_dir() => count_files(&e.path()),
            _ => 1,
        })
        .sum()
}

#[test]
fn the_flow_reviews_with_a_dry_run_then_installs_the_mod() {
    let f = Fixture::for_window("gui-ttw-flow", &manifest());
    let mut h = f.window();
    to_review(&mut h);
    assert!(!f.installed(), "the review writes nothing");
    assert!(!f.work().exists(), "the review's dry run wrote nothing");
    h.get_by_role_and_label(Role::Button, INSTALL).click();
    settle(&mut h);
    assert_eq!(
        h.state_mut().shell_mut().status.as_deref(),
        Some(INSTALLED_STATUS)
    );
    assert!(
        f.path("instance/mods")
            .join(MOD)
            .join("Example.esm")
            .is_file()
    );
    assert!(
        h.query_by_label("Example.esm").is_some(),
        "the plugins are listed"
    );
}

#[test]
fn a_refused_install_says_why_and_tries_again_from_the_review() {
    let f = Fixture::for_window("gui-ttw-flow-refused", &manifest());
    let mut h = f.window();
    to_review(&mut h);
    // The package turns unsupported after the review, so the install refuses it.
    let good = std::fs::read(&f.ttw.mpi).unwrap();
    f.unsupported_package();
    h.get_by_role_and_label(Role::Button, INSTALL).click();
    settle(&mut h);
    assert!(h.query_by_label(INSTALL_FAILED.what).is_some());
    assert!(!f.installed());
    std::fs::write(&f.ttw.mpi, &good).unwrap();
    h.get_by_label(TRY_AGAIN).click();
    settle(&mut h);
    assert!(h.query_by_label(INSTALL_FAILED.what).is_none());
    h.get_by_role_and_label(Role::Button, INSTALL).click();
    settle(&mut h);
    assert!(f.installed(), "checked again, then installed");
}

#[test]
fn a_stopped_install_resumes_where_it_left_off() {
    let f = Fixture::for_window("gui-ttw-flow-resume", &slow_manifest());
    let mut h = f.window();
    to_review(&mut h);
    h.get_by_role_and_label(Role::Button, INSTALL).click();
    until_writing(&mut h, &f);
    h.get_by_role_and_label(Role::Button, CANCEL).click();
    settle(&mut h);
    assert!(!f.installed(), "the Instance is as it was");
    assert!(
        f.work().join("out/.ttwright-journal.json").is_file(),
        "the journal is kept for --resume"
    );
    h.get_by_label(RESUME).click();
    settle(&mut h);
    // Back on the review, whose one button now continues the stopped run.
    h.get_by_role_and_label(Role::Button, RESUME).click();
    settle(&mut h);
    assert!(f.installed(), "resumed and installed");
    assert!(!f.work().exists(), "the staging folder is removed");
}

#[test]
fn cancel_stops_a_running_install_and_the_instance_stays_as_it_was() {
    let f = Fixture::for_window("gui-ttw-flow-cancel", &slow_manifest());
    let mut h = f.window();
    to_review(&mut h);
    let started = Instant::now();
    h.get_by_role_and_label(Role::Button, INSTALL).click();
    until_writing(&mut h, &f);
    h.get_by_role_and_label(Role::Button, CANCEL).click();
    settle(&mut h);
    assert!(
        started.elapsed() < Duration::from_secs(25),
        "{:?}",
        started.elapsed()
    );
    assert!(!f.installed());
    assert!(
        [
            INSTALL_CANCELLED.what,
            INSTALL_FAILED.what,
            INSTALL_UNKNOWN.what
        ]
        .iter()
        .any(|w| h.query_by_label(w).is_some()),
        "the page says how it ended"
    );
}
