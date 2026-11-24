// SPDX-License-Identifier: GPL-3.0-only
//! `raul scene` and `raul bench game` on synthetic data: a fake viewer (a shell script) stands in for the game child. The
//! engine's scene library runs in raul's own process; the viewer records what it was started with and makes empty
//! "pictures" and the log lines of a soak, so what is checked is what raul resolves from the Profile: the Mods in
//! priority order, the Plugins in Load order, the variants, the output folder, the label and the cap, and that
//! `raul bench` itself is untouched. No game and no engine is involved.
// These tests start the built binary, which is not a tool call (see clippy.toml).
#![allow(clippy::disallowed_methods)]
#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use nvmod_testkit::{BaseBuilder, TempDir};

struct Fixture {
    root: TempDir,
}

const LOD_MOD: &str = "Generated LOD (Default)";

impl Fixture {
    fn new(label: &str, with_lod: bool) -> Fixture {
        let f = Fixture {
            root: TempDir::new(label),
        };
        BaseBuilder::steam().write(&f.path("FalloutNV"));
        let (base, inst) = (f.path("FalloutNV"), f.path("instance"));
        for args in [
            &["init", "--base", s(&base), "--instance", s(&inst)][..],
            &["profile", "create", "--instance", s(&inst)],
            &["mods", "create", "Textures", "--instance", s(&inst)],
        ] {
            let out = f.raul(args, &[]);
            assert!(out.status.success(), "{args:?}: {out:?}");
        }
        let textures = inst.join("mods").join("Textures").join("textures");
        std::fs::create_dir_all(&textures).unwrap();
        std::fs::write(textures.join("t.dds"), "x").unwrap();
        if with_lod {
            let out = f.raul(&["mods", "create", LOD_MOD, "--instance", s(&inst)], &[]);
            assert!(out.status.success(), "{out:?}");
            std::fs::write(
                inst.join("mods").join(LOD_MOD).join("meta.ini"),
                "[raulGeneratedLod]\ngenerated=true\nprofile=\"Default\"\nrequestDigest=\"x\"\nlodwrightVersion=\"0.15.1\"\n",
            )
            .unwrap();
            let lod = inst.join("mods").join(LOD_MOD).join("meshes");
            std::fs::create_dir_all(&lod).unwrap();
            std::fs::write(lod.join("lod.nif"), "x").unwrap();
        }
        f
    }

    fn path(&self, p: &str) -> PathBuf {
        self.root.path().join(p)
    }

    fn raul(&self, args: &[&str], env: &[(&str, &str)]) -> Output {
        let mut c = Command::new(env!("CARGO_BIN_EXE_raul"));
        c.args(args)
            .env("RAUL_CONFIG_DIR", self.path("config"))
            .env("RAUL_DATA_DIR", self.path("data"))
            .env("NVMOD_STATE_DIR", self.path("state"))
            .env("HOME", self.path("home"));
        for (k, v) in env {
            c.env(k, v);
        }
        c.output().expect("raul runs")
    }

    /// A viewer that records its words and the folders it was given, one line a start, and plays the part of the
    /// engine: one picture, and the log lines of a soak for each place of the file.
    fn viewer(&self) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let png = self.path("one.png");
        std::fs::write(&png, fork_scene::png::encode_rgb(2, 2, &[7; 12]).unwrap()).unwrap();
        let file = self.path("fake-viewer");
        let log = self.path("viewer.log");
        let text = format!(
            "#!/bin/sh\necho \"$NV_SCENE_PLACES|$NV_SCENE|$NV_SCENE_OUT|$*\" >> '{log}'\n\
             if [ -n \"$NV_SCENE_OUT\" ]; then mkdir -p \"$NV_SCENE_OUT\" && cp '{png}' \"$NV_SCENE_OUT/goodsprings.png\"; fi\n\
             places=\"$NV_SCENE_PLACES\"\n\
             [ -z \"$places\" ] && places=$(sed -n 's/^name *= *\"\\(.*\\)\"/\\1/p' \"$NV_SCENE\" | tr '\\n' ',')\n\
             for p in $(echo \"$places\" | tr ',' ' '); do\n\
             echo '{{\"level\":\"info\",\"message\":\"scene: sending the player to '$p' from the Menu screen\"}}' >> \"$NV_LOG_JSON\"\n\
             echo '{{\"level\":\"info\",\"message\":\"scene: soak '$p' frames 10 avg_ms 5.00 p99_ms 6.00 max_ms 7.00 low1_fps 150.0\"}}' >> \"$NV_LOG_JSON\"\n\
             done\nexit ${{FAKE_EXIT:-0}}\n",
            log = log.display(),
            png = png.display()
        );
        std::fs::write(&file, text).unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();
        file
    }

    fn calls(&self) -> Vec<String> {
        std::fs::read_to_string(self.path("viewer.log"))
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }

    /// `raul scene <args>` with the fake viewer, the instance, a game folder and no card to read.
    fn scene(&self, args: &[&str], extra_env: &[(&str, &str)]) -> Output {
        let viewer = self.viewer();
        let inst = self.path("instance");
        let game = self.path("FalloutNV");
        let mut all = args.to_vec();
        all.extend_from_slice(&[
            "--instance",
            s(&inst),
            "--game",
            s(&game),
            "--viewer",
            s(&viewer),
            "--gpu-sysfs",
            "/nonexistent",
            "--no-cap",
        ]);
        self.raul(&all, extra_env)
    }

    /// The run's folder under raul's data folder (the one run of a test).
    fn run_dir(&self, who: &str) -> PathBuf {
        let parent = self.path("data/scene").join(who);
        let mut runs: Vec<PathBuf> = std::fs::read_dir(&parent)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        runs.sort();
        runs.pop().unwrap()
    }
}

fn s(p: &Path) -> &str {
    p.to_str().unwrap()
}

fn text(o: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

/// Whether the folder the game child was started in (the staged copy of the game) holds `rel` under Data.
fn staged_has(run: &Path, variant: &str, rel: &str) -> bool {
    run.join(variant).join("game/Data").join(rel).exists()
}

#[test]
fn a_profile_with_a_generated_lod_mod_gets_the_pair_and_the_difference() {
    let f = Fixture::new("scene-pair", true);
    let out = f.scene(&["scene", "run", "--places", "mojave-day"], &[]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let calls = f.calls();
    assert_eq!(
        calls.len(),
        2,
        "two starts, the comparison needs none: {calls:?}"
    );
    // The results are under raul's data folder, named by the Profile; the viewer got the staged game as its folder.
    let run = f.run_dir("Default");
    assert!(
        calls[0].contains(&format!("{}|", s(&run.join("mojave-day.toml")))),
        "{calls:?}"
    );
    assert!(
        calls[0].ends_with(s(&run.join("with-lod/game"))),
        "{calls:?}"
    );
    assert!(
        calls[1].ends_with(s(&run.join("without-lod/game"))),
        "{calls:?}"
    );
    // The Profile as it is lays both Mods over the game; without the LOD Mod only the other one.
    assert!(
        staged_has(&run, "with-lod", "textures/t.dds")
            && staged_has(&run, "with-lod", "meshes/lod.nif")
    );
    assert!(
        staged_has(&run, "without-lod", "textures/t.dds")
            && !staged_has(&run, "without-lod", "meshes/lod.nif")
    );
    let said = text(&out);
    assert!(said.contains("--- without-lod against with-lod"), "{said}");
    assert!(said.contains("Open both pictures"), "{said}");
}

#[test]
fn a_profile_without_a_lod_mod_gets_one_variant_and_no_comparison() {
    let f = Fixture::new("scene-single", false);
    let out = f.scene(
        &[
            "scene",
            "run",
            "--places",
            "mojave-night",
            "--only",
            "night-novac",
        ],
        &[],
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let calls = f.calls();
    assert_eq!(calls.len(), 1, "{calls:?}");
    assert!(calls[0].starts_with("night-novac|"), "{calls:?}");
    assert!(f.run_dir("Default").join("profile").is_dir());
    assert!(!text(&out).contains("against"), "{}", text(&out));
}

#[test]
fn named_variants_replace_the_default_and_a_bad_source_is_refused_before_any_start() {
    let f = Fixture::new("scene-named", true);
    let out = f.scene(
        &[
            "scene",
            "run",
            "--places",
            "mojave-day",
            "--variant",
            "plain=game",
            "--variant",
            "mine=profile",
        ],
        &[],
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let run = f.run_dir("Default");
    assert!(!staged_has(&run, "plain", "textures/t.dds"));
    assert!(staged_has(&run, "mine", "textures/t.dds"));
    let bad = f.scene(
        &[
            "scene",
            "run",
            "--places",
            "mojave-day",
            "--variant",
            "x=nonsense",
        ],
        &[],
    );
    assert_eq!(bad.status.code(), Some(2), "{}", text(&bad));
    assert!(
        text(&bad).contains("not a variant source"),
        "{}",
        text(&bad)
    );
    assert_eq!(f.calls().len(), 2, "the refusal started nothing");
}

/// Regression (#613): with an Instance and no `--game`, the game folder came from the `game.newvegas` setting, so a
/// Tale of Two Wastelands base was pictured as plain New Vegas. The base copy of the Instance is what `raul run` plays.
#[test]
fn without_game_the_run_pictures_the_instances_base_not_the_game_setting() {
    let f = Fixture::new("scene-base", false);
    let base_data = f.path("FalloutNV").join("Data");
    std::fs::write(base_data.join("TaleOfTwoWastelands.esm"), "x").unwrap();
    let other = f.path("PlainNV");
    BaseBuilder::steam().write(&other);
    std::fs::write(other.join("Data").join("OnlyInTheSetting.esm"), "x").unwrap();
    let set = f.raul(&["config", "set", "game.newvegas", s(&other)], &[]);
    assert!(set.status.success(), "{}", text(&set));
    let viewer = f.viewer();
    let out = f.raul(
        &[
            "scene",
            "run",
            "--places",
            "mojave-day",
            "--instance",
            s(&f.path("instance")),
            "--viewer",
            s(&viewer),
            "--gpu-sysfs",
            "/nonexistent",
            "--no-cap",
        ],
        &[],
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let run = f.run_dir("Default");
    assert!(
        staged_has(&run, "profile", "TaleOfTwoWastelands.esm"),
        "the base's plugin is in the pictured game"
    );
    assert!(
        !staged_has(&run, "profile", "OnlyInTheSetting.esm"),
        "the game.newvegas folder is not pictured"
    );
    // A soak takes the same game folder: the base's plugin is in the folder the game child is started in.
    let staged = f.raul(
        &["stage", "--instance", s(&f.path("instance")), "--copy"],
        &[],
    );
    assert!(staged.status.success(), "{staged:?}");
    let soak = f.raul(
        &[
            "scene",
            "soak",
            "--places",
            "mojave-day",
            "--instance",
            s(&f.path("instance")),
            "--viewer",
            s(&viewer),
            "--gpu-sysfs",
            "/nonexistent",
            "--no-cap",
        ],
        &[],
    );
    assert_eq!(soak.status.code(), Some(0), "{}", text(&soak));
    let calls = f.calls();
    let started_in = PathBuf::from(calls.last().unwrap().rsplit('|').next().unwrap());
    assert!(
        started_in.join("Data/TaleOfTwoWastelands.esm").exists(),
        "{calls:?}"
    );
    assert!(!started_in.join("Data/OnlyInTheSetting.esm").exists());
    // `--game` still names another folder.
    let named_dir = f.path("named");
    let named = f.raul(
        &[
            "scene",
            "run",
            "--places",
            "mojave-day",
            "--instance",
            s(&f.path("instance")),
            "--game",
            s(&other),
            "--out",
            s(&named_dir),
            "--viewer",
            s(&viewer),
            "--gpu-sysfs",
            "/nonexistent",
            "--no-cap",
        ],
        &[],
    );
    assert_eq!(named.status.code(), Some(0), "{}", text(&named));
    assert!(staged_has(&named_dir, "profile", "OnlyInTheSetting.esm"));
    assert!(!staged_has(
        &named_dir,
        "profile",
        "TaleOfTwoWastelands.esm"
    ));
}

#[test]
fn a_game_folder_that_is_not_there_is_refused_before_any_start() {
    let f = Fixture::new("scene-nobase", false);
    let viewer = f.viewer();
    let out = f.raul(
        &[
            "scene",
            "soak",
            "--places",
            "mojave-day",
            "--instance",
            s(&f.path("instance")),
            "--game",
            s(&f.path("gone")),
            "--viewer",
            s(&viewer),
            "--gpu-sysfs",
            "/nonexistent",
            "--no-cap",
        ],
        &[],
    );
    assert_eq!(out.status.code(), Some(2), "{}", text(&out));
    assert!(text(&out).contains("is not a folder"), "{}", text(&out));
    assert!(f.calls().is_empty(), "nothing was started");
}

#[test]
fn a_failed_start_stops_the_run_and_says_where_its_folder_is() {
    let f = Fixture::new("scene-fail", true);
    let out = f.scene(
        &["scene", "run", "--places", "mojave-day"],
        &[("FAKE_EXIT", "2")],
    );
    assert_ne!(out.status.code(), Some(0));
    assert_eq!(f.calls().len(), 1, "no second start, no comparison");
    assert!(text(&out).contains("did not finish"), "{}", text(&out));
    assert!(text(&out).contains("status 2"), "{}", text(&out));
}

#[test]
fn a_viewer_that_is_not_there_is_said_plainly() {
    let f = Fixture::new("scene-noviewer-file", false);
    let inst = f.path("instance");
    let game = f.path("FalloutNV");
    let out = f.raul(
        &[
            "scene",
            "run",
            "--places",
            "mojave-day",
            "--instance",
            s(&inst),
            "--game",
            s(&game),
            "--viewer",
            "/no/such/viewer",
            "--gpu-sysfs",
            "/nonexistent",
            "--no-cap",
        ],
        &[],
    );
    assert_ne!(out.status.code(), Some(0));
    assert!(text(&out).contains("did not finish"), "{}", text(&out));
}

#[test]
fn without_the_engine_or_a_viewer_the_run_is_refused() {
    let f = Fixture::new("scene-noviewer", false);
    let inst = f.path("instance");
    let game = f.path("FalloutNV");
    let out = f.raul(
        &[
            "scene",
            "soak",
            "--places",
            "mojave-day",
            "--instance",
            s(&inst),
            "--game",
            s(&game),
        ],
        &[],
    );
    assert_eq!(out.status.code(), Some(2), "{}", text(&out));
    assert!(text(&out).contains("no engine in it"), "{}", text(&out));
}

#[test]
fn a_soak_and_a_bench_start_once_over_the_staging_tree_and_the_bench_takes_the_lock() {
    let f = Fixture::new("scene-soak", false);
    let staged = f.raul(
        &["stage", "--instance", s(&f.path("instance")), "--copy"],
        &[],
    );
    assert!(staged.status.success(), "{staged:?}");
    let out = f.scene(
        &["scene", "soak", "--places", "mojave-day", "--frames", "50"],
        &[],
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    assert!(text(&out).contains("goodsprings"), "{}", text(&out));
    assert_eq!(f.calls().len(), 1, "{:?}", f.calls());
    let out = f.scene(
        &["bench", "game", "--suite", "quick", "--baseline", "mine"],
        &[],
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    assert_eq!(f.calls().len(), 2, "{:?}", f.calls());
    let bench_dir = f.run_dir("Default");
    assert!(
        std::fs::read_dir(&bench_dir)
            .unwrap()
            .flatten()
            .any(|e| e.file_name().to_string_lossy().starts_with("suite-quick")),
        "the suite file is written beside the results"
    );
    assert!(text(&out).contains("summary:"), "{}", text(&out));
}

#[test]
fn two_bench_summaries_are_compared_and_a_missing_one_is_said() {
    let f = Fixture::new("scene-bench-compare", false);
    let bad = f.scene(&["bench", "game", "compare", "a.json", "b.json"], &[]);
    assert_ne!(bad.status.code(), Some(0));
    assert!(text(&bad).contains("a.json"), "{}", text(&bad));
}

#[test]
fn the_place_files_raul_carries_are_written_and_never_overwritten() {
    let f = Fixture::new("scene-places", false);
    let dir = f.path("mine");
    let out = f.raul(&["scene", "places", "--out", s(&dir)], &[]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let day = std::fs::read_to_string(dir.join("mojave-day.toml")).unwrap();
    assert!(
        day.contains("goodsprings") && !day.contains("hour"),
        "{day}"
    );
    assert!(
        std::fs::read_to_string(dir.join("mojave-night.toml"))
            .unwrap()
            .contains("hour = 22.5")
    );
    std::fs::write(dir.join("mojave-day.toml"), "mine").unwrap();
    let again = f.raul(&["scene", "places", "--out", s(&dir)], &[]);
    assert!(text(&again).contains("left as it is"));
    assert_eq!(
        std::fs::read_to_string(dir.join("mojave-day.toml")).unwrap(),
        "mine"
    );
}

#[test]
fn the_synthetic_bench_is_unchanged_and_never_starts_the_game() {
    let f = Fixture::new("scene-bench-untouched", false);
    let out = f.raul(&["bench", "compare"], &[]);
    assert_eq!(out.status.code(), Some(5), "{}", text(&out));
    assert!(
        text(&out).contains("bench compare needs <base.json> <head.json>"),
        "{}",
        text(&out)
    );
    assert_eq!(f.calls().len(), 0);
}

#[test]
fn a_real_profile_is_pictured_on_its_staging_tree_with_shared_folders_in_any_case() {
    let f = Fixture::new("scene-shared", true);
    let inst = f.path("instance");
    // Three Mods share the extender's plugin folder, two of them in another case, and one file is hidden the MO2 way.
    for (m, rel) in [
        ("Alpha", "NVSE/Plugins/a.dll"),
        ("Beta", "nvse/plugins/b.dll"),
        ("Gamma", "nvse/Plugins/c.dll"),
        ("Gamma", "nvse/Plugins/d.dll.mohidden"),
    ] {
        if !inst.join("mods").join(m).is_dir() {
            let out = f.raul(&["mods", "create", m, "--instance", s(&inst)], &[]);
            assert!(out.status.success(), "{out:?}");
        }
        let p = inst.join("mods").join(m).join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, "x").unwrap();
    }
    let out = f.scene(&["scene", "run", "--places", "mojave-day"], &[]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let run = f.run_dir("Default");
    for variant in ["with-lod", "without-lod"] {
        for dll in ["a.dll", "b.dll", "c.dll"] {
            assert!(
                staged_has(&run, variant, &format!("nvse/plugins/{dll}")),
                "{variant} {dll}"
            );
        }
        assert!(
            !staged_has(&run, variant, "nvse/plugins/d.dll"),
            "{variant}"
        );
        assert!(
            !staged_has(&run, variant, "nvse/plugins/d.dll.mohidden"),
            "{variant}"
        );
    }
    assert!(staged_has(&run, "with-lod", "meshes/lod.nif"));
    assert!(!staged_has(&run, "without-lod", "meshes/lod.nif"));
    // The profile without its far view is built in raul's own folder of the Instance; the profile is not changed.
    assert!(inst.join(".raul/scene/Default-without-lod/data").is_dir());
    assert!(inst.join("staging/Default/data/meshes/lod.nif").is_file());
}

#[test]
fn a_profile_is_not_staged_for_pictures_while_another_program_holds_the_instance() {
    let f = Fixture::new("scene-locked", true);
    let inst = f.path("instance");
    let lock = nvmod_run::InstanceLock::acquire(&inst, "run test").unwrap();
    let out = f.scene(&["scene", "run", "--places", "mojave-day"], &[]);
    assert_ne!(out.status.code(), Some(0), "{}", text(&out));
    assert!(text(&out).contains("in use"), "{}", text(&out));
    assert!(f.calls().is_empty(), "no game was started");
    drop(lock);
    let out = f.scene(&["scene", "run", "--places", "mojave-day"], &[]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
}
