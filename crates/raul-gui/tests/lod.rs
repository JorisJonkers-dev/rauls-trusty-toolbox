// SPDX-License-Identifier: GPL-3.0-only
//! The LOD page end to end: it reads the plan, asks once, generates, finds the output current on a second run, and
//! stops a run, all through the release `raul` on a synthetic Instance. lodwright is a fake shell script: a wrapper
//! passes it to `raul lod generate` where the page uses the bundled one, so no real generator runs.
#![cfg(unix)]

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use nvmod_testkit::{BaseBuilder, TempDir, write_script};
use raul_gui::catalogue as msg;
use raul_gui::jobs::Jobs;
use raul_gui::pages::lod::{
    CURRENT_STATUS, DONE_STATUS, GENERATE, STATE_CURRENT, STATE_GENERATING_START, STOP,
    STOPPED_STATUS,
};
use raul_gui::shell::App;
use raul_ui_model::pages::lod::MOD_NAME;
use raul_ui_service::{Exec, ProcessExec};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// The fake lodwright: in mode `ok` it writes a small output with a manifest, and a folder that already has one is
/// up to date; in mode `slow` it waits to be stopped.
const FAKE: &str = r#"
here=$(dirname "$0")
mode=$(cat "$here/mode")
req=""; events=""
while [ $# -gt 0 ]; do
  case "$1" in
    --request) req="$2"; shift ;;
    --events) events="$2"; shift ;;
  esac
  shift
done
out=$(sed -n 's/.*"output_dir": *"\([^"]*\)".*/\1/p' "$req")
result() { printf '{"schema_version":1,"contract":"lodwright-result","tool":{"name":"lodwright","version":"0.3.0"},"status":"%s","code":"%s","reason":"fake"}\n' "$1" "$2"; }
case "$mode" in
  ok)
    if [ -f "$out/manifest.json" ]; then result ok up_to_date; exit 0; fi
    mkdir -p "$out/meshes/landscape/lod"
    echo synthetic > "$out/meshes/landscape/lod/block.nif"
    echo '{"event":"block","world":"SyntheticWorld","level":4,"done":1,"of":1,"up_to_date":false}' >> "$events"
    printf '{"schema_version":1,"contract":"lodwright-manifest","files":[]}\n' > "$out/manifest.json"
    result ok ok ;;
  slow) exec sleep 30 ;;
esac
"#;

struct Fixture {
    root: TempDir,
}

impl Fixture {
    fn new() -> Fixture {
        let f = Fixture {
            root: TempDir::new("gui-lod"),
        };
        BaseBuilder::steam()
            .exe_version([1, 4, 0, 525])
            .write(&f.path("base"));
        std::fs::create_dir_all(f.path("tools")).unwrap();
        write_script(&f.path("tools"), "lodwright", FAKE);
        f.mode("ok");
        // `raul` with this fixture's configuration and state, and the fake lodwright for `lod generate`.
        write_script(
            f.root.path(),
            "raul",
            &format!(
                "export RAUL_CONFIG_DIR='{c}' NVMOD_STATE_DIR='{s}'\nif [ \"$1 $2\" = 'lod generate' ]; then set -- \"$@\" --lodwright '{l}'; fi\nexec '{raul}' \"$@\"",
                c = f.path("config").display(),
                s = f.path("state").display(),
                l = f.path("tools/lodwright").display(),
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

    fn mode(&self, mode: &str) {
        std::fs::write(self.path("tools/mode"), mode).unwrap();
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
        assert!(h.state_mut().wait_for_jobs(Duration::from_secs(60)));
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
    let f = Fixture::new();
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

#[test]
fn stop_ends_the_run_and_adds_nothing() {
    let f = Fixture::new();
    f.mode("slow");
    let mut h = f.app();
    generate(&mut h);
    // Frames until the run has started, for up to 30 seconds; the fake waits to be stopped.
    for _ in 0..300 {
        h.run_steps(2);
        if h.query_by_label(STOP).is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(
        // The header and the status line can both say it, so ask for every match.
        h.query_all_by_label_contains(STATE_GENERATING_START)
            .next()
            .is_some(),
        "the run shows while it goes"
    );
    h.get_by_label(STOP).click();
    settle(&mut h);
    assert_eq!(status(&mut h).as_deref(), Some(STOPPED_STATUS));
    assert!(!f.lod_mod().exists(), "a stopped run adds no Mod");
    assert!(!f.modlist().contains(MOD_NAME), "{}", f.modlist());
    assert!(h.query_by_label(GENERATE).is_some(), "it can run again");
}
