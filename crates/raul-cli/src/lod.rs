// SPDX-License-Identifier: GPL-3.0-only
//! `raul lod plan`: the `lodwright-request` document `raul lod generate` would send to lodwright for a Profile,
//! checked against lodwright's published schema; nothing is run or written. `raul lod generate`: generate LOD with
//! lodwright, linked into `raul`, from that request and keep its output as the Profile's LOD Mod,
//! `Generated LOD (<Profile>)`; `--json` streams `raul-events` lines. `raul lod mods`: the generated LOD Mods the
//! Profile lists (`raul-lod-mods`); nothing is written.

use std::fmt::Write as _;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use nvmod_instance::NvmodInstance;
use raul_core::config::{Config, Key};
use raul_drivers::events::Event;
use raul_drivers::lod::{self, LodRequest};

use crate::printer::Printer;
use raul_core::context::Context;

const USAGE_LINE: &str = "lod takes `plan [--instance <dir>] [--profile <name>] [--world <name>]... [--allow-in-repo] \
     [--json]`, `generate [--instance <dir>] [--profile <name>] [--world <name>]... [--allow-in-repo] [--dry-run] \
     [--timeout-secs <n>] [--json]`, `mods [--instance <dir>] [--profile <name>] [--json]` \
     or `settings show|set|unset|reset|undo ...`";

struct Flags {
    instance: Option<PathBuf>,
    profile: Option<String>,
    worlds: Vec<String>,
    json: bool,
    dry_run: bool,
    allow_in_repo: bool,
    timeout: Duration,
}

/// How long a LOD run may take before it is stopped. The limit is checked between terrain blocks, worlds and steps;
/// a step already running when it passes finishes first.
const DEFAULT_TIMEOUT: Duration = Duration::from_hours(2);

/// The longest `--timeout-secs` taken: a week.
const MAX_TIMEOUT_SECS: u64 = 7 * 24 * 60 * 60;

fn parse(args: &[String], generate: bool) -> Result<Flags, String> {
    let mut f = Flags {
        instance: None,
        profile: None,
        worlds: Vec::new(),
        json: false,
        dry_run: false,
        allow_in_repo: false,
        timeout: DEFAULT_TIMEOUT,
    };
    let mut rest = args.iter();
    while let Some(a) = rest.next() {
        let mut value = || {
            rest.next()
                .cloned()
                .ok_or_else(|| format!("{a} needs a value"))
        };
        match a.as_str() {
            "--json" => f.json = true,
            "--instance" => f.instance = Some(PathBuf::from(value()?)),
            "--profile" => f.profile = Some(value()?),
            "--world" => f.worlds.push(value()?),
            "--allow-in-repo" => f.allow_in_repo = true,
            "--dry-run" if generate => f.dry_run = true,
            "--timeout-secs" if generate => {
                let secs = value()?
                    .parse::<u64>()
                    .ok()
                    .filter(|s| (1..=MAX_TIMEOUT_SECS).contains(s))
                    .ok_or_else(|| {
                        format!(
                            "--timeout-secs needs a number of seconds from 1 to {MAX_TIMEOUT_SECS}"
                        )
                    })?;
                f.timeout = Duration::from_secs(secs);
            }
            other => return Err(format!("unknown option {other}")),
        }
    }
    Ok(f)
}

pub(crate) fn run(ctx: &Context, args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    match args.first().map(String::as_str) {
        Some("plan") => plan(ctx, &args[1..], out, err),
        Some("generate") => generate(ctx, &args[1..], out, err),
        Some("mods") => mods(ctx, &args[1..], out, err),
        Some("settings") => crate::lod_settings::run(ctx, &args[1..], out, err),
        _ => crate::usage(err, USAGE_LINE),
    }
}

fn refused(err: &mut dyn Write, why: &str) -> u8 {
    refused_as(err, "plan", why)
}

fn refused_as(err: &mut dyn Write, verb: &str, why: &str) -> u8 {
    let _ = writeln!(err, "raul lod {verb}: refused: {why}");
    crate::REFUSED
}

/// `raul lod mods`: the generated LOD Mods the Profile lists, highest priority first.
fn mods(ctx: &Context, args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let flags = match parse(args, false) {
        Ok(f) if f.worlds.is_empty() => f,
        Ok(_) => {
            return crate::usage(
                err,
                &format!("--world is not an option of mods\n{USAGE_LINE}"),
            );
        }
        Err(why) => return crate::usage(err, &format!("{why}\n{USAGE_LINE}")),
    };
    let Some(root) = flags.instance.or_else(|| configured_instance(ctx)) else {
        return refused_as(
            err,
            "mods",
            "no Instance: pass --instance <dir> or run `raul config set storage.root <dir>`",
        );
    };
    let instance = match NvmodInstance::open_read_only(&root) {
        Ok(i) => i,
        Err(e) => return refused_as(err, "mods", &format!("{}: {e}", root.display())),
    };
    let profile = match instance.resolve_profile(flags.profile.as_deref()) {
        Ok(p) => p,
        Err(e) => return refused_as(err, "mods", &e.to_string()),
    };
    let doc = lod::generated_mods(&instance, &profile);
    let text = if flags.json {
        format!(
            "{}\n",
            serde_json::to_string_pretty(&doc).unwrap_or_default()
        )
    } else if doc.mods.is_empty() {
        format!(
            "raul lod mods: profile `{profile}` lists no generated LOD Mod; `raul lod generate` makes `{}`\n",
            doc.own_name
        )
    } else {
        let mut text = format!("raul lod mods: profile `{profile}`\n");
        for m in &doc.mods {
            let state = if m.enabled { "enabled" } else { "disabled" };
            let built = match (&m.lodwright_version, m.legacy) {
                (_, true) => " (old name; the next `raul lod generate` replaces it)".to_owned(),
                (Some(v), false) => format!(" (lodwright {v})"),
                (None, false) => String::new(),
            };
            let _ = writeln!(text, "  {:>3}  {}  {state}{built}", m.position, m.name);
        }
        text
    };
    crate::write_or_fail(out, err, &text)
}

/// The Instance from `raul config`, when the flags name none.
pub(crate) fn configured_instance(ctx: &Context) -> Option<PathBuf> {
    ctx.config_file()
        .and_then(|f| Config::load(&f).ok())
        .and_then(|c| c.get(Key::Instance))
}

fn plan(ctx: &Context, args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let flags = match parse(args, false) {
        Ok(f) => f,
        Err(why) => return crate::usage(err, &format!("{why}\n{USAGE_LINE}")),
    };
    let Some(root) = flags.instance.or_else(|| configured_instance(ctx)) else {
        return refused(
            err,
            "no Instance: pass --instance <dir> or run `raul config set storage.root <dir>`",
        );
    };
    let instance = match NvmodInstance::open_read_only(&root) {
        Ok(i) => i,
        Err(e) => return refused(err, &format!("{}: {e}", root.display())),
    };
    let profile = match instance.resolve_profile(flags.profile.as_deref()) {
        Ok(p) => p,
        Err(e) => return refused(err, &e.to_string()),
    };
    let plan = match lod::plan(&instance, &profile, &flags.worlds, flags.allow_in_repo) {
        Ok(p) => p,
        Err(why) => return refused(err, &why),
    };
    let text = if flags.json {
        format!(
            "{}\n",
            serde_json::to_string_pretty(&plan.request).unwrap_or_default()
        )
    } else {
        let worlds = plan.request["worlds"]
            .as_array()
            .filter(|w| !w.is_empty())
            .map_or_else(
                || "every world with terrain".to_owned(),
                |w| {
                    w.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                },
            );
        let n = plan.plugins.len();
        format!(
            "raul lod plan: profile `{profile}`, {n} plugin{}, {worlds}\n  data:   {}\n  output: {}\n",
            if n == 1 { "" } else { "s" },
            plan.data_folder.display(),
            plan.output_dir.display()
        )
    };
    crate::write_or_fail(out, err, &text)
}

/// A `raul lod generate` run, read from its command line and `raul config`, not yet started.
pub struct LodJob {
    request: LodRequest,
    json: bool,
}

impl LodJob {
    /// The Instance the run writes to, when the flags or `raul config` name one: the folder whose lock the caller holds.
    #[must_use]
    pub fn instance(&self) -> Option<&std::path::Path> {
        Some(self.request.instance.as_path()).filter(|p| !p.as_os_str().is_empty())
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
        let ended = job.run_to(stop, &mut out, &mut err);
        out.finish();
        ended
    }

    fn run_to(
        self,
        stop: raul_drivers::Stop<'_>,
        out: &mut dyn Write,
        err: &mut dyn Write,
    ) -> (u8, Option<String>) {
        let Self { request: req, json } = self;
        let mut printer = Printer {
            command: "lod",
            out,
            err,
            json,
            seq: 0,
            last_percent: None,
        };
        let outcome = lod::generate_with_stop(&req, stop, &mut |e| printer.emit(e));
        if json {
            printer.emit(Event::Result {
                result: serde_json::to_value(&outcome.result).unwrap_or_default(),
            });
            printer.emit(outcome.finished());
        } else if let Some(e) = &outcome.result.error {
            let _ = writeln!(printer.err, "raul lod generate: {}", e.message);
        } else {
            let r = &outcome.result;
            let text = if r.dry_run {
                "raul lod generate: dry run passed; the request is valid and nothing was written\n"
                    .to_owned()
            } else if r.up_to_date {
                format!(
                    "raul lod generate: `{}` is up to date; nothing was written\n",
                    r.mod_name.as_deref().unwrap_or_default()
                )
            } else {
                format!(
                    "raul lod generate: wrote `{}` ({} files) into profile `{}`\n",
                    r.mod_name.as_deref().unwrap_or_default(),
                    r.files.unwrap_or(0),
                    r.profile
                )
            };
            let _ = printer.out.write_all(text.as_bytes());
        }
        let note = outcome.result.error.as_ref().map(|e| e.message.clone());
        (outcome.exit_code, note)
    }
}

/// Reads a `raul lod generate` command line (the words after `lod generate`) and what `raul config` leaves open,
/// without starting it.
///
/// # Errors
/// The usage message when the command line cannot be understood.
pub fn generate_job(args: &[String]) -> Result<LodJob, String> {
    generate_job_in(args, &Context::default())
}

/// [`generate_job`] reading the configuration from the folders of `ctx` instead of the person's own.
///
/// # Errors
/// The usage message when the command line cannot be understood.
pub fn generate_job_in(args: &[String], ctx: &Context) -> Result<LodJob, String> {
    let flags = parse(args, true).map_err(|why| format!("{why}\n{USAGE_LINE}"))?;
    let absolute = |p: PathBuf| std::path::absolute(&p).unwrap_or(p);
    let instance = flags
        .instance
        .or_else(|| configured_instance(ctx))
        .map(absolute);
    Ok(LodJob {
        request: LodRequest {
            instance: instance.unwrap_or_default(),
            profile: flags.profile,
            worlds: flags.worlds,
            dry_run: flags.dry_run,
            allow_in_repo: flags.allow_in_repo,
            timeout: flags.timeout,
        },
        json: flags.json,
    })
}

fn generate(ctx: &Context, args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let job = match generate_job_in(args, ctx) {
        Ok(job) => job,
        Err(why) => return crate::usage(err, &why),
    };
    // A stop request (Ctrl+C, SIGTERM, the GUI's cancel) stops the generation; the Instance stays as it was.
    let _ = ctrlc::set_handler(nvmod_core::cancel::request);
    job.run_to(raul_drivers::Stop::Process, out, err).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use raul_drivers::Stop;

    fn words(line: &[&str]) -> Vec<String> {
        line.iter().map(|w| (*w).to_owned()).collect()
    }

    #[test]
    fn a_run_with_no_instance_streams_its_refusal_and_its_end() {
        let dir = std::env::temp_dir().join(format!("raul-lod-job-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let config = dir.join("config.toml");
        let job = generate_job_in(&words(&["--dry-run"]), Some(&config)).unwrap();
        assert!(job.instance().is_none());
        let mut lines = Vec::new();
        let (code, note) = job.run_lines(Stop::Process, &mut |l| lines.push(l.to_owned()));
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(code, crate::REFUSED);
        assert!(note.is_some(), "{lines:?}");
        assert!(lines.iter().any(|l| l.contains("\"event\":\"result\"")));
        assert!(
            lines
                .last()
                .is_some_and(|l| l.contains("\"event\":\"finished\""))
        );
    }

    #[test]
    fn a_command_line_that_cannot_be_understood_is_a_usage_message() {
        let why = generate_job(&words(&["--no-such-flag"])).err().unwrap();
        assert!(why.contains("unknown option --no-such-flag"), "{why}");
    }
}
