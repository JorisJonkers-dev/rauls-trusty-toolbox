// SPDX-License-Identifier: GPL-3.0-only
//! `raul lod plan`: the `lodwright-request` document `raul lod generate` would send to lodwright for a Profile,
//! checked against lodwright's published schema. Nothing is run or written.

use std::io::Write;
use std::path::PathBuf;

use nvmod_instance::NvmodInstance;
use raul_core::config::{Config, Key};
use raul_drivers::lod;

const USAGE_LINE: &str =
    "lod takes `plan [--instance <dir>] [--profile <name>] [--world <name>]... [--json]`";

struct Flags {
    instance: Option<PathBuf>,
    profile: Option<String>,
    worlds: Vec<String>,
    json: bool,
}

fn parse(args: &[String]) -> Result<Flags, String> {
    let mut f = Flags {
        instance: None,
        profile: None,
        worlds: Vec::new(),
        json: false,
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
            other => return Err(format!("unknown option {other}")),
        }
    }
    Ok(f)
}

pub(crate) fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    match args.first().map(String::as_str) {
        Some("plan") => plan(&args[1..], out, err),
        Some("generate") => {
            let _ = writeln!(
                err,
                "raul lod generate: refused: it comes with lodwright-rs 0.3.0, which is not in the Known-good set yet; `raul lod plan` shows the request it will send"
            );
            crate::REFUSED
        }
        _ => crate::usage(err, USAGE_LINE),
    }
}

fn refused(err: &mut dyn Write, why: &str) -> u8 {
    let _ = writeln!(err, "raul lod plan: refused: {why}");
    crate::REFUSED
}

fn plan(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let flags = match parse(args) {
        Ok(f) => f,
        Err(why) => return crate::usage(err, &format!("{why}\n{USAGE_LINE}")),
    };
    let from_config = || {
        raul_core::config::default_file()
            .and_then(|f| Config::load(&f).ok())
            .and_then(|c| c.get(Key::Instance))
    };
    let Some(root) = flags.instance.or_else(from_config) else {
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
    let plan = match lod::plan(&instance, &profile, &flags.worlds) {
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
        format!(
            "raul lod plan: profile `{profile}`, {} plugins, {worlds}\n  data:   {}\n  output: {}\n",
            plan.plugins.len(),
            plan.data_folder.display(),
            plan.output_dir.display()
        )
    };
    crate::write_or_fail(out, err, &text)
}
