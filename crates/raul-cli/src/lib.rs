// SPDX-License-Identifier: GPL-3.0-only
//! The `raul` command line. The Toolbox's own commands are handled here; every other command is nvmod's and is run
//! by `nvmod_cli` in this process (ADR-0002), so everything the GUI does is reachable from a shell.

mod known_good;
mod version;

use std::io::Write;

/// Exit code for success.
pub const OK: u8 = nvmod_cli::OK;
/// Exit code for a check that ran and found a problem.
pub const FOUND: u8 = nvmod_cli::FOUND;
/// Exit code for a command line that could not be understood.
pub const USAGE: u8 = nvmod_cli::USAGE;

/// The commands `raul` owns; anything else goes to nvmod. The GUI build exposes the same list (ADR-0004).
pub const OWN_COMMANDS: &[&str] = &["version", "known-good", "help"];

const HELP: &str = "raul: Raul's tRusty Toolbox

usage:
  raul --version | raul version [--json]
                              the Toolbox version and the Known-good set it ships
  raul known-good check [--repo <dir>] [--json]
                              check known-good.toml against the submodule pins and nvmod's tools.lock.toml
  raul <nvmod command> ...    every nvmod command (install, sort, lod, ttw, run, profile, doctor, ...);
                              `raul nvmod-help` shows them
";

/// Runs one command line (without the program name) and returns the exit code.
pub fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    match args.first().map(String::as_str) {
        None | Some("help" | "--help" | "-h") => write_or_fail(out, err, HELP),
        Some("--version" | "-V") => version::run(&[], out, err),
        Some("version") => version::run(&args[1..], out, err),
        Some("known-good") => known_good::run(&args[1..], out, err),
        Some("nvmod-help") => nvmod_cli::run_logged(&["--help".to_string()], out, err),
        Some(_) => nvmod_cli::run_logged(args, out, err),
    }
}

pub(crate) fn write_or_fail(out: &mut dyn Write, err: &mut dyn Write, text: &str) -> u8 {
    if out.write_all(text.as_bytes()).is_ok() {
        OK
    } else {
        let _ = writeln!(err, "raul: could not write the output");
        FOUND
    }
}

pub(crate) fn usage(err: &mut dyn Write, message: &str) -> u8 {
    let _ = writeln!(err, "raul: {message}\n\n{HELP}");
    USAGE
}
