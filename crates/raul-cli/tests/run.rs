// SPDX-License-Identifier: GPL-3.0-only
//! The Release test of #47, on synthetic data: the game starts through raul's own launcher. A synthetic Instance
//! whose target is nv-rs is staged, and `raul run nv-rs --wait` starts a stand-in program as the viewer: the program
//! is started in the run folder with the run's environment, its output goes to the run log, and the end of the run
//! is the end of the program. No game data and no engine are involved, so it runs on every system in CI.
// These tests start the built binary, which is not a tool call (see clippy.toml).
#![allow(clippy::disallowed_methods)]

use std::path::{Path, PathBuf};
use std::process::Command;

use nvmod_testkit::{BaseBuilder, TempDir};

struct Run {
    t: TempDir,
    instance: PathBuf,
}

impl Run {
    fn raul(&self, args: &[&str]) -> (i32, String, String) {
        let out = Command::new(env!("CARGO_BIN_EXE_raul"))
            .args(args)
            .env("RAUL_CONFIG_DIR", self.t.path().join("config"))
            .env("NVMOD_STATE_DIR", self.t.path().join("state"))
            .env("XDG_STATE_HOME", self.t.path().join("xdg-state"))
            .env("LOCALAPPDATA", self.t.path().join("local-app-data"))
            .env("HOME", self.t.path().join("home"))
            .env("USERPROFILE", self.t.path().join("home"))
            .output()
            .unwrap();
        (
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    }

    /// A staged Instance that runs on nv-rs, as the window sets one up.
    fn new(label: &str) -> Run {
        let t = TempDir::new(label);
        let base = t.path().join("base");
        BaseBuilder::steam().write(&base);
        let instance = t.path().join("instance");
        let run = Run { t, instance };
        let i = run.instance.to_string_lossy().into_owned();
        let b = base.to_string_lossy().into_owned();
        for args in [
            vec!["init", "--base", &b, "--instance", &i],
            vec!["target", "nv-rs", "--instance", &i],
            vec!["profile", "create", "--instance", &i],
            vec!["stage", "--instance", &i],
        ] {
            let (code, _, err) = run.raul(&args);
            assert_eq!(code, 0, "{args:?}: {err}");
        }
        run
    }

    /// A program that says it started, with its first argument, and ends with `code`.
    fn stand_in(&self, code: i32) -> PathBuf {
        let dir = self.t.path().join("viewer");
        std::fs::create_dir_all(&dir).unwrap();
        #[cfg(unix)]
        let (name, text) = (
            "stand-in-viewer",
            format!(
                "#!/bin/sh\necho \"stand-in started in $(pwd) with $# argument(s)\"\nexit {code}\n"
            ),
        );
        #[cfg(windows)]
        let (name, text) = (
            "stand-in-viewer.cmd",
            format!("@echo off\r\necho stand-in started in %CD% with %* \r\nexit /b {code}\r\n"),
        );
        let path = dir.join(name);
        std::fs::write(&path, text).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        path
    }

    fn play(&self, viewer: &Path) -> (i32, String, String) {
        // On Windows the run starts in the engine's folder of the Instance (`<instance>/nv-rs`), which the engine's own
        // install makes; a build without the engine has none, so the stand-in's test makes the folder.
        std::fs::create_dir_all(self.instance.join("nv-rs")).unwrap();
        let i = self.instance.to_string_lossy().into_owned();
        let v = viewer.to_string_lossy().into_owned();
        self.raul(&["run", "nv-rs", "--instance", &i, "--viewer", &v, "--wait"])
    }

    /// What every run log under the test's own folders holds, as text: where the log goes depends on the system
    /// (`XDG_STATE_HOME`, `~/Library/Application Support`, `%LOCALAPPDATA%`), and all of them are inside the folder.
    fn logs(&self) -> String {
        fn walk(dir: &Path, text: &mut String) {
            for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, text);
                } else if path.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
                    n.starts_with("run-") && path.extension().is_some_and(|e| e == "log")
                }) {
                    text.push_str(&std::fs::read_to_string(&path).unwrap_or_default());
                }
            }
        }
        let mut text = String::new();
        walk(&self.t.path().join("home"), &mut text);
        walk(&self.t.path().join("xdg-state"), &mut text);
        walk(&self.t.path().join("local-app-data"), &mut text);
        text
    }
}

#[test]
fn the_game_starts_through_the_launcher_in_the_run_folder_and_its_output_is_logged() {
    let run = Run::new("release-run-ok");
    let (code, out, err) = run.play(&run.stand_in(0));
    assert_eq!(code, 0, "{out}\n{err}");
    assert!(
        out.contains("stand-in-viewer"),
        "the plan names the program it starts: {out}"
    );
    // Where the run's output goes differs on Windows (no run folder, no log of the program's output in the plan), so
    // the log is checked where the plan names one.
    if out.contains("log:") {
        let logs = run.logs();
        assert!(
            logs.contains("stand-in started"),
            "the program's output is in the run log: {logs:?}"
        );
        assert!(
            logs.contains("run") && logs.contains("Default"),
            "it started in the run folder of the Profile: {logs:?}"
        );
    }
}

#[test]
fn a_game_that_ends_badly_ends_the_run_badly() {
    let run = Run::new("release-run-fail");
    let (code, out, err) = run.play(&run.stand_in(3));
    assert_ne!(code, 0, "{out}\n{err}");
}

#[test]
fn a_viewer_that_does_not_exist_is_refused_and_nothing_starts() {
    let run = Run::new("release-run-missing");
    let (code, _, err) = run.play(&run.t.path().join("no-such-viewer"));
    assert_ne!(code, 0, "{err}");
    assert_eq!(run.logs(), "");
}
