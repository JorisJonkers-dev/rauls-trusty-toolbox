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

/// Why a read-only `raul lod` command did not answer: a usage error or a refusal, with the words it ends with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The command line cannot be understood (exit code 5).
    Usage(String),
    /// The command was refused (exit code 2): `raul lod <verb>: refused: <why>`.
    Refused {
        /// The subcommand: `plan` or `mods`.
        verb: &'static str,
        /// Why, in words.
        why: String,
    },
}

impl Refusal {
    /// The line the command prints on standard error, and the exit code it ends with.
    #[must_use]
    pub fn render(&self) -> (String, u8) {
        match self {
            Refusal::Usage(message) => {
                (format!("raul: {message}\n\n{}", crate::HELP), crate::USAGE)
            }
            Refusal::Refused { verb, why } => {
                (format!("raul lod {verb}: refused: {why}"), crate::REFUSED)
            }
        }
    }
}

fn print_refusal(err: &mut dyn Write, r: &Refusal) -> u8 {
    let (line, code) = r.render();
    let _ = writeln!(err, "{line}");
    code
}

/// The Instance and the Profile a read-only `raul lod` command looks at.
fn open_for(
    ctx: &Context,
    verb: &'static str,
    instance: Option<PathBuf>,
    profile: Option<&str>,
) -> Result<(NvmodInstance, String), Refusal> {
    let refused = |why: String| Refusal::Refused { verb, why };
    let Some(root) = instance.or_else(|| configured_instance(ctx)) else {
        return Err(refused(
            "no Instance: pass --instance <dir> or run `raul config set storage.root <dir>`".into(),
        ));
    };
    let instance = NvmodInstance::open_read_only(&root)
        .map_err(|e| refused(format!("{}: {e}", root.display())))?;
    let profile = instance
        .resolve_profile(profile)
        .map_err(|e| refused(e.to_string()))?;
    Ok((instance, profile))
}

/// The generated LOD Mods of a Profile, as `raul lod mods` reads them from its command line.
fn mods_of(ctx: &Context, args: &[String]) -> Result<(bool, String, lod::LodMods), Refusal> {
    let flags = match parse(args, false) {
        Ok(f) if f.worlds.is_empty() => f,
        Ok(_) => {
            return Err(Refusal::Usage(format!(
                "--world is not an option of mods\n{USAGE_LINE}"
            )));
        }
        Err(why) => return Err(Refusal::Usage(format!("{why}\n{USAGE_LINE}"))),
    };
    let (instance, profile) = open_for(ctx, "mods", flags.instance, flags.profile.as_deref())?;
    let doc = lod::generated_mods(&instance, &profile);
    Ok((flags.json, profile, doc))
}

/// The document `raul lod mods --json` prints, for the window's direct job.
///
/// # Errors
/// The usage error or refusal the command ends with.
pub fn mods_document(ctx: &Context, args: &[String]) -> Result<serde_json::Value, Refusal> {
    let (_, _, doc) = mods_of(ctx, args)?;
    serde_json::to_value(&doc).map_err(|e| Refusal::Refused {
        verb: "mods",
        why: e.to_string(),
    })
}

/// `raul lod mods`: the generated LOD Mods the Profile lists, highest priority first.
fn mods(ctx: &Context, args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let (json, profile, doc) = match mods_of(ctx, args) {
        Ok(x) => x,
        Err(r) => return print_refusal(err, &r),
    };
    let text = if json {
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
            let made = m.generated_at.map_or_else(String::new, |t| {
                format!(", generated {}", raul_drivers::lod::utc_minute(t))
            });
            let files = m.files.map_or_else(String::new, |n| {
                format!(", {n} file{}", if n == 1 { "" } else { "s" })
            });
            let _ = writeln!(
                text,
                "  {:>3}  {}  {state}{built}{made}{files}",
                m.position, m.name
            );
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

/// The plan of a run, as `raul lod plan` reads it from its command line: whether `--json` was given, the Profile and
/// the plan.
fn plan_of(ctx: &Context, args: &[String]) -> Result<(bool, String, lod::Plan), Refusal> {
    let flags = match parse(args, false) {
        Ok(f) => f,
        Err(why) => return Err(Refusal::Usage(format!("{why}\n{USAGE_LINE}"))),
    };
    let (instance, profile) = open_for(ctx, "plan", flags.instance, flags.profile.as_deref())?;
    let plan = lod::plan(&instance, &profile, &flags.worlds, flags.allow_in_repo)
        .map_err(|why| Refusal::Refused { verb: "plan", why })?;
    Ok((flags.json, profile, plan))
}

/// The document `raul lod plan --json` prints (the run's request), for the window's direct job.
///
/// # Errors
/// The usage error or refusal the command ends with.
pub fn plan_document(ctx: &Context, args: &[String]) -> Result<serde_json::Value, Refusal> {
    plan_of(ctx, args).map(|(_, _, plan)| plan.request)
}

fn plan(ctx: &Context, args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let (json, profile, plan) = match plan_of(ctx, args) {
        Ok(x) => x,
        Err(r) => return print_refusal(err, &r),
    };
    let text = if json {
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
        let ctx = Context::default().with_config_folder(&dir);
        let job = generate_job_in(&words(&["--dry-run"]), &ctx).unwrap();
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

    /// A synthetic Instance with a Profile and no generated LOD Mod.
    fn instance(label: &str) -> (Context, PathBuf, PathBuf) {
        use nvmod_testkit::BaseBuilder;
        let dir =
            std::env::temp_dir().join(format!("raul-lod-read-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let base = dir.join("base");
        BaseBuilder::steam().write(&base);
        let root = dir.join("instance");
        let (progress, cancel) = (&mut nvmod_api::NoProgress, nvmod_api::CancelToken::new());
        nvmod_api::init::create(
            &nvmod_api::init::CreateRequest {
                instance: root.clone(),
                base,
            },
            progress,
            &cancel,
        )
        .unwrap();
        nvmod_api::profile::create(
            &nvmod_api::profile::CreateRequest {
                instance: root.clone(),
                name: None,
            },
            &mut nvmod_api::NoProgress,
            &cancel,
        )
        .unwrap();
        let ctx = Context::default()
            .with_config_folder(dir.join("config"))
            .without_run_log();
        (ctx, root, dir)
    }

    #[test]
    fn the_mods_document_lists_no_generated_mod_on_a_fresh_instance_and_refuses_what_it_cannot_open()
     {
        let (ctx, root, dir) = instance("mods");
        let i = root.to_string_lossy().into_owned();
        let doc = mods_document(&ctx, &words(&["--instance", &i])).unwrap();
        assert_eq!(doc["contract"], "raul-lod-mods");
        assert_eq!(doc["mods"].as_array().map(Vec::len), Some(0));
        let missing = dir.join("nothing").to_string_lossy().into_owned();
        let refusal = mods_document(&ctx, &words(&["--instance", &missing])).unwrap_err();
        let (line, code) = refusal.render();
        assert_eq!(code, crate::REFUSED);
        assert!(line.starts_with("raul lod mods: refused:"), "{line}");
        let none = mods_document(&ctx, &[]).unwrap_err();
        assert!(none.render().0.contains("no Instance"));
        let world = mods_document(&ctx, &words(&["--world", "X"])).unwrap_err();
        assert_eq!(world.render().1, crate::USAGE);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_plan_document_is_the_request_and_a_bad_line_is_a_usage_error() {
        let (ctx, root, dir) = instance("plan");
        let i = root.to_string_lossy().into_owned();
        let request = plan_document(&ctx, &words(&["--instance", &i, "--allow-in-repo"]));
        // A plan needs plugins that carry worlds, which the synthetic Instance has none of: either it is a request
        // document or a refusal in the command's words, never a panic.
        match request {
            Ok(doc) => assert!(doc.is_object()),
            Err(r) => assert!(r.render().0.starts_with("raul lod plan: refused:")),
        }
        let bad = plan_document(&ctx, &words(&["--no-such-flag"])).unwrap_err();
        assert_eq!(bad.render().1, crate::USAGE);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
