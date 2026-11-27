// SPDX-License-Identifier: GPL-3.0-only
//! The Beautiful list that raul carries (`lists/beautiful.toml`).

use raul_modlist::{BUNDLED_IDS, Source, bundled};
use raul_testkit::{covers, ids};

fn beautiful() -> raul_modlist::ModList {
    bundled("beautiful")
        .expect("raul carries Beautiful")
        .expect("Beautiful passes the rules")
}

#[test]
fn every_bundled_list_passes_the_rules() {
    covers(&[ids::mods::modlist::BEAUTIFUL_BUNDLED]);
    for id in BUNDLED_IDS {
        let list = bundled(id)
            .unwrap_or_else(|| panic!("{id} is listed but not carried"))
            .unwrap_or_else(|e| panic!("{id}: {e}"));
        assert_eq!(&list.id, id);
    }
}

#[test]
fn an_unknown_list_is_not_carried() {
    assert!(bundled("no-such-list").is_none());
}

#[test]
fn beautiful_holds_both_lucky_38_mods_at_their_nexus_files() {
    covers(&[ids::mods::modlist::BEAUTIFUL_BUNDLED]);
    let list = beautiful();
    let nexus = |id: &str| -> (u64, Option<u64>) {
        match &list
            .mods
            .iter()
            .find(|m| m.id == id)
            .unwrap_or_else(|| panic!("{id} missing"))
            .source
        {
            Source::Nexus { mod_id, file, .. } => (*mod_id, *file),
            other => panic!("{other:?}"),
        }
    };
    assert_eq!(
        nexus("the-mogul-mausoleum-a-lucky-38-overhaul"),
        (66_622, Some(1_000_059_826))
    );
    assert_eq!(
        nexus("lucky-38-exterior-remastered"),
        (81_005, Some(1_000_108_857))
    );
}

#[test]
fn beautiful_is_flat_and_the_generated_lod_comes_after_every_other_enabled_mod() {
    let list = beautiful();
    assert_eq!(list.mods.len(), 332);
    let last_enabled = list.mods.iter().rev().find(|m| m.enabled).unwrap();
    assert!(
        matches!(last_enabled.source, Source::Generated { .. }),
        "{}",
        last_enabled.id
    );
}

#[test]
fn the_lod_mods_the_generated_lod_replaces_are_switched_off_with_a_reason() {
    let list = beautiful();
    for id in ["terrain-lod", "object-lod"] {
        let m = list.mods.iter().find(|m| m.id == id).unwrap();
        assert!(!m.enabled && m.why_off.is_some(), "{id}");
    }
}

#[test]
fn the_nuclear_trees_are_moved_to_win_over_the_flora_project() {
    let list = beautiful();
    let pos = |id: &str| list.mods.iter().position(|m| m.id == id).unwrap();
    assert!(pos("nuclear-trees-vanilla") > pos("s3-mojave-flora-project"));
}
