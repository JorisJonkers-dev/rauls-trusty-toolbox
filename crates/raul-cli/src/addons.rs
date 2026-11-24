// SPDX-License-Identifier: GPL-3.0-only
//! `raul addons list`: raul's add-ons and where each stands in a Profile (Contract `raul-addons`); nothing is written.
//! `raul addons apply`: keep them as raul's list says (see [`raul_drivers::addons`]): a Profile with an add-on's
//! trigger plugin active gets the add-on's Mod, made on this machine; one without it is not touched. The same apply
//! runs at the end of `raul setup` and, as a quick check, before `raul run` and `raul scene` start the game; there a
//! failure is a warning line and the game starts anyway.

use std::fmt::Write as _;
use std::io::Write;
use std::path::{Path, PathBuf};

use nvmod_instance::NvmodInstance;
use raul_core::context::Context;
use raul_core::instance_lock::InstanceGuard;
use raul_drivers::addons::{self, AddonsReport, Status};

const USAGE_LINE: &str = "addons takes `list [--instance <dir>] [--profile <name>] [--json]` or \
     `apply [--instance <dir>] [--profile <name>] [--dry-run] [--json]`";

struct Flags {
    instance: Option<PathBuf>,
    profile: Option<String>,
    json: bool,
    dry_run: bool,
}

fn parse(args: &[String], apply: bool) -> Result<Flags, String> {
    let mut f = Flags {
        instance: None,
        profile: None,
        json: false,
        dry_run: !apply,
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
            "--dry-run" if apply => f.dry_run = true,
            other => return Err(format!("unknown option {other}")),
        }
    }
    Ok(f)
}

/// `raul addons ...`.
pub(crate) fn run(ctx: &Context, args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let verb = args.first().map(String::as_str);
    let flags = match verb {
        Some("list") => parse(&args[1..], false),
        Some("apply") => parse(&args[1..], true),
        _ => Err("addons needs `list` or `apply`".to_owned()),
    };
    let flags = match flags {
        Ok(f) => f,
        Err(why) => {
            let _ = writeln!(err, "raul: {why}\n{USAGE_LINE}\n\n{}", crate::HELP);
            return crate::USAGE;
        }
    };
    let verb = verb.unwrap_or_default();
    let refuse = |err: &mut dyn Write, why: String| {
        let _ = writeln!(err, "raul addons {verb}: refused: {why}");
        crate::REFUSED
    };
    let Some(root) = flags
        .instance
        .clone()
        .or_else(|| crate::lod::configured_instance(ctx))
    else {
        return refuse(
            err,
            "no Instance: pass --instance <dir> or run `raul config set storage.root <dir>`".into(),
        );
    };
    let opened = if flags.dry_run {
        NvmodInstance::open_read_only(&root)
    } else {
        NvmodInstance::open(&root)
    };
    let instance = match opened {
        Ok(i) => i,
        Err(e) => return refuse(err, format!("{}: {e}", root.display())),
    };
    let profile = match instance.resolve_profile(flags.profile.as_deref()) {
        Ok(p) => p,
        Err(e) => return refuse(err, e.to_string()),
    };
    let _guard = if flags.dry_run {
        None
    } else {
        match InstanceGuard::acquire(&root, "addons apply") {
            Ok(g) => Some(g),
            Err(e) => return refuse(err, e.to_string()),
        }
    };
    let report = addons::apply(&instance, &profile, flags.dry_run);
    let text = if flags.json {
        format!(
            "{}\n",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        )
    } else {
        text(&report)
    };
    let code = crate::write_or_fail(out, err, &text);
    if code != crate::OK {
        return code;
    }
    if report.failed().is_empty() {
        crate::OK
    } else {
        for a in report.failed() {
            let _ = writeln!(
                err,
                "raul addons {verb}: {} failed: {}",
                a.name,
                a.error.as_deref().unwrap_or_default()
            );
        }
        crate::FAILED
    }
}

fn status_words(status: Status) -> &'static str {
    match status {
        Status::NotNeeded => "not needed: its plugin is not active",
        Status::UpToDate => "in place",
        Status::OutOfDate => "would be added or made again (`raul addons apply`)",
        Status::Added => "added",
        Status::Updated => "made again",
        Status::TurnedOff => "turned off by you; raul leaves it off",
        Status::Failed => "failed",
    }
}

/// The report for people: one line per add-on, then one per change.
fn text(report: &AddonsReport) -> String {
    let mut text = format!("raul addons: profile `{}`\n", report.profile);
    for a in &report.addons {
        let _ = writeln!(
            text,
            "  {}  (for {}): {}",
            a.name,
            a.trigger_plugin,
            status_words(a.status)
        );
        for c in &a.changes {
            let _ = writeln!(text, "    {c}");
        }
        if let Some(e) = &a.error {
            let _ = writeln!(text, "    {e}");
        }
    }
    text
}

/// The lines that tell what an apply changed, for a command that applied the add-ons along the way.
#[must_use]
pub fn change_lines(report: &AddonsReport) -> Vec<String> {
    let mut lines = Vec::new();
    for a in &report.addons {
        for c in &a.changes {
            lines.push(format!("raul add-on {}: {c}", a.name));
        }
        if let Some(e) = &a.error {
            lines.push(format!("raul add-on {}: not applied: {e}", a.name));
        }
    }
    lines
}

/// The add-ons of `profile` (the selected one when `None`) of the Instance at `root`, applied before a game starts:
/// what changed is written to `err`, and anything that goes wrong is a warning line there; it never stops the game.
/// With `lock` the Instance lock is taken for the check; a caller that holds it already passes `false`.
pub fn before_game(root: &Path, profile: Option<&str>, lock: bool, err: &mut dyn Write) {
    let applied = (|| -> Result<AddonsReport, String> {
        let _guard = if lock {
            Some(InstanceGuard::acquire(root, "addons check").map_err(|e| e.to_string())?)
        } else {
            None
        };
        let instance = NvmodInstance::open(root).map_err(|e| e.to_string())?;
        let profile = instance
            .resolve_profile(profile)
            .map_err(|e| e.to_string())?;
        Ok(addons::apply(&instance, &profile, false))
    })();
    match applied {
        Ok(report) => {
            for line in change_lines(&report) {
                let _ = writeln!(err, "{line}");
            }
        }
        Err(why) => {
            let _ = writeln!(
                err,
                "raul: warning: the add-ons were not checked before the start: {why}"
            );
        }
    }
}
