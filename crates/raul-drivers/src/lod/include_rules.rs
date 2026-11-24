// SPDX-License-Identifier: GPL-3.0-only
//! Which placed objects of a load order enter the far view although nothing marks them visible from far, and which of
//! them get a far shape made from their full model. The list is data (`data/include-rules.json`); a rule that names a
//! plugin applies only to a load order that has that plugin, so any other load order keeps the request, and the
//! request digest, it had without rules.

use lodwright_contract::IncludeRule;

const RULES: &str = include_str!("../../data/include-rules.json");

fn known_rules() -> Vec<IncludeRule> {
    serde_json::from_str(RULES).expect("the include rules are valid")
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

/// The include rules for this load order.
pub(super) fn rules_for_load_order(plugins: &[String]) -> Vec<IncludeRule> {
    select(known_rules(), plugins)
}

#[cfg(test)]
mod tests {
    use super::{known_rules, select};

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn every_rule_names_a_plugin_and_a_model_pattern_and_is_valid() {
        let rules = known_rules();
        assert_ne!(rules.len(), 0);
        for rule in &rules {
            assert!(
                rule.plugin.is_some(),
                "a rule without a plugin applies to every load order"
            );
            assert!(!rule.models.trim().is_empty());
        }
        let mut request = lodwright_contract::Request::new("/data", &["FalloutNV.esm"], "/out");
        request.inclusion.include_rules = rules;
        assert_eq!(request.validate(), Ok(()));
    }

    #[test]
    fn a_rule_applies_only_when_its_plugin_is_in_the_load_order() {
        let rule = |plugin: Option<&str>| lodwright_contract::IncludeRule {
            models: "lights/*".to_owned(),
            plugin: plugin.map(str::to_owned),
            record_type: None,
            generate: None,
        };
        let plugins = names(&["FalloutNV.esm", "Lamps.ESP"]);
        assert_eq!(select(vec![rule(Some("lamps.esp"))], &plugins).len(), 1);
        assert_eq!(select(vec![rule(Some("Other.esp"))], &plugins).len(), 0);
        assert_eq!(select(vec![rule(None)], &plugins).len(), 1);
        assert_eq!(select(vec![rule(Some("Lamps.esp"))], &[]).len(), 0);
    }
}
