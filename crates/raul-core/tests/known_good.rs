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

#[test]
fn a_bump_adds_a_set_that_moves_only_that_component() {
    let text = raul_core::known_good::bump(ONE_SET, "lodwright-rs", "v0.3.0", "2026-11").unwrap();
    let kg = KnownGood::parse(&text).unwrap();
    let latest = kg.latest_known_good().unwrap();
    assert_eq!(latest.id, "2026-11-b");
    assert_eq!(latest.components["lodwright-rs"].version, "v0.3.0");
    assert_eq!(latest.components["nv-rs"].version, "v0.4.0");
    assert_eq!(latest.components["loot-rs"].contracts["loot-sort"], 1);
}

#[test]
fn a_bump_to_the_version_already_pinned_is_refused() {
    assert!(raul_core::known_good::bump(ONE_SET, "nv-rs", "v0.4.0", "2026-11").is_err());
}

#[test]
fn a_bump_of_something_that_is_not_a_component_is_refused() {
    assert!(raul_core::known_good::bump(ONE_SET, "xnvse", "v1.0.0", "2026-11").is_err());
}

#[test]
fn a_bump_of_two_components_makes_one_set_that_moves_both() {
    let text = raul_core::known_good::bump_all(
        ONE_SET,
        &[("nvmod", "v0.6.2"), ("loot-rs", "v0.6.0")],
        "2026-11",
    )
    .unwrap();
    let kg = KnownGood::parse(&text).unwrap();
    assert_eq!(kg.sets.len(), 2);
    let latest = kg.latest_known_good().unwrap();
    assert_eq!(latest.components["nvmod"].version, "v0.6.2");
    assert_eq!(latest.components["loot-rs"].version, "v0.6.0");
    assert_eq!(latest.components["nv-rs"].version, "v0.4.0");
    assert!(text.contains("# nvmod "), "{text}");
}

#[test]
fn a_component_already_at_its_version_stays_when_another_one_moves() {
    let text = raul_core::known_good::bump_all(
        ONE_SET,
        &[("nv-rs", "v0.4.0"), ("lodwright-rs", "v0.3.0")],
        "2026-11",
    )
    .unwrap();
    let kg = KnownGood::parse(&text).unwrap();
    assert_eq!(
        kg.latest_known_good().unwrap().components["lodwright-rs"].version,
        "v0.3.0"
    );
    assert!(!text.contains("# nv-rs"), "{text}");
}

#[test]
fn a_bump_that_moves_nothing_or_names_a_stranger_is_refused() {
    use raul_core::known_good::bump_all;
    assert!(bump_all(ONE_SET, &[("nv-rs", "v0.4.0")], "2026-11").is_err());
    assert!(bump_all(ONE_SET, &[], "2026-11").is_err());
    assert!(
        bump_all(
            ONE_SET,
            &[("nv-rs", "v0.5.0"), ("xnvse", "v1.0.0")],
            "2026-11"
        )
        .is_err()
    );
}

#[test]
fn the_loot_tag_nvmod_links_is_read_from_its_lock() {
    use raul_core::known_good::linked_loot_tag;
    assert_eq!(linked_loot_tag(LOCK_V050).as_deref(), Some("v0.5.0"));
    assert_eq!(linked_loot_tag("not toml ["), None);
    assert_eq!(linked_loot_tag("schema_version = 1\n"), None);
}

#[test]
fn the_workspace_loot_tag_is_set_and_nothing_else_changes() {
    use raul_core::known_good::with_loot_tag;
    let toml = "[workspace.dependencies]\nserde = \"1\"\nloot-rs = { git = \"https://github.com/JorisJonkers-dev/loot-rs\", tag = \"v0.5.0\" }\n";
    let out = with_loot_tag(toml, "v0.6.0").unwrap();
    assert_eq!(out, toml.replace("v0.5.0", "v0.6.0"));
    assert_eq!(with_loot_tag(toml, "v0.5.0"), None, "already at that tag");
    assert_eq!(
        with_loot_tag("serde = \"1\"\n", "v0.6.0"),
        None,
        "no loot-rs"
    );
}

#[test]
fn a_loot_line_without_a_closing_quote_is_left_alone() {
    let toml =
        "loot-rs = { git = \"https://github.com/JorisJonkers-dev/loot-rs\", tag = \"v0.5.0 }\n";
    assert_eq!(raul_core::known_good::with_loot_tag(toml, "v0.6.0"), None);
}
