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
fn the_version_names_the_known_good_set() {
    let (code, out) = raul(&["--version"]);
    assert_eq!(code, 0);
    assert!(
        out.starts_with("raul ") && out.contains("known-good set"),
        "{out}"
    );
}
