// SPDX-License-Identifier: GPL-3.0-only
//! The plan of a synthetic list against a synthetic set of present Mods.

use raul_modlist::ModList;
use raul_modlist::plan::{Acquire, FileResolver, NoResolver, State, nexus_page, plan};
use raul_testkit::{covers, ids};

const LIST: &str = r#"
schema = "raul-modlist/1"
id = "tiny"
name = "Tiny"
game = "fnv"
target = "ttw"
plugins = "loot"

[[mods]]
id = "base"
name = "Base"
source = { type = "tool", tool = "ttwright-rs", inputs = ["fallout3"] }
[[mods]]
id = "known"
name = "Known File"
source = { type = "nexus", game = "newvegas", mod = 100, file = 7, file_name = "Known.7z", size = 42 }
[[mods]]
id = "epoch"
name = "Epoch Only"
source = { type = "nexus", game = "newvegas", mod = 200, file_name = "Epoch-200.zip", uploaded = 1700000000 }
[[mods]]
id = "moddb"
name = "Moddb Thing"
source = { type = "manual", url = "https://moddb.example/thing", file_name = "thing.7z", note = "pick the 7z" }
[[mods]]
id = "lost"
name = "Lost Cut"
source = { type = "manual", note = "no archive recorded" }
[[mods]]
id = "off"
name = "Switched Off"
enabled = false
why_off = "replaced"
source = { type = "manual" }
[[mods]]
id = "glow"
name = "Glow"
source = { type = "addon", id = "small-lamp-glow" }
[[mods]]
id = "lod"
name = "LOD"
source = { type = "generated", tool = "lodwright-rs" }
"#;

fn list() -> ModList {
    ModList::parse(LIST).unwrap()
}

fn names(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn present_missing_and_switched_off_mods_are_told_apart() {
    covers(&[ids::mods::modlist::PLAN_PRESENT_MISSING]);
    let p = plan(&list(), &names(&["known file", "Base"]), &NoResolver);
    let state = |id: &str| p.items.iter().find(|i| i.id == id).unwrap().state;
    assert_eq!(state("base"), State::Present);
    assert_eq!(
        state("known"),
        State::Present,
        "names match without regard to case"
    );
    assert_eq!(state("epoch"), State::Missing);
    assert_eq!(state("off"), State::Skipped);
    let c = p.counts();
    assert_eq!((c.present, c.missing, c.skipped), (2, 5, 1));
}

#[test]
fn missing_mods_keep_the_lists_install_order() {
    let p = plan(&list(), &[], &NoResolver);
    let order: Vec<&str> = p.missing().map(|i| i.id.as_str()).collect();
    assert_eq!(
        order,
        ["base", "known", "epoch", "moddb", "lost", "glow", "lod"]
    );
    assert_eq!(p.items[0].position, 1);
}

#[test]
fn a_nexus_file_with_an_id_links_the_file_and_one_without_links_the_files_tab() {
    covers(&[ids::mods::modlist::PLAN_LINKS]);
    let p = plan(&list(), &[], &NoResolver);
    let acquire = |id: &str| {
        p.items
            .iter()
            .find(|i| i.id == id)
            .unwrap()
            .acquire
            .clone()
            .unwrap()
    };
    assert_eq!(
        acquire("known"),
        Acquire::Nexus {
            page: "https://www.nexusmods.com/newvegas/mods/100?tab=files&file_id=7".into(),
            file_id: Some(7),
            file_name: Some("Known.7z".into()),
            size: Some(42),
        }
    );
    match acquire("epoch") {
        Acquire::Nexus {
            page,
            file_id: None,
            ..
        } => {
            assert_eq!(
                page,
                "https://www.nexusmods.com/newvegas/mods/200?tab=files"
            );
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn other_links_and_hand_work_are_told_plainly() {
    covers(&[ids::mods::modlist::PLAN_LINKS]);
    let p = plan(&list(), &[], &NoResolver);
    let acquire = |id: &str| {
        p.items
            .iter()
            .find(|i| i.id == id)
            .unwrap()
            .acquire
            .clone()
            .unwrap()
    };
    assert_eq!(
        acquire("moddb"),
        Acquire::Link {
            url: "https://moddb.example/thing".into(),
            file_name: Some("thing.7z".into()),
            note: Some("pick the 7z".into()),
        }
    );
    assert_eq!(
        acquire("lost"),
        Acquire::Manual {
            file_name: None,
            note: Some("no archive recorded".into())
        }
    );
    assert_eq!(
        acquire("base"),
        Acquire::Tool {
            tool: "ttwright-rs".into()
        }
    );
    assert_eq!(
        acquire("glow"),
        Acquire::Addon {
            id: "small-lamp-glow".into()
        }
    );
    assert_eq!(
        acquire("lod"),
        Acquire::Generated {
            tool: "lodwright-rs".into()
        }
    );
}

struct Fixed(u64);

impl FileResolver for Fixed {
    fn nexus_file(
        &self,
        game: &str,
        mod_id: u64,
        uploaded: Option<u64>,
        _: Option<&str>,
    ) -> Option<u64> {
        assert_eq!(
            (game, mod_id, uploaded),
            ("newvegas", 200, Some(1_700_000_000))
        );
        Some(self.0)
    }
}

#[test]
fn a_resolver_supplies_the_file_id_the_list_lacks_and_is_not_asked_when_the_list_has_it() {
    covers(&[ids::mods::modlist::PLAN_RESOLVER]);
    let p = plan(&list(), &[], &Fixed(9));
    let page = |id: &str| match p
        .items
        .iter()
        .find(|i| i.id == id)
        .unwrap()
        .acquire
        .clone()
        .unwrap()
    {
        Acquire::Nexus { page, .. } => page,
        other => panic!("{other:?}"),
    };
    assert_eq!(page("epoch"), nexus_page("newvegas", 200, Some(9)));
    assert_eq!(
        page("known"),
        nexus_page("newvegas", 100, Some(7)),
        "the Fixed resolver asserts it is never asked about mod 100"
    );
}

#[test]
fn a_mod_that_is_present_or_off_needs_nothing() {
    let p = plan(&list(), &names(&["Base"]), &NoResolver);
    assert!(
        p.items
            .iter()
            .filter(|i| i.state != State::Missing)
            .all(|i| i.acquire.is_none())
    );
}
