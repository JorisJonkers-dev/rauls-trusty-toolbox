// SPDX-License-Identifier: GPL-3.0-only
//! `raul known-good check`: the latest Known-good set against the submodule pins and nvmod's `tools.lock.toml`.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use raul_core::known_good::{COMPONENTS, KnownGood, Pin, component_dir};

pub(crate) fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let mut repo = PathBuf::from(".");
    let mut json = false;
    let mut rest = args.iter();
    if rest.next().map(String::as_str) != Some("check") {
        return crate::usage(err, "known-good takes the subcommand `check`");
    }
    while let Some(a) = rest.next() {
        match a.as_str() {
            "--json" => json = true,
            "--repo" => match rest.next() {
                Some(dir) => repo = PathBuf::from(dir),
                None => return crate::usage(err, "--repo needs a folder"),
            },
            other => return crate::usage(err, &format!("unknown option {other}")),
        }
    }
    let problems = check(&repo);
    if json {
        let v = serde_json::json!({ "ok": problems.is_empty(), "problems": problems });
        crate::write_or_fail(out, err, &format!("{v}\n"));
    } else if problems.is_empty() {
        crate::write_or_fail(
            out,
            err,
            "known-good.toml matches the submodules and nvmod's tools.lock.toml\n",
        );
    } else {
        for p in &problems {
            let _ = writeln!(err, "{p}");
        }
    }
    if problems.is_empty() {
        crate::OK
    } else {
        crate::FOUND
    }
}

fn check(repo: &Path) -> Vec<String> {
    let text = match std::fs::read_to_string(repo.join("known-good.toml")) {
        Ok(t) => t,
        Err(e) => return vec![format!("known-good.toml: {e}")],
    };
    let kg = match KnownGood::parse(&text) {
        Ok(kg) => kg,
        Err(problems) => return problems,
    };
    let lock = std::fs::read_to_string(repo.join("components/nvmod-rs/tools.lock.toml"))
        .unwrap_or_default();
    let pins: BTreeMap<String, Pin> = COMPONENTS
        .iter()
        .filter_map(|c| {
            pin(&repo.join("components").join(component_dir(c))).map(|p| ((*c).to_string(), p))
        })
        .collect();
    kg.check(&pins, &lock)
}

fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// The release tag (`v*`) at the submodule's HEAD, or its short commit when it has none.
fn pin(dir: &Path) -> Option<Pin> {
    let tags = git(dir, &["tag", "--points-at", "HEAD"])?;
    if let Some(tag) = tags.lines().find(|t| t.starts_with('v')) {
        return Some(Pin::Tag(tag.to_string()));
    }
    git(dir, &["rev-parse", "--short", "HEAD"]).map(Pin::Untagged)
}
