// SPDX-License-Identifier: GPL-3.0-only
//! The `raul` program as a person or script runs it.
// These tests start the built binary, which is not a tool call (see clippy.toml).
#![allow(clippy::disallowed_methods)]

use std::process::{Command, Output};

fn raul(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_raul"))
        .args(args)
        .output()
        .expect("raul runs")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn version_as_json_names_the_toolbox_version() {
    let out = raul(&["version", "--json"]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["raul"], env!("CARGO_PKG_VERSION"));
    assert!(v.get("known_good_set").is_none(), "{v}");
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

#[test]
fn bench_writes_a_json_and_a_markdown_report() {
    let out_dir = nvmod_testkit::TempDir::new("raul-cli-bench");
    let out = raul(&[
        "bench",
        "--quick",
        "--out",
        out_dir.path().to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(out_dir.path().join("bench.json")).unwrap()).unwrap();
    assert!(json["results"].as_array().is_some_and(|r| !r.is_empty()));
    assert!(
        std::fs::read_to_string(out_dir.path().join("bench.md"))
            .unwrap()
            .contains("| sort/small |")
    );
}

#[test]
fn bench_compare_warns_but_succeeds_on_a_regression() {
    let dir = nvmod_testkit::TempDir::new("raul-cli-compare");
    let report = |ms: f64| {
        serde_json::json!({"schema_version":1,"raul":"0.0.0","os":"linux","arch":"x86_64","skipped":[],
        "results":[{"name":"sort/small","iterations":3,"median_ms":ms,"mean_ms":ms,"min_ms":ms,"max_ms":ms,"unit_per_s":null}]})
    };
    std::fs::write(dir.path().join("base.json"), report(10.0).to_string()).unwrap();
    std::fs::write(dir.path().join("head.json"), report(20.0).to_string()).unwrap();
    let out = raul(&[
        "bench",
        "compare",
        dir.path().join("base.json").to_str().unwrap(),
        dir.path().join("head.json").to_str().unwrap(),
    ]);
    assert!(out.status.success());
    assert!(
        stdout(&out).contains("Warning: sort/small"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn turning_update_checks_on_is_remembered() {
    let home = nvmod_testkit::TempDir::new("raul-cli-update");
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_raul"))
            .args(args)
            .env("HOME", home.path())
            .env("XDG_CONFIG_HOME", home.path().join(".config"))
            .env("APPDATA", home.path().join("AppData"))
            .output()
            .unwrap()
    };
    assert!(stdout(&run(&["update", "auto"])).contains("off"));
    assert!(run(&["update", "auto", "on"]).status.success());
    assert!(stdout(&run(&["update", "auto"])).contains("on"));
}

/// `raul update check` finds `curl` on the `PATH` and runs it through the Runner; a fake one answers here.
#[cfg(unix)]
#[test]
fn the_update_check_runs_curl_from_the_path_through_the_runner() {
    let home = nvmod_testkit::TempDir::new("raul-cli-update-check");
    let bin = home.path().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let releases = r#"[{"tag_name": "v999.0.0", "prerelease": false, "draft": false, "html_url": "https://example.invalid/999"},{"tag_name": "v1000.0.0-rc.1", "prerelease": false, "draft": false, "html_url": "https://example.invalid/rc"}]"#;
    nvmod_testkit::write_script(&bin, "curl", &format!("echo '{releases}'"));
    let out = Command::new(env!("CARGO_BIN_EXE_raul"))
        .args(["update", "check", "--json"])
        .env("PATH", &bin)
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join(".config"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["newer"]["version"], "999.0.0", "{v}");
    assert_eq!(v["current"], env!("CARGO_PKG_VERSION"));
}

/// A `curl` that fails gives a message and a failing exit code, not a wrong answer.
#[cfg(unix)]
#[test]
fn a_failing_update_check_says_why() {
    let home = nvmod_testkit::TempDir::new("raul-cli-update-fail");
    let bin = home.path().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    nvmod_testkit::write_script(
        &bin,
        "curl",
        "echo 'curl: (6) Could not resolve host' >&2\nexit 6",
    );
    let out = Command::new(env!("CARGO_BIN_EXE_raul"))
        .args(["update", "check"])
        .env("PATH", &bin)
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join(".config"))
        .output()
        .unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("raul update check") && err.contains("Could not resolve host"),
        "{err}"
    );
}
