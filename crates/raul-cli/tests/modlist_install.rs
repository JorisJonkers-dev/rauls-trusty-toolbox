// SPDX-License-Identifier: GPL-3.0-only
//! `raul modlist install` on a synthetic Instance and a synthetic list. No game or mod data is used.
#![allow(clippy::disallowed_methods)] // the tests run the built binary

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use nvmod_testkit::{BaseBuilder, PluginBuilder, TempDir, write_file};
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
}

/// Write a 7z archive of `files` at `path`, making its folder.
fn make_7z(path: &Path, files: &[(&str, &[u8])]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut writer = sevenz_rust2::ArchiveWriter::create(path).unwrap();
    for (name, data) in files {
        writer
            .push_archive_entry(sevenz_rust2::ArchiveEntry::new_file(name), Some(*data))
            .unwrap();
    }
    writer.finish().unwrap();
}

fn text(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn install(f: &Fixture, extra: &[&str]) -> Output {
    let (list, inst, dl) = (f.path("tiny.toml"), f.path("instance"), f.path("Downloads"));
    let mut args = vec![
        "modlist",
        "install",
        s(&list),
        "--instance",
        s(&inst),
        "--downloads",
        s(&dl),
        "--no-sort",
    ];
    args.extend_from_slice(extra);
    f.raul(&args)
}

fn modlist_txt(f: &Fixture) -> String {
    std::fs::read_to_string(f.path("instance/profiles/Default/modlist.txt")).unwrap_or_default()
}

#[test]
fn install_puts_the_mod_whose_archive_is_in_the_downloads_folder_into_the_profile() {
    let f = Fixture::new("modlist-install-one");
    make_7z(&f.path("Downloads/Beta.7z"), &[("Beta.esp", b"x")]);
    let out = install(&f, &[]);
    assert!(out.status.success(), "{out:?}");
    let t = text(&out);
    assert!(t.contains("installed: Beta <- Beta.7z"), "{t}");
    assert!(t.contains("No archive for these"), "{t}");
    assert!(modlist_txt(&f).contains("Beta"), "{}", modlist_txt(&f));
    assert!(f.path("instance/.raul/modlist-state.json").is_file());
}

#[test]
fn a_dry_run_says_what_would_be_installed_and_writes_nothing() {
    let f = Fixture::new("modlist-install-dry");
    make_7z(&f.path("Downloads/Beta.7z"), &[("Beta.esp", b"x")]);
    let before = modlist_txt(&f);
    let out = install(&f, &["--dry-run"]);
    assert!(out.status.success(), "{out:?}");
    assert!(text(&out).contains("would-install: Beta"), "{}", text(&out));
    assert_eq!(modlist_txt(&f), before);
    assert!(!f.path("instance/.raul/modlist-state.json").exists());
}

#[test]
fn the_json_report_names_each_step_and_its_result() {
    let f = Fixture::new("modlist-install-json");
    make_7z(&f.path("Downloads/Beta.7z"), &[("Beta.esp", b"x")]);
    let out = install(&f, &["--json"]);
    assert!(out.status.success(), "{out:?}");
    let doc: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(doc["steps"][0]["name"], "Beta");
    assert_eq!(doc["steps"][0]["result"], "installed");
}

#[test]
fn without_an_instance_the_install_is_refused() {
    let f = Fixture::new("modlist-install-none");
    let list = f.path("tiny.toml");
    let out = f.raul(&["modlist", "install", s(&list)]);
    assert_eq!(out.status.code(), Some(i32::from(raul_cli::REFUSED)));
}
