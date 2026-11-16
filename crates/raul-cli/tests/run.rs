// SPDX-License-Identifier: GPL-3.0-only
//! `raul run` refuses any game run without the memory-capped run guard, and nothing reaches nvmod's `run` then.
//! The fake guard is a shell script, so the tests that need one run on Linux and macOS.
// These tests start the built binary, which is not a tool call (see clippy.toml).
#![allow(clippy::disallowed_methods)]

use std::path::PathBuf;
use std::process::{Command, Output};

use nvmod_testkit::TempDir;
use serde_json::Value;

struct Env {
    root: TempDir,
}

impl Env {
    fn new(label: &str) -> Env {
        Env {
            root: TempDir::new(label),
        }
    }

    fn path(&self, p: &str) -> PathBuf {
        self.root.path().join(p)
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_raul"));
        c.args(args)
            .env("RAUL_CONFIG_DIR", self.path("config"))
            .env("NVMOD_STATE_DIR", self.path("state"));
        c
    }

    fn raul(&self, args: &[&str]) -> Output {
        self.command(args).output().expect("raul runs")
    }

    /// nvmod writes a run log for every command it runs.
    fn nvmod_ran(&self) -> bool {
        self.path("state/logs").exists()
    }

    #[cfg(unix)]
    fn guard(&self, body: &str) -> String {
        std::fs::create_dir_all(self.path("bin")).unwrap();
        nvmod_testkit::write_script(&self.path("bin"), "nv-rs-guard", body)
            .to_str()
            .unwrap()
            .to_owned()
    }
}

fn json(out: &Output) -> Value {
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "{e}: {}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

const GAME: [&str; 4] = ["run", "nv-rs", "--instance", "/nowhere"];

#[test]
fn without_a_guard_no_game_run_starts_and_nvmod_never_sees_it() {
    let env = Env::new("raul-run-none");
    let out = env.raul(&[&GAME[..], &["--json"]].concat());
    assert_eq!(out.status.code(), Some(2));
    let v = json(&out);
    assert_eq!(
        (v["status"].as_str(), v["started"].as_bool()),
        (Some("refused"), Some(false))
    );
    assert!(
        v["reason"]
            .as_str()
            .unwrap()
            .contains("memory-capped run guard"),
        "{v}"
    );
    let out = env.raul(&GAME);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("raul run: refused"));
    assert!(!env.nvmod_ran());
}

/// No flag, no guard path and no variable turns the cap off.
#[test]
fn no_argument_or_variable_starts_an_uncapped_run() {
    let env = Env::new("raul-run-uncapped");
    let missing = env.path("no-guard");
    for extra in [
        &["--no-guard"][..],
        &["--uncapped"],
        &["--guard", missing.to_str().unwrap()],
        &["--guard", ""],
        &["--dry-run"],
        &["--", "--no-guard"],
    ] {
        let out = env.raul(&[&GAME[..], extra].concat());
        assert_eq!(out.status.code(), Some(2), "{extra:?}");
    }
    for (name, value) in [
        ("RAUL_RUN_GUARD", "off"),
        ("RAUL_NO_GUARD", "1"),
        ("NV_RS_GUARD", "0"),
        ("NV_GPU_BUDGET_MB", "0"),
    ] {
        let out = env.command(&GAME).env(name, value).output().unwrap();
        assert_eq!(out.status.code(), Some(2), "{name}={value}");
    }
    assert!(!env.nvmod_ran());
}

#[cfg(unix)]
const ON: &str = r#"echo '{"schema_version":1,"contract":"nv-rs-run-guard","enabled":true,"caps":{"host_mib":12288,"vram_mib":12280,"gtt_mib":6000}}'"#;

#[cfg(unix)]
#[test]
fn a_guard_that_is_off_or_answers_badly_refuses_the_run() {
    let env = Env::new("raul-run-bad");
    for (body, expect) in [
        (
            r#"echo '{"schema_version":1,"contract":"nv-rs-run-guard","enabled":false,"reason":"no amdgpu memory files"}'"#,
            "is off (no amdgpu memory files)",
        ),
        ("echo not json", "not JSON"),
        (
            r#"echo '{"schema_version":1,"contract":"something-else"}'"#,
            "not nv-rs-run-guard",
        ),
        (
            r#"echo '{"schema_version":2,"contract":"nv-rs-run-guard","enabled":true}'"#,
            "version 2",
        ),
        (
            r#"echo '{"schema_version":1,"contract":"nv-rs-run-guard","enabled":true,"caps":{"host_mib":0,"vram_mib":1,"gtt_mib":1}}'"#,
            "a cap of 0",
        ),
        (
            r#"echo '{"schema_version":1,"contract":"nv-rs-run-guard","enabled":true}'"#,
            "names no caps",
        ),
        ("exit 1", "did not answer"),
    ] {
        let guard = env.guard(body);
        let out = env.raul(&[&GAME[..], &["--guard", &guard, "--json"]].concat());
        assert_eq!(out.status.code(), Some(2), "{body}");
        let v = json(&out);
        assert!(
            v["reason"].as_str().unwrap().contains(expect),
            "{body}: {v}"
        );
        assert_eq!(v["started"], false);
    }
    assert!(!env.nvmod_ran());
}

#[cfg(unix)]
#[test]
fn with_the_guard_on_a_dry_run_names_its_caps_and_a_real_run_still_starts_nothing() {
    let env = Env::new("raul-run-on");
    let guard = env.guard(ON);
    let out = env.raul(&[&GAME[..], &["--guard", &guard, "--dry-run", "--json"]].concat());
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let v = json(&out);
    assert_eq!(v["status"], "dry_run");
    assert_eq!(v["started"], false);
    assert_eq!(v["caps"]["host_mib"], 12288);
    assert_eq!(v["caps"]["vram_mib"], 12280);
    assert_eq!(v["caps"]["gtt_mib"], 6000);
    assert_eq!(v["guard"]["path"], guard.as_str());
    assert_eq!(v["guard"]["sha256"].as_str().unwrap().len(), 64);
    // nvmod planned the run (and refused this one: there is no Instance), read only.
    assert!(v["plan_error"].is_string(), "{v}");
    let out = env.raul(&[&GAME[..], &["--guard", &guard, "--dry-run"]].concat());
    assert!(String::from_utf8_lossy(&out.stderr).contains("12280 MiB of VRAM"));

    let logs = env.path("state/logs");
    let _ = std::fs::remove_dir_all(&logs);
    let out = env.raul(&[&GAME[..], &["--guard", &guard, "--json"]].concat());
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(json(&out)["started"], false);
    assert!(!logs.exists(), "nvmod's run was reached");
}
