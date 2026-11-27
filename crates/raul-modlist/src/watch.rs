// SPDX-License-Identifier: GPL-3.0-only
//! Recognising a manually downloaded archive (ADR-0023, "Manual is the floor"). The matching is pure: it takes a
//! candidate's name and size, the files the list still wants, and a way to read the candidate's md5 on demand, and says
//! which list entry the file is for, by file name, then size, then md5. [`Poller`] decides, from successive listings of a
//! folder, which archives are new and finished. The only disk access is [`md5_file`].

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::io::Read;
use std::path::{Path, PathBuf};

use md5::{Digest, Md5};

use crate::model::{ModList, Source};

/// A file the list still wants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wanted {
    /// The Mod's id in the list.
    pub id: String,
    /// The Mod folder name, for display and for naming the install.
    pub name: String,
    /// The archive's name, when the list has it.
    pub file_name: Option<String>,
    /// The archive's size, when the list has it.
    pub size: Option<u64>,
    /// The archive's md5 (lower case hex), when the list has it.
    pub md5: Option<String>,
    /// For a Nexus source: the game domain, the mod id and the file id (when known).
    pub nexus: Option<(String, u64, Option<u64>)>,
}

/// The files `list` wants for the Mods in `missing_ids`: only enabled Mods with a source that is a file.
#[must_use]
pub fn wanted(list: &ModList, missing_ids: &BTreeSet<String>) -> Vec<Wanted> {
    list.mods
        .iter()
        .filter(|m| m.enabled && missing_ids.contains(&m.id))
        .filter_map(|m| {
            let (file_name, size, md5, nexus) = match &m.source {
                Source::Nexus {
                    game,
                    mod_id,
                    file,
                    file_name,
                    size,
                    md5,
                    ..
                } => (
                    file_name.clone(),
                    *size,
                    md5.as_ref().map(|h| h.to_lowercase()),
                    Some((game.clone(), *mod_id, *file)),
                ),
                Source::Url { file_name, .. } | Source::Manual { file_name, .. } => {
                    (file_name.clone(), None, None, None)
                }
                _ => return None,
            };
            if file_name.is_none() && size.is_none() && md5.is_none() && nexus.is_none() {
                return None; // nothing to recognise it by
            }
            Some(Wanted {
                id: m.id.clone(),
                name: m.name.clone(),
                file_name,
                size,
                md5,
                nexus,
            })
        })
        .collect()
}

/// A downloaded file as the matcher sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// The file name, without a folder.
    pub name: String,
    /// The size in bytes.
    pub size: u64,
}

/// How a file was recognised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum By {
    /// The file name (and the size and md5, wherever the list has them, agree).
    Name,
    /// The size alone named exactly one entry (and the md5 agrees, when the list has one).
    Size,
    /// The md5 of the content.
    Md5,
    /// The Nexus md5 lookup named the Mod.
    Lookup,
}

impl By {
    /// The word printed.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            By::Name => "name",
            By::Size => "size",
            By::Md5 => "md5",
            By::Lookup => "nexus md5 lookup",
        }
    }
}

/// What the matcher decided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// The file is for this entry.
    Match {
        /// The entry's id.
        id: String,
        /// How it was recognised.
        by: By,
    },
    /// The name fits an entry but the content does not: not imported.
    Rejected {
        /// The entry's id.
        id: String,
        /// Why.
        why: String,
    },
    /// No entry wants this file.
    Unmatched,
}

/// A Nexus file found by its md5.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NexusRef {
    /// The game domain.
    pub game: String,
    /// The mod id.
    pub mod_id: u64,
    /// The file id.
    pub file_id: u64,
}

/// The seam for the Nexus md5 lookup (`md5_search`). Today [`NoLookup`] is used; the Nexus client plugs in later
/// without a change to the matcher or the commands.
pub trait Md5Lookup {
    /// Whether a lookup is wired; when not, the matcher does not hash a file for it.
    fn active(&self) -> bool {
        true
    }
    /// The Nexus file with this md5.
    fn by_md5(&self, md5: &str) -> Option<NexusRef>;
}

/// A lookup that finds nothing.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoLookup;

impl Md5Lookup for NoLookup {
    fn active(&self) -> bool {
        false
    }
    fn by_md5(&self, _: &str) -> Option<NexusRef> {
        None
    }
}

/// A name as compared: lower case, and without the " (1)" a browser adds to a second copy.
#[must_use]
pub fn normalize(name: &str) -> String {
    let lower = name.to_lowercase();
    let (stem, ext) = lower
        .rfind('.')
        .map_or((lower.as_str(), ""), |i| (&lower[..i], &lower[i..]));
    let stem = stem.trim_end();
    let stem = stem
        .strip_suffix(')')
        .and_then(|s| s.rsplit_once('('))
        .filter(|(_, n)| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        .map_or(stem, |(head, _)| head.trim_end());
    format!("{stem}{ext}")
}

/// Which entry of `wanted` the candidate is for. `hash` reads the candidate's md5 and is called at most once, and only
/// when a decision needs it.
pub fn classify(
    candidate: &Candidate,
    wanted: &[Wanted],
    hash: &mut dyn FnMut() -> Option<String>,
    lookup: &dyn Md5Lookup,
) -> Verdict {
    let mut cache: Option<Option<String>> = None;
    let mut md5 = || cache.get_or_insert_with(&mut *hash).clone();
    let name = normalize(&candidate.name);

    // 1. By file name; the size and md5 the list has must agree.
    if let Some(w) = wanted
        .iter()
        .find(|w| w.file_name.as_deref().map(normalize).as_deref() == Some(name.as_str()))
    {
        return confirm(w, candidate, &mut md5, By::Name);
    }
    // 2. By size, when it names exactly one entry.
    let by_size: Vec<&Wanted> = wanted
        .iter()
        .filter(|w| w.size == Some(candidate.size))
        .collect();
    if let [w] = by_size.as_slice() {
        return confirm(w, candidate, &mut md5, By::Size);
    }
    // 3. By md5, among the entries that carry one and could have this size.
    let hashed: Vec<&Wanted> = wanted
        .iter()
        .filter(|w| w.md5.is_some() && w.size.is_none_or(|s| s == candidate.size))
        .collect();
    if !hashed.is_empty()
        && let Some(h) = md5()
        && let Some(w) = hashed.iter().find(|w| w.md5.as_deref() == Some(h.as_str()))
    {
        return Verdict::Match {
            id: w.id.clone(),
            by: By::Md5,
        };
    }
    // 4. By the Nexus lookup, when one is wired.
    if lookup.active()
        && let Some(found) = md5().and_then(|h| lookup.by_md5(&h))
        && let Some(w) = wanted.iter().find(|w| {
            w.nexus.as_ref().is_some_and(|(g, m, f)| {
                *g == found.game && *m == found.mod_id && f.is_none_or(|f| f == found.file_id)
            })
        })
    {
        return Verdict::Match {
            id: w.id.clone(),
            by: By::Lookup,
        };
    }
    Verdict::Unmatched
}

fn confirm(
    w: &Wanted,
    candidate: &Candidate,
    md5: &mut dyn FnMut() -> Option<String>,
    by: By,
) -> Verdict {
    if let Some(size) = w.size.filter(|s| *s != candidate.size) {
        return Verdict::Rejected {
            id: w.id.clone(),
            why: format!(
                "{} is {} bytes, the list says {size}",
                candidate.name, candidate.size
            ),
        };
    }
    if let Some(want) = &w.md5 {
        match md5() {
            Some(got) if got == *want => {}
            Some(got) => {
                return Verdict::Rejected {
                    id: w.id.clone(),
                    why: format!("md5 {got} is not the list's {want}"),
                };
            }
            None => {
                return Verdict::Rejected {
                    id: w.id.clone(),
                    why: "the file could not be read to check its md5".to_owned(),
                };
            }
        }
    }
    Verdict::Match {
        id: w.id.clone(),
        by,
    }
}

/// The md5 of a file as lower case hex.
///
/// # Errors
/// The file could not be read.
pub fn md5_file(path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Md5::new();
    let mut buf = vec![0_u8; 1 << 16];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let mut hex = String::with_capacity(32);
    for b in hasher.finalize() {
        let _ = write!(hex, "{b:02x}");
    }
    Ok(hex)
}

/// Whether the name is one an archive has (`.7z`, `.zip`, `.rar`) and not a hidden or temporary file.
#[must_use]
pub fn is_archive(name: &str) -> bool {
    if name.starts_with('.') || name.starts_with("~$") {
        return false;
    }
    let lower = name.to_lowercase();
    [".7z", ".zip", ".rar"].iter().any(|e| lower.ends_with(e))
}

/// A file seen in a folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The full path.
    pub path: PathBuf,
    /// The size in bytes.
    pub size: u64,
}

impl Entry {
    /// The file name.
    #[must_use]
    pub fn name(&self) -> String {
        self.path
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
    }
}

/// Decides which archives of successive listings are new and finished. A browser's partial files (`.crdownload`,
/// `.part`) are not archives by name; an archive written in place is finished once two polls see the same size.
#[derive(Debug, Default)]
pub struct Poller {
    known: BTreeSet<PathBuf>,
    pending: BTreeMap<PathBuf, u64>,
}

impl Poller {
    /// A poller that treats nothing as known.
    #[must_use]
    pub fn new() -> Poller {
        Poller::default()
    }

    /// Mark what is already in the folder, so only later arrivals count.
    pub fn baseline(&mut self, listing: &[Entry]) {
        self.known.extend(listing.iter().map(|e| e.path.clone()));
    }

    /// The new, finished archives of this listing.
    pub fn poll(&mut self, listing: &[Entry]) -> Vec<Entry> {
        let mut ready = Vec::new();
        let present: BTreeSet<&PathBuf> = listing.iter().map(|e| &e.path).collect();
        self.pending.retain(|p, _| present.contains(p));
        for e in listing {
            if self.known.contains(&e.path) || e.size == 0 || !is_archive(&e.name()) {
                continue;
            }
            if self.pending.get(&e.path) == Some(&e.size) {
                self.pending.remove(&e.path);
                self.known.insert(e.path.clone());
                ready.push(e.clone());
            } else {
                self.pending.insert(e.path.clone(), e.size);
            }
        }
        ready
    }

    /// Every archive of the listing not yet handled, without waiting for the size to settle (a single scan).
    pub fn take_all(&mut self, listing: &[Entry]) -> Vec<Entry> {
        let mut out = Vec::new();
        for e in listing {
            if !self.known.contains(&e.path) && e.size > 0 && is_archive(&e.name()) {
                self.known.insert(e.path.clone());
                out.push(e.clone());
            }
        }
        out
    }
}

/// The folders a browser may save to: the XDG download folder (`user_dirs` is the text of `user-dirs.dirs`), the
/// folder `Downloads` of the home, and the Instance's own downloads folder. `env` reads an environment variable.
#[must_use]
pub fn download_dirs(
    env: &dyn Fn(&str) -> Option<String>,
    user_dirs: Option<&str>,
    instance_downloads: Option<PathBuf>,
) -> Vec<PathBuf> {
    let home = env("HOME")
        .or_else(|| env("USERPROFILE"))
        .map(PathBuf::from);
    let mut dirs = Vec::new();
    if let Some(dir) = env("XDG_DOWNLOAD_DIR") {
        dirs.push(PathBuf::from(dir));
    }
    if let (Some(text), Some(home)) = (user_dirs, &home)
        && let Some(rel) = xdg_download(text, home)
    {
        dirs.push(rel);
    }
    if let Some(home) = &home {
        dirs.push(home.join("Downloads"));
    }
    dirs.extend(instance_downloads);
    let mut seen = BTreeSet::new();
    dirs.retain(|d| seen.insert(d.clone()));
    dirs
}

fn xdg_download(text: &str, home: &Path) -> Option<PathBuf> {
    let line = text
        .lines()
        .find_map(|l| l.trim().strip_prefix("XDG_DOWNLOAD_DIR="))?;
    let value = line.trim().trim_matches('"');
    let path = value.strip_prefix("$HOME").map_or_else(
        || PathBuf::from(value),
        |r| home.join(r.trim_start_matches('/')),
    );
    Some(path)
}
