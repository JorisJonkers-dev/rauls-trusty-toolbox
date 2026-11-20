// SPDX-License-Identifier: GPL-3.0-only
//! `raul ttw`: install Tale of Two Wastelands with ttwright-rs and add it to the Instance as one Mod. The game
//! folders, the package and the Instance come from flags or from `raul config`. With `--json` the run is a stream of
//! `raul-events` lines on standard output (`contracts/README.md`); without it, progress goes to standard error.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use raul_core::config::{Config, Key};
use raul_drivers::events::Event;

use crate::printer::Printer;
use raul_drivers::ttw::{self, TtwErrorCode, TtwRequest};

/// The longest `--timeout-secs` taken: a week.
const MAX_TIMEOUT_SECS: u64 = 7 * 24 * 60 * 60;

const USAGE_LINE: &str = "ttw takes [--instance <dir>] [--profile <name>] [--fallout3 <dir>] [--newvegas <dir>] [--package <file>] [--dry-run] [--resume] [--timeout-secs <n>] [--json]";

fn value<'a>(rest: &mut impl Iterator<Item = &'a String>, flag: &str) -> Result<&'a str, String> {
    rest.next()
        .map(String::as_str)
        .ok_or_else(|| format!("{flag} needs a value"))
}

struct Flags {
    req: TtwRequest,
    instance: Option<PathBuf>,
    json: bool,
}

fn parse(args: &[String]) -> Result<Flags, String> {
    let mut req = TtwRequest {
        instance: PathBuf::new(),
        profile: None,
        fallout3: None,
        newvegas: None,
        package: None,
        dry_run: false,
        resume: false,
        timeout: ttw::DEFAULT_TIMEOUT,
        free_space: None,
    };
    let (mut instance, mut json) = (None, false);
    let mut rest = args.iter();
    while let Some(a) = rest.next() {
        match a.as_str() {
            "--json" => json = true,
            "--dry-run" => req.dry_run = true,
            "--resume" => req.resume = true,
            "--instance" => instance = Some(PathBuf::from(value(&mut rest, a)?)),
            "--profile" => req.profile = Some(value(&mut rest, a)?.to_owned()),
            "--fallout3" => req.fallout3 = Some(PathBuf::from(value(&mut rest, a)?)),
            "--newvegas" => req.newvegas = Some(PathBuf::from(value(&mut rest, a)?)),
            "--package" => req.package = Some(PathBuf::from(value(&mut rest, a)?)),
            "--timeout-secs" => {
                let secs = value(&mut rest, a)?
                    .parse::<u64>()
                    .ok()
                    .filter(|s| (1..=MAX_TIMEOUT_SECS).contains(s))
                    .ok_or_else(|| {
                        format!(
                            "--timeout-secs needs a number of seconds from 1 to {MAX_TIMEOUT_SECS}"
                        )
                    })?;
                req.timeout = Duration::from_secs(secs);
            }
            other => return Err(format!("unknown option {other}")),
        }
    }
    Ok(Flags {
        req,
        instance,
        json,
    })
}

pub(crate) fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let Flags {
        mut req,
        instance,
        json,
    } = match parse(args) {
        Ok(f) => f,
        Err(why) => return crate::usage(err, &format!("{why}\n{USAGE_LINE}")),
    };
    // A stop request (Ctrl+C, SIGTERM, the GUI's cancel) stops ttwright; its journal lets `--resume` continue.
    let _ = ctrlc::set_handler(nvmod_core::cancel::request);

    // What the flags leave open comes from `raul config`; a configuration that cannot be read is named, not fatal.
    let config = raul_core::config::default_file().map(|f| Config::load(&f));
    let config = match config {
        Some(Ok(c)) => Some(c),
        Some(Err(e)) => {
            let _ = writeln!(err, "raul ttw: the configuration is not used: {e}");
            None
        }
        None => None,
    };
    let from_config = |key| config.as_ref().and_then(|c: &Config| c.get(key));
    req.fallout3 = req.fallout3.or_else(|| from_config(Key::GameFallout3));
    req.newvegas = req.newvegas.or_else(|| from_config(Key::GameNewVegas));
    req.package = req.package.or_else(|| from_config(Key::TtwPackage));

    let mut printer = Printer {
        command: "ttw",
        out,
        err,
        json,
        seq: 0,
        last_percent: None,
    };
    // Paths in the result are absolute, whatever the working folder was.
    let absolute = |p: PathBuf| std::path::absolute(&p).unwrap_or(p);
    for path in [&mut req.fallout3, &mut req.newvegas, &mut req.package] {
        *path = path.take().map(absolute);
    }
    let outcome = match instance.or_else(|| from_config(Key::Instance)) {
        Some(dir) => {
            req.instance = absolute(dir);
            ttw::install(&req, &mut |e| printer.emit(e))
        }
        None => ttw::refused(
            &req,
            TtwErrorCode::InstanceInvalid,
            "no Instance: pass --instance <dir> or run `raul config set storage.root <dir>`",
        ),
    };
    let finished = outcome.finished();
    if json {
        printer.emit(Event::Result {
            result: serde_json::to_value(&outcome.result).unwrap_or_default(),
        });
        printer.emit(finished);
    } else {
        report(printer.out, printer.err, &outcome);
    }
    outcome.exit_code
}

fn report(out: &mut dyn Write, err: &mut dyn Write, outcome: &ttw::Outcome) {
    let r = &outcome.result;
    if let Some(e) = &r.error {
        let _ = writeln!(err, "raul ttw: {}", e.message);
        if let Some(staging) = r.staging.as_deref().filter(|_| r.resumable) {
            let _ = writeln!(
                err,
                "raul ttw: the work so far is kept in {}; run the same command with --resume to continue",
                staging.display()
            );
        }
        return;
    }
    let text = match &r.mod_name {
        Some(name) => format!(
            "raul ttw: added `{name}` to profile `{}` ({} files); enabled {}\n",
            r.profile,
            r.files.unwrap_or(0),
            if r.plugins_enabled.is_empty() {
                "no plugins".to_owned()
            } else {
                r.plugins_enabled.join(", ")
            }
        ),
        None => format!(
            "raul ttw: dry run passed for {}; nothing was written\n",
            show(&r.instance)
        ),
    };
    let _ = out.write_all(text.as_bytes());
}

fn show(p: &Path) -> String {
    p.display().to_string()
}
