// SPDX-License-Identifier: GPL-3.0-only
//! The `raul` command line. The Toolbox's own commands are handled here; every other command is nvmod's and is run
//! by `nvmod_cli` in this process (ADR-0002), so everything the GUI does is reachable from a shell.

mod bench;
mod config;
mod detect;
mod install_nv_rs;
mod lod;
mod printer;
mod setup;
mod ttw;
mod update;
mod version;

use std::io::Write;

/// Exit code for success.
pub const OK: u8 = nvmod_cli::OK;
/// Exit code for a check that ran and found a problem.
pub const FOUND: u8 = nvmod_cli::FOUND;
/// Exit code for a command that was refused before it changed anything.
pub const REFUSED: u8 = nvmod_cli::REFUSED;
/// Exit code for a step that failed; what it had changed was put back.
pub const FAILED: u8 = nvmod_cli::ROLLED_BACK;
/// Exit code for a command line that could not be understood.
pub const USAGE: u8 = nvmod_cli::USAGE;

/// The commands `raul` owns; anything else goes to nvmod. The GUI build exposes the same list (ADR-0004). An own
/// command takes precedence over an nvmod command of the same name, which `raul` then never reaches.
pub const OWN_COMMANDS: &[&str] = &[
    "version",
    "detect",
    "bench",
    "update",
    "config",
    "setup",
    "ttw",
    "lod",
    "install-nv-rs",
    "help",
];

/// Who runs a command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// One of the Toolbox's own commands ([`OWN_COMMANDS`], plus `--version`, `--help` and no command at all).
    Own,
    /// nvmod, in this process.
    Nvmod,
}

/// Who runs `args`.
#[must_use]
pub fn route(args: &[String]) -> Route {
    match args.first().map(String::as_str) {
        None | Some("--help" | "-h" | "--version" | "-V") => Route::Own,
        Some(command) if OWN_COMMANDS.contains(&command) => Route::Own,
        Some(_) => Route::Nvmod,
    }
}

const HELP: &str = "raul: Raul's tRusty Toolbox

usage:
  raul --version | raul version [--json]
                              the Toolbox version
  raul detect [--folder <dir>] [--json]
                              find Fallout: New Vegas and Fallout 3 copies (Steam, GOG, Epic), or check one folder
  raul bench [--quick] [--out <dir>]
                              run the benchmark suite on synthetic fixtures; writes bench.json and bench.md.
                              the LOD case stays skipped until a synthetic plan fixture exists
  raul bench compare <base.json> <head.json> [--threshold <percent>]
                              compare two reports; slower cases are warnings, never failures
  raul update check [--json]  look for a newer Toolbox release on GitHub (nothing is downloaded or sent)
  raul update auto [on|off]   show or set the GUI's update check at start (off until turned on)
  raul config list|get <key>|set <key> <path>|unset <key> [--json]
                              the game folders, the TTW package and the storage folder (game.newvegas,
                              game.fallout3, ttw.package, storage.root; instance is derived)
  raul setup --base <dir> --instance <dir> --masterlist <file> [--import-mo2 <dir>] [--mod <archive>]...
             [--fallout3 <dir> [--package <file>]] [--profile <name>] [--source <dir> [--cargo <path>]] [--skip-nv-rs]
             [--sort] [--copy] [--keep-partial] [--dry-run] [--json]
  raul setup --resume --instance <dir> [--json]
  raul setup --restart --instance <dir> --yes [--json]
                              a base copy to a modded, checked Instance in one run, resumable: each finished step is
                              kept in the Instance's journal, so --resume continues at the first unfinished step and
                              --restart removes an unfinished Instance; --json streams the events the setup wizard reads
  raul ttw [--instance <dir>] [--profile <name>] [--fallout3 <dir>] [--newvegas <dir>] [--package <file>]
           [--dry-run] [--resume] [--json]
                              install TTW with ttwright, which runs inside raul, and add it to the Instance as one Mod; what
                              the flags leave open comes from `raul config`; --json streams raul-events lines
  raul lod plan [--instance <dir>] [--profile <name>] [--world <name>]... [--allow-in-repo] [--json]
                              the lodwright request a LOD run would send for the Profile (its active plugins,
                              its Staging tree and its raul-lod.toml); nothing is run. An output folder inside a
                              git working tree is refused unless --allow-in-repo (this run only) or the Profile's
                              allow_in_repo key allows it
  raul lod generate [--instance <dir>] [--profile <name>] [--world <name>]... [--allow-in-repo] [--dry-run]
                    [--timeout-secs <n>] [--json]
                              generate LOD with lodwright, linked into raul, from that request and keep the output
                              as the Mod `LOD (lodwright)`; --json streams raul-events lines. --timeout-secs is
                              checked between terrain blocks, worlds and steps, so a step already running finishes
                              first
  raul install-nv-rs --instance <dir> [--source <dir> [--cargo <path>]]
                              put the engine that ships with raul in an Instance and check it; nothing is
                              downloaded. --source builds an nv-rs workspace instead (for developers)
  raul <nvmod command> ...    every nvmod command (install, sort, run, profile, doctor, ...);
                              `raul nvmod-help` shows them
";

/// Runs one command line (without the program name) and returns the exit code.
pub fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    match args.first().map(String::as_str) {
        None | Some("help" | "--help" | "-h") => write_or_fail(out, err, HELP),
        Some("--version" | "-V") => version::run(&[], out, err),
        Some("version") => version::run(&args[1..], out, err),
        Some("detect") => detect::run(&args[1..], out, err),
        Some("bench") => bench::run(&args[1..], out, err),
        Some("update") => update::run(&args[1..], out, err),
        Some("config") => config::run(&args[1..], out, err),
        Some("setup") => setup::run(&args[1..], out, err),
        Some("ttw") => ttw::run(&args[1..], out, err),
        Some("lod") => lod::run(&args[1..], out, err),
        Some("install-nv-rs") => install_nv_rs::run(&args[1..], out, err),
        Some("nvmod-help") => nvmod_cli::run_logged(&["--help".to_string()], out, err),
        Some(_) if route(args) == Route::Own => {
            let _ = writeln!(err, "raul: {} has no handler", args[0]);
            FAILED
        }
        Some(_) => nvmod_cli::run_logged(args, out, err),
    }
}

/// Prints a failed engine install as `raul install-nv-rs` does (exposed for its test).
#[doc(hidden)]
pub fn print_install_failure(e: &nvmod_api::JobError, err: &mut dyn Write) -> u8 {
    install_nv_rs::failure(e, err)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn words(line: &[&str]) -> Vec<String> {
        line.iter().map(|w| (*w).to_owned()).collect()
    }

    #[test]
    fn every_own_command_is_routed_to_the_toolbox_and_the_rest_to_nvmod() {
        for command in OWN_COMMANDS {
            assert_eq!(route(&words(&[command, "--json"])), Route::Own, "{command}");
        }
        for line in [&[][..], &["--help"], &["-V"]] {
            assert_eq!(route(&words(line)), Route::Own);
        }
        for command in ["catalog", "sort", "init", "plugins", "nvmod-help"] {
            assert_eq!(route(&words(&[command])), Route::Nvmod, "{command}");
        }
    }

    #[test]
    fn the_commands_that_moved_from_nvmod_are_own_commands() {
        for command in ["config", "setup", "ttw", "lod"] {
            assert!(OWN_COMMANDS.contains(&command), "{command}");
        }
    }
}
