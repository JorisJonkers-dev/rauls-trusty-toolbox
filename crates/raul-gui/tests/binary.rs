// SPDX-License-Identifier: GPL-3.0-only
//! The release `raul` (ADR-0004): with arguments it is the same command line as the umbrella's `raul`, through the
//! same `raul_cli::run`. With none it opens the window, which the headless smoke step of `gui.yml` checks.
// Starts the built binary, which is not a tool call.
#![allow(clippy::disallowed_methods)]

use std::process::Command;

fn raul(args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_raul"))
        .args(args)
        .output()
        .expect("run raul");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

fn in_process(args: &[&str]) -> (i32, String) {
    let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = raul_cli::run(&args, &mut out, &mut err);
    (i32::from(code), String::from_utf8_lossy(&out).into_owned())
}

#[test]
fn every_argument_list_runs_the_command_line() {
    for args in [
        &["--version"][..],
        &["help"],
        &["nvmod-help"],
        &["no-such-command"],
    ] {
        assert_eq!(raul(args), in_process(args), "raul {args:?}");
    }
}

#[test]
fn the_version_names_the_toolbox_version() {
    let (code, out) = raul(&["--version"]);
    assert_eq!(code, 0);
    assert!(out.starts_with("raul "), "{out}");
}

#[test]
fn the_version_document_reads_as_the_toolbox_version() {
    // Versions shows this document's `raul` and nothing else; the page model reads it whether or not the command
    // still prints the release check's set and the Component versions beside it.
    let (code, out) = raul(&["version", "--json"]);
    assert_eq!(code, 0, "{out}");
    let doc: serde_json::Value = serde_json::from_str(out.trim()).expect("a raul version document");
    let version = doc
        .get("raul")
        .and_then(serde_json::Value::as_str)
        .expect("the Toolbox version");
    let shown = raul_ui_model::pages::versions::parse_version(&out).expect("the page reads it");
    assert_eq!(
        shown.raul, version,
        "the page reads only the Toolbox version"
    );
}

#[test]
fn the_update_check_is_a_command_of_the_release_build() {
    // The GUI's Check for updates runs exactly this; it reads the release list and downloads nothing (ADR-0005).
    let out = Command::new(env!("CARGO_BIN_EXE_raul"))
        .args(["update", "check", "--json"])
        // An absolute folder on every system (a bare "/nonexistent" has no drive on Windows, so it is not absolute).
        .env(
            "RAUL_CONFIG_DIR",
            std::env::temp_dir().join("raul-config-nonexistent"),
        )
        .output()
        .expect("run raul");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stdout.contains("\"current\"") || stderr.contains("raul update check"),
        "{stdout}{stderr}"
    );
    assert!(
        !stderr.contains("unknown command") && !stderr.contains("usage:"),
        "{stderr}"
    );
}

/// A copy of the built program in a folder of its own, with the files of a portable install around it.
fn portable_copy(dir: &std::path::Path) -> std::path::PathBuf {
    let exe = dir.join(format!("raul{}", std::env::consts::EXE_SUFFIX));
    std::fs::copy(env!("CARGO_BIN_EXE_raul"), &exe).expect("copy raul");
    exe
}

// macOS does not update itself (the update layout is `NotHere(Mac)`), so there is nothing to settle there.
#[cfg(not(target_os = "macos"))]
#[test]
fn a_new_version_that_fails_its_check_at_start_goes_back_to_the_old_one() {
    // The folder holds a finished, unconfirmed update whose list names a file that is not there: the new version's own
    // check fails at start, so the old files are put back and the program says so and stops (ADR-0020).
    let dir = std::env::temp_dir().join(format!("raul-settle-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let work = dir.join(".raul-update");
    std::fs::create_dir_all(work.join("previous")).unwrap();
    let exe = portable_copy(&dir);
    std::fs::write(dir.join("NOTICES"), "new").unwrap();
    std::fs::write(work.join("previous").join("NOTICES"), "old").unwrap();
    let missing = format!("{}  missing-file\n", "0".repeat(64));
    std::fs::write(dir.join("INSTALLED-FILES.sha256"), missing).unwrap();
    let journal = r#"{"state":"swapped","from":"0.0.1","to":"9.9.9","files":[{"path":"NOTICES","had_old":true}]}"#;
    std::fs::write(work.join("journal.json"), journal).unwrap();
    let out = Command::new(&exe).arg("--version").output().expect("run");
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(4), "{err}");
    assert!(err.contains("went back to the old version"), "{err}");
    assert_eq!(std::fs::read_to_string(dir.join("NOTICES")).unwrap(), "old");
    // The next start finds nothing to settle and runs.
    let again = Command::new(&exe)
        .arg("--version")
        .output()
        .expect("run again");
    assert_eq!(again.status.code(), Some(0));
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(not(feature = "engine"))]
#[test]
fn the_play_mode_without_the_engine_is_one_plain_refusal() {
    let out = Command::new(env!("CARGO_BIN_EXE_raul"))
        .args(["--play-mode", "/nowhere"])
        .output()
        .expect("run raul");
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(out.stdout, Vec::<u8>::new());
    assert_eq!(
        String::from_utf8_lossy(&out.stderr).trim(),
        raul_engine::NO_ENGINE_LINE
    );
}
