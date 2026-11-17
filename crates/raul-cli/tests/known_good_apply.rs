// SPDX-License-Identifier: GPL-3.0-only
//! `raul known-good apply` on a synthetic repository: submodule-like folders that are small git repositories
//! with tags, a `known-good.toml` and a workspace `Cargo.toml`.
#![allow(clippy::disallowed_methods)]

use std::path::Path;
use std::process::{Command, Output};

const SET: &str = r#"schema_version = 1

[[set]]
id = "2026-11-a"
status = "known-good"

[set.components.nv-rs]
version = "v0.4.0"

[set.components.nvmod]
version = "v0.6.0"

[set.components.loot-rs]
version = "v0.5.0"

[set.components.lodwright-rs]
version = "v0.3.0"

[set.components.ttwright-rs]
version = "v0.3.0"
"#;

const CARGO: &str = "[workspace.dependencies]\nloot-rs = { git = \"https://github.com/JorisJonkers-dev/loot-rs\", tag = \"v0.5.0\" }\n";

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git runs");
    assert!(out.status.success(), "{args:?}: {out:?}");
}

/// A folder that is a repository with its own `origin` and a tag per release in `tags`; `file` is written at each.
fn component(root: &Path, name: &str, tags: &[(&str, Option<&str>)]) {
    let dir = root.join("components").join(name);
    std::fs::create_dir_all(&dir).unwrap();
    git(&dir, &["init", "--quiet", "-b", "main"]);
    for (tag, lock) in tags {
        std::fs::write(dir.join("marker"), tag).unwrap();
        if let Some(lock) = lock {
            std::fs::write(dir.join("tools.lock.toml"), lock).unwrap();
        }
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "--quiet", "-m", tag]);
        git(&dir, &["tag", tag]);
    }
    let here = dir.to_string_lossy().into_owned();
    git(&dir, &["remote", "add", "origin", &here]);
    git(&dir, &["checkout", "--quiet", "--detach", tags[0].0]);
}

fn lock(loot: &str) -> String {
    format!("schema_version = 1\n\n[[tool]]\nname = \"loot-rs\"\nversion = \"{loot}\"\n")
}

fn repo() -> nvmod_testkit::TempDir {
    let t = nvmod_testkit::TempDir::new("raul-known-good-apply");
    let root = t.path();
    std::fs::write(root.join("known-good.toml"), SET).unwrap();
    std::fs::write(root.join("Cargo.toml"), CARGO).unwrap();
    component(root, "nv-rs", &[("v0.4.0", None), ("v0.5.0", None)]);
    component(
        root,
        "nvmod-rs",
        &[
            ("v0.6.0", Some(&lock("0.5.0"))),
            ("v0.6.2", Some(&lock("0.5.0"))),
            ("v0.7.0", Some(&lock("0.6.0"))),
        ],
    );
    component(root, "loot-rs", &[("v0.5.0", None), ("v0.6.0", None)]);
    component(root, "lodwright-rs", &[("v0.3.0", None)]);
    component(root, "ttwright-rs", &[("v0.3.0", None)]);
    t
}

fn apply(root: &Path, component: &str, version: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_raul"))
        .args([
            "known-good",
            "apply",
            component,
            version,
            "--month",
            "2026-11",
        ])
        .arg("--repo")
        .arg(root)
        .output()
        .unwrap()
}

fn head_tag(root: &Path, name: &str) -> String {
    let out = Command::new("git")
        .args(["tag", "--points-at", "HEAD"])
        .current_dir(root.join("components").join(name))
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn a_component_moves_its_submodule_and_adds_a_set() {
    let t = repo();
    let out = apply(t.path(), "nv-rs", "v0.5.0");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(head_tag(t.path(), "nv-rs"), "v0.5.0");
    let text = std::fs::read_to_string(t.path().join("known-good.toml")).unwrap();
    assert!(text.contains("id = \"2026-11-b\""), "{text}");
    assert!(text.contains("# nv-rs v0.4.0 -> v0.5.0"), "{text}");
    assert_eq!(
        std::fs::read_to_string(t.path().join("Cargo.toml")).unwrap(),
        CARGO
    );
}

#[test]
fn nvmod_takes_the_loot_rs_it_links_with_it() {
    let t = repo();
    let out = apply(t.path(), "nvmod", "v0.7.0");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(head_tag(t.path(), "nvmod-rs"), "v0.7.0");
    assert_eq!(head_tag(t.path(), "loot-rs"), "v0.6.0");
    let text = std::fs::read_to_string(t.path().join("known-good.toml")).unwrap();
    assert_eq!(text.matches("[[set]]").count(), 2, "{text}");
    assert!(
        text.contains("nvmod v0.6.0 -> v0.7.0, loot-rs v0.5.0 -> v0.6.0"),
        "{text}"
    );
    let cargo = std::fs::read_to_string(t.path().join("Cargo.toml")).unwrap();
    assert!(cargo.contains("tag = \"v0.6.0\""), "{cargo}");
}

#[test]
fn nvmod_with_the_same_loot_rs_moves_only_itself() {
    let t = repo();
    let out = apply(t.path(), "nvmod", "v0.6.2");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(head_tag(t.path(), "loot-rs"), "v0.5.0");
    let text = std::fs::read_to_string(t.path().join("known-good.toml")).unwrap();
    assert!(text.contains("# nvmod v0.6.0 -> v0.6.2\n"), "{text}");
}

#[test]
fn a_tag_that_does_not_exist_or_is_not_a_tag_name_changes_nothing() {
    let t = repo();
    for bad in ["v9.9.9", "main", "v1.0.0; rm"] {
        let out = apply(t.path(), "nv-rs", bad);
        assert!(!out.status.success(), "{bad}: {out:?}");
    }
    assert_eq!(head_tag(t.path(), "nv-rs"), "v0.4.0");
    assert_eq!(
        std::fs::read_to_string(t.path().join("known-good.toml")).unwrap(),
        SET
    );
    assert!(!apply(t.path(), "xnvse", "v1.0.0").status.success());
}

#[test]
fn the_month_defaults_to_this_month() {
    let t = repo();
    let out = Command::new(env!("CARGO_BIN_EXE_raul"))
        .args(["known-good", "apply", "nv-rs", "v0.5.0", "--repo"])
        .arg(t.path())
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let text = std::fs::read_to_string(t.path().join("known-good.toml")).unwrap();
    let id = text
        .lines()
        .filter_map(|l| l.strip_prefix("id = \""))
        .next_back()
        .unwrap();
    let (month, letter) = id.trim_end_matches('"').rsplit_once('-').unwrap();
    assert_eq!(letter, "b");
    let (year, mon) = month.split_once('-').unwrap();
    assert!(
        year.len() == 4 && year.chars().all(|c| c.is_ascii_digit()),
        "{id}"
    );
    assert!(matches!(mon.parse::<u32>(), Ok(1..=12)), "{id}");
}
