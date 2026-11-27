// SPDX-License-Identifier: GPL-3.0-only
//! The `raul` command line. The Toolbox's own commands are handled here; every other command is nvmod's and is run
//! by `nvmod_cli` in this process (ADR-0002), so everything the GUI does is reachable from a shell.

pub mod addons;
mod bench;
mod config;
mod detect;
mod lod;
mod lod_settings;
mod modlist;

mod nexus;
pub mod play;
mod printer;
mod program_name;
pub mod scene;
mod setup;
mod sort;
mod space;
mod support_bundle;
mod target;
mod ttw;
mod update;
mod version;

pub use config::json_answer as config_answer;
pub use detect::copies as detect_copies;
pub use lod::{LodJob, generate_job as lod_generate_job, generate_job_in as lod_generate_job_in};
pub use lod::{
    Refusal as LodRefusal, mods_document as lod_mods_document, plan_document as lod_plan_document,
};
pub use lod_settings::answer as lod_settings_answer;
pub use ttw::{TtwJob, install_job as ttw_install_job, install_job_in as ttw_install_job_in};
pub use update::{Failure as UpdateFailure, answer as update_answer};
pub use version::document as version_document;

use std::io::Write;

pub use raul_core::context::Context;

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
    "space",
    "bench",
    "scene",
    "update",
    "config",
    "setup",
    "ttw",
    "lod",
    "addons",
    "modlist",

    "nexus",
    "support-bundle",
    "target",
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
  raul space --folder <dir> [--json]
                              the free space on the storage the folder is, or would be made, on
  raul detect [--folder <dir>] [--json]
                              find Fallout: New Vegas and Fallout 3 copies (Steam, GOG, Epic), or check one folder
  raul bench [--quick] [--out <dir>]
                              run the benchmark suite on synthetic fixtures; writes bench.json and bench.md.
                              the LOD case stays skipped until a synthetic plan fixture exists
  raul bench compare <base.json> <head.json> [--threshold <percent>]
                              compare two reports; slower cases are warnings, never failures
  raul scene run --places <file|mojave-day|mojave-night> [--instance <dir>] [--profile <name>]
                 [--variant <name>=profile|profile-without-lod|game|<folder>]... [--only <place>,<place>]
                 [--out <dir>] [--resolution WxH] [--grids <n>] [--quick] [--game <dir>] [--no-cap]
                              pictures of the places, one game start per variant, then the difference between the
                              variants. With a Profile that has a generated LOD Mod and no --variant: the Profile
                              with it (with-lod) and without it (without-lod). Pictures go under raul's data folder
                              unless --out says otherwise. Open both pictures before you say they are equal
  raul scene compare <a> <b> [--noise <a2>]
                              the difference between two pictures, or two folders of pictures by name
  raul scene soak --places <file> [--frames <n>] [--instance <dir>] [--profile <name>]
                              hold every place for n frames and count stops, errors and warnings
  raul scene sweep --places <file> [--instance <dir>] [--profile <name>]
                              the stability sweep: a place that stops or hangs is recorded and the sweep goes on
  raul scene places [--out <dir>]
                              write the place files raul carries (mojave-day, mojave-night) to edit
  raul bench game --suite quick|mojave|soak|capital [--baseline <name> | --save-baseline <name>] [--instance <dir>]
                  [--profile <name>]
                              the in-engine benchmark: starts the game, takes the machine's game lock exclusively and
                              is never run by CI. raul bench game compare <a.json> <b.json> compares two summaries.
                              `raul bench` and `raul bench compare` (the synthetic suite) are unchanged
  raul update check [--json]  look for a newer Toolbox release on GitHub (nothing is downloaded or sent)
  raul update status [--json] whether this install can update itself, and what is downloaded or kept
  raul update download [--tag <tag>] [--allow-older] [--json]
                              download the newest release (or that tag), check it against the release's checksums
                              and get it ready; this is the only update command that downloads. An older version
                              is refused unless --allow-older
  raul update install [--json]
                              install what was downloaded, check that the new version starts and works, and go back
                              at once if it does not
  raul update rollback [--json]
                              go back to the version before the last update
  raul update self-check [--expect <version>] [--json]
                              the check a new version runs on itself (used by install)
  raul update auto [on|off]   show or set the GUI's update check at start (off until turned on)
  raul config list|get <key>|set <key> <path>|unset <key> [--json]
                              the game folders, the TTW package and the storage folder (game.newvegas,
                              game.fallout3, ttw.package, storage.root; instance is derived)
  raul setup --base <dir> --instance <dir> --masterlist <file> [--import-mo2 <dir>] [--mod <archive>]...
             [--fallout3 <dir> [--package <file>]] [--profile <name>] [--skip-nv-rs]
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
                              as the Mod `Generated LOD (<Profile>)`; --json streams raul-events lines. --timeout-secs is
                              checked between terrain blocks, worlds and steps, so a step already running finishes
                              first
  raul lod mods [--instance <dir>] [--profile <name>] [--json]
                              the generated LOD Mods the profile lists, highest priority first; nothing is written
  raul lod settings show [--instance <dir>] [--profile <name>] [--json]
                              every LOD option of the Profile with its value and default; * marks one its
                              raul-lod.toml sets
  raul lod settings set <key> <value>... [--instance <dir>] [--profile <name>] [--json]
                              set one option (true/false, a number, or the names of a list); checked first, and a
                              refused value changes nothing
  raul lod settings unset <key> [--instance <dir>] [--profile <name>] [--json]
                              remove one key from raul-lod.toml, so the option takes its default
  raul lod settings reset [--dry-run] [--instance <dir>] [--profile <name>] [--json]
                              Restore defaults: remove every LOD generation setting (allow_in_repo stays), keeping
                              the file as it was in raul-lod.toml.previous; --dry-run names what it would remove
  raul lod settings undo [--instance <dir>] [--profile <name>] [--json]
                              put back the file the last reset kept (one level; a later set or unset ends it)
  raul addons list [--instance <dir>] [--profile <name>] [--json]
                              raul's add-ons (small Mods raul makes on this machine for a mod the Profile has, such
                              as a small lamp glow for Street Lights TTW) and where each stands; nothing is written
  raul addons apply [--instance <dir>] [--profile <name>] [--dry-run] [--json]
                              add or update them: a Profile with the add-on's plugin active gets its Mod, above
                              that mod; one without it is not touched; one you turned off stays off. `raul setup`
                              does this at its end, and `raul run` and `raul scene` check it before the game starts
  raul modlist list [--json]
                              the Mod lists raul carries
  raul modlist show <list> [--json]
                              what a Mod list holds, in the order the Mods go in; <list> is a bundled id (such as
                              beautiful) or a file; nothing is written
  raul modlist plan <list> [--instance <dir>] [--profile <name>] [--json]
                              what the Profile has of the list and what it lacks, where to download each missing Mod
                              (a Nexus page or another link, or what to do by hand) and the install order; nothing
                              is read from Nexus and nothing is written

  raul nexus key set [--file]|clear|status [--json]
                              the Nexus Mods API key: set reads it from standard input (never the command line) and keeps it
                              in the OS keyring, or with --file in a file only you can read; NEXUS_API_KEY overrides both.
                              status asks Nexus who the key belongs to and whether the account is premium
  raul support-bundle [--instance <dir>] [--hide-names] [--out <file>] [--apply] [--json]
                              what a bug report needs, in one zip: versions, system, the folders of `raul config`,
                              the Instance's Mods, plugins and Doctor report, and the last run logs. Without --apply
                              it prints the plan; nothing is sent and no game file's contents are read. The home
                              folder is written as `~`; --hide-names also replaces Mod and plugin names with hashes
  raul run --stop --instance <dir> [--profile <name>] [--json]
                              ask the running game to stop between two frames (the play mode's stop file)
  raul <nvmod command> ...    every nvmod command (install, sort, run, profile, doctor, ...);
                              `raul nvmod-help` shows them
";

/// Runs one command line (without the program name) and returns the exit code.
pub fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    run_in(&Context::default(), args, out, err)
}

/// [`run`] with the folders and switches of `ctx` instead of the person's own: how the window and the tests run a
/// command without touching the environment of the process.
pub fn run_in(ctx: &Context, args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    match args.first().map(String::as_str) {
        None | Some("help" | "--help" | "-h") => write_or_fail(out, err, HELP),
        Some("--version" | "-V") => version::run(&[], out, err),
        Some("version") => version::run(&args[1..], out, err),
        Some("detect") => detect::run(&args[1..], out, err),
        Some("space") => space::run(&args[1..], out, err),
        Some("bench") if args.get(1).map(String::as_str) == Some("game") => {
            scene::bench_game(ctx, &args[2..], out, err)
        }
        Some("bench") => bench::run(&args[1..], out, err),
        Some("scene") => scene::run(ctx, &args[1..], out, err),
        Some("update") => update::run(ctx, &args[1..], out, err),
        Some("config") => config::run(ctx, &args[1..], out, err),
        Some("setup") => setup::run(&args[1..], out, err),
        Some("ttw") => ttw::run(ctx, &args[1..], out, err),
        Some("lod") => lod::run(ctx, &args[1..], out, err),
        Some("addons") => addons::run(ctx, &args[1..], out, err),
        Some("modlist") => modlist::run(ctx, &args[1..], out, err),

        Some("nexus") => nexus::run(ctx, &args[1..], out, err),
        Some("support-bundle") => support_bundle::run(ctx, &args[1..], out, err),
        Some("install-nv-rs") => gone_install_nv_rs(err),
        Some("target") => target::run(args, out, err),
        Some("run") if args.iter().any(|a| a == "--stop") => play::stop(&args[1..], out, err),
        Some("run") if play::enabled() => {
            addons_before_run(args, err);
            run_in_play_mode(args, out, err)
        }
        Some("run") if args.iter().any(|a| a == "--json") => {
            addons_before_run(args, err);
            play::run_plain(args, ctx.run_log(), out, err)
        }
        Some("run") => {
            addons_before_run(args, err);
            let mut out = program_name::Renamed::new(out);
            let mut err = program_name::Renamed::new(err);
            play::run_plain(args, ctx.run_log(), &mut out, &mut err)
        }
        Some("sort") => match sort::prepare(&args[1..], err) {
            Ok(None) => nvmod(ctx, args, out, err),
            Ok(Some(rest)) => {
                let mut words = vec!["sort".to_owned()];
                words.extend(rest);
                nvmod(ctx, &words, out, err)
            }
            Err(code) => code,
        },
        Some("nvmod-help") => nvmod(ctx, &["--help".to_string()], out, err),
        Some(_) if route(args) == Route::Own => {
            let _ = writeln!(err, "raul: {} has no handler", args[0]);
            FAILED
        }
        Some(_) => nvmod(ctx, args, out, err),
    }
}

/// An nvmod command, with its run log unless the context says none.
fn nvmod(ctx: &Context, args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    // A `--json` document is data for a program: its text is not rewritten.
    if args.iter().any(|a| a == "--json") {
        return if ctx.run_log() {
            nvmod_cli::run_logged(args, out, err)
        } else {
            nvmod_cli::run(args, out, err)
        };
    }
    let mut out = program_name::Renamed::new(out);
    let mut err = program_name::Renamed::new(err);
    if ctx.run_log() {
        nvmod_cli::run_logged(args, &mut out, &mut err)
    } else {
        nvmod_cli::run(args, &mut out, &mut err)
    }
}

/// `raul run` with the game started as raul's own play mode, logged like every nvmod command.
/// The add-ons check before `raul run` starts the game (not for a dry run): the Instance and Profile come from the
/// command line; without `--instance` nvmod finds none either, and nothing is checked. It never stops the run.
fn addons_before_run(args: &[String], err: &mut dyn Write) {
    let value = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
    };
    if args.iter().any(|a| a == "--dry-run") {
        return;
    }
    if let Some(root) = value("--instance") {
        addons::before_game(
            std::path::Path::new(root),
            value("--profile").map(String::as_str),
            true,
            err,
        );
    }
}

fn run_in_play_mode(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    match nvmod_cli::begin_run_log(args) {
        Ok((rest, session)) => {
            let code = play::run(&rest, out, err);
            session.finish(code);
            code
        }
        Err(why) => {
            let _ = writeln!(err, "nvmod: {why}");
            USAGE
        }
    }
}

/// `raul install-nv-rs` is gone: the engine is part of `raul` (ADR-0016). The name stays reserved so that nvmod's
/// command of the same name, which fetches a release, is never reached through `raul`.
fn gone_install_nv_rs(err: &mut dyn Write) -> u8 {
    let _ = writeln!(
        err,
        "raul install-nv-rs: this command is gone: the engine is part of raul and needs no install; \
         `raul target nv-rs --instance <dir>` makes an Instance run on it"
    );
    USAGE
}

/// At start, before anything else: settle an update that is half done or not yet confirmed (ADR-0020). Returns an exit
/// code when this program must stop because the old version was put back (the person starts raul again), else `None`.
#[must_use]
pub fn settle_update_at_start(args: &[String], err: &mut dyn Write) -> Option<u8> {
    update::settle_at_start(args, err)
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
    use raul_testkit::{covers, ids};

    fn words(line: &[&str]) -> Vec<String> {
        line.iter().map(|w| (*w).to_owned()).collect()
    }

    #[test]
    fn every_own_command_is_routed_to_the_toolbox_and_the_rest_to_nvmod() {
        covers(&[ids::cli::commands::OWN_COMMANDS]);
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
        for command in ["config", "setup", "ttw", "lod", "target"] {
            assert!(OWN_COMMANDS.contains(&command), "{command}");
        }
    }

    /// `raul run` without a target is refused by nvmod's usage rule before anything starts; the words say `raul`,
    /// except in a `--json` run, where the text is left as it is.
    #[test]
    fn a_run_with_no_target_is_a_usage_error_that_names_raul() {
        let ctx = Context::default().without_run_log();
        for (line, renamed) in [(&["run"][..], true), (&["run", "--json"], false)] {
            let (mut out, mut err) = (Vec::new(), Vec::new());
            let code = run_in(&ctx, &words(line), &mut out, &mut err);
            assert_eq!(code, USAGE, "{line:?}");
            let text = String::from_utf8(err).unwrap() + &String::from_utf8(out).unwrap();
            assert_eq!(text.contains("raul run"), renamed, "{line:?}: {text}");
            assert!(renamed || text.contains("nvmod run"), "{text}");
        }
    }
}
