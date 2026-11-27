// SPDX-License-Identifier: GPL-3.0-only
//! Reading a Mod list: synthetic text only, no mod or game data.

use raul_modlist::{Error, ModList, Source};
use raul_testkit::{covers, ids};

const HEAD: &str = r#"
schema = "raul-modlist/1"
id = "tiny"
name = "Tiny"
game = "fnv"
target = "vanilla"
plugins = "loot"

[[groups]]
id = "fixes"
title = "Fixes"
"#;

fn with(mods: &str) -> String {
    format!("{HEAD}\n{mods}")
}

fn problems(text: &str) -> Vec<String> {
    match ModList::parse(text) {
        Err(Error::Invalid(p)) => p,
        other => panic!("expected broken rules, got {other:?}"),
    }
}

#[test]
fn a_list_is_read_with_its_mods_in_order() {
    covers(&[ids::mods::modlist::LIST_READ]);
    let list = ModList::parse(&with(
        r#"
[[mods]]
id = "first"
name = "First"
group = "fixes"
source = { type = "nexus", game = "newvegas", mod = 11, file = 22, file_name = "First.7z", size = 5 }

[[mods]]
id = "second"
name = "Second"
source = { type = "manual", note = "ask nicely" }
"#,
    ))
    .unwrap();
    let ids: Vec<&str> = list.mods.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, ["first", "second"]);
    assert!(list.mods[0].enabled);
    assert!(matches!(
        list.mods[0].source,
        Source::Nexus {
            mod_id: 11,
            file: Some(22),
            ..
        }
    ));
}

#[test]
fn a_nexus_file_may_be_known_only_by_its_upload_time() {
    let list = ModList::parse(&with(
        r#"
[[mods]]
id = "a"
name = "A"
source = { type = "nexus", game = "newvegas", mod = 5, uploaded = 1745010761 }
"#,
    ))
    .unwrap();
    assert!(matches!(
        list.mods[0].source,
        Source::Nexus {
            file: None,
            uploaded: Some(1_745_010_761),
            ..
        }
    ));
}

#[test]
fn every_source_type_is_read() {
    let list = ModList::parse(&with(
        r#"
[[mods]]
id = "u"
name = "U"
source = { type = "url", url = "https://example.org/u.7z", sha256 = "0000000000000000000000000000000000000000000000000000000000000000" }
[[mods]]
id = "t"
name = "T"
source = { type = "tool", tool = "ttwright-rs", inputs = ["fallout3"] }
[[mods]]
id = "g"
name = "G"
source = { type = "generated", tool = "lodwright-rs" }
[[mods]]
id = "d"
name = "D"
source = { type = "addon", id = "small-lamp-glow" }
"#,
    ))
    .unwrap();
    assert_eq!(list.mods.len(), 4);
}

#[test]
fn a_list_that_breaks_several_rules_names_each_one() {
    covers(&[ids::mods::modlist::BROKEN_LIST_REFUSED]);
    let broken = problems(&with(
        r#"
[[mods]]
id = "dup"
name = "One"
source = { type = "manual" }
[[mods]]
id = "dup"
name = "Two"
group = "nowhere"
source = { type = "manual" }
"#,
    ));
    assert!(
        broken.iter().any(|p| p.contains("`dup` is used twice")),
        "{broken:?}"
    );
    assert!(
        broken.iter().any(|p| p.contains("group `nowhere`")),
        "{broken:?}"
    );
}

#[test]
fn a_switched_off_mod_must_say_why() {
    let p = problems(&with(
        r#"
[[mods]]
id = "off"
name = "Off"
enabled = false
source = { type = "manual" }
"#,
    ));
    assert!(p[0].contains("does not say why"), "{p:?}");
}

#[test]
fn a_url_source_needs_https_and_a_sha256() {
    let p = problems(&with(
        r#"
[[mods]]
id = "u"
name = "U"
source = { type = "url", url = "http://example.org/u.7z", sha256 = "abc" }
"#,
    ));
    assert_eq!(p.len(), 2, "{p:?}");
}

#[test]
fn a_nexus_source_needs_a_real_mod_id_and_a_well_formed_md5() {
    let p = problems(&with(
        r#"
[[mods]]
id = "n"
name = "N"
source = { type = "nexus", game = "newvegas", mod = 0, file = 0, md5 = "xyz" }
"#,
    ));
    assert_eq!(p.len(), 3, "{p:?}");
}

#[test]
fn an_id_that_is_not_a_slug_is_refused() {
    let p = problems(&with(
        r#"
[[mods]]
id = "Not A Slug"
name = "N"
source = { type = "manual" }
"#,
    ));
    assert!(p[0].contains("not a slug"), "{p:?}");
}

#[test]
fn a_misspelt_field_is_an_error_not_an_omission() {
    let err = ModList::parse(&with(
        r#"
[[mods]]
id = "a"
name = "A"
sorce = { type = "manual" }
"#,
    ))
    .unwrap_err();
    assert!(matches!(err, Error::Syntax(_)), "{err:?}");
}

#[test]
fn text_that_is_not_toml_is_a_syntax_error() {
    assert!(matches!(
        ModList::parse("this is = = not toml"),
        Err(Error::Syntax(_))
    ));
}

#[test]
fn another_schema_version_is_refused_with_a_plain_line() {
    covers(&[ids::mods::modlist::OTHER_SCHEMA_REFUSED]);
    let err = ModList::parse(&HEAD.replace("raul-modlist/1", "raul-modlist/2")).unwrap_err();
    assert_eq!(
        err.to_string(),
        "the Mod list is `raul-modlist/2` and this raul reads `raul-modlist/1`; update raul or the list"
    );
}

#[test]
fn a_list_without_a_schema_line_is_refused() {
    let err = ModList::parse("id = \"x\"").unwrap_err();
    assert_eq!(
        err,
        Error::Schema("the Mod list has no `schema` line".into())
    );
}

#[test]
fn the_refusal_reads_as_a_list_of_the_broken_rules() {
    let err = ModList::parse(&with(
        r#"
[[mods]]
id = "a"
name = "A"
enabled = false
source = { type = "manual" }
"#,
    ))
    .unwrap_err();
    assert_eq!(
        err.to_string(),
        "the Mod list breaks 1 rule(s):\n  Mod `a` is switched off and does not say why (`why_off`)"
    );
}
