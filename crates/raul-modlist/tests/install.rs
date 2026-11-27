// SPDX-License-Identifier: GPL-3.0-only
//! The install steps of a synthetic list against synthetic Instances and download folders.

use raul_modlist::ModList;
use raul_modlist::install::{Place, plan_install};

const LIST: &str = r#"
schema = "raul-modlist/1"
id = "tiny"
name = "Tiny"
game = "fnv"
target = "vanilla"
plugins = "loot"

[[mods]]
id = "a"
name = "A"
source = { type = "nexus", game = "newvegas", mod = 1, file = 1, file_name = "A.zip" }
[[mods]]
id = "b"
name = "B"
source = { type = "url", url = "https://x.example/b.7z", sha256 = "00", file_name = "B.7z" }
[[mods]]
id = "c"
name = "C"
enabled = false
why_off = "replaced"
source = { type = "manual", file_name = "C.zip" }
[[mods]]
id = "d"
name = "D"
source = { type = "manual", note = "find it" }
[[mods]]
id = "e"
name = "E"
enabled = false
why_off = "replaced"
source = { type = "manual", file_name = "E.zip" }
[[mods]]
id = "lod"
name = "LOD"
source = { type = "generated", tool = "lodwright-rs" }
[[mods]]
id = "f"
name = "F"
source = { type = "manual", file_name = "F.zip" }
"#;

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| (*x).to_owned()).collect()
}

fn list() -> ModList {
    ModList::parse(LIST).unwrap()
}

#[test]
fn present_archives_become_steps_in_list_order_and_the_rest_is_reported() {
    let p = plan_install(&list(), &[], &s(&["f.zip", "a.ZIP", "B.7z", "C.zip"]));
    let order: Vec<(&str, bool)> = p.steps.iter().map(|x| (x.id.as_str(), x.enabled)).collect();
    assert_eq!(
        order,
        [("a", true), ("b", true), ("c", false), ("f", true)]
    );
    assert_eq!(p.steps[0].archive, "a.ZIP", "the archive is named as it is on disk");
    assert_eq!(p.missing.len(), 1, "D has no archive named, E is off");
    assert_eq!(p.missing[0].id, "d");
    assert_eq!(p.skipped, 1);
    assert_eq!(p.elsewhere[0].id, "lod");
}

#[test]
fn each_mod_goes_above_the_earlier_ones_and_below_a_later_one_that_is_there() {
    // Nothing installed: every new Mod goes on top, so the last of the list ends up first.
    let p = plan_install(&list(), &[], &s(&["A.zip", "B.7z"]));
    assert!(p.steps.iter().all(|x| x.place == Place::Top));
    // F is already there: A and B go right under it, and B (later in the list) goes above A.
    let p = plan_install(&list(), &s(&["f"]), &s(&["A.zip", "B.7z"]));
    assert_eq!(p.present, 1);
    assert!(
        p.steps
            .iter()
            .all(|x| x.place == Place::After("F".to_owned()))
    );
}

#[test]
fn a_run_with_everything_present_has_nothing_to_do() {
    let p = plan_install(&list(), &s(&["a", "B", "c", "d", "e", "LOD", "f"]), &s(&["A.zip"]));
    assert!(p.steps.is_empty() && p.missing.is_empty());
    assert_eq!(p.present, 7);
}
