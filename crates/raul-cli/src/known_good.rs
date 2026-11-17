// SPDX-License-Identifier: GPL-3.0-only
//! `raul known-good check`: the latest Known-good set against the submodule pins and nvmod's `tools.lock.toml`.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use raul_core::known_good::{COMPONENTS, KnownGood, Pin, component_dir};
use raul_runner::{Call, Runner};

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
        [cmd, component, version] if cmd == "apply" => {
            let month = month.unwrap_or_else(current_month);
            apply_command(&repo, component, version, &month, out, err)
        }
        _ => crate::usage(
            err,
            "known-good takes `check`, `bump <component> <version> --month YYYY-MM` or `apply <component> <version> [--month YYYY-MM]`",
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

/// This month in UTC as `YYYY-MM`.
fn current_month() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    month_of_days(i64::try_from(secs / 86_400).unwrap_or(0))
}

/// The `YYYY-MM` of a day count since 1970-01-01 (Howard Hinnant's civil-from-days).
fn month_of_days(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}")
}

/// Moves a Component to a release the way a bump pull request does: its submodule goes to the tag, a new
/// Known-good set is added, and when the Component is nvmod the loot-rs it links moves too (submodule, set and the
/// workspace `loot-rs` tag, ADR-0015). Lock files are not touched: the caller runs `cargo update` after.
fn apply_command(
    repo: &Path,
    component: &str,
    version: &str,
    month: &str,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    match apply(repo, component, version, month) {
        Ok(moved) => crate::write_or_fail(out, err, &format!("moved {}\n", moved.join(", "))),
        Err(e) => {
            let _ = writeln!(err, "{e}");
            crate::FOUND
        }
    }
}

fn apply(repo: &Path, component: &str, version: &str, month: &str) -> Result<Vec<String>, String> {
    if !COMPONENTS.contains(&component) {
        return Err(format!("{component} is not a Component"));
    }
    if !version.starts_with('v')
        || !version
            .chars()
            .skip(1)
            .all(|c| c.is_ascii_digit() || c == '.')
    {
        return Err(format!("{version} is not a vX.Y.Z tag"));
    }
    check_out_tag(repo, component, version)?;
    let mut moves = vec![(component.to_string(), version.to_string())];
    if component == "nvmod" {
        let lock = std::fs::read_to_string(repo.join("components/nvmod-rs/tools.lock.toml"))
            .map_err(|e| format!("nvmod tools.lock.toml: {e}"))?;
        let loot = raul_core::known_good::linked_loot_tag(&lock)
            .ok_or("nvmod's tools.lock.toml has no loot-rs entry")?;
        check_out_tag(repo, "loot-rs", &loot)?;
        let cargo_path = repo.join("Cargo.toml");
        let cargo = std::fs::read_to_string(&cargo_path).map_err(|e| format!("Cargo.toml: {e}"))?;
        if let Some(text) = raul_core::known_good::with_loot_tag(&cargo, &loot) {
            std::fs::write(&cargo_path, text).map_err(|e| format!("Cargo.toml: {e}"))?;
        }
        moves.push(("loot-rs".to_string(), loot));
    }
    let path = repo.join("known-good.toml");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("known-good.toml: {e}"))?;
    let pairs: Vec<(&str, &str)> = moves
        .iter()
        .map(|(c, v)| (c.as_str(), v.as_str()))
        .collect();
    let text = raul_core::known_good::bump_all(&text, &pairs, month)?;
    std::fs::write(&path, text).map_err(|e| format!("known-good.toml: {e}"))?;
    Ok(moves.into_iter().map(|(c, v)| format!("{c} {v}")).collect())
}

fn check_out_tag(repo: &Path, component: &str, tag: &str) -> Result<(), String> {
    let dir = repo.join("components").join(component_dir(component));
    git(&dir, &["fetch", "--quiet", "--tags", "origin"])
        .ok_or_else(|| format!("{component}: could not fetch the tags"))?;
    git(&dir, &["checkout", "--quiet", &format!("refs/tags/{tag}")])
        .ok_or_else(|| format!("{component}: could not check out {tag}"))?;
    match pin(&dir) {
        Some(Pin::Tag(now)) if now == tag => Ok(()),
        _ => Err(format!("{component}: {tag} is not a tag of its repository")),
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

/// Run `git` in `dir` through the Runner and return its trimmed standard output, or `None` when it fails.
fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
    let mut call = Call::new("git", Path::new("git"), &args);
    call.cwd = Some(dir);
    let runner = Runner {
        timeout: Duration::from_secs(30),
        max_output: 64 * 1024,
        ..Runner::default()
    };
    let out = runner.run(&call).ok()?;
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// The release tag (`v*`) at the submodule's HEAD, or its short commit when it has none.
fn pin(dir: &Path) -> Option<Pin> {
    let tags = git(dir, &["tag", "--points-at", "HEAD"])?;
    if let Some(tag) = tags.lines().find(|t| t.starts_with('v')) {
        return Some(Pin::Tag(tag.to_string()));
    }
    git(dir, &["rev-parse", "--short", "HEAD"]).map(Pin::Untagged)
}

#[cfg(test)]
mod tests {
    use super::month_of_days;

    #[test]
    fn the_month_of_a_day_count() {
        assert_eq!(month_of_days(0), "1970-01");
        assert_eq!(month_of_days(20_409), "2025-11");
        assert_eq!(month_of_days(20_454), "2026-01");
        assert_eq!(month_of_days(20_789), "2026-12");
        assert_eq!(month_of_days(11_016), "2000-02");
    }
}
