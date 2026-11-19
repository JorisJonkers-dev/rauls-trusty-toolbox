// SPDX-License-Identifier: GPL-3.0-only
//! `raul lod plan`: the `lodwright-request` document `raul lod generate` would send to lodwright for a Profile,
//! checked against lodwright's published schema; nothing is run or written. `raul lod generate`: generate LOD with
//! lodwright, linked into `raul`, from that request and keep its output as the Mod `LOD (lodwright)`; `--json` streams
//! `raul-events` lines.

use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use nvmod_instance::NvmodInstance;
use raul_core::config::{Config, Key};
use raul_drivers::events::Event;
use raul_drivers::lod::{self, LodRequest};

use crate::printer::Printer;

const USAGE_LINE: &str = "lod takes `plan [--instance <dir>] [--profile <name>] [--world <name>]... [--allow-in-repo] \
     [--json]` or `generate [--instance <dir>] [--profile <name>] [--world <name>]... [--allow-in-repo] [--dry-run] \
     [--timeout-secs <n>] [--json]` or `settings show|set|unset|reset|undo ...`";

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

pub(crate) fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    match args.first().map(String::as_str) {
        Some("plan") => plan(&args[1..], out, err),
        Some("generate") => generate(&args[1..], out, err),
        Some("settings") => crate::lod_settings::run(&args[1..], out, err),
        _ => crate::usage(err, USAGE_LINE),
    }
}

fn refused(err: &mut dyn Write, why: &str) -> u8 {
    let _ = writeln!(err, "raul lod plan: refused: {why}");
    crate::REFUSED
}

/// The Instance from `raul config`, when the flags name none.
pub(crate) fn configured_instance() -> Option<PathBuf> {
    raul_core::config::default_file()
        .and_then(|f| Config::load(&f).ok())
        .and_then(|c| c.get(Key::Instance))
}

fn plan(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let flags = match parse(args, false) {
        Ok(f) => f,
        Err(why) => return crate::usage(err, &format!("{why}\n{USAGE_LINE}")),
    };
    let Some(root) = flags.instance.or_else(configured_instance) else {
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

fn generate(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let flags = match parse(args, true) {
        Ok(f) => f,
        Err(why) => return crate::usage(err, &format!("{why}\n{USAGE_LINE}")),
    };
    // A stop request (Ctrl+C, SIGTERM, the GUI's cancel) stops the generation; the Instance stays as it was.
    let _ = ctrlc::set_handler(nvmod_core::cancel::request);
    let absolute = |p: PathBuf| std::path::absolute(&p).unwrap_or(p);
    let instance = flags.instance.or_else(configured_instance).map(absolute);
    let req = LodRequest {
        instance: instance.clone().unwrap_or_default(),
        profile: flags.profile,
        worlds: flags.worlds,
        dry_run: flags.dry_run,
        allow_in_repo: flags.allow_in_repo,
        timeout: flags.timeout,
    };
    let json = flags.json;
    let mut printer = Printer {
        command: "lod",
        out,
        err,
        json,
        seq: 0,
        last_percent: None,
    };
    let outcome = lod::generate(&req, &mut |e| printer.emit(e));
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
                lod::MOD_NAME
            )
        } else {
            format!(
                "raul lod generate: wrote `{}` ({} files) into profile `{}`\n",
                lod::MOD_NAME,
                r.files.unwrap_or(0),
                r.profile
            )
        };
        let _ = printer.out.write_all(text.as_bytes());
    }
    outcome.exit_code
}
