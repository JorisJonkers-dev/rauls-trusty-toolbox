// SPDX-License-Identifier: GPL-3.0-only
//! `raul lod plan`: the lodwright request a Profile gives, checked against lodwright's published schema, and that
//! `raul lod` never reaches nvmod.
// These tests start the built binary, which is not a tool call (see clippy.toml).
#![allow(clippy::disallowed_methods)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use nvmod_testkit::{BaseBuilder, TempDir};
use serde_json::Value;

struct Fixture {
    root: TempDir,
}

impl Fixture {
    fn new(label: &str) -> Fixture {
        let f = Fixture {
            root: TempDir::new(label),
        };
        BaseBuilder::steam()
            .exe_version([1, 4, 0, 525])
            .write(&f.path("FalloutNV"));
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

    fn plan(&self, extra: &[&str]) -> (Option<i32>, Value) {
        let inst = self.instance();
        let mut args = vec!["lod", "plan", "--json", "--instance", s(&inst)];
        args.extend_from_slice(extra);
        let out = self.raul(&args);
        let v = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
        assert!(
            out.status.success() || v.is_null(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        (out.status.code(), v)
    }
}

fn s(p: &Path) -> &str {
    p.to_str().unwrap()
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/fixtures/lodwright-request/profile-settings.json")
}

/// The request with the two folders of this machine replaced by `<data>` and `<output>`.
fn portable(mut request: Value) -> Value {
    request["data_folder"] = "<data>".into();
    request["output_dir"] = "<output>".into();
    request
}

#[test]
fn a_profile_with_settings_gives_the_request_in_the_fixture() {
    let f = Fixture::new("raul-lod-fixture");
    std::fs::write(
        f.instance().join("profiles/Default/raul-lod.toml"),
        r#"
worlds = ["SyntheticWorld"]
threads = 2

[inclusion]
exclude_plugins = ["Noisy.esp"]
exclude_models = ["clutter/*"]

[output]
trees = false
"#,
    )
    .unwrap();
    let (code, request) = f.plan(&[]);
    assert_eq!(code, Some(0));
    raul_drivers::lod::check_request(&request).unwrap();
    let data = PathBuf::from(request["data_folder"].as_str().unwrap());
    assert!(
        data.ends_with("data") && data.join("falloutnv.esm").is_file(),
        "{request}"
    );
    assert!(
        Path::new(request["output_dir"].as_str().unwrap()).ends_with(".raul/lod/out"),
        "{request}"
    );
    let expected: Value =
        serde_json::from_str(&std::fs::read_to_string(fixture()).unwrap()).unwrap();
    assert_eq!(portable(request), expected);
}

#[test]
fn the_fixture_is_a_valid_request_and_reads_back_unchanged() {
    let text = std::fs::read_to_string(fixture()).unwrap();
    let mut request: Value = serde_json::from_str(&text).unwrap();
    request["data_folder"] = "/data".into();
    request["output_dir"] = "/out".into();
    raul_drivers::lod::check_request(&request).unwrap();
    let again: Value = serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
    assert_eq!(again, request);
    let mut broken = request;
    broken["contract"] = "lodwright-plan".into();
    let why = raul_drivers::lod::check_request(&broken).unwrap_err();
    assert!(why.contains("lodwright-request v1"), "{why}");
}

#[test]
fn without_settings_the_request_takes_every_world_and_world_flags_choose_them() {
    let f = Fixture::new("raul-lod-defaults");
    let (_, request) = f.plan(&[]);
    assert_eq!(request["worlds"], serde_json::json!([]));
    assert_eq!(request["plugins"], serde_json::json!(["FalloutNV.esm"]));
    assert_eq!(request["trees"]["enabled"], true);
    let (_, request) = f.plan(&["--world", "A", "--world", "B"]);
    assert_eq!(request["worlds"], serde_json::json!(["A", "B"]));
    let out = f.raul(&["lod", "plan", "--instance", s(&f.instance())]);
    assert!(String::from_utf8_lossy(&out.stdout).contains("1 plugin, every world with terrain"));
}

#[test]
fn a_profile_that_cannot_give_a_request_is_refused() {
    let f = Fixture::new("raul-lod-refuse");
    let settings = f.instance().join("profiles/Default/raul-lod.toml");
    std::fs::write(&settings, "colour = \"blue\"\n").unwrap();
    assert_eq!(f.plan(&[]).0, Some(2));
    std::fs::remove_file(&settings).unwrap();
    std::fs::remove_dir_all(f.instance().join("staging")).unwrap();
    let out = f.raul(&["lod", "plan", "--instance", s(&f.instance())]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("Staging tree"),
        "{out:?}"
    );
    assert_eq!(f.plan(&["--profile", "Nope"]).0, Some(2));
    let out = f.raul(&["lod", "plan", "--instance", s(&f.path("nowhere"))]);
    assert_eq!(out.status.code(), Some(2));
    let out = f.raul(&["lod", "plan"]);
    assert_eq!(out.status.code(), Some(2), "no Instance named anywhere");
    assert_eq!(f.raul(&["lod", "plan", "--bogus"]).status.code(), Some(5));
    assert_eq!(f.raul(&["lod"]).status.code(), Some(5));
}

/// A Fixture whose Instance lies inside a git working tree (a `.git` folder above it).
fn in_a_git_folder(label: &str) -> Fixture {
    let f = Fixture::new(label);
    std::fs::create_dir_all(f.path(".git")).unwrap();
    f
}

fn allow_in_the_settings(f: &Fixture) {
    std::fs::write(
        f.instance().join("profiles/Default/raul-lod.toml"),
        "allow_in_repo = true\n",
    )
    .unwrap();
}

#[test]
fn an_output_folder_in_a_git_folder_is_refused_and_the_refusal_names_both_ways_to_allow_it() {
    let f = in_a_git_folder("raul-lod-git-neither");
    let out = f.raul(&["lod", "plan", "--instance", s(&f.instance())]);
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("git working tree"), "{err}");
    assert!(
        err.contains("allow_in_repo = true") && err.contains("--allow-in-repo"),
        "{err}"
    );
}

#[test]
fn the_settings_key_allows_an_output_folder_in_a_git_folder() {
    let f = in_a_git_folder("raul-lod-git-key");
    allow_in_the_settings(&f);
    let (code, request) = f.plan(&[]);
    assert_eq!(code, Some(0));
    raul_drivers::lod::check_request(&request).unwrap();
    assert!(
        request.get("allow_in_repo").is_none(),
        "the switch is not part of lodwright's request: {request}"
    );
}

#[test]
fn the_flag_allows_an_output_folder_in_a_git_folder_for_that_run_and_writes_no_settings() {
    let f = in_a_git_folder("raul-lod-git-flag");
    let settings = f.instance().join("profiles/Default/raul-lod.toml");
    assert_eq!(f.plan(&["--allow-in-repo"]).0, Some(0));
    assert!(!settings.exists(), "the flag wrote the settings");
    assert_eq!(f.plan(&[]).0, Some(2), "the flag lasted past its run");
}

#[test]
fn the_settings_key_and_the_flag_together_allow_an_output_folder_in_a_git_folder() {
    let f = in_a_git_folder("raul-lod-git-both");
    allow_in_the_settings(&f);
    assert_eq!(f.plan(&["--allow-in-repo"]).0, Some(0));
}

/// nvmod has its own `lod`, and writes a run log for every command it runs: `raul lod` leaves none.
#[test]
fn lod_is_the_toolboxs_own_command_and_never_reaches_nvmod() {
    let f = Fixture::new("raul-lod-dispatch");
    let logs = f.path("state/logs");
    std::fs::remove_dir_all(&logs).unwrap();
    assert_eq!(f.plan(&[]).0, Some(0));
    assert!(!logs.exists(), "nvmod ran");
}
