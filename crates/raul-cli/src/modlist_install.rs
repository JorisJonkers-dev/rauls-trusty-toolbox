// SPDX-License-Identifier: GPL-3.0-only
//! `raul modlist install <list>` (ADR-0023, #681): install the archives of a Mod list that are in the downloads folder
//! into a Profile, in list order, then sort the plugins with loot-rs as the list says. The steps are
//! `raul_modlist::install` (pure); this file reads the Instance, calls nvmod's own jobs in process (so no program is
//! started) and keeps `.raul/modlist-state.json`.
//!
//! It is safe to run again: a Mod the Profile already has is left alone. A Mod with no archive, a FOMOD that needs
//! answers (#682) and Mods built elsewhere are reported and do not stop the run.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Write;
use std::path::{Path, PathBuf};

use nvmod_api::downloads::{InstallRequest, downloads_dir};
use nvmod_api::install::{InstallOutput, LimitOverrides, Placement};
use nvmod_api::loadorder::{SortMode, SortRequest};
use nvmod_api::mods::{Change, ChangeRequest};
use nvmod_api::{CancelToken, NoProgress, Outcome};
use nvmod_instance::NvmodInstance;
use nvmod_modinstall::mods;
use raul_core::context::Context;
use raul_core::pinned::{MasterlistPin, masterlist};
use raul_modlist::install::{InstallPlan, Place, Step, plan_install};
use raul_modlist::{ModList, PluginOrder};
use serde_json::{Value, json};

const USAGE_LINE: &str = "modlist install takes `<list> [--instance <dir>] [--profile <name>] [--downloads <dir>] \
     [--dry-run] [--no-sort] [--masterlist <file>] [--json]`";
const STATE_FILE: &str = ".raul/modlist-state.json";

struct Flags {
    list: Option<String>,
    instance: Option<PathBuf>,
    profile: Option<String>,
    downloads: Option<PathBuf>,
    masterlist: Option<PathBuf>,
    dry_run: bool,
    no_sort: bool,
    json: bool,
}

fn parse(args: &[String]) -> Result<Flags, String> {
    let mut f = Flags {
        list: None,
        instance: None,
        profile: None,
        downloads: None,
        masterlist: None,
        dry_run: false,
        no_sort: false,
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
            "--dry-run" => f.dry_run = true,
            "--no-sort" => f.no_sort = true,
            "--instance" => f.instance = Some(PathBuf::from(value()?)),
            "--profile" => f.profile = Some(value()?),
            "--downloads" => f.downloads = Some(PathBuf::from(value()?)),
            "--masterlist" => f.masterlist = Some(PathBuf::from(value()?)),
            other if other.starts_with("--") => return Err(format!("unknown option {other}")),
            other if f.list.is_none() => f.list = Some(other.to_owned()),
            other => return Err(format!("unexpected argument {other}")),
        }
    }
    Ok(f)
}

/// What happened to one step.
#[derive(Debug)]
enum Result_ {
    Installed,
    Planned,
    NeedsAnswers,
    Failed(String),
}

fn place_text(place: &Place) -> String {
    match place {
        Place::Top => "on top".to_owned(),
        Place::After(n) => format!("under {n}"),
    }
}

fn placement(place: &Place) -> Placement {
    match place {
        Place::Top => Placement::Top,
        Place::After(n) => Placement::After(n.clone()),
    }
}

/// `raul modlist install ...`; `args` are the words after `install`.
pub(crate) fn run(ctx: &Context, args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let flags = match parse(args) {
        Ok(f) => f,
        Err(why) => return crate::usage(err, &format!("{why}\n{USAGE_LINE}")),
    };
    let Some(name) = flags.list.as_deref() else {
        return crate::usage(err, &format!("modlist install needs a list\n{USAGE_LINE}"));
    };
    let refuse = |err: &mut dyn Write, why: String| {
        let _ = writeln!(err, "raul modlist install: refused: {why}");
        crate::REFUSED
    };
    let list = match crate::modlist::load(name) {
        Ok(l) => l,
        Err(why) => return refuse(err, why),
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
    let (profile, present) = match present_mods(&root, flags.profile.as_deref()) {
        Ok(x) => x,
        Err(why) => return refuse(err, why),
    };
    let (dir, _) = downloads_dir(&root, flags.downloads.as_deref());
    let plan = plan_install(&list, &present, &archives_in(&dir));

    let mut results: Vec<(&Step, Result_)> = Vec::new();
    for step in &plan.steps {
        let r = if flags.dry_run {
            Result_::Planned
        } else {
            install_step(&root, &profile, flags.downloads.as_deref(), step)
        };
        results.push((step, r));
    }
    let installed = results
        .iter()
        .filter(|(_, r)| matches!(r, Result_::Installed))
        .count();
    let failed = results
        .iter()
        .filter(|(_, r)| matches!(r, Result_::Failed(_)))
        .count();
    let mut sort_note: Option<String> = None;
    let mut sort_failed = false;
    if installed > 0 && !flags.no_sort && list.plugins == PluginOrder::Loot {
        match sort_plugins(&root, &profile, flags.masterlist.as_deref()) {
            Ok(()) => sort_note = Some("sorted with loot-rs".to_owned()),
            Err(why) => {
                sort_failed = true;
                sort_note = Some(format!("the sort failed, the Mods stay installed: {why}"));
            }
        }
    }
    if installed > 0 {
        if let Err(why) = write_state(&root, &list, &profile, &results) {
            let _ = writeln!(err, "raul modlist install: could not write {STATE_FILE}: {why}");
        }
    }
    let text = if flags.json {
        let doc = document(&plan, &profile, &results, flags.dry_run, sort_note.as_deref());
        format!("{}\n", serde_json::to_string_pretty(&doc).unwrap_or_default())
    } else {
        text_of(&list, &plan, &results, flags.dry_run, sort_note.as_deref())
    };
    let code = crate::write_or_fail(out, err, &text);
    if code != crate::OK {
        code
    } else if failed > 0 || sort_failed {
        crate::FAILED
    } else {
        crate::OK
    }
}

/// The Profile and the names of the Mods it lists whose folders exist.
fn present_mods(root: &Path, profile: Option<&str>) -> Result<(String, Vec<String>), String> {
    let instance =
        NvmodInstance::open_read_only(root).map_err(|e| format!("{}: {e}", root.display()))?;
    let profile = instance.resolve_profile(profile).map_err(|e| e.to_string())?;
    let present = mods::list(&instance, &profile)
        .into_iter()
        .filter(|row| row.kind == "mod" && row.installed)
        .map(|row| row.name)
        .collect();
    Ok((profile, present))
}

/// The files of the downloads folder that can be archives (not the `.meta` files), by name.
fn archives_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| !n.to_ascii_lowercase().ends_with(".meta"))
        .collect();
    names.sort();
    names
}

fn install_step(root: &Path, profile: &str, downloads: Option<&Path>, step: &Step) -> Result_ {
    let request = InstallRequest {
        instance: root.to_path_buf(),
        downloads: downloads.map(Path::to_path_buf),
        archive: step.archive.clone(),
        profile: Some(profile.to_owned()),
        name: Some(step.name.clone()),
        dry_run: false,
        answers: None,
        placement: placement(&step.place),
        max_extract_bytes: None,
        extract_limits: LimitOverrides::default(),
    };
    match nvmod_api::downloads::install(&request, &mut NoProgress, &CancelToken::new()) {
        Ok(Outcome::Done(done)) => match done.install {
            InstallOutput::Questions(_) => return Result_::NeedsAnswers,
            InstallOutput::Installed(_) => {}
        },
        Ok(Outcome::Cancelled) => return Result_::Failed("cancelled".to_owned()),
        Err(e) => return Result_::Failed(e.summary),
    }
    if !step.enabled {
        let change = ChangeRequest {
            instance: root.to_path_buf(),
            profile: Some(profile.to_owned()),
            change: Change::Disable {
                mods: vec![step.name.clone()],
            },
            dry_run: false,
        };
        if let Err(e) = nvmod_api::mods::change(&change, &mut NoProgress, &CancelToken::new()) {
            return Result_::Failed(format!("installed, but switching it off failed: {}", e.summary));
        }
    }
    Result_::Installed
}

fn sort_plugins(root: &Path, profile: &str, given: Option<&Path>) -> Result<(), String> {
    let file = match given {
        Some(f) => f.to_path_buf(),
        None => masterlist(root, &MasterlistPin::default()).map_err(|e| format!("{e:?}"))?,
    };
    let request = SortRequest {
        instance: root.to_path_buf(),
        profile: Some(profile.to_owned()),
        masterlist: file,
        apply: true,
        mode: SortMode::default(),
    };
    match nvmod_api::loadorder::sort(&request, &mut NoProgress, &CancelToken::new()) {
        Ok(Outcome::Done(_)) => Ok(()),
        Ok(Outcome::Cancelled) => Err("cancelled".to_owned()),
        Err(e) => Err(e.summary),
    }
}

/// Adds what this run installed to `.raul/modlist-state.json`, which keeps for each list and Profile the Mods the
/// list put there. It is a record for the person and for later runs; the Profile itself says what is installed.
fn write_state(
    root: &Path,
    list: &ModList,
    profile: &str,
    results: &[(&Step, Result_)],
) -> Result<(), String> {
    let path = root.join(STATE_FILE);
    let mut doc: Value = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_else(|| json!({}));
    let mut known: BTreeMap<String, Value> = doc
        .get("lists")
        .and_then(Value::as_object)
        .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();
    let key = format!("{}@{profile}", list.id);
    let entry = known
        .entry(key)
        .or_insert_with(|| json!({ "list": list.id, "profile": profile, "installed": {} }));
    if let Some(map) = entry.get_mut("installed").and_then(Value::as_object_mut) {
        for (step, r) in results {
            if matches!(r, Result_::Installed) {
                map.insert(
                    step.id.clone(),
                    json!({ "name": step.name, "archive": step.archive, "enabled": step.enabled }),
                );
            }
        }
    }
    doc = json!({ "schema": "raul-modlist-state/1", "lists": known });
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(
        &tmp,
        format!("{}\n", serde_json::to_string_pretty(&doc).unwrap_or_default()),
    )
    .map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

fn status(r: &Result_) -> (&'static str, Option<&str>) {
    match r {
        Result_::Installed => ("installed", None),
        Result_::Planned => ("would-install", None),
        Result_::NeedsAnswers => ("needs-answers", None),
        Result_::Failed(why) => ("failed", Some(why.as_str())),
    }
}

fn document(
    plan: &InstallPlan,
    profile: &str,
    results: &[(&Step, Result_)],
    dry_run: bool,
    sort: Option<&str>,
) -> Value {
    let steps: Vec<Value> = results
        .iter()
        .map(|(s, r)| {
            let (state, why) = status(r);
            json!({ "position": s.position, "id": s.id, "name": s.name, "archive": s.archive,
                "place": s.place, "enabled": s.enabled, "result": state, "why": why })
        })
        .collect();
    json!({ "list": plan.list, "profile": profile, "dry_run": dry_run, "present": plan.present,
        "steps": steps, "missing": plan.missing, "elsewhere": plan.elsewhere, "sort": sort })
}

fn text_of(
    list: &ModList,
    plan: &InstallPlan,
    results: &[(&Step, Result_)],
    dry_run: bool,
    sort: Option<&str>,
) -> String {
    let mut s = format!(
        "{}: {} present, {} to install, {} without an archive{}\n",
        list.name,
        plan.present,
        plan.steps.len(),
        plan.missing.len(),
        if dry_run { " (dry run, nothing written)" } else { "" }
    );
    for (step, r) in results {
        let (state, why) = status(r);
        let off = if step.enabled { "" } else { ", switched off" };
        let _ = write!(
            s,
            "{:>4}. {state}: {} <- {} ({}{off})",
            step.position,
            step.name,
            step.archive,
            place_text(&step.place)
        );
        if let Some(why) = why {
            let _ = write!(s, ": {why}");
        }
        s.push('\n');
    }
    if !plan.missing.is_empty() {
        s.push_str("\nNo archive for these (put them in the downloads folder and run again):\n");
        for m in &plan.missing {
            let file = m.archive.as_deref().unwrap_or("no file name recorded");
            let _ = writeln!(s, "{:>4}. {} ({file})", m.position, m.name);
        }
    }
    if !plan.elsewhere.is_empty() {
        let names: Vec<&str> = plan.elsewhere.iter().map(|e| e.name.as_str()).collect();
        let _ = writeln!(s, "\nBuilt by other steps, not installed here: {}", names.join(", "));
    }
    if results.iter().any(|(_, r)| matches!(r, Result_::NeedsAnswers)) {
        s.push_str("\nSome archives hold a FOMOD with questions; install those with `raul install --interactive`.\n");
    }
    if let Some(note) = sort {
        let _ = writeln!(s, "\nPlugins: {note}");
    } else if dry_run && list.plugins == PluginOrder::Loot && !plan.steps.is_empty() {
        s.push_str("\nPlugins: would be sorted with loot-rs\n");
    }
    s
}
