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
        .env("RAUL_CONFIG_DIR", "/nonexistent/raul-config")
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
