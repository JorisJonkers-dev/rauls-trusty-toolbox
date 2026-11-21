// SPDX-License-Identifier: GPL-3.0-only
//! Keeps the GUI workspace's lock and manifest consistent with nv-rs, whose crates it links in process (ADR-0017).
//!
//! Checks, all against `components/nv-rs` at the pinned commit:
//!
//! 1. Families: the versions of windows, winit, ash, wgpu*, naga and raw-window-handle in the GUI lock are exactly the
//!    versions nv-rs locks (no extra duplicate version of a big family).
//! 2. Closure: every registry crate that `fork-api` or `fork-play` reaches in both locks is at a version both have, or
//!    the difference is listed, with the reason, in `scripts/component-locks.allow` (and a listed one that has gone is
//!    an error). Each lock resolves on its own (the 7-day cooldown, other consumers), so some drift is normal; the
//!    file is where it is reviewed.
//! 3. Settings that do not travel with a dependency: the `[patch.crates-io]` wgpu-hal path and the release profile
//!    entries for engine-core and engine-platform equal those of nv-rs's root `Cargo.toml`.
//! 4. The Rust toolchain channel equals nv-rs's.
//!
//! ```text
//! check-component-locks [--root <repository>]   check the repository (default: the folder above with a .git)
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

use toml::Table;

/// The crates whose dependency closure the engine link reaches.
const ROOTS: [&str; 2] = ["fork-api", "fork-play"];

/// Names of the families whose versions must be the same in both locks.
fn is_family(name: &str) -> bool {
    matches!(
        name,
        "windows" | "winit" | "ash" | "naga" | "raw-window-handle"
    ) || name == "wgpu"
        || name.starts_with("wgpu-")
}

/// One package of a parsed `Cargo.lock`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Package {
    name: String,
    version: String,
    registry: bool,
    dependencies: Vec<String>,
}

/// The packages of a parsed `Cargo.lock`.
fn packages(lock: &Table) -> Vec<Package> {
    let text = |t: &Table, key: &str| {
        t.get(key)
            .and_then(toml::Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    lock.get("package")
        .and_then(toml::Value::as_array)
        .map(|all| {
            all.iter()
                .filter_map(toml::Value::as_table)
                .map(|p| Package {
                    name: text(p, "name"),
                    version: text(p, "version"),
                    registry: text(p, "source").starts_with("registry"),
                    dependencies: p
                        .get("dependencies")
                        .and_then(toml::Value::as_array)
                        .map(|d| {
                            d.iter()
                                .filter_map(|s| s.as_str().map(str::to_owned))
                                .collect()
                        })
                        .unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default()
}

type Versions = BTreeMap<String, BTreeSet<String>>;

/// name -> versions of the registry packages.
fn versions(pkgs: &[Package]) -> Versions {
    let mut out = Versions::new();
    for p in pkgs.iter().filter(|p| p.registry) {
        out.entry(p.name.clone())
            .or_default()
            .insert(p.version.clone());
    }
    out
}

/// name -> versions of every registry package reachable from the root packages.
fn closure(pkgs: &[Package]) -> Versions {
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut todo: Vec<&Package> = pkgs
        .iter()
        .filter(|p| ROOTS.contains(&p.name.as_str()))
        .collect();
    while let Some(pkg) = todo.pop() {
        if !seen.insert((pkg.name.clone(), pkg.version.clone())) {
            continue;
        }
        for dep in &pkg.dependencies {
            let mut parts = dep.split(' ');
            let name = parts.next().unwrap_or_default();
            let version = parts.next();
            todo.extend(
                pkgs.iter()
                    .filter(|c| c.name == name && version.is_none_or(|v| c.version == v)),
            );
        }
    }
    let registry: BTreeSet<(&str, &str)> = pkgs
        .iter()
        .filter(|p| p.registry)
        .map(|p| (p.name.as_str(), p.version.as_str()))
        .collect();
    let mut out = Versions::new();
    for (name, version) in seen {
        if registry.contains(&(name.as_str(), version.as_str())) {
            out.entry(name).or_default().insert(version);
        }
    }
    out
}

fn join(set: &BTreeSet<String>) -> String {
    set.iter().cloned().collect::<Vec<_>>().join(",")
}

fn list(set: &BTreeSet<String>) -> String {
    format!("{:?}", set.iter().collect::<Vec<_>>())
}

fn check_families(gui: &Versions, nv: &Versions) -> Vec<String> {
    gui.iter()
        .filter(|(name, v)| is_family(name) && nv.get(*name).is_some_and(|w| w != *v))
        .map(|(name, v)| {
            format!(
                "family {name}: GUI lock {}, nv-rs lock {}",
                list(v),
                list(&nv[name])
            )
        })
        .collect()
}

/// The registry crates both closures reach at no common version, as `name gui-versions nv-versions`.
fn drifts(gui: &[Package], nv: &[Package]) -> BTreeSet<String> {
    let (gui, nv) = (closure(gui), closure(nv));
    gui.iter()
        .filter_map(|(name, g)| {
            let n = nv.get(name)?;
            g.is_disjoint(n)
                .then(|| format!("{name} {} {}", join(g), join(n)))
        })
        .collect()
}

/// Drift must be listed in `scripts/component-locks.allow`, and a listed drift must still exist.
fn check_closure(gui: &[Package], nv: &[Package], accepted: &BTreeSet<String>) -> Vec<String> {
    let found = drifts(gui, nv);
    let mut problems: Vec<String> = found
        .difference(accepted)
        .map(|d| format!("drift not accepted in scripts/component-locks.allow: {d}"))
        .collect();
    problems.extend(
        accepted
            .difference(&found)
            .map(|d| format!("accepted drift no longer exists, remove it: {d}")),
    );
    problems
}

/// `a/b/../c` as `a/c`, without touching the file system.
fn normal(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::ParentDir if out.file_name().is_some() => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

fn at<'a>(table: &'a Table, path: &[&str]) -> Option<&'a toml::Value> {
    let (last, parents) = path.split_last()?;
    let mut t = table;
    for key in parents {
        t = t.get(*key)?.as_table()?;
    }
    t.get(*last)
}

fn check_settings(gui: &Table, nv: &Table) -> Vec<String> {
    let mut problems = Vec::new();
    let hal = |t: &Table| {
        at(t, &["patch", "crates-io", "wgpu-hal", "path"])
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
    };
    match (hal(nv), hal(gui)) {
        (None, _) => problems.push(
            "nv-rs has no [patch.crates-io] wgpu-hal: remove ours or update this check".to_owned(),
        ),
        (Some(theirs), ours) => {
            let want = normal(&Path::new("../../components/nv-rs").join(theirs));
            if ours.as_deref().map(|p| normal(Path::new(p))) != Some(want.clone()) {
                problems.push(format!(
                    "[patch.crates-io] wgpu-hal is {ours:?} in crates/raul-gui, expected {:?}",
                    want.display().to_string()
                ));
            }
        }
    }
    for krate in ["engine-core", "engine-platform"] {
        let key = ["profile", "release", "package", krate];
        let (theirs, ours) = (at(nv, &key), at(gui, &key));
        if theirs != ours {
            problems.push(format!(
                "[profile.release.package.{krate}] is {ours:?} in crates/raul-gui, nv-rs has {theirs:?}"
            ));
        }
    }
    problems
}

fn check_toolchain(ours: &Table, theirs: &Table) -> Vec<String> {
    let channel = |t: &Table| {
        at(t, &["toolchain", "channel"])
            .and_then(toml::Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    let (a, b) = (channel(ours), channel(theirs));
    if a == b {
        Vec::new()
    } else {
        vec![format!("toolchain channel {a} here, {b} in nv-rs")]
    }
}

fn load(path: &Path) -> Result<Table, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    text.parse::<Table>()
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// Lines `name gui-versions nv-versions  # why`.
fn accepted(text: &str) -> BTreeSet<String> {
    text.lines()
        .map(|l| {
            l.split('#')
                .next()
                .unwrap_or_default()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|l| !l.is_empty())
        .collect()
}

/// Every problem the repository at `root` has.
fn run(root: &Path) -> Result<Vec<String>, String> {
    let nv_dir = root.join("components").join("nv-rs");
    let gui = packages(&load(&root.join("crates/raul-gui/Cargo.lock"))?);
    let nv = packages(&load(&nv_dir.join("Cargo.lock"))?);
    let allow = root.join("scripts/component-locks.allow");
    let allow = std::fs::read_to_string(&allow).map_err(|e| format!("{}: {e}", allow.display()))?;
    let mut problems = check_families(&versions(&gui), &versions(&nv));
    problems.extend(check_closure(&gui, &nv, &accepted(&allow)));
    problems.extend(check_settings(
        &load(&root.join("crates/raul-gui/Cargo.toml"))?,
        &load(&nv_dir.join("Cargo.toml"))?,
    ));
    problems.extend(check_toolchain(
        &load(&root.join("rust-toolchain.toml"))?,
        &load(&nv_dir.join("rust-toolchain.toml"))?,
    ));
    Ok(problems)
}

/// The nearest folder above the current one that holds `.git` (a folder, or the file a worktree has).
fn top_level() -> Result<PathBuf, String> {
    let here = std::env::current_dir().map_err(|e| format!("current folder: {e}"))?;
    here.ancestors()
        .find(|dir| dir.join(".git").exists())
        .map(Path::to_path_buf)
        .ok_or_else(|| "not inside a git working tree; give --root".to_owned())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = match args.as_slice() {
        [] => top_level(),
        [flag, dir] if flag == "--root" => Ok(PathBuf::from(dir)),
        _ => Err("usage: check-component-locks [--root <repository>]".to_owned()),
    };
    match root.and_then(|r| run(&r)) {
        Ok(problems) if problems.is_empty() => {
            println!("ok   the GUI lock and manifest agree with nv-rs");
            ExitCode::SUCCESS
        }
        Ok(problems) => {
            for p in problems {
                eprintln!("FAIL {p}");
            }
            ExitCode::FAILURE
        }
        Err(why) => {
            eprintln!("check-component-locks: {why}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pkg(name: &str, version: &str, deps: &[&str], registry: bool) -> Package {
        Package {
            name: name.to_owned(),
            version: version.to_owned(),
            registry,
            dependencies: deps.iter().map(|d| (*d).to_owned()).collect(),
        }
    }

    fn lock(tokio: &str) -> Vec<Package> {
        vec![
            pkg("fork-play", "0.1.0", &["tokio", "wgpu"], false),
            pkg("tokio", tokio, &[], true),
            pkg("wgpu", "30.0.1", &[], true),
            pkg("windows", "0.58.0", &[], true),
            pkg("windows", "0.62.2", &[], true),
        ]
    }

    fn set(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| (*s).to_owned()).collect()
    }

    fn table(text: &str) -> Table {
        text.parse().unwrap()
    }

    #[test]
    fn equal_families_pass_and_an_extra_version_fails() {
        let (gui, nv) = (lock("1.54.1"), lock("1.53.1"));
        assert!(check_families(&versions(&gui), &versions(&nv)).is_empty());
        let mut more = gui.clone();
        more.push(pkg("windows", "0.61.0", &[], true));
        assert_eq!(check_families(&versions(&more), &versions(&nv)).len(), 1);
    }

    #[test]
    fn drift_is_found_and_must_be_listed() {
        let (gui, nv) = (lock("1.54.1"), lock("1.53.1"));
        assert_eq!(drifts(&gui, &nv), set(&["tokio 1.54.1 1.53.1"]));
        assert!(check_closure(&gui, &nv, &set(&["tokio 1.54.1 1.53.1"])).is_empty());
        assert_eq!(
            check_closure(&gui, &nv, &set(&[])).len(),
            1,
            "an unlisted drift must fail"
        );
        let stale = set(&["tokio 1.54.1 1.53.1", "gone 1 2"]);
        assert_eq!(
            check_closure(&gui, &nv, &stale).len(),
            1,
            "a stale entry must fail"
        );
    }

    #[test]
    fn the_closure_follows_versioned_dependencies_and_skips_what_is_unreachable() {
        let pkgs = vec![
            pkg("fork-api", "0.1.0", &["a 1.0.0"], false),
            pkg("a", "1.0.0", &["b"], true),
            pkg("a", "2.0.0", &["c"], true),
            pkg("b", "1.0.0", &[], true),
            pkg("c", "1.0.0", &[], true),
        ];
        let names: Vec<_> = closure(&pkgs).into_keys().collect();
        assert_eq!(names, ["a", "b"], "a 2.0.0 and c are not reached");
    }

    #[test]
    fn packages_read_a_lock_and_tell_registry_from_path_packages() {
        let lock = table(
            "[[package]]\nname = \"x\"\nversion = \"1.0.0\"\nsource = \"registry+https://example\"\ndependencies = [\"y 2.0.0\"]\n\n[[package]]\nname = \"y\"\nversion = \"2.0.0\"\n",
        );
        let p = packages(&lock);
        assert_eq!(p.len(), 2);
        assert!(p[0].registry && !p[1].registry);
        assert_eq!(p[0].dependencies, ["y 2.0.0"]);
    }

    const THEIRS: &str = "[patch.crates-io]\nwgpu-hal = { path = \"vendor/wgpu-hal\" }\n[profile.release.package.engine-core]\ncodegen-units = 1\n[profile.release.package.engine-platform]\ncodegen-units = 1\n";

    #[test]
    fn settings_must_equal_the_engines() {
        let nv = table(THEIRS);
        let ours =
            table(&THEIRS.replace("vendor/wgpu-hal", "../../components/nv-rs/vendor/wgpu-hal"));
        assert!(check_settings(&ours, &nv).is_empty());
        let changed = table(
            "[patch.crates-io]\nwgpu-hal = { path = \"../../components/nv-rs/vendor/wgpu-hal\" }\n[profile.release.package.engine-core]\ncodegen-units = 16\n",
        );
        assert_eq!(
            check_settings(&changed, &nv).len(),
            2,
            "changed profile lines must fail"
        );
        let wrong_path = table(THEIRS);
        assert!(
            !check_settings(&wrong_path, &nv).is_empty(),
            "the path must point into nv-rs"
        );
        assert_eq!(
            check_settings(&ours, &table("")).len(),
            3,
            "no patch in nv-rs, no profile in nv-rs"
        );
    }

    #[test]
    fn the_toolchain_channel_must_equal_the_engines() {
        let a = table("[toolchain]\nchannel = \"1\"\n");
        let b = table("[toolchain]\nchannel = \"2\"\n");
        assert!(check_toolchain(&a, &a).is_empty());
        assert_eq!(check_toolchain(&a, &b).len(), 1);
    }

    #[test]
    fn the_repository_is_found_from_inside_it() {
        let top = top_level().unwrap();
        assert!(top.join(".git").exists());
        assert!(top.join("tools/check-component-locks/Cargo.toml").is_file());
    }

    #[test]
    fn the_allow_list_drops_comments_and_blank_lines() {
        let text = "# why\n\ntokio  1.54.1   1.53.1  # reason\n";
        assert_eq!(accepted(text), set(&["tokio 1.54.1 1.53.1"]));
    }

    #[test]
    fn paths_are_normalised_without_the_file_system() {
        assert_eq!(
            normal(Path::new("../../components/nv-rs/../x")),
            PathBuf::from("../../components/x")
        );
    }
}
