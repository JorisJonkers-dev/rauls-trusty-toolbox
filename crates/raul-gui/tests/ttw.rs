// SPDX-License-Identifier: GPL-3.0-only
//! The TTW request end to end: `raul_ui_service::CliService` runs the release `raul ttw --json` through the
//! streaming Runner against a fake ttwright on synthetic fixtures, reads its `raul-events` stream and its
//! `raul-ttw-result`, and feeds the job table. The fake is a shell script, so these tests run on Linux and macOS.
//!
//! The TTW flow page is driven the same way: through the window, with the sources set by `raul config`, Next to the
//! review (a dry run that writes nothing) and Install, which adds the Mod to the Instance.
#![cfg(unix)]

use eframe::egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use nvmod_testkit::{BaseBuilder, PeBuilder, TempDir, write_file, write_script};
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
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// The fake ttwright of `crates/raul-cli/tests/ttw.rs`: it reads its mode from `mode` next to it and behaves as
/// ttwright v0.3.0 would.
const FAKE: &str = r#"
here=$(dirname "$0")
mode=$(cat "$here/mode")
echo "$@" > "$here/args"
echo $$ > "$here/pid"
dest=""; events=""; resume=false
while [ $# -gt 0 ]; do
  case "$1" in
    --dest) dest="$2"; shift ;;
    --events) events="$2"; shift ;;
    --resume) resume=true ;;
  esac
  shift
done
ev() { echo "{\"schema_version\":1,$1}" >> "$events"; }
result() { # result <status> <exit> <error code or empty> <schema version>
  if [ -n "$3" ]; then err="{\"code\":\"$3\",\"message\":\"fake $3\",\"feature\":null}"; else err=null; fi
  printf '{"schema_version":%s,"contract":"ttwright-result","tool":{"name":"ttwright","version":"0.3.0"},"status":"%s","exit_code":%s,"checks_passed":true,"assets":{"total":2,"done":2,"failed":0},"archives":{"total":0,"done":0,"failed":0},"post_install":{"total":0,"done":0,"failed":0},"resumed":%s,"dry_run":false,"elapsed_ms":5,"output_dir":"%s","error":%s}\n' "${4:-1}" "$1" "$2" "$resume" "$dest" "$err"
}
ev '"event":"started","package":"Synthetic","version":"3.4.0","assets_total":2,"resumed":false'
ev '"event":"phase","phase":"package"'
case "$mode" in
  success)
    ev '"event":"phase","phase":"assets"'
    ev '"event":"progress","phase":"assets","done":1,"of":2,"message":"first"'
    ev '"event":"warning","message":"a synthetic warning"'
    mkdir -p "$dest/textures"
    echo synthetic > "$dest/Example.esm"
    echo synthetic > "$dest/Example - Patch.esp"
    echo synthetic > "$dest/textures/a.dds"
    ev '"event":"progress","phase":"assets","done":2,"of":2'
    ev '"event":"finished","status":"success","code":null'
    result success 0 "" ;;
  failed) echo '{}' > "$dest/.ttwright-journal.json"; result failed 1 check_failed; exit 1 ;;
  usage) result failed 2 usage; exit 2 ;;
  unsupported) result unsupported 3 unsupported_version; exit 3 ;;
  cancelled) echo '{}' > "$dest/.ttwright-journal.json"; echo half > "$dest/half"; result cancelled 4 cancelled; exit 4 ;;
  newer) result success 0 "" 2 ;;
  garbage) echo "this is not a result" ;;
  empty) result success 0 "" ;;
  slow) echo '{}' > "$dest/.ttwright-journal.json"; ev '"event":"phase","phase":"assets"'; exec sleep 30 ;;
esac
"#;

struct Fixture {
    root: TempDir,
}

impl Fixture {
    fn new(label: &str) -> Fixture {
        let f = Fixture {
            root: TempDir::new(label),
        };
        BaseBuilder::steam()
            .exe_version([1, 4, 0, 525])
            .write(&f.path("FalloutNV"));
        write_file(
            &f.path("Fallout3"),
            "Fallout3.exe",
            &PeBuilder::exe().build(),
        );
        write_file(&f.path("Fallout3"), "Data/Fallout3.esm", b"synthetic");
        write_file(f.root.path(), "TTW.mpi", b"synthetic package");
        std::fs::create_dir_all(f.path("tools")).unwrap();
        write_script(&f.path("tools"), "ttwright", FAKE);
        // `raul` with this fixture's own configuration and state, so nothing reaches the real home.
        write_script(
            f.root.path(),
            "raul",
            &format!(
                "export RAUL_CONFIG_DIR='{c}' NVMOD_STATE_DIR='{s}'\nexec '{raul}' \"$@\"",
                c = f.path("config").display(),
                s = f.path("state").display(),
                raul = env!("CARGO_BIN_EXE_raul"),
            ),
        );
        let service = f.service();
        let _ = &service;
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

    fn path(&self, p: &str) -> PathBuf {
        self.root.path().join(p)
    }

    fn s(&self, p: &str) -> String {
        self.path(p).to_string_lossy().into_owned()
    }

    fn raul_ok(&self, args: &[&str]) {
        use raul_ui_service::Exec;
        let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
        let out = ProcessExec::new(self.path("raul")).run(&args).unwrap();
        assert_eq!(out.code, Some(0), "{args:?}: {}", out.stderr);
    }

    fn mode(&self, mode: &str) {
        std::fs::write(self.path("tools/mode"), mode).unwrap();
    }

    fn service(&self) -> CliService<ProcessExec> {
        CliService::new(ProcessExec::new(self.path("raul")).with_grace(Duration::from_secs(5)))
    }

    /// A request with every input given.
    fn request(&self) -> TtwRequest {
        TtwRequest {
            instance: Some(self.s("instance")),
            fallout3: Some(self.s("Fallout3")),
            package: Some(self.s("TTW.mpi")),
            ttwright: Some(self.s("tools/ttwright")),
            ..TtwRequest::default()
        }
    }
}

#[test]
fn a_successful_install_is_a_report_with_the_mod() {
    let f = Fixture::new("gui-ttw-ok");
    f.mode("success");
    let report = f.service().ttw(&f.request()).unwrap();
    assert_eq!(report.status, Status::Success);
    assert_eq!(report.mod_name.as_deref(), Some("Tale of Two Wastelands"));
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
        f.path("instance/mods/Tale of Two Wastelands/Example.esm")
            .is_file()
    );
}

#[test]
fn a_dry_run_writes_nothing() {
    let f = Fixture::new("gui-ttw-dry");
    f.mode("success");
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
    assert!(!f.path("instance/mods/Tale of Two Wastelands").exists());
}

#[test]
fn a_failure_or_a_refusal_is_a_report_that_says_why() {
    let f = Fixture::new("gui-ttw-fail");
    for (mode, status, code) in [
        ("failed", Status::Failed, "ttwright_failed"),
        ("unsupported", Status::Unsupported, "unsupported"),
        ("newer", Status::Refused, "contract_refused"),
    ] {
        f.mode(mode);
        let report = f.service().ttw(&f.request()).unwrap();
        assert_eq!(report.status, status, "{mode}: {report:?}");
        assert_eq!(
            report.error.as_ref().map(|e| e.code.as_str()),
            Some(code),
            "{mode}"
        );
        assert!(report.mod_name.is_none(), "{mode}");
    }
    // Nothing given and nothing configured: refused before ttwright starts.
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
    f.mode("success");
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
        events.iter().any(|e| matches!(e, JobEvent::Progress { phase, done: 2, total: Some(2), .. } if phase == "assets")),
        "{events:?}"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, JobEvent::Log { line, .. } if line.contains("warning")))
    );
    assert_eq!(
        table.get(JobId(1)).map(|j| j.state),
        Some(JobState::Finished(Outcome::Completed))
    );
}

#[test]
fn a_cancelled_ttw_job_stops_ttwright() {
    let f = Fixture::new("gui-ttw-cancel");
    f.mode("slow");
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
    assert!(
        matches!(
            state,
            Some(JobState::Finished(
                Outcome::Cancelled | Outcome::RolledBack | Outcome::Failed
            ))
        ),
        "{state:?}"
    );
    let pid: u32 = std::fs::read_to_string(f.path("tools/pid"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert!(
        !PathBuf::from(format!("/proc/{pid}")).exists() || cfg!(target_os = "macos"),
        "ttwright outlived the cancel"
    );
}

impl Fixture {
    /// The fixture as the window sees it: the storage folder holds the Instance, the sources are set with
    /// `raul config`, and a wrapper hands `raul ttw` the fake ttwright, which the page never names.
    fn for_window(label: &str) -> Fixture {
        let f = Fixture::new(label);
        let storage = f.root.path().to_string_lossy().into_owned();
        f.raul_ok(&["config", "set", "storage.root", &storage]);
        f.raul_ok(&["config", "set", "game.fallout3", &f.s("Fallout3")]);
        f.raul_ok(&["config", "set", "ttw.package", &f.s("TTW.mpi")]);
        write_script(
            f.root.path(),
            "raul-window",
            &format!(
                "if [ \"$1\" = ttw ]; then set -- \"$@\" --ttwright '{t}'; fi\nexec '{raul}' \"$@\"",
                t = f.s("tools/ttwright"),
                raul = f.s("raul"),
            ),
        );
        f
    }

    fn window(&self) -> Harness<'static, App> {
        let exec = ProcessExec::new(self.path("raul-window")).with_grace(Duration::from_secs(5));
        let app = App::new(Jobs::new(Arc::new(exec)));
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

    fn installed(&self) -> bool {
        self.path("instance/mods/Tale of Two Wastelands").exists()
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

#[test]
fn the_flow_reviews_with_a_dry_run_then_installs_the_mod() {
    let f = Fixture::for_window("gui-ttw-flow");
    f.mode("success");
    let mut h = f.window();
    to_review(&mut h);
    assert!(!f.installed(), "the review writes nothing");
    let args = std::fs::read_to_string(f.path("tools/args")).unwrap_or_default();
    assert!(args.is_empty() || args.contains("--dry-run"), "{args}");
    h.get_by_role_and_label(Role::Button, INSTALL).click();
    settle(&mut h);
    assert_eq!(
        h.state_mut().shell_mut().status.as_deref(),
        Some(INSTALLED_STATUS)
    );
    assert!(
        f.path("instance/mods/Tale of Two Wastelands/Example.esm")
            .is_file()
    );
    assert!(
        h.query_by_label("Example.esm").is_some(),
        "the plugins are listed"
    );
}

#[test]
fn a_refused_install_says_why_and_tries_again_from_the_review() {
    let f = Fixture::for_window("gui-ttw-flow-refused");
    f.mode("success");
    let mut h = f.window();
    to_review(&mut h);
    f.mode("unsupported");
    h.get_by_role_and_label(Role::Button, INSTALL).click();
    settle(&mut h);
    assert!(h.query_by_label(INSTALL_FAILED.what).is_some());
    assert!(!f.installed());
    f.mode("success");
    h.get_by_label(TRY_AGAIN).click();
    settle(&mut h);
    assert!(h.query_by_label(INSTALL_FAILED.what).is_none());
    h.get_by_role_and_label(Role::Button, INSTALL).click();
    settle(&mut h);
    assert!(f.installed(), "checked again, then installed");
}

#[test]
fn a_stopped_install_resumes_where_it_left_off() {
    let f = Fixture::for_window("gui-ttw-flow-resume");
    f.mode("success");
    let mut h = f.window();
    to_review(&mut h);
    f.mode("failed");
    h.get_by_role_and_label(Role::Button, INSTALL).click();
    settle(&mut h);
    assert!(h.query_by_label(INSTALL_FAILED.what).is_some());
    assert!(!f.installed(), "the Instance is as it was");
    f.mode("success");
    h.get_by_label(RESUME).click();
    settle(&mut h);
    h.get_by_role_and_label(Role::Button, INSTALL).click();
    settle(&mut h);
    let args = std::fs::read_to_string(f.path("tools/args")).unwrap();
    assert!(args.contains("--resume"), "{args}");
    assert!(f.installed());
}

#[test]
fn cancel_stops_a_running_install_and_the_instance_stays_as_it_was() {
    let f = Fixture::for_window("gui-ttw-flow-cancel");
    f.mode("success");
    let mut h = f.window();
    to_review(&mut h);
    f.mode("slow");
    let _ = std::fs::remove_file(f.path("tools/pid"));
    h.get_by_role_and_label(Role::Button, INSTALL).click();
    let started = Instant::now();
    while !f.path("tools/pid").exists() {
        assert!(
            started.elapsed() < Duration::from_secs(20),
            "ttwright never started"
        );
        h.run_steps(2);
        std::thread::sleep(Duration::from_millis(20));
    }
    h.run_steps(4);
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
