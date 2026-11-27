// SPDX-License-Identifier: GPL-3.0-only
//! Writing a Mod list from an Instance (ADR-0023): the Profile's `modlist.txt` gives the Mods, their order and whether they are on,
//! `plugins.txt` and the Mod folders give the Plugins, and each Mod's `meta.ini` gives the Nexus mod id, file id, version and the
//! archive it came from. The archive name carries the upload time Nexus put in it, which names the file when the id is not known.
//!
//! The result is a [`ModList`] that passes [`ModList::problems`]; [`ModList::to_toml`] writes it and [`ModList::parse`] reads it back.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{Game, Group, ModEntry, ModList, PluginOrder, SCHEMA, Source, Target};

/// What to export and how to name the list.
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// The Profile; `None` takes the only one the Instance has.
    pub profile: Option<String>,
    /// The list id (a slug); `None` derives it from the Profile name.
    pub id: Option<String>,
    /// The list name; `None` uses the Profile name.
    pub name: Option<String>,
    /// The base game the list builds on.
    pub target: Target,
}

impl Default for ExportOptions {
    fn default() -> Self {
        ExportOptions {
            profile: None,
            id: None,
            name: None,
            target: Target::Vanilla,
        }
    }
}

/// Why an export did not happen. `Display` is a plain sentence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportError {
    /// A file or folder of the Instance could not be read.
    Io(String),
    /// The Instance has no such Profile, or several and none was named.
    Profile(String),
    /// The list built from the Instance breaks a rule of the format (for example an id that is not a slug).
    Invalid(Vec<String>),
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportError::Io(why) | ExportError::Profile(why) => write!(f, "{why}"),
            ExportError::Invalid(p) => write!(
                f,
                "the exported list breaks {} rule(s): {}",
                p.len(),
                p.join("; ")
            ),
        }
    }
}

impl std::error::Error for ExportError {}

impl ModList {
    /// Write the list as TOML text that [`ModList::parse`] reads back.
    ///
    /// # Errors
    /// The message of the serializer, which only fails for a value TOML cannot hold.
    pub fn to_toml(&self) -> Result<String, String> {
        let body = toml::to_string(self).map_err(|e| e.to_string())?;
        Ok(format!(
            "# SPDX-License-Identifier: CC0-1.0\n# A Mod list (schema {SCHEMA}, docs/adr/0023): references only, no mod file is in it.\n{body}"
        ))
    }
}

/// Export the Profile of the Instance at `instance` as a Mod list.
///
/// Mods keep the Profile's order, lowest priority first (`modlist.txt` has the highest first). A Mod with a Nexus id becomes a `nexus`
/// source; one without becomes `manual` with a note. Separators become groups.
///
/// # Errors
/// [`ExportError`] when the Instance or Profile cannot be read, or the result would break a rule of the format.
pub fn export_modlist(instance: &Path, options: &ExportOptions) -> Result<ModList, ExportError> {
    let profile = pick_profile(instance, options.profile.as_deref())?;
    let profile_dir = ci_child(&instance.join("profiles"), &profile)
        .ok_or_else(|| ExportError::Profile(format!("the Profile `{profile}` has no folder")))?;
    let order = read_text(&profile_dir.join("modlist.txt"))?;
    let plugin_text = fs::read(profile_dir.join("plugins.txt"))
        .ok()
        .map(|b| String::from_utf8_lossy(&b).into_owned());
    let active = plugin_text.as_deref().map(profile_plugins);
    let mods_dir = instance.join("mods");
    let folders = folder_names(&mods_dir);

    let mut groups: Vec<Group> = Vec::new();
    let mut group_ids = BTreeSet::new();
    let mut used = BTreeSet::new();
    let mut mods = Vec::new();
    let mut current: Option<String> = None;
    for (name, on) in modlist_entries(&order).into_iter().rev() {
        if let Some(title) = name.strip_suffix("_separator") {
            let id = unique(&slug(title, "group"), &mut group_ids);
            groups.push(Group {
                id: id.clone(),
                title: title.to_string(),
            });
            current = Some(id);
            continue;
        }
        let folder = folders.get(&name.to_lowercase()).map(|f| mods_dir.join(f));
        let meta = folder
            .as_ref()
            .map_or_else(BTreeMap::new, |f| read_meta(&f.join("meta.ini")));
        let plugins = folder
            .as_deref()
            .map_or_else(Vec::new, |f| mod_plugins(f, active.as_ref()));
        let (source, version) = source_of(&meta, folder.is_some());
        mods.push(ModEntry {
            id: unique(&slug(&name, "mod"), &mut used),
            name,
            group: current.clone(),
            enabled: on,
            why_off: (!on).then(|| format!("switched off in the Profile `{profile}`")),
            version,
            plugins,
            hide: Vec::new(),
            source,
            note: None,
        });
    }

    let list = ModList {
        schema: SCHEMA.to_string(),
        id: options
            .id
            .clone()
            .unwrap_or_else(|| slug(&profile, "profile")),
        name: options.name.clone().unwrap_or_else(|| profile.clone()),
        description: Some(format!(
            "Exported from the Profile `{profile}`: {} Mods.",
            mods.len()
        )),
        game: Game::Fnv,
        target: options.target,
        plugins: PluginOrder::Loot,
        requires: match options.target {
            Target::Vanilla => vec!["fnv".to_string()],
            Target::Ttw => vec!["fnv".to_string(), "fo3".to_string()],
        },
        groups,
        mods,
    };
    let problems = list.problems();
    if problems.is_empty() {
        Ok(list)
    } else {
        Err(ExportError::Invalid(problems))
    }
}

fn read_text(path: &Path) -> Result<String, ExportError> {
    fs::read(path)
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .map_err(|e| ExportError::Io(format!("could not read {}: {e}", path.display())))
}

/// The Profile to export: the one named, else the Instance's only one.
fn pick_profile(instance: &Path, wanted: Option<&str>) -> Result<String, ExportError> {
    let dir = instance.join("profiles");
    let mut found: Vec<String> = fs::read_dir(&dir)
        .map_err(|e| ExportError::Io(format!("could not read {}: {e}", dir.display())))?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    found.sort();
    match wanted {
        Some(w) => found
            .iter()
            .find(|f| f.eq_ignore_ascii_case(w))
            .cloned()
            .ok_or_else(|| {
                ExportError::Profile(format!(
                    "the Instance has no Profile `{w}`; it has: {}",
                    found.join(", ")
                ))
            }),
        None => match found.len() {
            1 => Ok(found.remove(0)),
            0 => Err(ExportError::Profile("the Instance has no Profile".into())),
            _ => Err(ExportError::Profile(format!(
                "the Instance has several Profiles ({}); name one with --profile",
                found.join(", ")
            ))),
        },
    }
}

fn ci_child(dir: &Path, name: &str) -> Option<PathBuf> {
    let exact = dir.join(name);
    if exact.exists() {
        return Some(exact);
    }
    fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .find(|e| e.file_name().to_string_lossy().eq_ignore_ascii_case(name))
        .map(|e| e.path())
}

/// Lower-cased folder name to real folder name.
fn folder_names(dir: &Path) -> BTreeMap<String, String> {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .map(|e| {
            let n = e.file_name().to_string_lossy().into_owned();
            (n.to_lowercase(), n)
        })
        .collect()
}

/// `modlist.txt` as (Mod name, enabled), highest priority first as the file has it. `*` (game-owned) and `#` lines are skipped.
fn modlist_entries(text: &str) -> Vec<(String, bool)> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim_end_matches('\r');
            let on = match line.chars().next()? {
                '+' => true,
                '-' => false,
                _ => return None,
            };
            let name = line[1..].trim();
            (!name.is_empty()).then(|| (name.to_string(), on))
        })
        .collect()
}

/// The Plugins `plugins.txt` lists, lower-cased, with the names as written.
fn profile_plugins(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .map(|l| l.trim_end_matches('\r').trim_start_matches('*').trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| (l.to_lowercase(), l.to_string()))
        .collect()
}

/// The Plugins in the top of a Mod folder that the Profile knows (all of them when there is no `plugins.txt`).
fn mod_plugins(folder: &Path, active: Option<&BTreeMap<String, String>>) -> Vec<String> {
    let mut out: Vec<String> = fs::read_dir(folder)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .filter_map(|e| {
            let n = e.file_name().to_string_lossy().into_owned();
            let l = n.to_lowercase();
            [".esp", ".esm", ".esl"]
                .iter()
                .any(|x| l.ends_with(x))
                .then_some((l, n))
        })
        .filter_map(|(l, n)| match active {
            Some(a) => a.get(&l).cloned(),
            None => Some(n),
        })
        .collect();
    out.sort_by_key(|n| n.to_lowercase());
    out
}

/// The keys of `meta.ini` this export reads, from `[General]`, keys lower-cased.
fn read_meta(path: &Path) -> BTreeMap<String, String> {
    let Ok(bytes) = fs::read(path) else {
        return BTreeMap::new();
    };
    let text = String::from_utf8_lossy(&bytes);
    let mut section = String::new();
    let mut out = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if let Some(s) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = s.to_lowercase();
        } else if section == "general"
            && let Some((k, v)) = line.split_once('=')
        {
            out.insert(
                k.trim().to_lowercase(),
                v.trim().trim_matches('"').to_string(),
            );
        }
    }
    out
}

/// The source and the version a person reads for one Mod.
fn source_of(meta: &BTreeMap<String, String>, has_folder: bool) -> (Source, Option<String>) {
    let archive = meta
        .get("installationfile")
        .map(|f| f.rsplit(['/', '\\']).next().unwrap_or(f).trim().to_string())
        .filter(|f| !f.is_empty());
    let mod_id = meta
        .get("modid")
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|id| *id > 0);
    let from_name = mod_id
        .zip(archive.as_deref())
        .map(|(id, a)| archive_parts(a, id));
    let meta_version = meta
        .get("version")
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty() && !is_placeholder(v));
    let version = meta_version.or_else(|| from_name.as_ref().and_then(|p| p.version.clone()));
    let source = match mod_id {
        Some(mod_id) => Source::Nexus {
            game: meta
                .get("gamename")
                .filter(|g| !g.is_empty())
                .cloned()
                .unwrap_or_else(|| "newvegas".to_string()),
            mod_id,
            file: meta
                .get("fileid")
                .and_then(|v| v.parse::<u64>().ok())
                .filter(|f| *f > 0),
            file_name: archive,
            size: None,
            md5: None,
            uploaded: from_name.and_then(|p| p.uploaded),
        },
        None => Source::Manual {
            url: None,
            file_name: archive,
            note: Some(
                if has_folder {
                    "no Nexus mod id in this Mod's meta.ini; find the file yourself"
                } else {
                    "the Profile lists this Mod but the Instance has no folder for it; find the file yourself"
                }
                .to_string(),
            ),
        },
    };
    (source, version)
}

/// MO2 writes `d2026.3.1.0` (a date) when a Mod has no version of its own.
fn is_placeholder(v: &str) -> bool {
    v.strip_prefix('d')
        .is_some_and(|r| r.starts_with(|c: char| c.is_ascii_digit()))
}

struct ArchiveParts {
    version: Option<String>,
    uploaded: Option<u64>,
}

/// `Name-<modid>-<v1>-<v2>-<epoch>.7z`: the upload time (Unix seconds) is the last segment and the version is what sits between the
/// mod id and it, joined with dots.
fn archive_parts(archive: &str, mod_id: u64) -> ArchiveParts {
    let stem = archive.rsplit_once('.').map_or(archive, |(s, _)| s);
    let mut segs: Vec<&str> = stem.split('-').collect();
    let uploaded = segs
        .last()
        .filter(|s| (9..=10).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|s| s.parse::<u64>().ok());
    if uploaded.is_some() {
        segs.pop();
    }
    let version = segs
        .iter()
        .rposition(|s| s.parse::<u64>() == Ok(mod_id))
        .map(|i| segs[i + 1..].join("."))
        .filter(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.'));
    ArchiveParts { version, uploaded }
}

fn slug(text: &str, fallback: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        fallback.to_string()
    } else {
        out
    }
}

fn unique(base: &str, used: &mut BTreeSet<String>) -> String {
    let mut id = base.to_string();
    let mut n = 2;
    while !used.insert(id.clone()) {
        id = format!("{base}-{n}");
        n += 1;
    }
    id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_upload_time_and_version_come_out_of_the_archive_name() {
        let p = archive_parts("Some Mod-12345-2-0-1-1700000000.7z", 12345);
        assert_eq!(p.uploaded, Some(1_700_000_000));
        assert_eq!(p.version.as_deref(), Some("2.0.1"));
        let p = archive_parts("Plain.zip", 7);
        assert_eq!((p.uploaded, p.version), (None, None));
    }

    #[test]
    fn slugs_are_unique_and_never_empty() {
        let mut used = BTreeSet::new();
        assert_eq!(unique(&slug("3D Rain!", "mod"), &mut used), "3d-rain");
        assert_eq!(unique(&slug("3d rain", "mod"), &mut used), "3d-rain-2");
        assert_eq!(slug("???", "mod"), "mod");
    }
}
