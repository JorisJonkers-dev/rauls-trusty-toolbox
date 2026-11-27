// SPDX-License-Identifier: GPL-3.0-only
//! `raul modlist list|show|plan` on a synthetic Instance and a synthetic list. No game or mod data is used.
#![allow(clippy::disallowed_methods)] // the tests run the built binary

use raul_testkit::{covers, ids};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use nvmod_testkit::{BaseBuilder, PluginBuilder, TempDir, snapshot, write_file};
use serde_json::Value;

const LIST: &str = r#"
schema = "raul-modlist/1"
id = "tiny"
name = "Tiny"
game = "fnv"
target = "vanilla"
plugins = "loot"

[[mods]]
id = "alpha"
name = "Alpha"
source = { type = "nexus", game = "newvegas", mod = 100, file = 7, file_name = "Alpha.7z" }
[[mods]]
id = "beta"
name = "Beta"
source = { type = "nexus", game = "newvegas", mod = 200, file_name = "Beta.7z", uploaded = 1700000000 }
[[mods]]
id = "gamma"
name = "Gamma"
source = { type = "manual", note = "ask the author" }
[[mods]]
id = "delta"
name = "Delta"
enabled = false
why_off = "replaced"
source = { type = "manual" }
"#;

struct Fixture {
    root: TempDir,
}

fn s(p: &Path) -> &str {
    p.to_str().unwrap()
}

impl Fixture {
    /// An Instance with the Profile `Default` and the Mod `Alpha` installed.
    fn new(label: &str) -> Fixture {
        let f = Fixture {
            root: TempDir::new(label),
        };
        let base = f.path("FalloutNV");
        BaseBuilder::steam()
            .exe_version([1, 4, 0, 525])
            .write(&base);
        write_file(
            &base,
            "Data/FalloutNV.esm",
            &PluginBuilder::master().build(),
        );
        let inst = f.path("instance");
        f.ok(&["init", "--base", s(&base), "--instance", s(&inst)]);
        f.ok(&["profile", "create", "--instance", s(&inst)]);
        write_file(
            &f.path("src/Alpha"),
            "Alpha.esp",
            &PluginBuilder::new().build(),
        );
        f.ok(&[
            "install",
            s(&f.path("src/Alpha")),
            "--instance",
            s(&inst),
            "--name",
            "Alpha",
        ]);
        std::fs::write(f.path("tiny.toml"), LIST).unwrap();
        f
    }

    fn path(&self, p: &str) -> PathBuf {
        self.root.path().join(p)
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

    fn plan(&self, extra: &[&str]) -> Output {
        let (list, inst) = (self.path("tiny.toml"), self.path("instance"));
        let mut args = vec!["modlist", "plan", s(&list), "--instance", s(&inst)];
        args.extend_from_slice(extra);
        self.raul(&args)
    }
}

fn text(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn the_plan_says_what_the_profile_has_and_where_to_get_the_rest_in_install_order() {
    covers(&[ids::mods::modlist::COMMAND_PLAN]);
    let f = Fixture::new("modlist-plan");
    let out = f.plan(&[]);
    assert!(out.status.success(), "{out:?}");
    let t = text(&out);
    assert!(
        t.starts_with("Tiny: 1 present, 2 to get, 1 switched off by the list\n"),
        "{t}"
    );
    assert!(t.contains("   2. Beta\n      Nexus: https://www.nexusmods.com/newvegas/mods/200?tab=files -> Beta.7z\n"), "{t}");
    assert!(
        t.contains("   3. Gamma\n      by hand: ask the author\n"),
        "{t}"
    );
    assert!(
        !t.contains("Alpha") && !t.contains("Delta"),
        "a present or switched-off Mod is not listed to get: {t}"
    );
    assert!(t.find("Beta").unwrap() < t.find("Gamma").unwrap());
}

#[test]
fn the_plan_as_json_carries_counts_and_each_missing_mods_page() {
    let f = Fixture::new("modlist-plan-json");
    let out = f.plan(&["--json"]);
    assert!(out.status.success(), "{out:?}");
    let doc: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        doc["counts"],
        serde_json::json!({ "present": 1, "missing": 2, "skipped": 1 })
    );
    let beta = &doc["items"][1];
    assert_eq!(
        (beta["id"].as_str(), beta["state"].as_str()),
        (Some("beta"), Some("missing"))
    );
    assert_eq!(
        beta["acquire"]["page"],
        "https://www.nexusmods.com/newvegas/mods/200?tab=files"
    );
}

#[test]
fn the_plan_writes_nothing() {
    covers(&[ids::mods::modlist::COMMAND_PLAN]);
    let f = Fixture::new("modlist-plan-readonly");
    let before = snapshot(&f.path("instance"));
    assert!(f.plan(&[]).status.success());
    assert_eq!(snapshot(&f.path("instance")), before);
}

#[test]
fn the_plan_of_a_list_the_profile_has_none_of_is_every_mod_in_order() {
    let f = Fixture::new("modlist-plan-bundled");
    let inst = f.path("instance");
    let out = f.ok(&[
        "modlist",
        "plan",
        "beautiful",
        "--instance",
        s(&inst),
        "--json",
    ]);
    let doc: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(doc["counts"]["present"], 0);
    assert_eq!(doc["counts"]["missing"], 330);
    assert_eq!(doc["items"][0]["name"], "Tale of Two Wastelands");
}

#[test]
fn without_an_instance_the_plan_is_refused_and_says_how_to_name_one() {
    let f = Fixture::new("modlist-plan-no-instance");
    let list = f.path("tiny.toml");
    let out = f.raul(&["modlist", "plan", s(&list)]);
    assert_eq!(out.status.code(), Some(i32::from(raul_cli::REFUSED)));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("no Instance: pass --instance <dir>"),
        "{out:?}"
    );
}

#[test]
fn a_list_that_is_neither_carried_nor_a_file_is_refused() {
    let f = Fixture::new("modlist-unknown");
    let out = f.raul(&["modlist", "show", "no-such-list"]);
    assert_eq!(out.status.code(), Some(i32::from(raul_cli::REFUSED)));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("neither a list raul carries (beautiful)"),
        "{out:?}"
    );
}

#[test]
fn a_broken_list_file_is_refused_with_its_problems() {
    let f = Fixture::new("modlist-broken");
    std::fs::write(
        f.path("bad.toml"),
        LIST.replace("why_off = \"replaced\"\n", ""),
    )
    .unwrap();
    let bad = f.path("bad.toml");
    let out = f.raul(&["modlist", "show", s(&bad)]);
    assert_eq!(out.status.code(), Some(i32::from(raul_cli::REFUSED)));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("does not say why"),
        "{out:?}"
    );
}

#[test]
fn list_and_show_name_the_bundled_list_and_its_mods() {
    let f = Fixture::new("modlist-show");
    assert_eq!(
        text(&f.ok(&["modlist", "list"])),
        "beautiful: Beautiful, 332 Mods\n"
    );
    let shown = text(&f.ok(&["modlist", "show", "beautiful"]));
    assert!(
        shown.contains("Lucky 38 Exterior Remastered  [Nexus]"),
        "{shown}"
    );
    let json: Value =
        serde_json::from_str(&text(&f.ok(&["modlist", "show", "beautiful", "--json"]))).unwrap();
    assert_eq!(json["mods"].as_array().unwrap().len(), 332);
}

#[test]
fn a_command_line_that_is_not_understood_is_a_usage_error() {
    let f = Fixture::new("modlist-usage");
    assert_eq!(
        f.raul(&["modlist"]).status.code(),
        Some(i32::from(raul_cli::USAGE))
    );
    assert_eq!(
        f.raul(&["modlist", "plan"]).status.code(),
        Some(i32::from(raul_cli::USAGE))
    );
    assert_eq!(
        f.raul(&["modlist", "list", "--bogus"]).status.code(),
        Some(i32::from(raul_cli::USAGE))
    );
}
