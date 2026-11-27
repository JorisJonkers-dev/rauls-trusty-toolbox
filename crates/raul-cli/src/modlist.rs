// SPDX-License-Identifier: GPL-3.0-only
//! `raul modlist list|show|plan|fetch|watch` (ADR-0023): the Mod lists raul carries, what is in one, what an Instance
//! has and lacks of it, opening the download pages of what is missing, and importing the archives a person downloads
//! (`fetch` and `watch` are in `modlist_watch`). Only `watch` writes. Installing a whole list is #681.

use std::fmt::Write as _;
use std::io::Write;
use std::path::PathBuf;

use nvmod_instance::NvmodInstance;
use nvmod_modinstall::mods;
use raul_core::context::Context;
use raul_modlist::plan::{self, Acquire, NoResolver, Plan, State};
use raul_modlist::{BUNDLED_IDS, ModList, Source, bundled};
use serde_json::json;

const USAGE_LINE: &str = "modlist takes `list [--json]`, `show <list> [--json]` or \
     `plan <list> [--instance <dir>] [--profile <name>] [--json]`, \
     `fetch <list> [--open] [--all] [--limit <n>] [--instance <dir>] [--profile <name>]`, \
     `watch <list> [--downloads <dir>] [--install] [--once] [--scan-existing] [--instance <dir>] [--profile <name>]` \
     or `install <list>` (see `raul help`); <list> is a bundled id or a file";

#[derive(Default)]
#[allow(clippy::struct_excessive_bools)] // one switch per command-line flag
pub(crate) struct Flags {
    pub(crate) list: Option<String>,
    pub(crate) instance: Option<PathBuf>,
    pub(crate) profile: Option<String>,
    pub(crate) json: bool,
    /// `fetch`: open the pages instead of printing them only.
    pub(crate) open: bool,
    /// `fetch`: every Mod, not only the ones the Instance lacks.
    pub(crate) all: bool,
    /// `fetch --open`: the most pages to open in one go.
    pub(crate) limit: Option<usize>,
    /// `watch`: the folder to watch instead of the usual download folders.
    pub(crate) downloads: Option<PathBuf>,
    /// `watch`: install into the Profile after staging.
    pub(crate) install: bool,
    /// `watch`: one scan of what is there, then stop.
    pub(crate) once: bool,
    /// `watch`: also take archives that were in the folder before it started.
    pub(crate) scan_existing: bool,
}

fn parse(args: &[String]) -> Result<Flags, String> {
    let mut f = Flags::default();
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
            "--open" => f.open = true,
            "--all" => f.all = true,
            "--limit" => {
                f.limit = Some(
                    value()?
                        .parse()
                        .map_err(|_| "--limit needs a number".to_owned())?,
                );
            }
            "--downloads" => f.downloads = Some(PathBuf::from(value()?)),
            "--install" => f.install = true,
            "--once" => f.once = true,
            "--scan-existing" => f.scan_existing = true,
            other if other.starts_with("--") => return Err(format!("unknown option {other}")),
            other if f.list.is_none() => f.list = Some(other.to_owned()),
            other => return Err(format!("unexpected argument {other}")),
        }
    }
    Ok(f)
}

/// `raul modlist ...`.
pub(crate) fn run(ctx: &Context, args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let verb = args.first().map_or("", String::as_str);
    if verb == "install" {
        return crate::modlist_install::run(ctx, &args[1..], out, err);
    }
    if verb == "export" {
        return crate::modlist_export::run(ctx, &args[1..], out, err);
    }
    let flags = match verb {
        "list" | "show" | "plan" | "fetch" | "watch" => parse(&args[1..]),
        _ => Err(
            "modlist needs `list`, `show`, `plan`, `fetch`, `watch`, `export` or `install`"
                .to_owned(),
        ),
    };
    let flags = match flags {
        Ok(f) => f,
        Err(why) => return crate::usage(err, &format!("{why}\n{USAGE_LINE}")),
    };
    let refuse = |err: &mut dyn Write, why: String| {
        let _ = writeln!(err, "raul modlist {verb}: refused: {why}");
        crate::REFUSED
    };
    if verb == "list" {
        return list(flags.json, out, err);
    }
    let Some(name) = flags.list.as_deref() else {
        return crate::usage(err, &format!("modlist {verb} needs a list\n{USAGE_LINE}"));
    };
    let list = match load(name) {
        Ok(l) => l,
        Err(why) => return refuse(err, why),
    };
    if verb == "show" {
        let text = if flags.json {
            format!(
                "{}\n",
                serde_json::to_string_pretty(&list).unwrap_or_default()
            )
        } else {
            show_text(&list)
        };
        return crate::write_or_fail(out, err, &text);
    }
    if verb == "fetch" {
        return match crate::modlist_watch::fetch(ctx, &list, &flags, out) {
            Ok(code) => code,
            Err(why) => refuse(err, why),
        };
    }
    if verb == "watch" {
        return match crate::modlist_watch::watch(ctx, &list, &flags, out) {
            Ok(code) => code,
            Err(why) => refuse(err, why),
        };
    }
    let plan = match present_in_instance(ctx, &flags) {
        Ok(present) => plan::plan(&list, &present, &NoResolver),
        Err(why) => return refuse(err, why),
    };
    let text = if flags.json {
        let doc = json!({ "list": plan.list, "name": plan.name, "counts": plan.counts(), "items": plan.items });
        format!(
            "{}\n",
            serde_json::to_string_pretty(&doc).unwrap_or_default()
        )
    } else {
        plan_text(&plan)
    };
    crate::write_or_fail(out, err, &text)
}

fn list(json: bool, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let rows: Vec<(String, String, usize)> = BUNDLED_IDS
        .iter()
        .filter_map(|id| bundled(id)?.ok())
        .map(|l| (l.id, l.name, l.mods.len()))
        .collect();
    let text = if json {
        let lists: Vec<_> = rows
            .iter()
            .map(|(id, name, mods)| json!({ "id": id, "name": name, "mods": mods }))
            .collect();
        format!(
            "{}\n",
            serde_json::to_string_pretty(&json!({ "lists": lists })).unwrap_or_default()
        )
    } else {
        rows.iter().fold(String::new(), |mut s, (id, name, mods)| {
            let _ = writeln!(s, "{id}: {name}, {mods} Mods");
            s
        })
    };
    crate::write_or_fail(out, err, &text)
}

/// A bundled list by id, else a file by path.
pub(crate) fn load(name: &str) -> Result<ModList, String> {
    if let Some(found) = bundled(name) {
        return found.map_err(|e| e.to_string());
    }
    let text = std::fs::read_to_string(name).map_err(|e| {
        format!(
            "`{name}` is neither a list raul carries ({}) nor a readable file: {e}",
            BUNDLED_IDS.join(", ")
        )
    })?;
    ModList::parse(&text).map_err(|e| format!("{name}: {e}"))
}

/// The Instance folder: `--instance`, else the configured one.
pub(crate) fn instance_root(ctx: &Context, flags: &Flags) -> Result<PathBuf, String> {
    flags
        .instance
        .clone()
        .or_else(|| crate::lod::configured_instance(ctx))
        .ok_or_else(|| {
            "no Instance: pass --instance <dir> or run `raul config set storage.root <dir>`"
                .to_owned()
        })
}

/// The names of the Mods the Profile lists and whose folders exist.
pub(crate) fn present_in_instance(ctx: &Context, flags: &Flags) -> Result<Vec<String>, String> {
    let root = instance_root(ctx, flags)?;
    let instance =
        NvmodInstance::open_read_only(&root).map_err(|e| format!("{}: {e}", root.display()))?;
    let profile = instance
        .resolve_profile(flags.profile.as_deref())
        .map_err(|e| e.to_string())?;
    Ok(mods::list(&instance, &profile)
        .into_iter()
        .filter(|row| row.kind == "mod" && row.installed)
        .map(|row| row.name)
        .collect())
}

fn source_words(source: &Source) -> &'static str {
    match source {
        Source::Nexus { .. } => "Nexus",
        Source::Url { .. } => "link",
        Source::Manual { .. } => "by hand",
        Source::Tool { .. } => "built from your own files",
        Source::Generated { .. } => "generated",
        Source::Addon { .. } => "raul add-on",
    }
}

fn show_text(list: &ModList) -> String {
    let mut s = format!(
        "{}: {} Mods ({} off)\n",
        list.name,
        list.mods.len(),
        list.mods.iter().filter(|m| !m.enabled).count()
    );
    if let Some(d) = &list.description {
        let _ = writeln!(s, "{d}");
    }
    let _ = writeln!(s, "needs: {}\n", list.requires.join(", "));
    for (i, m) in list.mods.iter().enumerate() {
        let off = if m.enabled { "" } else { "  (off)" };
        let _ = writeln!(
            s,
            "{:>4}. {}  [{}]{off}",
            i + 1,
            m.name,
            source_words(&m.source)
        );
    }
    s
}

fn plan_text(plan: &Plan) -> String {
    let c = plan.counts();
    let mut s = format!(
        "{}: {} present, {} to get, {} switched off by the list\n",
        plan.name, c.present, c.missing, c.skipped
    );
    if c.missing == 0 {
        s.push_str("Nothing to get.\n");
        return s;
    }
    s.push_str("\nTo get, in the order they go in:\n");
    for item in plan.items.iter().filter(|i| i.state == State::Missing) {
        let _ = writeln!(s, "{:>4}. {}", item.position, item.name);
        if let Some(a) = &item.acquire {
            let _ = writeln!(s, "      {}", acquire_text(a));
        }
    }
    s
}

fn acquire_text(a: &Acquire) -> String {
    let with =
        |lead: &str, what: Option<&str>| what.map_or_else(String::new, |w| format!("{lead}{w}"));
    match a {
        Acquire::Nexus {
            page, file_name, ..
        } => format!("Nexus: {page}{}", with(" -> ", file_name.as_deref())),
        Acquire::Link {
            url,
            file_name,
            note,
        } => {
            let note = note
                .as_deref()
                .map_or_else(String::new, |n| format!(" ({n})"));
            format!(
                "download: {url}{}{note}",
                with(" -> ", file_name.as_deref())
            )
        }
        Acquire::Manual { file_name, note } => {
            format!(
                "by hand: {}{}",
                note.as_deref()
                    .unwrap_or("no download is recorded for this one"),
                with(" -> ", file_name.as_deref())
            )
        }
        Acquire::Tool { tool } => format!("built by {tool} from your own game files"),
        Acquire::Generated { tool } => format!("generated by {tool} once the other Mods are in"),
        Acquire::Addon { id } => format!("raul makes it (`raul addons apply`, add-on {id})"),
    }
}
