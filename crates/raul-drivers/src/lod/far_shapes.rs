// SPDX-License-Identifier: GPL-3.0-only
//! Which models of a load order take their far shape from the far blocks the games ship. The list is data
//! (`data/far-shape-rules.json`); a rule applies only to a load order that has its world, so any other load order
//! keeps the request, and the request digest, it had without rules.

use std::path::Path;

use lodwright_contract::ShippedShapeRule;
use serde::Deserialize;

const RULES: &str = include_str!("../../data/far-shape-rules.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rule {
    world: String,
    models: Vec<String>,
}

fn known_rules() -> Vec<ShippedShapeRule> {
    let rules: Vec<Rule> = serde_json::from_str(RULES).expect("the far-shape rules are valid");
    rules
        .into_iter()
        .map(|r| ShippedShapeRule {
            world: r.world,
            models: r.models,
        })
        .collect()
}

fn named(names: &[String], world: &str) -> bool {
    names.iter().any(|n| n.eq_ignore_ascii_case(world))
}

/// The rules whose world is in `present` and, when `requested` names worlds, among them.
fn select(
    rules: Vec<ShippedShapeRule>,
    present: &[String],
    requested: &[String],
) -> Vec<ShippedShapeRule> {
    rules
        .into_iter()
        .filter(|r| {
            named(present, &r.world) && (requested.is_empty() || named(requested, &r.world))
        })
        .collect()
}

/// The rules for this load order. The plugins are read only when a rule could apply; a load order that cannot be read
/// gets no rules here, and the generation that follows reports why it cannot be read.
pub(super) fn rules_for_load_order(
    data_folder: &Path,
    plugins: &[String],
    requested: &[String],
) -> Vec<ShippedShapeRule> {
    let rules = known_rules();
    if !rules
        .iter()
        .any(|r| requested.is_empty() || named(requested, &r.world))
    {
        return Vec::new();
    }
    let Ok(worlds) = lodwright_api::load::load_worlds(data_folder, plugins) else {
        return Vec::new();
    };
    let present: Vec<String> = worlds.iter().map(|w| w.name().to_owned()).collect();
    select(rules, &present, requested)
}

#[cfg(test)]
mod tests {
    use super::{known_rules, select};
    use raul_testkit::{covers, ids};

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn every_rule_names_a_world_and_models() {
        let rules = known_rules();
        assert_ne!(rules.len(), 0);
        assert!(
            rules
                .iter()
                .all(|r| !r.world.is_empty() && !r.models.is_empty())
        );
    }

    #[test]
    fn a_rule_applies_only_when_its_world_is_present_and_requested() {
        covers(&[ids::lod::settings::FAR_SHAPE_RULES]);
        let rule = |w: &str| lodwright_contract::ShippedShapeRule {
            world: w.to_owned(),
            models: names(&["a/b.nif"]),
        };
        assert_eq!(
            select(vec![rule("World")], &names(&["WORLD"]), &[]).len(),
            1
        );
        assert_eq!(
            select(vec![rule("World")], &names(&["Other"]), &[]).len(),
            0
        );
        assert_eq!(
            select(vec![rule("World")], &names(&["World"]), &names(&["Other"])).len(),
            0
        );
        assert_eq!(
            select(vec![rule("World")], &names(&["World"]), &names(&["world"])).len(),
            1
        );
    }
}
