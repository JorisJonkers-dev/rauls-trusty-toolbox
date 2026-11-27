// SPDX-License-Identifier: GPL-3.0-only
//! `raul modlist export --instance <dir> [--profile <name>] [--id <slug>] [--name <text>] [--target vanilla|ttw] -o <file>`
//! (ADR-0023): write a Profile as a Mod list. The Instance is only read; with no `-o` the list goes to standard output.

use std::io::Write;
use std::path::PathBuf;

use raul_core::context::Context;
use raul_modlist::{ExportOptions, Target, export_modlist};

struct Flags {
    instance: Option<PathBuf>,
    output: Option<PathBuf>,
    options: ExportOptions,
}

fn parse(args: &[String]) -> Result<Flags, String> {
    let mut f = Flags {
        instance: None,
        output: None,
        options: ExportOptions::default(),
    };
    let mut rest = args.iter();
    while let Some(a) = rest.next() {
        let mut value = || {
            rest.next()
                .cloned()
                .ok_or_else(|| format!("{a} needs a value"))
        };
        match a.as_str() {
            "--instance" => f.instance = Some(PathBuf::from(value()?)),
            "--profile" => f.options.profile = Some(value()?),
            "--id" => f.options.id = Some(value()?),
            "--name" => f.options.name = Some(value()?),
            "-o" | "--output" => f.output = Some(PathBuf::from(value()?)),
            "--target" => {
                f.options.target = match value()?.as_str() {
                    "vanilla" => Target::Vanilla,
                    "ttw" => Target::Ttw,
                    other => return Err(format!("--target is vanilla or ttw, not {other}")),
                };
            }
            other => return Err(format!("unexpected argument {other}")),
        }
    }
    Ok(f)
}

/// `raul modlist export ...`; `args` are what follows `export`.
pub(crate) fn run(ctx: &Context, args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let flags = match parse(args) {
        Ok(f) => f,
        Err(why) => return crate::usage(err, &format!("modlist export: {why}")),
    };
    let refuse = |err: &mut dyn Write, why: String| {
        let _ = writeln!(err, "raul modlist export: refused: {why}");
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
    let text = match export_modlist(&root, &flags.options)
        .map_err(|e| e.to_string())
        .and_then(|l| l.to_toml())
    {
        Ok(t) => t,
        Err(why) => return refuse(err, why),
    };
    match &flags.output {
        Some(path) => match std::fs::write(path, &text) {
            Ok(()) => {
                let _ = writeln!(out, "wrote {}", path.display());
                0
            }
            Err(e) => refuse(err, format!("could not write {}: {e}", path.display())),
        },
        None => crate::write_or_fail(out, err, &text),
    }
}
