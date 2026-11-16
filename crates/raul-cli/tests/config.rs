// SPDX-License-Identifier: GPL-3.0-only
//! `raul config`: the paths the GUI needs, in `config.toml` of the Toolbox's configuration folder.
// These tests start the built binary, which is not a tool call (see clippy.toml).
#![allow(clippy::disallowed_methods)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use nvmod_testkit::{BaseBuilder, PeBuilder, TempDir, write_file};
use serde_json::Value;

struct Env {
    home: TempDir,
}

impl Env {
    fn new(label: &str) -> Env {
        Env {
            home: TempDir::new(label),
        }
    }

    fn dir(&self) -> PathBuf {
        self.home.path().join("config")
    }

    fn file(&self) -> PathBuf {
        self.dir().join("config.toml")
    }

    fn state(&self) -> PathBuf {
        self.home.path().join("state")
    }

    fn raul(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_raul"))
            .args(args)
            .env("RAUL_CONFIG_DIR", self.dir())
            .env("NVMOD_STATE_DIR", self.state())
            .output()
            .expect("raul runs")
    }

    fn json(&self, args: &[&str]) -> (Option<i32>, Value) {
        let out = self.raul(args);
        let v = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
            panic!(
                "{e}: {}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            )
        });
        (out.status.code(), v)
    }
}

fn s(p: &Path) -> &str {
    p.to_str().unwrap()
}

fn new_vegas(dir: &Path) {
    BaseBuilder::steam().exe_version([1, 4, 0, 525]).write(dir);
}

fn fallout3(dir: &Path) {
    write_file(dir, "Fallout3.exe", &PeBuilder::exe().build());
    write_file(dir, "Data/Fallout3.esm", b"synthetic");
}

fn schema() -> Value {
    serde_json::from_str(&raul_core::config::schema()).unwrap()
}

fn assert_valid(v: &Value) {
    let schema = schema();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let errors: Vec<String> = validator.iter_errors(v).map(|e| e.to_string()).collect();
    assert!(errors.is_empty(), "{errors:?}\n{v}");
}

#[test]
fn list_prints_every_key_with_null_on_a_fresh_machine() {
    let env = Env::new("raul-config-fresh");
    let (code, v) = env.json(&["config", "list", "--json"]);
    assert_eq!(code, Some(0));
    assert_valid(&v);
    let keys: Vec<&str> = v["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["key"].as_str().unwrap())
        .collect();
    assert_eq!(
        keys,
        [
            "game.newvegas",
            "game.fallout3",
            "ttw.package",
            "storage.root",
            "instance"
        ]
    );
    assert!(
        v["entries"]
            .as_array()
            .unwrap()
            .iter()
            .all(|e| e["value"].is_null())
    );
    assert_eq!(v["file"], s(&env.file()));
    assert!(!env.file().exists(), "list writes nothing");
}

#[test]
fn set_then_get_and_list_show_the_paths_and_the_derived_instance() {
    let env = Env::new("raul-config-set");
    let nv = env.home.path().join("FalloutNV");
    let fo3 = env.home.path().join("Fallout3");
    new_vegas(&nv);
    fallout3(&fo3);
    let package = env.home.path().join("TTW.mpi");
    std::fs::write(&package, b"synthetic").unwrap();
    let storage = env.home.path().join("storage");
    for (key, value) in [
        ("game.newvegas", &nv),
        ("game.fallout3", &fo3),
        ("ttw.package", &package),
        ("storage.root", &storage),
    ] {
        let (code, v) = env.json(&["config", "set", key, s(value), "--json"]);
        assert_eq!(code, Some(0), "{v}");
        assert_valid(&v);
        assert_eq!(v["entries"][0]["key"], key);
        assert_eq!(v["entries"][0]["value"], s(value));
    }
    let (_, v) = env.json(&["config", "get", "instance", "--json"]);
    assert_eq!(v["entries"][0]["value"], s(&storage.join("instance")));
    assert_eq!(v["entries"][0]["read_only"], true);
    let out = env.raul(&["config", "get", "game.fallout3"]);
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), s(&fo3));
    let out = env.raul(&["config", "list"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains(&format!("ttw.package = {}", s(&package))),
        "{text}"
    );
    assert!(text.contains("(derived)"), "{text}");
    let (code, v) = env.json(&["config", "unset", "ttw.package", "--json"]);
    assert_eq!(code, Some(0));
    assert!(v["entries"][0]["value"].is_null());
}

#[test]
fn a_folder_detect_does_not_accept_is_refused_with_exit_2_and_nothing_written() {
    let env = Env::new("raul-config-refuse");
    let empty = env.home.path().join("Downloads");
    std::fs::create_dir_all(&empty).unwrap();
    let (code, v) = env.json(&["config", "set", "game.newvegas", s(&empty), "--json"]);
    assert_eq!(code, Some(2));
    assert_valid(&v);
    assert_eq!(v["error"]["code"], "invalid_value");
    assert_eq!(v["error"]["key"], "game.newvegas");
    assert!(!env.file().exists(), "nothing written");

    // A Fallout 3 copy is not a New Vegas folder, and the other way round.
    let fo3 = env.home.path().join("Fallout3");
    fallout3(&fo3);
    let out = env.raul(&["config", "set", "game.newvegas", s(&fo3)]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("other game"));
    let nv = env.home.path().join("FalloutNV");
    new_vegas(&nv);
    assert_eq!(
        env.raul(&["config", "set", "game.fallout3", s(&nv)])
            .status
            .code(),
        Some(2)
    );
    assert!(!env.file().exists(), "nothing written");
}

#[test]
fn other_bad_values_are_refused() {
    let env = Env::new("raul-config-bad");
    let folder = env.home.path().join("folder");
    std::fs::create_dir_all(&folder).unwrap();
    let file = env.home.path().join("file");
    std::fs::write(&file, b"x").unwrap();
    for (key, value, code) in [
        ("ttw.package", s(&folder), "invalid_value"),
        ("ttw.package", "", "invalid_value"),
        ("storage.root", s(&file), "invalid_value"),
        ("instance", s(&folder), "read_only"),
    ] {
        let (exit, v) = env.json(&["config", "set", key, value, "--json"]);
        assert_eq!(
            (exit, v["error"]["code"].as_str()),
            (Some(2), Some(code)),
            "{key} {v}"
        );
    }
    let (exit, v) = env.json(&["config", "unset", "instance", "--json"]);
    assert_eq!(
        (exit, v["error"]["code"].as_str()),
        (Some(2), Some("read_only"))
    );
    let (exit, v) = env.json(&["config", "get", "game.oblivion", "--json"]);
    assert_eq!(exit, Some(5));
    assert_valid(&v);
    assert_eq!(v["error"]["code"], "unknown_key");
    assert!(!env.file().exists());
}

#[test]
fn a_relative_path_is_stored_as_an_absolute_one() {
    let env = Env::new("raul-config-relative");
    let work = env.home.path().join("work");
    std::fs::create_dir_all(work.join("pkg")).unwrap();
    std::fs::write(work.join("pkg/TTW.mpi"), b"x").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_raul"))
        .args(["config", "set", "ttw.package", "pkg/TTW.mpi", "--json"])
        .current_dir(&work)
        .env("RAUL_CONFIG_DIR", env.dir())
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["entries"][0]["value"], s(&work.join("pkg/TTW.mpi")));
}

#[test]
fn a_newer_schema_version_is_refused_and_the_file_is_left_alone() {
    let env = Env::new("raul-config-newer");
    std::fs::create_dir_all(env.dir()).unwrap();
    let text = "schema_version = 2\n[storage]\nroot = \"/s\"\n";
    std::fs::write(env.file(), text).unwrap();
    let (code, v) = env.json(&["config", "list", "--json"]);
    assert_eq!(code, Some(2));
    assert_valid(&v);
    assert_eq!(v["error"]["code"], "newer_schema");
    assert!(
        v["error"]["message"]
            .as_str()
            .unwrap()
            .contains("schema_version 2")
    );
    let storage = env.home.path().join("storage");
    assert_eq!(
        env.raul(&["config", "set", "storage.root", s(&storage)])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(std::fs::read_to_string(env.file()).unwrap(), text);
}

#[test]
fn a_broken_file_is_refused() {
    let env = Env::new("raul-config-broken");
    std::fs::create_dir_all(env.dir()).unwrap();
    for text in [
        "this is not toml = = =",
        "schema_version = \"one\"",
        "game = \"x\"",
        "[game]\nnewvegas = 3\n",
    ] {
        std::fs::write(env.file(), text).unwrap();
        let (code, v) = env.json(&["config", "list", "--json"]);
        assert_eq!(
            (code, v["error"]["code"].as_str()),
            (Some(2), Some("unreadable")),
            "{text}"
        );
    }
}

#[test]
fn unknown_keys_survive_a_set() {
    let env = Env::new("raul-config-unknown");
    std::fs::create_dir_all(env.dir()).unwrap();
    std::fs::write(
        env.file(),
        "schema_version = 1\nfuture = \"kept\"\n[game]\nnewvegas = \"/nv\"\nlanguage = \"de\"\n[later]\nx = 1\n",
    )
    .unwrap();
    let storage = env.home.path().join("storage");
    assert!(
        env.raul(&["config", "set", "storage.root", s(&storage)])
            .status
            .success()
    );
    let table: toml::Table = std::fs::read_to_string(env.file())
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(table["schema_version"].as_integer(), Some(1));
    assert_eq!(table["future"].as_str(), Some("kept"));
    assert_eq!(table["game"]["language"].as_str(), Some("de"));
    assert_eq!(table["game"]["newvegas"].as_str(), Some("/nv"));
    assert_eq!(table["later"]["x"].as_integer(), Some(1));
    assert_eq!(table["storage"]["root"].as_str(), Some(s(&storage)));
}

#[test]
fn without_a_configuration_folder_config_refuses() {
    let out = Command::new(env!("CARGO_BIN_EXE_raul"))
        .args(["config", "list", "--json"])
        .env_clear()
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["error"]["code"], "no_config_folder");
    assert!(v["file"].is_null());
}

#[test]
fn a_malformed_command_line_is_a_usage_error() {
    let env = Env::new("raul-config-usage");
    for args in [
        &["config"][..],
        &["config", "set", "storage.root"],
        &["config", "drop"],
    ] {
        let out = env.raul(args);
        assert_eq!(out.status.code(), Some(5), "{args:?}");
        assert!(String::from_utf8_lossy(&out.stderr).starts_with("raul: config takes"));
    }
}

/// nvmod writes a run log for every command it runs; `raul config` leaves none, so nvmod never received it.
#[test]
fn config_is_the_toolboxs_own_command_and_never_reaches_nvmod() {
    let env = Env::new("raul-config-dispatch");
    assert!(env.raul(&["config", "list"]).status.success());
    assert_eq!(env.raul(&["config", "bogus"]).status.code(), Some(5));
    assert!(!env.state().join("logs").exists(), "nvmod ran");
    // The same check notices nvmod when it does run.
    assert!(env.raul(&["catalog", "--json"]).status.success());
    assert!(env.state().join("logs").exists());
}
