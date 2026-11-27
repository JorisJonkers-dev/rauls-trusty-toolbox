// SPDX-License-Identifier: GPL-3.0-only
//! `raul modlist export` on a synthetic MO2-style Instance. No game or mod data is used.
#![allow(clippy::disallowed_methods)] // the tests run the built binary

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use nvmod_testkit::{TempDir, snapshot, write_file};
use raul_testkit::{covers, ids};

struct Fixture {
    root: TempDir,
}

impl Fixture {
    fn new(label: &str) -> Fixture {
        let f = Fixture {
            root: TempDir::new(label),
        };
        let inst = f.path("instance");
        write_file(
            &inst,
            "profiles/Default/modlist.txt",
            b"# generated\n+Beta\n-Alpha\n+Tools_separator\n",
        );
        write_file(&inst, "profiles/Default/plugins.txt", b"*Beta.esp\n");
        write_file(
            &inst,
            "mods/Alpha/meta.ini",
            b"[General]\nmodid=100\nversion=1.0\ninstallationFile=Alpha-100-1-0-1700000000.7z\n",
        );
        write_file(&inst, "mods/Beta/meta.ini", b"[General]\n");
        write_file(&inst, "mods/Beta/Beta.esp", b"");
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
}

fn s(p: &Path) -> &str {
    p.to_str().unwrap()
}

#[test]
fn export_writes_a_list_that_show_reads_back_and_leaves_the_instance_alone() {
    covers(&[ids::mods::modlist::COMMAND_EXPORT]);
    let f = Fixture::new("modlist-export");
    let (inst, file) = (f.path("instance"), f.path("mine.toml"));
    let before = snapshot(&inst);
    let out = f.raul(&["modlist", "export", "--instance", s(&inst), "-o", s(&file)]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(snapshot(&inst), before);

    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("mod = 100"), "{text}");
    assert!(text.contains("uploaded = 1700000000"), "{text}");
    let shown = f.raul(&["modlist", "show", s(&file)]);
    assert!(shown.status.success(), "{shown:?}");
    let t = String::from_utf8_lossy(&shown.stdout).into_owned();
    assert!(t.starts_with("Default: 2 Mods (1 off)"), "{t}");
    assert!(t.find("Alpha").unwrap() < t.find("Beta").unwrap(), "{t}");
    assert!(t.contains("Beta  [by hand]"), "{t}");
}

#[test]
fn export_without_a_file_prints_the_list() {
    covers(&[ids::mods::modlist::COMMAND_EXPORT]);
    let f = Fixture::new("modlist-export-stdout");
    let inst = f.path("instance");
    let out = f.raul(&[
        "modlist",
        "export",
        "--instance",
        s(&inst),
        "--target",
        "ttw",
    ]);
    assert!(out.status.success(), "{out:?}");
    let t = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(t.contains("schema = \"raul-modlist/1\""), "{t}");
    assert!(t.contains("target = \"ttw\""), "{t}");
}

#[test]
fn a_missing_profile_is_refused_and_bad_flags_are_usage_errors() {
    let f = Fixture::new("modlist-export-refused");
    let inst = f.path("instance");
    let out = f.raul(&[
        "modlist",
        "export",
        "--instance",
        s(&inst),
        "--profile",
        "Nope",
    ]);
    assert_eq!(out.status.code(), Some(i32::from(raul_cli::REFUSED)));
    assert!(String::from_utf8_lossy(&out.stderr).contains("no Profile `Nope`"));
    let out = f.raul(&["modlist", "export", "--bogus"]);
    assert_eq!(out.status.code(), Some(i32::from(raul_cli::USAGE)));
}
