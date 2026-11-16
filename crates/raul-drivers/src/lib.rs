// SPDX-License-Identifier: GPL-3.0-only
//! Drivers of the process Components (ADR-0007): the Toolbox runs ttwright and lodwright as programs through
//! `raul-runner`, reads their result and event Contracts, and reports to its own callers as `raul-events` lines.
//! The [`events`] and [`ttw`] modules hold the Toolbox's own Contracts of those commands; their schemas are in
//! `contracts/` at the repository root.

pub mod events;
pub mod lod;
pub mod run;
pub mod ttw;

/// A schema as pretty JSON with a final newline, the form the files in `contracts/` have. Every property of every
/// object is listed as required: the Toolbox always prints every field, with `null` for one that has no value.
///
/// # Panics
///
/// Never: a schema is a JSON value.
#[must_use]
pub fn schema_text(schema: &schemars::Schema) -> String {
    let mut value = schema.as_value().clone();
    require_every_property(&mut value);
    let mut text = serde_json::to_string_pretty(&value).expect("a schema is JSON");
    text.push('\n');
    text
}

fn require_every_property(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::Object(properties)) = map.get("properties") {
                let names: Vec<serde_json::Value> = properties
                    .keys()
                    .cloned()
                    .map(serde_json::Value::String)
                    .collect();
                map.insert("required".into(), serde_json::Value::Array(names));
            }
            map.values_mut().for_each(require_every_property);
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(require_every_property),
        _ => {}
    }
}
