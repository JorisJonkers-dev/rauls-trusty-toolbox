// SPDX-License-Identifier: GPL-3.0-only
//! `raul lod generate` against the real lodwright program of the shipped Known-good set, on a synthetic world built
//! with lodwright's own test kit. The program comes from `RAUL_TEST_LODWRIGHT`; CI downloads the release asset of
//! the pinned tag, checks it against the release's `SHA256SUMS` and runs
//! `cargo test -p raul-cli --features real-lodwright --test lod_generate_real`. Without the feature this file
//! builds nothing, so a plain `cargo test` needs no lodwright.
#![cfg(feature = "real-lodwright")]
// These tests start the built binary, which is not a tool call (see clippy.toml).
#![allow(clippy::disallowed_methods)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use lodwright_testkit::land::Heights;
use lodwright_testkit::world::{SyntheticWorld, world_plugin};
use nvmod_testkit::{BaseBuilder, TempDir, snapshot};
use raul_core::known_good::KnownGood;
use serde_json::Value;

const MOD: &str = "LOD (lodwright)";

fn lodwright() -> PathBuf {
    let path = std::env::var_os("RAUL_TEST_LODWRIGHT")
        .expect("RAUL_TEST_LODWRIGHT names the real lodwright program; this test needs it");
    let path = PathBuf::from(path);
    assert!(path.is_file(), "{}: no such program", path.display());
    path
}

/// The lodwright-rs version of the latest Known-good set, without its `v`.
fn shipped_lodwright_version() -> String {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../known-good.toml"),
    )
    .expect("read known-good.toml");
    let set = KnownGood::parse(&text)
        .expect("known-good.toml parses")
        .latest_known_good()
        .expect("a known-good set")
        .clone();
    set.components["lodwright-rs"]
        .version
        .trim_start_matches('v')
        .to_owned()
}

struct Fixture {
    root: TempDir,
}

impl Fixture {
    /// An Instance whose game master is a synthetic world of four cells with terrain.
    fn new(label: &str) -> Fixture {
        let f = Fixture {
            root: TempDir::new(label),
        };
        BaseBuilder::steam()
            .exe_version([1, 4, 0, 525])
            .write(&f.path("FalloutNV"));
        let world = SyntheticWorld::new(0xB00, "Island")
            .cell(2, 2, Heights::flat(16.0))
            .cell(3, 2, Heights::flat(24.0))
            .cell(2, 3, Heights::flat(32.0))
            .cell(3, 3, Heights::flat(8.0));
        std::fs::write(
            f.path("FalloutNV/Data/FalloutNV.esm"),
            world_plugin(&[], &world, false),
        )
        .unwrap();
        let (base, inst) = (f.path("FalloutNV"), f.instance());
        for args in [
            &["init", "--base", s(&base), "--instance", s(&inst)][..],
            &["profile", "create", "--instance", s(&inst)],
            &["stage", "--instance", s(&inst), "--copy"],
        ] {
            let out = f.raul(args);
            assert!(out.status.success(), "{args:?}: {out:?}");
        }
        f
    }

    fn path(&self, p: &str) -> PathBuf {
        self.root.path().join(p)
    }

    fn instance(&self) -> PathBuf {
        self.path("instance")
    }

    fn raul(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_raul"))
            .args(args)
            .env("RAUL_CONFIG_DIR", self.path("config"))
            .env("NVMOD_STATE_DIR", self.path("state"))
            .output()
            .expect("raul runs")
    }

    /// Runs `raul lod generate --json` with the real lodwright; returns the exit code, the result and `finished`.
    fn generate(&self) -> (Option<i32>, Value, Value) {
        let (inst, tool) = (self.instance(), lodwright());
        let out = self.raul(&[
            "lod",
            "generate",
            "--json",
            "--instance",
            s(&inst),
            "--lodwright",
            s(&tool),
        ]);
        let lines: Vec<Value> = String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(|l| {
                serde_json::from_str(l).unwrap_or_else(|e| {
                    panic!("{e}: {l}\n{}", String::from_utf8_lossy(&out.stderr))
                })
            })
            .collect();
        let events = validator(&raul_drivers::events::schema());
        for l in &lines {
            let errors: Vec<String> = events.iter_errors(l).map(|e| e.to_string()).collect();
            assert!(errors.is_empty(), "{errors:?} {l}");
        }
        assert!(
            lines.len() >= 2,
            "a result and a finished line: {lines:?}\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let finished = lines.last().expect("a finished line").clone();
        assert_eq!(finished["event"], "finished", "{finished}");
        let result = lines[lines.len() - 2]["result"].clone();
        let lod = validator(&raul_drivers::lod::schema());
        let errors: Vec<String> = lod.iter_errors(&result).map(|e| e.to_string()).collect();
        assert!(errors.is_empty(), "{errors:?} {result}");
        (out.status.code(), result, finished)
    }

    fn state(&self) -> BTreeMap<String, Vec<u8>> {
        let mut all = snapshot(&self.instance().join("mods"));
        all.extend(snapshot(&self.instance().join("profiles")));
        all
    }
}

fn s(p: &Path) -> &str {
    p.to_str().unwrap()
}

fn validator(schema: &str) -> jsonschema::Validator {
    jsonschema::validator_for(&serde_json::from_str(schema).unwrap()).unwrap()
}

#[test]
fn the_real_lodwright_generates_one_enabled_lod_mod_and_a_second_run_changes_nothing() {
    let f = Fixture::new("raul-lodgen-real");

    let (code, result, finished) = f.generate();
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(result["status"], "success", "{result}");
    assert_eq!(result["mod"], MOD);
    assert_eq!(result["up_to_date"], false);
    assert_eq!(finished["changed"], true);
    assert_eq!(
        result["tool"]["version"],
        shipped_lodwright_version().as_str()
    );
    assert_eq!(result["lodwright"]["code"], "generated", "{result}");
    let lod_mod = f.instance().join("mods").join(MOD);
    assert!(
        lod_mod.join("lodsettings/island.dlodsettings").is_file(),
        "{result}"
    );
    assert!(lod_mod.join("meshes/landscape/lod/island").is_dir());
    assert!(!lod_mod.join("manifest.json").exists());
    let modlist =
        std::fs::read_to_string(f.instance().join("profiles/Default/modlist.txt")).unwrap();
    assert!(modlist.lines().any(|l| l == format!("+{MOD}")), "{modlist}");

    let before = f.state();
    let (code, result, finished) = f.generate();
    assert_eq!(code, Some(0), "{result}");
    assert_eq!(result["up_to_date"], true, "{result}");
    assert_eq!(result["lodwright"]["code"], "up_to_date");
    assert_eq!(finished["changed"], false);
    assert_eq!(f.state(), before, "an up-to-date run changed the Instance");
}
