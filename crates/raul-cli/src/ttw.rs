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

/// A `raul ttw` run, read from its command line and `raul config`, not yet started.
pub struct TtwJob {
    request: TtwRequest,
    instance: Option<PathBuf>,
    json: bool,
    config_warning: Option<String>,
}

impl TtwJob {
    /// The Instance the run writes to, when the flags or `raul config` name one: the folder whose lock the caller holds.
    #[must_use]
    pub fn instance(&self) -> Option<&Path> {
        self.instance.as_deref()
    }

    /// Runs it as a stream of `raul-events` lines, handing each to `sink` as it is made, stopping when `stop` says so.
    /// Returns the exit code and, when the run failed or was stopped, the reason in one line.
    pub fn run_lines(
        self,
        stop: raul_drivers::Stop<'_>,
        sink: &mut dyn FnMut(&str),
    ) -> (u8, Option<String>) {
        let mut out = crate::printer::LineSink::new(sink);
        let mut err = Vec::new();
        let job = Self { json: true, ..self };
        let (code, note) = job.run_to(stop, &mut out, &mut err);
        out.finish();
        (code, note)
    }

    fn run_to(
        self,
        stop: raul_drivers::Stop<'_>,
        out: &mut dyn Write,
        err: &mut dyn Write,
    ) -> (u8, Option<String>) {
        let Self {
            mut request,
            instance,
            json,
            config_warning,
        } = self;
        if let Some(warning) = config_warning {
            let _ = writeln!(err, "raul ttw: the configuration is not used: {warning}");
        }
        let mut printer = Printer {
            command: "ttw",
            out,
            err,
            json,
            seq: 0,
            last_percent: None,
        };
        let outcome = match instance {
            Some(dir) => {
                request.instance = dir;
                ttw::install_with_stop(&request, stop, &mut |e| printer.emit(e))
            }
            None => ttw::refused(
                &request,
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
        let note = outcome.result.error.as_ref().map(|e| e.message.clone());
        (outcome.exit_code, note)
    }
}

/// Reads a `raul ttw` command line (the words after `ttw`) and what `raul config` leaves open, without starting it.
///
/// # Errors
/// The usage message when the command line cannot be understood.
pub fn install_job(args: &[String]) -> Result<TtwJob, String> {
    install_job_in(args, &raul_core::context::Context::default())
}

/// [`install_job`] reading the configuration from the folders of `ctx` instead of the person's own: how a test points a
/// run at a configuration of its own without touching the environment of the process.
///
/// # Errors
/// The usage message when the command line cannot be understood.
pub fn install_job_in(
    args: &[String],
    ctx: &raul_core::context::Context,
) -> Result<TtwJob, String> {
    let Flags {
        mut req,
        instance,
        json,
    } = parse(args).map_err(|why| format!("{why}\n{USAGE_LINE}"))?;

    // What the flags leave open comes from `raul config`; a configuration that cannot be read is named, not fatal.
    let config = ctx.config_file().map(|f| Config::load(&f));
    let (config, config_warning) = match config {
        Some(Ok(c)) => (Some(c), None),
        Some(Err(e)) => (None, Some(e.to_string())),
        None => (None, None),
    };
    let from_config = |key| config.as_ref().and_then(|c: &Config| c.get(key));
    req.fallout3 = req.fallout3.or_else(|| from_config(Key::GameFallout3));
    req.newvegas = req.newvegas.or_else(|| from_config(Key::GameNewVegas));
    req.package = req.package.or_else(|| from_config(Key::TtwPackage));
    // Paths in the result are absolute, whatever the working folder was.
    let absolute = |p: PathBuf| std::path::absolute(&p).unwrap_or(p);
    for path in [&mut req.fallout3, &mut req.newvegas, &mut req.package] {
        *path = path.take().map(absolute);
    }
    let instance = instance
        .or_else(|| from_config(Key::Instance))
        .map(absolute);
    Ok(TtwJob {
        request: req,
        instance,
        json,
        config_warning,
    })
}

pub(crate) fn run(
    ctx: &raul_core::context::Context,
    args: &[String],
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let job = match install_job_in(args, ctx) {
        Ok(job) => job,
        Err(why) => return crate::usage(err, &why),
    };
    // A stop request (Ctrl+C, SIGTERM, the GUI's cancel) stops ttwright; its journal lets `--resume` continue.
    let _ = ctrlc::set_handler(nvmod_core::cancel::request);
    job.run_to(raul_drivers::Stop::Process, out, err).0
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

#[cfg(test)]
mod tests {
    use super::*;
    use raul_drivers::Stop;

    fn words(line: &[&str]) -> Vec<String> {
        line.iter().map(|w| (*w).to_owned()).collect()
    }

    #[test]
    fn a_run_with_no_instance_streams_its_refusal_and_its_end_without_starting_anything() {
        let dir = std::env::temp_dir().join(format!("raul-ttw-job-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let config = dir.join("config.toml");
        let job = install_job_in(&words(&["--dry-run"]), Some(&config)).unwrap();
        assert!(job.instance().is_none());
        let mut lines = Vec::new();
        let (code, note) = job.run_lines(Stop::Process, &mut |l| lines.push(l.to_owned()));
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(code, crate::REFUSED);
        assert!(note.is_some_and(|n| n.contains("no Instance")), "{lines:?}");
        assert!(lines.iter().any(|l| l.contains("\"event\":\"result\"")));
        assert!(
            lines
                .last()
                .is_some_and(|l| l.contains("\"event\":\"finished\""))
        );
    }

    #[test]
    fn a_command_line_that_cannot_be_understood_is_a_usage_message() {
        let why = install_job(&words(&["--no-such-flag"])).err().unwrap();
        assert!(why.contains("unknown option --no-such-flag"), "{why}");
    }
}
