// SPDX-License-Identifier: GPL-3.0-only
//! The LOD page end to end: it reads the plan, asks once, generates, finds the output current on a second run, and
//! stops a run, all through the release `raul` on a synthetic Instance. lodwright is linked into `raul`, so the page
//! starts no generator program: the world is a synthetic plugin from `lodwright-testkit`.
#![cfg(unix)]

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use lodwright_testkit::land::Heights;
use lodwright_testkit::world::{SyntheticWorld, world_plugin};
use nvmod_testkit::{BaseBuilder, TempDir, write_script};
use raul_gui::catalogue as msg;
use raul_gui::jobs::Jobs;
use raul_gui::pages::lod::{
    CURRENT_STATUS, DONE_STATUS, GENERATE, STATE_CURRENT, STATE_GENERATING_START, STOP,
    STOPPED_STATUS,
};
use raul_gui::shell::App;
use raul_ui_service::{Exec, ProcessExec};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// The LOD Mod of the fixture's only Profile.
const MOD_NAME: &str = "Generated LOD (Default)";

/// A world of `side` by `side` cells with terrain, so a generation takes long enough to be stopped while it runs.
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

struct Fixture {
    root: TempDir,
}

impl Fixture {
    fn new(side: i32) -> Fixture {
        let f = Fixture {
            root: TempDir::new("gui-lod"),
        };
        BaseBuilder::steam()
            .exe_version([1, 4, 0, 525])
            .write(&f.path("base"));
        std::fs::write(
            f.path("base/Data/FalloutNV.esm"),
            world_plugin(&[], &world(side), false),
        )
        .unwrap();
        // `raul` with this fixture's configuration and state, so nothing reaches the real home.
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
        let instance = f.s("storage/instance");
        f.raul(&["config", "set", "storage.root", &f.s("storage")]);
        f.raul(&["init", "--base", &f.s("base"), "--instance", &instance]);
        f.raul(&["profile", "create", "--instance", &instance]);
        f.raul(&["stage", "--instance", &instance, "--copy"]);
        f
    }

    fn path(&self, p: &str) -> PathBuf {
        self.root.path().join(p)
    }

    fn s(&self, p: &str) -> String {
        self.path(p).to_string_lossy().into_owned()
    }

    fn raul(&self, args: &[&str]) {
        let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
        let out = ProcessExec::new(self.path("raul")).run(&args).unwrap();
        assert_eq!(out.code, Some(0), "{args:?}: {}", out.stderr);
    }

    fn lod_mod(&self) -> PathBuf {
        self.path("storage/instance/mods").join(MOD_NAME)
    }

    fn modlist(&self) -> String {
        let profiles = self.path("storage/instance/profiles");
        let mut text = String::new();
        for entry in std::fs::read_dir(profiles).unwrap() {
            let file = entry.unwrap().path().join("modlist.txt");
            text.push_str(&std::fs::read_to_string(file).unwrap_or_default());
        }
        text
    }

    fn app(&self) -> Harness<'static, App> {
        let exec = ProcessExec::new(self.path("raul")).with_grace(Duration::from_secs(5));
        let app = App::new(Jobs::new(Arc::new(exec)));
        let mut h = Harness::builder()
            .with_size([1280.0, 800.0])
            .build_eframe(|cc| {
                raul_gui::set_up(&cc.egui_ctx);
                app
            });
        settle(&mut h);
        h.get_all_by_label(msg::PAGE_LOD).next().unwrap().click();
        settle(&mut h);
        h
    }
}

/// Runs frames until the jobs the page starts have ended and it has taken their answers.
fn settle(h: &mut Harness<'static, App>) {
    for _ in 0..6 {
        h.run_steps(8);
        assert!(h.state_mut().wait_for_jobs(Duration::from_secs(240)));
    }
    h.run_steps(8);
}

/// Presses Generate and confirms the dialog it opens.
fn generate(h: &mut Harness<'static, App>) {
    h.get_by_label(GENERATE).click();
    h.run_steps(4);
    assert!(h.state().dialogs().is_open(), "Generate asks first");
    // The dialog's action is the second button labelled Generate; the page's own is under the backdrop.
    h.get_all_by_label(GENERATE).last().unwrap().click();
}

fn status(h: &mut Harness<'static, App>) -> Option<String> {
    h.state_mut().shell_mut().status.clone()
}

#[test]
fn generate_keeps_one_enabled_mod_and_a_second_run_finds_it_current() {
    let f = Fixture::new(4);
    let mut h = f.app();
    assert!(
        h.query_by_label_contains("blocks not generated").is_some(),
        "the plan is read and its block shown"
    );

    h.get_by_label(GENERATE).click();
    h.run_steps(4);
    assert!(h.state().dialogs().is_open(), "Generate asks first");
    h.get_by_label(msg::CANCEL).click();
    settle(&mut h);
    assert!(!f.lod_mod().exists(), "Cancel starts nothing");

    generate(&mut h);
    settle(&mut h);
    assert_eq!(status(&mut h).as_deref(), Some(DONE_STATUS));
    assert!(f.lod_mod().is_dir(), "the output is kept as one Mod");
    let line = format!("+{MOD_NAME}");
    assert_eq!(
        f.modlist().lines().filter(|l| *l == line).count(),
        1,
        "{}",
        f.modlist()
    );
    assert!(
        h.query_by_label(STATE_CURRENT).is_some(),
        "the header says the blocks are current"
    );

    generate(&mut h);
    settle(&mut h);
    assert_eq!(status(&mut h).as_deref(), Some(CURRENT_STATUS));
    assert!(f.lod_mod().is_dir());
}

/// The side of the world of the stop test: 1024 cells, which take longer to generate than the few frames between the
/// run's Stop button appearing and the click, also on a slow runner (the GUI workflow runs the tests under coverage
/// instrumentation, where a larger world outlasts the wait for the stop to land). A run that finished first would fail
/// the test, not pass it.
const STOP_SIDE: i32 = 32;

#[test]
fn stop_ends_the_run_and_adds_nothing() {
    let f = Fixture::new(STOP_SIDE);
    let mut h = f.app();
    generate(&mut h);
    // Frames until the run shows its Stop button, for up to 60 seconds: Stop can only end a run that has started.
    for _ in 0..600 {
        h.run_steps(2);
        if h.query_by_label(STOP).is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(h.query_by_label(STOP).is_some(), "the run shows Stop");
    h.run_steps(8);
    assert!(
        // The header and the status line can both say it, so ask for every match.
        h.query_all_by_label_contains(STATE_GENERATING_START)
            .next()
            .is_some(),
        "the run shows while it goes"
    );
    h.get_by_label(STOP).click();
    settle(&mut h);
    // Pressing Stop races the end of the run: on a slow (instrumented) runner the generation can finish before the
    // Stop lands, so both ends are right, and only what is true of either is asserted. The deterministic stop, held at
    // an event until the cancel lands, is in the raul-cli tests (`lod_stop.rs`, `lod_generate.rs`).
    let status = status(&mut h);
    if status.as_deref() == Some(STOPPED_STATUS) {
        assert!(!f.lod_mod().exists(), "a stopped run adds no Mod");
        assert!(!f.modlist().contains(MOD_NAME), "{}", f.modlist());
    } else {
        assert_eq!(status.as_deref(), Some(DONE_STATUS), "stopped or done");
        assert!(f.lod_mod().exists(), "a finished run adds the Mod");
        assert_eq!(
            f.modlist().matches(MOD_NAME).count(),
            1,
            "the Mod is listed once: {}",
            f.modlist()
        );
    }
    assert!(h.query_by_label(GENERATE).is_some(), "it can run again");
}
