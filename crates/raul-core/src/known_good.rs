// SPDX-License-Identifier: GPL-3.0-only
//! Known-good sets (`known-good.toml`): the chain of Component versions that passed the contract and end-to-end
//! tests together (ADR-0001, ADR-0005). The chain rule of nv-linux ADR-0007 holds: sets are listed oldest first, and
//! a release pins the latest set whose status is `known-good`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

/// Every Component a set must state, in display order.
pub const COMPONENTS: [&str; 5] = ["nv-rs", "nvmod", "loot-rs", "lodwright-rs", "ttwright-rs"];

/// The version a Component has before its repository has any release tag.
pub const UNRELEASED: &str = "unreleased";

/// The parsed `known-good.toml`.
#[derive(Debug, Clone, Serialize)]
pub struct KnownGood {
    /// The sets, oldest first.
    pub sets: Vec<Set>,
}

/// One Known-good set.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Set {
    /// Unique lowercase slug, such as `2026-11-a`.
    pub id: String,
    /// `known-good`, or `candidate` while a bump pull request is open.
    pub status: Status,
    /// The version and Contracts of each Component.
    pub components: BTreeMap<String, ComponentVersion>,
}

/// Whether a set has passed its tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    /// Passed the contract and end-to-end tests.
    KnownGood,
    /// Under test in a bump pull request; never on `main`.
    Candidate,
}

/// One Component's entry in a set.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentVersion {
    /// A release tag of the Component's repository, or `unreleased`.
    pub version: String,
    /// Contract name to schema version.
    #[serde(default)]
    pub contracts: BTreeMap<String, u32>,
}

/// What a submodule points at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pin {
    /// A commit that a release tag names.
    Tag(String),
    /// A commit with no tag (abbreviated SHA).
    Untagged(String),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    schema_version: u32,
    #[serde(default, rename = "set")]
    sets: Vec<Set>,
}

#[derive(Deserialize)]
struct Lock {
    #[serde(default, rename = "tool")]
    tools: Vec<LockTool>,
}

#[derive(Deserialize)]
struct LockTool {
    name: String,
    version: String,
}

fn is_slug(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn bare(version: &str) -> &str {
    version.strip_prefix('v').unwrap_or(version)
}

impl KnownGood {
    /// Parses `known-good.toml` and validates its shape.
    ///
    /// # Errors
    ///
    /// Every problem found, each naming the set and Component it is about.
    pub fn parse(text: &str) -> Result<KnownGood, Vec<String>> {
        let file: File = toml::from_str(text).map_err(|e| vec![format!("known-good.toml: {e}")])?;
        let mut problems = Vec::new();
        if file.schema_version != 1 {
            problems.push(format!("schema_version {} is not 1", file.schema_version));
        }
        let mut ids = BTreeSet::new();
        for set in &file.sets {
            if !is_slug(&set.id) {
                problems.push(format!("set {:?}: the id must be a lowercase slug", set.id));
            }
            if !ids.insert(set.id.as_str()) {
                problems.push(format!("set {}: the id is used twice", set.id));
            }
            for name in COMPONENTS {
                match set.components.get(name) {
                    None => problems.push(format!("set {}: {name} is not stated", set.id)),
                    Some(c) if c.version.is_empty() => {
                        problems.push(format!("set {}: {name} has an empty version", set.id));
                    }
                    Some(_) => {}
                }
            }
            for name in set.components.keys() {
                if !COMPONENTS.contains(&name.as_str()) {
                    problems.push(format!("set {}: {name} is not a Component", set.id));
                }
            }
        }
        if problems.is_empty() {
            Ok(KnownGood { sets: file.sets })
        } else {
            Err(problems)
        }
    }

    /// The set a release ships: the last one whose status is `known-good`.
    #[must_use]
    pub fn latest_known_good(&self) -> Option<&Set> {
        self.sets
            .iter()
            .rev()
            .find(|s| s.status == Status::KnownGood)
    }

    /// Checks the latest Known-good set against what the submodules point at, and against nvmod's
    /// `tools.lock.toml` (loot-rs is linked into nvmod at the version that lock names).
    #[must_use]
    pub fn check(&self, pins: &BTreeMap<String, Pin>, tools_lock: &str) -> Vec<String> {
        let Some(set) = self.latest_known_good() else {
            return vec!["known-good.toml has no known-good set".into()];
        };
        let mut problems = Vec::new();
        for (name, entry) in &set.components {
            match (pins.get(name), entry.version.as_str()) {
                (None, _) => problems.push(format!("{name}: no submodule found")),
                (Some(Pin::Untagged(_)), UNRELEASED) => {}
                (Some(Pin::Untagged(sha)), want) => problems.push(format!(
                    "{name}: the submodule is at untagged commit {sha}, set {} names {want}",
                    set.id
                )),
                (Some(Pin::Tag(tag)), want) if bare(tag) == bare(want) => {}
                (Some(Pin::Tag(tag)), want) => problems.push(format!(
                    "{name}: the submodule is at {tag}, set {} names {want}",
                    set.id
                )),
            }
        }
        match toml::from_str::<Lock>(tools_lock) {
            Err(e) => problems.push(format!("nvmod tools.lock.toml: {e}")),
            Ok(lock) => {
                let locked = lock
                    .tools
                    .iter()
                    .find(|t| t.name == "loot-rs")
                    .map(|t| t.version.as_str());
                let want = set.components.get("loot-rs").map(|c| c.version.as_str());
                match (want, locked) {
                    (Some(want), Some(locked)) if bare(want) != bare(locked) => {
                        problems.push(format!(
                            "loot-rs: set {} names {want}, nvmod's tools.lock.toml links {locked}",
                            set.id
                        ));
                    }
                    (_, None) => problems
                        .push("loot-rs: nvmod's tools.lock.toml has no loot-rs entry".into()),
                    _ => {}
                }
            }
        }
        problems
    }
}

/// The `known-good.toml` this build was made from.
pub const SHIPPED_TEXT: &str = include_str!("../../../known-good.toml");

/// The Known-good sets this build was made from.
///
/// # Panics
///
/// Never in a built binary: the `known_good` tests parse the same file.
#[must_use]
pub fn shipped() -> KnownGood {
    KnownGood::parse(SHIPPED_TEXT).expect("the shipped known-good.toml is valid")
}

/// The folder under `components/` that holds a Component.
#[must_use]
pub fn component_dir(name: &str) -> &str {
    match name {
        "nvmod" => "nvmod-rs",
        other => other,
    }
}

/// `text` with one more set appended that copies the latest Known-good set and moves only `component` to
/// `version`. The new set's id is the first free `<month>-<letter>` (`month` is `YYYY-MM`). It is written as
/// `known-good`: the bump pull request's required check is what makes that true before it can merge.
///
/// # Errors
///
/// When `text` is not a valid file, `component` is not a Component, it is already at `version`, or the month has
/// no free letter.
pub fn bump(text: &str, component: &str, version: &str, month: &str) -> Result<String, String> {
    if !COMPONENTS.contains(&component) {
        return Err(format!("{component} is not a Component"));
    }
    let kg = KnownGood::parse(text).map_err(|p| p.join("; "))?;
    let latest = kg
        .latest_known_good()
        .ok_or("known-good.toml has no known-good set")?;
    let current = &latest.components[component];
    if bare(&current.version) == bare(version) {
        return Err(format!(
            "{component} is already at {version} in set {}",
            latest.id
        ));
    }
    let id = ('a'..='z')
        .map(|c| format!("{month}-{c}"))
        .find(|id| kg.sets.iter().all(|s| &s.id != id))
        .ok_or_else(|| format!("no free set id left in {month}"))?;
    let mut out = text.trim_end().to_string();
    let _ = write!(
        out,
        "\n\n# {component} {} -> {version}\n[[set]]\nid = \"{id}\"\nstatus = \"known-good\"\n",
        current.version
    );
    for name in COMPONENTS {
        let mut entry = latest.components[name].clone();
        if name == component {
            entry.version = version.to_string();
        }
        let _ = write!(
            out,
            "\n[set.components.{name}]\nversion = \"{}\"\n",
            entry.version
        );
        if !entry.contracts.is_empty() {
            let contracts: Vec<String> = entry
                .contracts
                .iter()
                .map(|(k, v)| format!("{k} = {v}"))
                .collect();
            let _ = writeln!(out, "contracts = {{ {} }}", contracts.join(", "));
        }
    }
    Ok(out)
}
