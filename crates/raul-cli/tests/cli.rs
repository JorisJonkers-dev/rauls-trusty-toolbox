// SPDX-License-Identifier: GPL-3.0-only
//! The `raul` program as a person or script runs it.

use std::path::PathBuf;
use std::process::{Command, Output};

fn raul(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_raul"))
        .args(args)
        .output()
        .expect("raul runs")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn version_names_the_toolbox_and_every_component_of_the_shipped_set() {
    let out = raul(&["--version"]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.starts_with("raul "), "{text}");
    for (name, version) in [
        ("nv-rs", "v0.4.0"),
        ("nvmod", "v0.5.0"),
        ("loot-rs", "v0.2.0"),
        ("lodwright-rs", "v0.2.0"),
        ("ttwright-rs", "unreleased"),
    ] {
        assert!(
            text.lines()
                .any(|l| l.contains(name) && l.contains(version)),
            "{name} {version} missing:\n{text}"
        );
    }
}

#[test]
fn version_as_json_carries_the_set_id_and_the_components() {
    let out = raul(&["version", "--json"]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["known_good_set"], "2026-11-a");
    assert_eq!(v["components"]["nv-rs"], "v0.4.0");
}

#[test]
fn a_command_raul_does_not_own_is_run_by_nvmod() {
    let out = raul(&["catalog", "--json"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v.is_object() || v.is_array());
}

#[test]
fn an_unknown_command_exits_with_the_usage_code() {
    let out = raul(&["no-such-command"]);
    assert_eq!(out.status.code(), Some(5));
}

#[test]
fn the_known_good_check_passes_on_this_repository() {
    let out = raul(&[
        "known-good",
        "check",
        "--repo",
        repo_root().to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "{}{}",
        stdout(&out),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn detect_on_a_picked_folder_reports_the_copy_as_json() {
    let dir = nvmod_testkit::TempDir::new("raul-cli-detect");
    nvmod_testkit::BaseBuilder::steam()
        .exe_version([1, 4, 0, 525])
        .write(dir.path());
    let out = raul(&["detect", "--folder", dir.path().to_str().unwrap(), "--json"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v[0]["game"], "new-vegas");
    assert_eq!(v[0]["version"], "1.4.0.525");
}

#[test]
fn detect_on_a_folder_without_the_game_exits_with_found() {
    let dir = nvmod_testkit::TempDir::new("raul-cli-nogame");
    let out = raul(&["detect", "--folder", dir.path().to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1));
}
