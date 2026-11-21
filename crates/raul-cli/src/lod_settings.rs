// SPDX-License-Identifier: GPL-3.0-only
//! `raul lod settings show|set|unset|reset|undo`: a Profile's LOD settings, option by option, through the same
//! functions the GUI's LOD settings view calls (`raul_drivers::lod::settings`). `--json` prints one
//! `raul-lod-settings` document.

use std::io::Write;
use std::path::PathBuf;

use nvmod_instance::NvmodInstance;
use raul_drivers::lod::settings::{self, ProfileSettings, Refusal, RefusalCode, SettingsDoc};
use serde_json::Value;

const USAGE_LINE: &str = "lod settings takes `show`, `set <key> <value>...`, `unset <key>`, `reset [--dry-run]` or \
     `undo`, each with [--instance <dir>] [--profile <name>] [--json]";

struct Flags {
    instance: Option<PathBuf>,
    profile: Option<String>,
    json: bool,
    dry_run: bool,
    words: Vec<String>,
}

fn parse(args: &[String], dry_run: bool) -> Result<Flags, String> {
    let mut f = Flags {
        instance: None,
        profile: None,
        json: false,
        dry_run: false,
        words: Vec::new(),
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
            "--dry-run" if dry_run => f.dry_run = true,
            other if other.starts_with("--") => return Err(format!("unknown option {other}")),
            word => f.words.push(word.to_owned()),
        }
    }
    Ok(f)
}

pub(crate) fn run(
    ctx: &raul_core::context::Context,
    args: &[String],
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let verb = args.first().map(String::as_str);
    let Some(verb @ ("show" | "set" | "unset" | "reset" | "undo")) = verb else {
        return crate::usage(err, USAGE_LINE);
    };
    let flags = match parse(&args[1..], verb == "reset") {
        Ok(f) => f,
        Err(why) => return crate::usage(err, &format!("{why}\n{USAGE_LINE}")),
    };
    let words_ok = match verb {
        "show" | "reset" | "undo" => flags.words.is_empty(),
        "unset" => flags.words.len() == 1,
        _ => !flags.words.is_empty(),
    };
    if !words_ok {
        return crate::usage(err, USAGE_LINE);
    }
    let (doc, code) = evaluate(ctx, verb, &flags);
    render(&flags, verb, &doc, code, out, err)
}

/// Checks the words of a `lod settings` command line: `Some` with the flags when they are one of the five forms.
fn understood(verb: &str, rest: &[String]) -> Option<Flags> {
    let flags = parse(rest, verb == "reset").ok()?;
    let words_ok = match verb {
        "show" | "reset" | "undo" => flags.words.is_empty(),
        "unset" => flags.words.len() == 1,
        "set" => !flags.words.is_empty(),
        _ => false,
    };
    words_ok.then_some(flags)
}

/// The document `raul lod settings <verb> --json` prints and the exit code it ends with, for the window's direct job;
/// `rest` is what follows the verb. `None` is a usage error.
#[must_use]
pub fn answer(
    ctx: &raul_core::context::Context,
    verb: &str,
    rest: &[String],
) -> Option<(Value, u8)> {
    let flags = understood(verb, rest)?;
    let (doc, code) = evaluate(ctx, verb, &flags);
    serde_json::to_value(&doc).ok().map(|json| (json, code))
}

/// Runs `verb` on the Profile's settings: the document and the exit code.
fn evaluate(ctx: &raul_core::context::Context, verb: &str, flags: &Flags) -> (SettingsDoc, u8) {
    let refused = |root: &std::path::Path, at: Option<&ProfileSettings>, refusal: Refusal| {
        let code = exit_code(&refusal);
        (settings::document(root, at, Err(refusal), Vec::new()), code)
    };
    let Some(root) = flags
        .instance
        .clone()
        .or_else(|| crate::lod::configured_instance(ctx))
    else {
        return refused(
            &PathBuf::new(),
            None,
            settings::instance_invalid(
                "no Instance: pass --instance <dir> or set storage.root with raul config"
                    .to_owned(),
            ),
        );
    };
    let instance = match NvmodInstance::open_read_only(&root) {
        Ok(i) => i,
        Err(e) => {
            return refused(
                &root,
                None,
                settings::instance_invalid(format!("{}: {e}", root.display())),
            );
        }
    };
    let at = match ProfileSettings::of(&instance, flags.profile.as_deref()) {
        Ok(a) => a,
        Err(refusal) => return refused(&root, None, refusal),
    };
    let outcome = match verb {
        "show" => settings::show(&at).map(|_| None),
        "set" => settings::parse_value(&flags.words[0], &flags.words[1..])
            .and_then(|v| settings::set(&at, &flags.words[0], &v))
            .map(Some),
        "unset" => settings::unset(&at, &flags.words[0]).map(Some),
        "reset" => settings::reset(&at, flags.dry_run).map(Some),
        _ => settings::undo(&at).map(Some),
    };
    let warnings = if matches!(verb, "show" | "set") && outcome.is_ok() {
        raul_drivers::lod::settings_warnings(&instance, &at.profile)
    } else {
        Vec::new()
    };
    let code = outcome.as_ref().err().map_or(crate::OK, exit_code);
    (
        settings::document(&root, Some(&at), outcome, warnings),
        code,
    )
}

fn exit_code(refusal: &Refusal) -> u8 {
    if refusal.code == RefusalCode::WriteFailed {
        crate::FAILED
    } else {
        crate::REFUSED
    }
}

fn render(
    flags: &Flags,
    verb: &str,
    doc: &SettingsDoc,
    code: u8,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    if flags.json {
        let text = format!(
            "{}\n",
            serde_json::to_string_pretty(doc).unwrap_or_default()
        );
        let written = crate::write_or_fail(out, err, &text);
        return if written == crate::OK { code } else { written };
    }
    if let Some(e) = &doc.error {
        let _ = writeln!(err, "raul lod settings {verb}: refused: {}", e.line());
        return code;
    }
    for w in &doc.warnings {
        let value = w
            .value
            .as_deref()
            .map_or_else(String::new, |v| format!(" `{v}`"));
        let reason = w
            .reason
            .as_deref()
            .map_or_else(String::new, |r| format!(" ({r})"));
        let _ = writeln!(
            err,
            "raul lod settings {verb}: warning: {}{value}: {}{reason}",
            w.key, w.what
        );
    }
    crate::write_or_fail(out, err, &text(verb, doc))
}

/// A value as a line shows it. Nothing set is `(none)` for a number that keeps everything when empty (the water
/// depth) and `(decided at run time)` for the settings the program decides itself (workers, memory budget).
fn shown(key: &str, v: &Value) -> String {
    match v {
        Value::Null if key == "terrain.drop_under_water" => "(none)".to_owned(),
        Value::Null => "(decided at run time)".to_owned(),
        Value::Array(items) if items.is_empty() => "[]".to_owned(),
        other => other.to_string(),
    }
}

fn text(verb: &str, doc: &SettingsDoc) -> String {
    use std::fmt::Write as _;
    let mut t = String::new();
    if let Some(c) = &doc.change {
        let keys = if c.keys.is_empty() {
            "nothing".to_owned()
        } else {
            c.keys.join(", ")
        };
        let line = match (verb, c.dry_run, c.changed) {
            ("reset", true, _) => format!("would remove {keys}; nothing was written"),
            ("reset", false, true) => format!(
                "removed {keys}; the file as it was is kept at {}",
                c.restore_point
                    .as_ref()
                    .map_or_else(String::new, |p| p.display().to_string())
            ),
            ("undo", _, _) => format!("put back {keys}"),
            (_, _, false) => "nothing to change".to_owned(),
            _ => format!("changed {keys}"),
        };
        let _ = writeln!(t, "raul lod settings {verb}: {line}");
    }
    if doc.run_in_progress {
        let _ = writeln!(
            t,
            "raul lod settings {verb}: {}",
            settings::catalogue::RUN_IN_PROGRESS
        );
    }
    if verb == "show" {
        let _ = writeln!(
            t,
            "raul lod settings: profile `{}` ({})",
            doc.profile,
            doc.file
                .as_ref()
                .map_or_else(String::new, |f| f.display().to_string())
        );
        for o in &doc.options {
            let mark = if o.overridden { "*" } else { " " };
            // The memory budget is an estimate checked before the build, never a limit while it runs.
            let note = if o.key == "memory_mib" {
                ", advisory"
            } else {
                ""
            };
            let _ = writeln!(
                t,
                "{mark} {} = {}  (default {}{note})",
                o.key,
                shown(&o.key, &o.value),
                shown(&o.key, &o.default)
            );
        }
        t.push_str("  * set in the file; Restore defaults (`raul lod settings reset`) removes every one but allow_in_repo\n");
        if let Some(p) = &doc.restore_point {
            let _ = writeln!(
                t,
                "  a restore point is kept at {}; `raul lod settings undo` puts it back",
                p.display()
            );
        }
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(line: &[&str]) -> Vec<String> {
        line.iter().map(|w| (*w).to_owned()).collect()
    }

    /// A synthetic Instance with a Profile.
    fn instance(label: &str) -> (raul_core::context::Context, PathBuf, PathBuf) {
        use nvmod_testkit::BaseBuilder;
        let dir = std::env::temp_dir().join(format!("raul-lod-set-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let base = dir.join("base");
        BaseBuilder::steam().write(&base);
        let root = dir.join("instance");
        let cancel = nvmod_api::CancelToken::new();
        nvmod_api::init::create(
            &nvmod_api::init::CreateRequest {
                instance: root.clone(),
                base,
            },
            &mut nvmod_api::NoProgress,
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
        let ctx = raul_core::context::Context::default()
            .with_config_folder(dir.join("config"))
            .without_run_log();
        (ctx, root, dir)
    }

    #[test]
    fn the_answer_shows_resets_refuses_and_reports_a_usage_error_as_none() {
        let (ctx, root, dir) = instance("answer");
        let i = root.to_string_lossy().into_owned();
        let (doc, code) = answer(&ctx, "show", &words(&["--instance", &i])).unwrap();
        assert_eq!(code, crate::OK, "{doc}");
        assert_eq!(doc["contract"], "raul-lod-settings");
        let (_, code) = answer(&ctx, "reset", &words(&["--instance", &i, "--dry-run"])).unwrap();
        assert_eq!(code, crate::OK);
        // An option that does not exist is a refusal document with the command's code.
        let (refused, code) =
            answer(&ctx, "unset", &words(&["no.such.option", "--instance", &i])).unwrap();
        assert_eq!(code, crate::REFUSED, "{refused}");
        assert!(refused["error"].is_object(), "{refused}");
        // No Instance and an Instance that cannot be opened are refusals too.
        let none = answer(&ctx, "show", &[]).unwrap();
        assert_eq!(none.1, crate::REFUSED);
        let missing = dir.join("nothing").to_string_lossy().into_owned();
        assert_eq!(
            answer(&ctx, "show", &words(&["--instance", &missing]))
                .unwrap()
                .1,
            crate::REFUSED
        );
        // Words that are not one of the five forms.
        assert!(answer(&ctx, "set", &[]).is_none());
        assert!(answer(&ctx, "show", &words(&["extra"])).is_none());
        assert!(answer(&ctx, "nonsense", &[]).is_none());
        assert!(answer(&ctx, "show", &words(&["--bogus"])).is_none());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
