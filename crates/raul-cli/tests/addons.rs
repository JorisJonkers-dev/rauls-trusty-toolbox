// SPDX-License-Identifier: GPL-3.0-only
//! `raul addons list|apply` on a synthetic Instance: a base copy with a game master, and a Mod `Street Lights` whose
//! plugin `Street Lights TTW.esm` is made in the test (one glow activator and one other). No game or mod data is used.
#![allow(clippy::disallowed_methods)] // the tests run the built binary

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use nvmod_testkit::{BaseBuilder, PluginBuilder, TempDir, snapshot, write_file};
use raul_drivers::addons::plugin::synthetic_plugin;
use serde_json::Value;

const ADDON: &str = "Raul Small Lamp Glow";
const ADDON_PLUGIN: &str = "Raul Small Lamp Glow.esp";
const TRIGGER: &str = "Street Lights TTW.esm";
const GLOW: &str = "Effects\\Ambient\\FXGlowSimpFillRnd.nif";

struct Fixture {
    root: TempDir,
}

fn s(p: &Path) -> &str {
    p.to_str().unwrap()
}

/// The trigger plugin; `glow` is the editor id of its glow activator, so a new copy can differ in what is copied.
fn street_lights(glow: &str) -> Vec<u8> {
    synthetic_plugin(
        &["FalloutNV.esm"],
        &[
            ("ACTI", 0x0100_0801, glow, GLOW),
            ("ACTI", 0x0100_0802, "Other", "clutter\\box.nif"),
        ],
    )
}

impl Fixture {
    /// An Instance with the Profile `Default`; with `lamps`, the Mod `Street Lights` installed with its plugin
    /// active, and a plain Mod `Other` above it.
    fn new(label: &str, lamps: bool) -> Fixture {
        let f = Fixture {
            root: TempDir::new(label),
        };
        BaseBuilder::steam()
            .exe_version([1, 4, 0, 525])
            .write(&f.path("FalloutNV"));
        write_file(
            &f.path("FalloutNV"),
            "Data/FalloutNV.esm",
            &PluginBuilder::master().build(),
        );
        let (base, inst) = (f.path("FalloutNV"), f.instance());
        f.ok(&["init", "--base", s(&base), "--instance", s(&inst)]);
        f.ok(&["profile", "create", "--instance", s(&inst)]);
        if lamps {
            write_file(
                &f.path("src/Street Lights"),
                TRIGGER,
                &street_lights("LampGlow"),
            );
            f.install("Street Lights");
        }
        write_file(
            &f.path("src/Other"),
            "Other.esp",
            &PluginBuilder::new().build(),
        );
        f.install("Other");
        f
    }

    fn install(&self, name: &str) {
        let (src, inst) = (self.path(&format!("src/{name}")), self.instance());
        self.ok(&["install", s(&src), "--instance", s(&inst), "--name", name]);
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

    fn ok(&self, args: &[&str]) -> Output {
        let out = self.raul(args);
        assert!(out.status.success(), "{args:?}: {out:?}");
        out
    }

    /// `raul addons <verb> --json`: the add-on's entry of the document.
    fn addons(&self, verb: &str) -> Value {
        let inst = self.instance();
        let out = self.ok(&["addons", verb, "--instance", s(&inst), "--json"]);
        let doc: Value = serde_json::from_slice(&out.stdout).unwrap();
        let schema: Value =
            serde_json::from_str(&raul_drivers::addons::schema()).expect("the schema is JSON");
        let validator = jsonschema::validator_for(&schema).unwrap();
        assert!(validator.is_valid(&doc), "{doc:#}");
        assert_eq!(doc["contract"], "raul-addons");
        doc["addons"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["id"] == "small-lamp-glow")
            .unwrap()
            .clone()
    }

    fn lines(&self, file: &str) -> Vec<String> {
        std::fs::read_to_string(self.instance().join("profiles/Default").join(file))
            .unwrap_or_default()
            .lines()
            .map(|l| l.trim().trim_start_matches('*').to_owned())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect()
    }

    fn state(&self) -> BTreeMap<String, Vec<u8>> {
        let mut all = snapshot(&self.instance().join("mods"));
        all.extend(snapshot(&self.instance().join("profiles")));
        all
    }

    fn mods(&self, verb: &str, name: &str) {
        let inst = self.instance();
        self.ok(&["mods", verb, name, "--instance", s(&inst)]);
    }
}

fn index(list: &[String], name: &str) -> usize {
    list.iter()
        .position(|l| l.trim_start_matches(['+', '-']).eq_ignore_ascii_case(name))
        .unwrap_or_else(|| panic!("{name} in {list:?}"))
}

#[test]
fn a_profile_without_street_lights_is_never_touched() {
    let f = Fixture::new("raul-addons-none", false);
    let before = f.state();
    assert_eq!(f.addons("list")["status"], "not_needed");
    let glow = f.addons("apply");
    assert_eq!(glow["status"], "not_needed");
    assert_eq!(glow["changes"], serde_json::json!([]));
    assert_eq!(f.state(), before, "nothing in the mods or profiles changed");
    assert!(!f.instance().join("mods").join(ADDON).exists());
}

#[test]
fn the_add_on_goes_right_above_street_lights_with_its_plugin_after_the_trigger() {
    let f = Fixture::new("raul-addons-add", true);
    assert_eq!(f.addons("list")["status"], "out_of_date");
    let glow = f.addons("apply");
    assert_eq!(glow["status"], "added", "{glow:#}");
    assert_eq!(glow["enabled"], true);
    let modlist = f.lines("modlist.txt");
    assert_eq!(
        index(&modlist, ADDON) + 1,
        index(&modlist, "Street Lights"),
        "{modlist:?}"
    );
    assert!(
        modlist.iter().any(|l| l == &format!("+{ADDON}")),
        "{modlist:?}"
    );
    for file in ["plugins.txt", "loadorder.txt"] {
        let list = f.lines(file);
        assert_eq!(
            index(&list, ADDON_PLUGIN),
            index(&list, TRIGGER) + 1,
            "{file}: {list:?}"
        );
    }
    let dir = f.instance().join("mods").join(ADDON);
    let meta = std::fs::read_to_string(dir.join("meta.ini")).unwrap();
    assert!(
        meta.contains("[raulGeneratedAddon]") && meta.contains("sourceDigest="),
        "{meta}"
    );
    let esp = std::fs::read(dir.join(ADDON_PLUGIN)).unwrap();
    assert!(
        esp.windows(TRIGGER.len()).any(|w| w == TRIGGER.as_bytes()),
        "the trigger is a master"
    );
    assert!(
        dir.join("meshes/raul/effects/raulsmallglow01.nif")
            .is_file()
    );

    // A second run changes nothing.
    let before = f.state();
    let again = f.addons("apply");
    assert_eq!(again["status"], "up_to_date");
    assert_eq!(f.state(), before);
}

#[test]
fn a_turned_off_add_on_stays_off_and_a_removed_one_is_not_added_back() {
    let f = Fixture::new("raul-addons-off", true);
    f.addons("apply");
    f.mods("disable", ADDON);
    let before = f.state();
    let glow = f.addons("apply");
    assert_eq!(glow["status"], "turned_off");
    assert_eq!(glow["enabled"], false);
    assert_eq!(f.state(), before);

    let inst = f.instance();
    f.ok(&["mods", "remove", ADDON, "--instance", s(&inst), "--force"]);
    let glow = f.addons("apply");
    assert_eq!(glow["status"], "turned_off");
    assert!(!f.instance().join("mods").join(ADDON).exists());
}

#[test]
fn a_changed_trigger_plugin_makes_the_add_on_again_in_its_place() {
    let f = Fixture::new("raul-addons-update", true);
    f.addons("apply");
    let modlist = f.lines("modlist.txt");
    let esp = f.instance().join("mods").join(ADDON).join(ADDON_PLUGIN);
    let old = std::fs::read(&esp).unwrap();
    std::fs::write(
        f.instance().join("mods/Street Lights").join(TRIGGER),
        street_lights("LampGlowRenamed"),
    )
    .unwrap();
    let glow = f.addons("apply");
    assert_eq!(glow["status"], "updated", "{glow:#}");
    assert_eq!(f.lines("modlist.txt"), modlist, "its place and state stay");
    assert_ne!(std::fs::read(&esp).unwrap(), old);
    assert_eq!(f.addons("apply")["status"], "up_to_date");
}

#[test]
fn without_its_trigger_the_add_on_is_turned_off_and_comes_back_with_it() {
    let f = Fixture::new("raul-addons-pause", true);
    f.addons("apply");
    f.mods("disable", "Street Lights");
    let glow = f.addons("apply");
    assert_eq!(glow["status"], "not_needed");
    assert_eq!(glow["enabled"], false, "{glow:#}");
    f.mods("enable", "Street Lights");
    let inst = f.instance();
    f.ok(&["plugins", "enable", TRIGGER, "--instance", s(&inst)]);
    let glow = f.addons("apply");
    assert_eq!(glow["enabled"], true, "{glow:#}");
    assert_eq!(glow["status"], "updated");
}

#[test]
fn the_check_before_a_game_start_never_stops_it() {
    let dir = TempDir::new("raul-addons-before");
    let mut err = Vec::new();
    raul_cli::addons::before_game(&dir.path().join("none"), None, true, &mut err);
    let text = String::from_utf8(err).unwrap();
    assert!(
        text.starts_with("raul: warning: the add-ons were not checked"),
        "{text}"
    );
    // On a real Instance it prints what it changed.
    let f = Fixture::new("raul-addons-before-real", true);
    let mut err = Vec::new();
    raul_cli::addons::before_game(&f.instance(), Some("Default"), true, &mut err);
    let text = String::from_utf8(err).unwrap();
    assert!(
        text.contains("raul add-on Raul Small Lamp Glow: added"),
        "{text}"
    );
}
