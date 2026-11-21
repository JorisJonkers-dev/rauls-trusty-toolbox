// SPDX-License-Identifier: GPL-3.0-only
//! The LOD generation and the TTW install as direct jobs (ADR-0016): lodwright and ttwright run in this process, their
//! progress reaches the job table as `raul-events` lines, and a Cancel reaches the run through the job's own token; the
//! job never sets the process-wide stop request. (The nvmod steps that add the Mod still read that request: nvmod-rs#212.) Synthetic fixtures from the Components' testkits; no game data.
#![cfg(unix)]

use lodwright_testkit::land::Heights;
use lodwright_testkit::world::{SyntheticWorld, world_plugin};
use nvmod_api::{CancelToken, NoProgress};
use nvmod_testkit::{BaseBuilder, TempDir};
use raul_core::instance_lock::InstanceGuard;
use raul_ui_model::job::{JobEvent, JobId, JobState, JobTable, Outcome};
use raul_ui_service::registry::Action;
use raul_ui_service::{CancelHandle, JobSpec, TtwRequest, direct, run_direct};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use ttwright_testkit::{Manifest, loc};

const LOD_MOD: &str = "Generated LOD (Default)";
const TTW_MOD: &str = "Tale of Two Wastelands";

/// The process-wide stop request is one flag, so the tests that set it take turns.
static FLAG: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn turn() -> std::sync::MutexGuard<'static, ()> {
    FLAG.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn s(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

/// Makes the Instance of a fixture with nvmod's own jobs: init, profile `Default`, a staging tree.
fn instance_from(base: &Path, root: &Path, stage: bool) {
    let (progress, cancel) = (&mut NoProgress, &CancelToken::new());
    nvmod_api::init::create(
        &nvmod_api::init::CreateRequest {
            instance: root.to_owned(),
            base: base.to_owned(),
        },
        progress,
        cancel,
    )
    .unwrap();
    nvmod_api::profile::create(
        &nvmod_api::profile::CreateRequest {
            instance: root.to_owned(),
            name: None,
        },
        progress,
        cancel,
    )
    .unwrap();
    if stage {
        nvmod_api::stage::stage(
            &nvmod_api::stage::StageRequest {
                instance: root.to_owned(),
                profile: None,
                copy: true,
                full: false,
            },
            progress,
            cancel,
        )
        .unwrap();
    }
}

/// Runs `job` as a writing job of `action`, asking to stop once `stop_at` says an event is the one to stop at.
fn run(
    action: Action,
    job: direct::DirectJob,
    stop_at: impl Fn(&JobEvent) -> bool,
) -> (JobTable, raul_ui_service::Output) {
    let mut table = JobTable::default();
    let cancel = CancelHandle::new();
    let asker = cancel.clone();
    let spec = JobSpec::new(JobId(1), action, Vec::new()).writing();
    let out = run_direct(
        &spec,
        job,
        &mut |event| {
            if stop_at(&event) {
                asker.request();
            }
            table.apply(event);
        },
        &cancel,
    );
    (table, out)
}

// The cell indexes are small and the heights are synthetic, so the casts cannot lose anything that matters.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss
)]
fn world(side: i32) -> SyntheticWorld {
    let mut world = SyntheticWorld::new(0xB00, "Island");
    for x in 0..side {
        for y in 0..side {
            let heights =
                Heights::from_fn(|i, j| ((x * 7 + y * 13 + i as i32 * 3 + j as i32) % 40) as f32);
            world = world.cell(x, y, heights);
        }
    }
    world
}

struct Lod {
    root: TempDir,
}

impl Lod {
    fn new(side: i32) -> Self {
        let f = Self {
            root: TempDir::new("direct-lod"),
        };
        let base = f.root.path().join("base");
        BaseBuilder::steam()
            .exe_version([1, 4, 0, 525])
            .write(&base);
        std::fs::write(
            base.join("Data/FalloutNV.esm"),
            world_plugin(&[], &world(side), false),
        )
        .unwrap();
        instance_from(&base, &f.instance(), true);
        f
    }

    fn instance(&self) -> PathBuf {
        self.root.path().join("instance")
    }

    fn job(&self) -> direct::DirectJob {
        let rest = ["--instance".to_owned(), s(&self.instance())];
        direct::for_command("lod-generate", &rest).expect("lod-generate is a direct job")
    }

    fn lod_mod(&self) -> PathBuf {
        self.instance().join("mods").join(LOD_MOD)
    }
}

#[test]
fn a_lod_run_reports_its_progress_and_adds_the_mod_in_process() {
    let _turn = turn();
    let f = Lod::new(2);
    let (table, out) = run(Action::Lod, f.job(), |_| false);
    assert_eq!(out.code, Some(0), "{out:?}");
    assert!(
        out.stdout.contains("\"event\":\"progress\"") || out.stdout.contains("\"event\":\"phase\""),
        "the stream carries its progress: {}",
        out.stdout
    );
    assert_eq!(
        table.get(JobId(1)).map(|j| j.state),
        Some(JobState::Finished(Outcome::Completed))
    );
    assert!(f.lod_mod().exists(), "the run adds the Mod");
}

#[test]
fn cancel_stops_a_lod_run_through_its_own_token_and_adds_nothing() {
    let _turn = turn();
    let f = Lod::new(32);
    let started = Instant::now();
    let (table, _) = run(Action::Lod, f.job(), |e| {
        matches!(e, JobEvent::Progress { .. })
    });
    let state = table.get(JobId(1)).map(|j| j.state);
    // Pressing Cancel races the end of a run on a slow runner: both ends are right, and each is asserted whole.
    match state {
        Some(JobState::Finished(Outcome::Cancelled)) => {
            assert!(!f.lod_mod().exists(), "a stopped run adds no Mod");
            assert!(started.elapsed() < Duration::from_secs(60));
        }
        Some(JobState::Finished(Outcome::Completed)) => assert!(f.lod_mod().exists()),
        other => panic!("a run ends stopped or done, not {other:?}"),
    }
    assert!(
        !nvmod_core::cancel::requested(),
        "the process-wide request was not used"
    );
}

#[test]
fn a_held_instance_lock_refuses_a_lod_run_before_it_starts() {
    let _turn = turn();
    let f = Lod::new(2);
    let held = InstanceGuard::acquire(&f.instance(), "test").unwrap();
    let (table, out) = run(Action::Lod, f.job(), |_| false);
    drop(held);
    assert_ne!(out.code, Some(0), "{out:?}");
    assert!(!f.lod_mod().exists());
    assert!(matches!(
        table.get(JobId(1)).map(|j| j.state),
        Some(JobState::Finished(Outcome::Refused))
    ));
}

/// The package with many small copies, so an install runs long enough to be stopped while it works.
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

struct Ttw {
    root: TempDir,
    ttw: ttwright_testkit::Fixture,
}

impl Ttw {
    fn with_manifest(label: &str, manifest: &Manifest) -> Self {
        let f = Self {
            root: TempDir::new(label),
            ttw: ttwright_testkit::Fixture::with_manifest(label, manifest),
        };
        let base = f.root.path().join("FalloutNV");
        BaseBuilder::steam()
            .exe_version([1, 4, 0, 525])
            .write(&base);
        instance_from(&base, &f.instance(), false);
        f
    }

    fn instance(&self) -> PathBuf {
        self.root.path().join("instance")
    }

    fn job(&self) -> direct::DirectJob {
        let request = TtwRequest {
            instance: Some(s(&self.instance())),
            fallout3: Some(s(&self.ttw.fallout3)),
            package: Some(s(&self.ttw.mpi)),
            ..TtwRequest::default()
        };
        direct::for_command("ttw", &request.args()).expect("ttw is a direct job")
    }

    fn installed(&self) -> bool {
        self.instance().join("mods").join(TTW_MOD).exists()
    }
}

#[test]
fn a_ttw_install_streams_its_phases_and_adds_the_mod_in_process() {
    let _turn = turn();
    let f = Ttw::with_manifest("direct-ttw-ok", &ttwright_testkit::Fixture::manifest());
    let (table, out) = run(Action::Ttw, f.job(), |_| false);
    assert_eq!(out.code, Some(0), "{out:?}");
    assert_eq!(
        table.get(JobId(1)).map(|j| j.state),
        Some(JobState::Finished(Outcome::Completed))
    );
    assert!(
        out.stdout.contains("\"phase\":\"assets\""),
        "{}",
        out.stdout
    );
    assert!(f.installed());
}

#[test]
fn cancel_stops_a_ttw_install_through_its_own_token_and_adds_nothing() {
    let _turn = turn();
    let f = Ttw::with_manifest("direct-ttw-cancel", &slow_manifest());
    let started = Instant::now();
    let (table, _) = run(
        Action::Ttw,
        f.job(),
        |e| matches!(e, JobEvent::Progress { phase, .. } if phase == "assets"),
    );
    assert!(
        started.elapsed() < Duration::from_secs(25),
        "{:?}",
        started.elapsed()
    );
    assert_eq!(
        table.get(JobId(1)).map(|j| j.state),
        Some(JobState::Finished(Outcome::Cancelled))
    );
    assert!(!f.installed(), "the cancelled install added the Mod");
    assert!(
        !nvmod_core::cancel::requested(),
        "the process-wide request was not used"
    );
}
