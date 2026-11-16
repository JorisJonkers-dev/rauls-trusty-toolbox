// SPDX-License-Identifier: GPL-3.0-only
//! The Known-good set check: `known-good.toml` against the submodule pins and nvmod's `tools.lock.toml`.

use std::collections::BTreeMap;

use raul_core::known_good::{KnownGood, Pin};

const ONE_SET: &str = r#"
schema_version = 1

[[set]]
id = "2026-11-a"
status = "known-good"

[set.components.nv-rs]
version = "v0.4.0"
contracts = { nv-rs-release-manifest = 1, nv-rs-capabilities = 1 }

[set.components.nvmod]
version = "v0.5.0"

[set.components.loot-rs]
version = "v0.5.0"
contracts = { loot-sort = 1 }

[set.components.lodwright-rs]
version = "v0.2.0"
contracts = { lodwright-plan = 1 }

[set.components.ttwright-rs]
version = "unreleased"
"#;

fn pins() -> BTreeMap<String, Pin> {
    [
        ("nv-rs", Pin::Tag("v0.4.0".into())),
        ("nvmod", Pin::Tag("v0.5.0".into())),
        ("loot-rs", Pin::Tag("v0.5.0".into())),
        ("lodwright-rs", Pin::Tag("v0.2.0".into())),
        ("ttwright-rs", Pin::Untagged("bc8a951".into())),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect()
}

const LOCK_V050: &str = r#"
schema_version = 1
[[tool]]
name = "loot-rs"
version = "0.5.0"
kind = "linked"
"#;

#[test]
fn a_set_that_matches_the_pins_and_the_lock_has_no_problems() {
    let kg = KnownGood::parse(ONE_SET).unwrap();
    assert_eq!(kg.check(&pins(), LOCK_V050), Vec::<String>::new());
}

#[test]
fn a_set_that_leaves_out_a_component_is_refused() {
    let text = ONE_SET.replace(
        "[set.components.ttwright-rs]\nversion = \"unreleased\"\n",
        "",
    );
    let problems = KnownGood::parse(&text).unwrap_err();
    assert!(
        problems.iter().any(|p| p.contains("ttwright-rs")),
        "{problems:?}"
    );
}

#[test]
fn two_sets_with_the_same_id_are_refused() {
    let second = ONE_SET.split_once("[[set]]").unwrap().1;
    let text = format!("{ONE_SET}\n[[set]]{second}");
    let problems = KnownGood::parse(&text).unwrap_err();
    assert!(
        problems.iter().any(|p| p.contains("2026-11-a")),
        "{problems:?}"
    );
}

#[test]
fn a_status_other_than_known_good_or_candidate_is_refused() {
    let text = ONE_SET.replace("status = \"known-good\"", "status = \"tested\"");
    assert!(KnownGood::parse(&text).is_err());
}

#[test]
fn the_latest_known_good_set_skips_a_later_candidate() {
    let second = ONE_SET
        .split_once("[[set]]")
        .unwrap()
        .1
        .replace("2026-11-a", "2026-11-b")
        .replace("known-good", "candidate");
    let kg = KnownGood::parse(&format!("{ONE_SET}\n[[set]]{second}")).unwrap();
    assert_eq!(kg.latest_known_good().unwrap().id, "2026-11-a");
}

#[test]
fn a_submodule_on_another_tag_than_the_latest_set_is_reported() {
    let kg = KnownGood::parse(ONE_SET).unwrap();
    let mut pins = pins();
    pins.insert("nv-rs".into(), Pin::Tag("v0.3.0".into()));
    let problems = kg.check(&pins, LOCK_V050);
    assert!(
        problems
            .iter()
            .any(|p| p.contains("nv-rs") && p.contains("v0.3.0")),
        "{problems:?}"
    );
}

#[test]
fn an_untagged_submodule_is_reported_when_the_set_names_a_release() {
    let kg = KnownGood::parse(ONE_SET).unwrap();
    let mut pins = pins();
    pins.insert("lodwright-rs".into(), Pin::Untagged("8604089".into()));
    let problems = kg.check(&pins, LOCK_V050);
    assert!(
        problems.iter().any(|p| p.contains("lodwright-rs")),
        "{problems:?}"
    );
}

#[test]
fn a_loot_rs_version_other_than_nvmods_lock_is_reported() {
    let kg = KnownGood::parse(ONE_SET).unwrap();
    let problems = kg.check(&pins(), &LOCK_V050.replace("0.5.0", "0.4.0"));
    assert!(
        problems.iter().any(|p| p.contains("tools.lock.toml")),
        "{problems:?}"
    );
}
