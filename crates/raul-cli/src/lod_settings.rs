// SPDX-License-Identifier: GPL-3.0-only
//! `raul lod settings show|set|unset|reset|undo`: a Profile's LOD settings, option by option, through the same
//! functions the GUI's LOD settings view calls (`raul_drivers::lod::settings`). `--json` prints one
//! `raul-lod-settings` document.

use std::io::Write;
use std::path::PathBuf;

use nvmod_instance::NvmodInstance;
use raul_drivers::lod::settings::{
    self, Change, ProfileSettings, Refusal, RefusalCode, SettingsDoc, Warning,
};
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

pub(crate) fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
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
    let Some(root) = flags
        .instance
        .clone()
        .or_else(crate::lod::configured_instance)
    else {
        let refusal = settings::instance_invalid(
            "no Instance: pass --instance <dir> or set storage.root with raul config".to_owned(),
        );
        return finish(
            &flags,
            verb,
            &PathBuf::new(),
            None,
            Err(refusal),
            Vec::new(),
            out,
            err,
        );
    };
    let instance = match NvmodInstance::open_read_only(&root) {
        Ok(i) => i,
        Err(e) => {
            let refusal = settings::instance_invalid(format!("{}: {e}", root.display()));
            return finish(
                &flags,
                verb,
                &root,
                None,
                Err(refusal),
                Vec::new(),
                out,
                err,
            );
        }
    };
    let at = match ProfileSettings::of(&instance, flags.profile.as_deref()) {
        Ok(a) => a,
        Err(refusal) => {
            return finish(
                &flags,
                verb,
                &root,
                None,
                Err(refusal),
                Vec::new(),
                out,
                err,
            );
        }
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
    finish(&flags, verb, &root, Some(&at), outcome, warnings, out, err)
}

fn exit_code(refusal: &Refusal) -> u8 {
    if refusal.code == RefusalCode::WriteFailed {
        crate::FAILED
    } else {
        crate::REFUSED
    }
}

#[allow(clippy::too_many_arguments)] // one call per way out of `run`
fn finish(
    flags: &Flags,
    verb: &str,
    root: &std::path::Path,
    at: Option<&ProfileSettings>,
    outcome: Result<Option<Change>, Refusal>,
    warnings: Vec<Warning>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let code = outcome.as_ref().err().map_or(crate::OK, exit_code);
    let doc = settings::document(root, at, outcome, warnings);
    if flags.json {
        let text = format!(
            "{}\n",
            serde_json::to_string_pretty(&doc).unwrap_or_default()
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
    crate::write_or_fail(out, err, &text(verb, &doc))
}

fn shown(v: &Value) -> String {
    match v {
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
                shown(&o.value),
                shown(&o.default)
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
