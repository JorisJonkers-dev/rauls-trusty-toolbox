// SPDX-License-Identifier: GPL-3.0-only
//! Which placed objects of a load order enter the far view although nothing marks them visible from far, and which of
//! them get a far shape made from their full model. The list is data (`data/include-rules.json`): rules without a
//! plugin name models of the base game (its street lamps and lamp posts) and apply to every load order; a rule that
//! names a plugin applies only to a load order that has that plugin, so a load order without it keeps the request,
//! and the request digest, it had without that rule.
//!
//! With far glow on, the rules of `data/glow-include-rules.json` come too: the light strips the Strip casinos show
//! towards the Mojave are models of their own (activators whose shapes are drawn without lighting), which only far
//! glow can draw; without it they would give no far shape.

use lodwright_contract::IncludeRule;

const RULES: &str = include_str!("../../data/include-rules.json");
const GLOW_RULES: &str = include_str!("../../data/glow-include-rules.json");

fn known_rules() -> Vec<IncludeRule> {
    serde_json::from_str(RULES).expect("the include rules are valid")
}

fn glow_rules() -> Vec<IncludeRule> {
    serde_json::from_str(GLOW_RULES).expect("the far glow include rules are valid")
}

/// The rules of `rules` whose plugin is in `plugins` (any case), and those that name no plugin.
fn select(rules: Vec<IncludeRule>, plugins: &[String]) -> Vec<IncludeRule> {
    rules
        .into_iter()
        .filter(|r| {
            r.plugin
                .as_deref()
                .is_none_or(|name| plugins.iter().any(|p| p.eq_ignore_ascii_case(name)))
        })
        .collect()
}

/// The include rules for this load order, with the far glow rules when `far_glow` is on, as the request carries them.
/// They go through their own serialisation, so a limit reads as written (`0.05`) and not as the nearest `f64` of its
/// `f32` value.
pub(super) fn rules_for_load_order(plugins: &[String], far_glow: bool) -> Vec<serde_json::Value> {
    let mut rules = known_rules();
    if far_glow {
        rules.extend(glow_rules());
    }
    select(rules, plugins)
        .iter()
        .map(|rule| {
            let text = serde_json::to_string(rule).expect("a rule serialises");
            serde_json::from_str(&text).expect("a rule reads back")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{glow_rules, known_rules, rules_for_load_order, select};
    use raul_testkit::{covers, ids};

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn every_rule_names_a_model_pattern_and_a_far_shape_and_is_valid() {
        let rules = known_rules();
        assert_ne!(rules.len(), 0);
        for rule in &rules {
            assert_ne!(rule.models.trim(), "");
            assert_eq!(rule.record_type.as_deref(), Some("STAT"), "{rule:?}");
            // The lamps have no LOD model: without a far shape of their own a rule would draw nothing.
            assert!(rule.generate.is_some(), "{rule:?}");
            if rule.plugin.is_none() {
                // A rule for every load order names models, never every model.
                assert!(!rule.models.starts_with('*'), "{rule:?}");
                assert!(
                    std::path::Path::new(&rule.models)
                        .extension()
                        .is_some_and(|e| e.eq_ignore_ascii_case("nif")),
                    "{rule:?}"
                );
            }
        }
        let mut request = lodwright_contract::Request::new("/data", &["FalloutNV.esm"], "/out");
        request.inclusion.include_rules = rules;
        assert_eq!(request.validate(), Ok(()));
    }

    #[test]
    fn the_far_glow_rules_name_the_strip_light_activators_and_come_only_with_far_glow() {
        let rules = glow_rules();
        assert_ne!(rules.len(), 0);
        for rule in &rules {
            assert_eq!(rule.record_type.as_deref(), Some("ACTI"), "{rule:?}");
            assert!(rule.generate.is_some(), "{rule:?}");
            assert!(rule.plugin.is_none(), "{rule:?}");
            assert!(!rule.models.starts_with('*'), "{rule:?}");
        }
        let mut request = lodwright_contract::Request::new("/data", &["FalloutNV.esm"], "/out");
        request.inclusion.include_rules = rules;
        assert_eq!(request.validate(), Ok(()));
        let plugins = names(&["FalloutNV.esm"]);
        let off = rules_for_load_order(&plugins, false);
        let on = rules_for_load_order(&plugins, true);
        assert_eq!(on.len(), off.len() + glow_rules().len());
        assert_eq!(on[..off.len()], off[..]);
        assert!(!off.iter().any(|r| r["record_type"] == "ACTI"));
    }

    #[test]
    fn the_base_game_lamp_posts_are_named_for_every_load_order() {
        let general = select(known_rules(), &names(&["FalloutNV.esm"]));
        assert!(general.iter().all(|r| r.plugin.is_none()));
        for model in [
            "architecture/strip/nvstripstreetlamp*.nif",
            "architecture/urban/streetdressing/streetlight0*.nif",
            "dungeons/caves/lamplight/lamplightlamp.nif",
        ] {
            assert!(general.iter().any(|r| r.models == model), "{model}");
        }
        let with_mod = select(
            known_rules(),
            &names(&["FalloutNV.esm", "Street Lights TTW.esm"]),
        );
        assert_eq!(with_mod.len(), general.len() + 1);
    }

    #[test]
    fn a_rule_applies_only_when_its_plugin_is_in_the_load_order() {
        covers(&[ids::lod::settings::INCLUDE_RULES]);
        let rule = |plugin: Option<&str>| lodwright_contract::IncludeRule {
            models: "lights/*".to_owned(),
            plugin: plugin.map(str::to_owned),
            record_type: None,
            generate: None,
            replace_lod_model: false,
        };
        let plugins = names(&["FalloutNV.esm", "Lamps.ESP"]);
        assert_eq!(select(vec![rule(Some("lamps.esp"))], &plugins).len(), 1);
        assert_eq!(select(vec![rule(Some("Other.esp"))], &plugins).len(), 0);
        assert_eq!(select(vec![rule(None)], &plugins).len(), 1);
        assert_eq!(select(vec![rule(Some("Lamps.esp"))], &[]).len(), 0);
    }
}
