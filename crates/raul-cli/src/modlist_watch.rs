// SPDX-License-Identifier: GPL-3.0-only
//! `raul modlist fetch` and `raul modlist watch` (ADR-0023, #680): open the download page of each missing Mod, and
//! import the archives a person downloads by hand. The matching is `raul_modlist::watch` (pure); this file reads
//! folders, copies files and calls nvmod's own install job.
//!
//! Where an import goes: the file is copied into the Instance's downloads folder (`<Instance>/downloads`), which is
//! where `nvmod downloads` already looks. With `--install` the Mod is then installed into the Profile through
//! `nvmod_api::downloads::install`, the existing install path. Install-into-profile in list order (#681) does not
//! exist yet, so an installed Mod is placed last, and the Mod is named as the list names it.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use nvmod_api::downloads::{InstallRequest, downloads_dir};
use nvmod_api::install::InstallOutput;
use nvmod_api::{CancelToken, NoProgress, Outcome};
use raul_core::context::Context;
use raul_modlist::ModList;
use raul_modlist::plan::{self, Acquire, NoResolver};
use raul_modlist::watch::{
    Candidate, Entry, NoLookup, Poller, Verdict, Wanted, classify, download_dirs, md5_file,
};

use crate::modlist::{Flags, instance_root, present_in_instance};

/// How often the folders are read.
const POLL: Duration = Duration::from_secs(2);
/// Pages opened by `fetch --open` unless `--limit` says otherwise.
const DEFAULT_LIMIT: usize = 10;
/// Names a program to open addresses with instead of the system's (`xdg-open`, `open`, `rundll32`).
const OPENER_ENV: &str = "RAUL_URL_OPENER";

fn present(ctx: &Context, flags: &Flags) -> Result<Vec<String>, String> {
    if flags.all {
        Ok(Vec::new())
    } else {
        present_in_instance(ctx, flags)
    }
}

/// `raul modlist fetch`.
pub(crate) fn fetch(
    ctx: &Context,
    list: &ModList,
    flags: &Flags,
    out: &mut dyn Write,
) -> Result<u8, String> {
    let plan = plan::plan(list, &present(ctx, flags)?, &NoResolver);
    let mut pages: Vec<(usize, &str, String)> = Vec::new();
    let mut by_hand = Vec::new();
    for item in plan.missing() {
        match &item.acquire {
            Some(Acquire::Nexus { page, .. }) => {
                pages.push((item.position, &item.name, page.clone()));
            }
            Some(Acquire::Link { url, .. }) => pages.push((item.position, &item.name, url.clone())),
            Some(Acquire::Manual { note, .. }) => by_hand.push((&item.name, note.clone())),
            _ => {}
        }
    }
    let mut text = String::new();
    for (pos, name, url) in &pages {
        let _ = write!(text, "{pos:>4}. {name}\n      {url}\n");
    }
    for (name, note) in &by_hand {
        let _ = writeln!(
            text,
            "      {name}: by hand, {}",
            note.as_deref().unwrap_or("no download is recorded")
        );
    }
    if pages.is_empty() && by_hand.is_empty() {
        text.push_str("Nothing to download.\n");
    }
    let _ = out.write_all(text.as_bytes());
    if !flags.open {
        return Ok(crate::OK);
    }
    let limit = flags.limit.unwrap_or(DEFAULT_LIMIT);
    let mut failed = 0;
    for (_, name, url) in pages.iter().take(limit) {
        match open_url(url) {
            Ok(()) => {}
            Err(why) => {
                failed += 1;
                let _ = writeln!(out, "could not open the page of {name}: {why}");
            }
        }
    }
    let opened = pages.len().min(limit) - failed;
    let rest = pages.len().saturating_sub(limit);
    let _ = writeln!(out, "Opened {opened} page(s).");
    if rest > 0 {
        let _ = writeln!(
            out,
            "{rest} more are listed above; run it again with a higher --limit to open them."
        );
    }
    Ok(if failed == 0 {
        crate::OK
    } else {
        crate::REFUSED
    })
}

/// Open an address in the person's browser through the system's opener, started by the runner (ADR-0007).
fn open_url(url: &str) -> Result<(), String> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("only http and https addresses are opened".to_owned());
    }
    let program = std::env::var(OPENER_ENV).unwrap_or_else(|_| {
        if cfg!(target_os = "macos") {
            "open"
        } else if cfg!(windows) {
            "rundll32"
        } else {
            "xdg-open"
        }
        .to_owned()
    });
    let args: Vec<String> = if program == "rundll32" {
        vec!["url.dll,FileProtocolHandler".to_owned(), url.to_owned()]
    } else {
        vec![url.to_owned()]
    };
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let started = raul_runner::start_program(&program, &args, &BTreeMap::new(), &cwd, false, None)?;
    match started.wait()? {
        0 => Ok(()),
        code => Err(format!("{program} exited with {code}")),
    }
}

fn read_dir(dir: &Path) -> Vec<Entry> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    rd.filter_map(Result::ok)
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            meta.is_file().then(|| Entry {
                path: e.path(),
                size: meta.len(),
            })
        })
        .collect()
}

fn env_var(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

/// `raul modlist watch`.
pub(crate) fn watch(
    ctx: &Context,
    list: &ModList,
    flags: &Flags,
    out: &mut dyn Write,
) -> Result<u8, String> {
    let root = instance_root(ctx, flags)?;
    let plan = plan::plan(list, &present_in_instance(ctx, flags)?, &NoResolver);
    let missing: BTreeSet<String> = plan.missing().map(|i| i.id.clone()).collect();
    let mut wanted = raul_modlist::watch::wanted(list, &missing);
    let (staging, _) = downloads_dir(&root, None);
    let dirs = if let Some(d) = &flags.downloads {
        vec![d.clone()]
    } else {
        let user_dirs = env_var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| env_var("HOME").map(|h| PathBuf::from(h).join(".config")))
            .and_then(|base| std::fs::read_to_string(base.join("user-dirs.dirs")).ok());
        download_dirs(&env_var, user_dirs.as_deref(), Some(staging.clone()))
    };
    if dirs.is_empty() {
        return Err("no download folder is known: pass --downloads <dir>".to_owned());
    }
    if wanted.is_empty() {
        let _ = writeln!(
            out,
            "Nothing to wait for: no missing Mod has a file to download."
        );
        return Ok(crate::OK);
    }
    let _ = writeln!(
        out,
        "Watching {} for {} file(s). Ctrl+C stops.",
        dirs.iter()
            .map(|d| d.display().to_string())
            .collect::<Vec<_>>()
            .join(", "),
        wanted.len()
    );
    let _ = ctrlc::set_handler(nvmod_core::cancel::request);
    let mut poller = Poller::new();
    if !flags.once && !flags.scan_existing {
        for d in &dirs {
            poller.baseline(&read_dir(d));
        }
    }
    let mut imported = 0_usize;
    let mut problems = 0_usize;
    loop {
        for d in &dirs {
            let listing = read_dir(d);
            let ready = if flags.once {
                poller.take_all(&listing)
            } else {
                poller.poll(&listing)
            };
            for entry in ready {
                if d == &staging {
                    // Our own staging folder: a file there is already imported.
                    continue;
                }
                match handle(&entry, &mut wanted, &root, &staging, flags, out) {
                    Handled::Imported => imported += 1,
                    Handled::Problem => problems += 1,
                    Handled::Ignored => {}
                }
            }
        }
        if wanted.is_empty() || flags.once || nvmod_core::cancel::requested() {
            break;
        }
        std::thread::sleep(POLL);
    }
    let _ = writeln!(out, "Imported {imported}; {} still wanted.", wanted.len());
    Ok(if problems == 0 {
        crate::OK
    } else {
        crate::REFUSED
    })
}

enum Handled {
    Imported,
    Problem,
    Ignored,
}

fn handle(
    entry: &Entry,
    wanted: &mut Vec<Wanted>,
    root: &Path,
    staging: &Path,
    flags: &Flags,
    out: &mut dyn Write,
) -> Handled {
    let candidate = Candidate {
        name: entry.name(),
        size: entry.size,
    };
    let verdict = classify(
        &candidate,
        wanted,
        &mut || md5_file(&entry.path).ok(),
        &NoLookup,
    );
    let (id, by) = match verdict {
        Verdict::Unmatched => return Handled::Ignored,
        Verdict::Rejected { id, why } => {
            let _ = writeln!(
                out,
                "Not importing {}: for `{id}`, but {why}.",
                candidate.name
            );
            return Handled::Problem;
        }
        Verdict::Match { id, by } => (id, by),
    };
    let Some(pos) = wanted.iter().position(|w| w.id == id) else {
        return Handled::Ignored;
    };
    let name = wanted[pos].name.clone();
    if let Err(why) = stage(&entry.path, staging, &candidate.name) {
        let _ = writeln!(out, "Could not stage {}: {why}", candidate.name);
        return Handled::Problem;
    }
    let _ = writeln!(
        out,
        "{} is {name} (by {}); staged in {}.",
        candidate.name,
        by.as_str(),
        staging.display()
    );
    wanted.remove(pos);
    if flags.install {
        match install(
            root,
            staging,
            &candidate.name,
            &name,
            flags.profile.as_deref(),
        ) {
            Ok(Some(report)) => {
                let _ = writeln!(out, "  installed {report}.");
            }
            Ok(None) => {
                let _ = writeln!(
                    out,
                    "  left staged: its installer asks questions (answer them with `nvmod downloads install`)."
                );
            }
            Err(why) => {
                let _ = writeln!(out, "  not installed: {why}");
                return Handled::Problem;
            }
        }
    }
    Handled::Imported
}

/// Copy `src` into the staging folder as `name`; a file of that name that is the same size is taken as already there.
fn stage(src: &Path, staging: &Path, name: &str) -> Result<(), String> {
    std::fs::create_dir_all(staging).map_err(|e| e.to_string())?;
    let dest = staging.join(name);
    if src == dest {
        return Ok(());
    }
    if let Ok(meta) = std::fs::metadata(&dest) {
        let same = std::fs::metadata(src).is_ok_and(|m| m.len() == meta.len());
        return if same {
            Ok(())
        } else {
            Err(format!("a different file named {name} is already staged"))
        };
    }
    let partial = staging.join(format!("{name}.raul-part"));
    std::fs::copy(src, &partial).map_err(|e| e.to_string())?;
    std::fs::rename(&partial, &dest).map_err(|e| e.to_string())
}

fn install(
    root: &Path,
    staging: &Path,
    archive: &str,
    name: &str,
    profile: Option<&str>,
) -> Result<Option<String>, String> {
    let request = InstallRequest {
        instance: root.to_path_buf(),
        downloads: Some(staging.to_path_buf()),
        archive: archive.to_owned(),
        profile: profile.map(str::to_owned),
        name: Some(name.to_owned()),
        dry_run: false,
        answers: None,
        max_extract_bytes: None,
        extract_limits: nvmod_modinstall::LimitOverrides::default(),
    };
    match nvmod_api::downloads::install(&request, &mut NoProgress, &CancelToken::new()) {
        Ok(Outcome::Done(o)) => Ok(match o.install {
            InstallOutput::Installed(r) => Some(format!("{} ({} files)", r.r#mod, r.files.len())),
            InstallOutput::Questions(_) => None,
        }),
        Ok(Outcome::Cancelled) => Err("cancelled".to_owned()),
        Err(e) => Err(e.to_string()),
    }
}
