// SPDX-License-Identifier: GPL-3.0-only
//! The Toolbox's own Contracts (ADR-0009): the JSON documents `raul` prints for the GUI and for scripts. Each one has
//! a schema in `contracts/<name>.v<N>.json` at the repository root, generated from its Rust type; a test keeps the
//! two equal and checks the fixtures in `contracts/fixtures/` against the schema.

/// One Contract the Toolbox produces.
#[derive(Debug, Clone, Copy)]
pub struct Own {
    /// Its name, for example `raul-config`.
    pub name: &'static str,
    /// The version this build prints.
    pub version: u32,
    /// Its JSON Schema, as the file in `contracts/` holds it.
    pub schema: fn() -> String,
}

impl Own {
    /// The schema's file name, for example `raul-config.v1.json`.
    #[must_use]
    pub fn file_name(&self) -> String {
        format!("{}.v{}.json", self.name, self.version)
    }
}

/// Every Contract the Toolbox produces.
pub const OWN: &[Own] = &[Own {
    name: crate::config::CONTRACT,
    version: crate::config::SCHEMA_VERSION,
    schema: crate::config::schema,
}];

/// A schema as pretty JSON with a final newline, the form the files in `contracts/` have. Every property of every
/// object is listed as required: the Toolbox always prints every field, with `null` for one that has no value.
pub(crate) fn schema_text(schema: &schemars::Schema) -> String {
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
