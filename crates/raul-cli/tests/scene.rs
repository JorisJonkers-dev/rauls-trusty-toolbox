// SPDX-License-Identifier: GPL-3.0-only
//! `raul scene` and `raul bench game` on synthetic data: a fake tool program stands in for the engine's scene tool (and
//! so for the game child). It records the words it was started with and makes empty "pictures", so what is checked is
//! what raul resolves from the Profile: the Mods in priority order, the Plugins in Load order, the variants, the
//! output folder, the label and the cap, and that `raul bench` itself is untouched. No game and no engine is involved.
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
        if with_lod {
            let out = f.raul(&["mods", "create", LOD_MOD, "--instance", s(&inst)], &[]);
            assert!(out.status.success(), "{out:?}");
            std::fs::write(
                inst.join("mods").join(LOD_MOD).join("meta.ini"),
                "[raulGeneratedLod]\ngenerated=true\nprofile=\"Default\"\nrequestDigest=\"x\"\nlodwrightVersion=\"0.15.1\"\n",
            )
            .unwrap();
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

    /// A tool that records its words, one line a start, and makes one picture for each `run`.
    fn tool(&self) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let file = self.path("fake-scene-tool");
        let log = self.path("tool.log");
        let text = format!(
            "#!/bin/sh\necho \"$@\" >> '{}'\nif [ \"$1\" = run ]; then\n  while [ $# -gt 0 ]; do\n    case \"$1\" in --out) out=\"$2\";; --variant) v=\"${{2%%=*}}\";; esac\n    shift\n  done\n  mkdir -p \"$out/$v/pictures\" && echo png > \"$out/$v/pictures/goodsprings.png\"\nfi\nexit ${{FAKE_EXIT:-0}}\n",
            log.display()
        );
        std::fs::write(&file, text).unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();
        file
    }

    fn calls(&self) -> Vec<String> {
        std::fs::read_to_string(self.path("tool.log"))
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }

    /// `raul scene <args>` with the fake tool, the instance, a game folder and a made viewer name.
    fn scene(&self, args: &[&str], extra_env: &[(&str, &str)]) -> Output {
        let tool = self.tool();
        let inst = self.path("instance");
        let game = self.path("FalloutNV");
        let mut all = args.to_vec();
        all.extend_from_slice(&[
            "--instance",
            s(&inst),
            "--game",
            s(&game),
            "--viewer",
            "/the/viewer",
            "--no-cap",
        ]);
        let mut env = vec![("RAUL_SCENE_TOOL", s(&tool))];
        env.extend_from_slice(extra_env);
        self.raul(&all, &env)
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

#[test]
fn a_profile_with_a_generated_lod_mod_gets_the_pair_and_the_difference() {
    let f = Fixture::new("scene-pair", true);
    let out = f.scene(&["scene", "run", "--places", "mojave-day"], &[]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let calls = f.calls();
    assert_eq!(calls.len(), 3, "two starts and one compare: {calls:?}");
    let (with, without, compare) = (&calls[0], &calls[1], &calls[2]);
    // The Profile as it is lays both Mods over the game, the lowest priority first; without the LOD Mod only one.
    assert!(with.contains("--variant with-lod="), "{with}");
    assert!(
        with.contains("Textures") && with.contains(LOD_MOD),
        "{with}"
    );
    assert!(without.contains("--variant without-lod="), "{without}");
    assert!(
        without.contains("Textures") && !without.contains(LOD_MOD),
        "{without}"
    );
    assert!(
        with.contains("--label raul-scene-run-Default-with-lod --who raul"),
        "{with}"
    );
    assert!(
        with.contains("--places") && with.contains("mojave-day.toml"),
        "{with}"
    );
    assert!(compare.starts_with("compare "), "{compare}");
    assert!(
        compare.contains("with-lod/pictures") && compare.contains("without-lod/pictures"),
        "{compare}"
    );
    // The results are under raul's data folder, named by the Profile.
    assert!(with.contains(s(&f.path("data/scene/Default"))), "{with}");
    assert!(text(&out).contains("Open both pictures"), "{}", text(&out));
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
    assert!(
        calls[0].contains("--variant profile=") && calls[0].contains("--only night-novac"),
        "{calls:?}"
    );
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
    assert!(f.calls()[0].contains("--variant plain "), "{:?}", f.calls());
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
    assert_eq!(f.calls().len(), 3, "the refusal started nothing");
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
}

#[test]
fn a_missing_tool_is_said_plainly() {
    let f = Fixture::new("scene-notool", false);
    let out = f.scene(
        &["scene", "run", "--places", "mojave-day"],
        &[("RAUL_SCENE_TOOL", "/no/such/tool")],
    );
    assert_ne!(out.status.code(), Some(0));
    assert!(
        text(&out).contains("is not installed here"),
        "{}",
        text(&out)
    );
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
fn a_soak_and_a_bench_start_once_over_the_staging_tree_and_the_bench_takes_the_lock_by_name() {
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
    let out = f.scene(
        &["bench", "game", "--suite", "quick", "--baseline", "mine"],
        &[],
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let calls = f.calls();
    assert!(
        calls[0].starts_with("soak ")
            && calls[0].contains("--frames 50")
            && calls[0].contains("--overlay"),
        "{calls:?}"
    );
    assert!(
        calls[1].starts_with("bench ")
            && calls[1].contains("--suite quick")
            && calls[1].contains("--lock")
            && calls[1].contains("--baseline mine"),
        "{calls:?}"
    );
    let compare = f.scene(&["bench", "game", "compare", "a.json", "b.json"], &[]);
    assert_eq!(compare.status.code(), Some(0), "{}", text(&compare));
    assert_eq!(f.calls()[2], "bench-compare a.json b.json");
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
fn the_synthetic_bench_is_unchanged_and_never_reaches_the_scene_tool() {
    let f = Fixture::new("scene-bench-untouched", false);
    let out = f.raul(&["bench", "compare"], &[("RAUL_SCENE_TOOL", s(&f.tool()))]);
    assert_eq!(out.status.code(), Some(5), "{}", text(&out));
    assert!(
        text(&out).contains("bench compare needs <base.json> <head.json>"),
        "{}",
        text(&out)
    );
    assert_eq!(f.calls().len(), 0);
}
