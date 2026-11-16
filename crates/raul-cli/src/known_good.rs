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
    let mut month = None;
    let mut positional = Vec::new();
    let mut rest = args.iter();
    while let Some(a) = rest.next() {
        match a.as_str() {
            "--json" => json = true,
            "--repo" | "--month" => {
                let Some(value) = rest.next() else {
                    return crate::usage(err, &format!("{a} needs a value"));
                };
                if a == "--repo" {
                    repo = PathBuf::from(value);
                } else {
                    month = Some(value.clone());
                }
            }
            other if other.starts_with("--") => {
                return crate::usage(err, &format!("unknown option {other}"));
            }
            other => positional.push(other.to_string()),
        }
    }
    match positional.as_slice() {
        [cmd] if cmd == "check" => check_command(&repo, json, out, err),
        [cmd, component, version] if cmd == "bump" => {
            let Some(month) = month else {
                return crate::usage(err, "bump needs --month YYYY-MM");
            };
            bump_command(&repo, component, version, &month, out, err)
        }
        _ => crate::usage(
            err,
            "known-good takes `check` or `bump <component> <version> --month YYYY-MM`",
        ),
    }
}

fn check_command(repo: &Path, json: bool, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let problems = check(repo);
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

fn bump_command(
    repo: &Path,
    component: &str,
    version: &str,
    month: &str,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let path = repo.join("known-good.toml");
    let result = std::fs::read_to_string(&path)
        .map_err(|e| format!("known-good.toml: {e}"))
        .and_then(|text| raul_core::known_good::bump(&text, component, version, month))
        .and_then(|text| std::fs::write(&path, text).map_err(|e| format!("known-good.toml: {e}")));
    match result {
        Ok(()) => crate::write_or_fail(
            out,
            err,
            &format!("known-good.toml: added a set with {component} {version}\n"),
        ),
        Err(e) => {
            let _ = writeln!(err, "{e}");
            crate::FOUND
        }
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
