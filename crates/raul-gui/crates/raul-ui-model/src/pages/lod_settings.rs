// SPDX-License-Identifier: GPL-3.0-only
//! The toolkit-free state of the LOD settings view: the options of a Profile's `raul-lod.toml` as
//! `raul lod settings show` lists them, one field per option, and the changes the view makes: one option at a time
//! when a field is left (UI-RULES T6), Restore defaults and Undo restore (T4, T5).
//!
//! Every change is one `raul lod settings` command; the options are read again after it, so the view always shows
//! what the file holds. The view never builds a value the command line could not take.

use super::settings::{Command, Fetch, Save};
use serde::Deserialize;
use serde_json::Value;

/// `raul lod settings show --instance <dir>`.
pub fn show_command(instance: &str) -> Command {
    Command::read(
        "lod-settings-show",
        3,
        &["lod", "settings", "show", "--instance", instance],
    )
}

/// `raul lod settings set <key> <value>... --instance <dir>`.
pub fn set_command(instance: &str, key: &str, values: &[String]) -> Command {
    let mut words = vec!["lod", "settings", "set", key];
    words.extend(values.iter().map(String::as_str));
    words.extend(["--instance", instance]);
    Command::write("lod-settings-set", 3, &words)
}

/// `raul lod settings unset <key> --instance <dir>`.
pub fn unset_command(instance: &str, key: &str) -> Command {
    Command::write(
        "lod-settings-unset",
        3,
        &["lod", "settings", "unset", key, "--instance", instance],
    )
}

/// `raul lod settings reset --instance <dir>`: Restore defaults.
pub fn reset_command(instance: &str) -> Command {
    Command::write(
        "lod-settings-reset",
        3,
        &["lod", "settings", "reset", "--instance", instance],
    )
}

/// `raul lod settings undo --instance <dir>`: Undo restore.
pub fn undo_command(instance: &str) -> Command {
    Command::write(
        "lod-settings-undo",
        3,
        &["lod", "settings", "undo", "--instance", instance],
    )
}

/// What kind of value an option holds (`kind` of `raul-lod-settings`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// On or off.
    Switch,
    /// A whole number.
    Count,
    /// A number that may have a fraction.
    Number,
    /// A texture edge in pixels.
    TextureSize,
    /// A list of names or patterns.
    Names,
    /// A list of record ids.
    Records,
}

/// One option as the command lists it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct OptionRow {
    /// Its key.
    pub key: String,
    /// 1: shown at once; 2: advanced.
    pub tier: u8,
    /// What kind of value it holds.
    pub kind: Kind,
    /// The value it takes.
    pub value: Value,
    /// The value it takes when the file does not set it.
    pub default: Value,
    /// The file sets it.
    pub overridden: bool,
    /// Restore defaults removes it.
    pub restores: bool,
}

/// A warning the command gave (`warnings` of `raul-lod-settings`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Warning {
    /// The stable code: `unknown_world`, `worlds_unchecked`, `plugin_not_active` or `no_object_left`.
    pub code: String,
    /// The option it is about.
    pub key: String,
    /// The value it is about.
    #[serde(default)]
    pub value: Option<String>,
    /// What it means.
    pub what: String,
}

/// A refusal (`error` of `raul-lod-settings`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Refusal {
    /// The stable code.
    pub code: String,
    /// The option it is about.
    #[serde(default)]
    pub key: Option<String>,
    /// What went wrong.
    pub what: String,
    /// A further reason.
    #[serde(default)]
    pub reason: Option<String>,
}

impl Refusal {
    /// What went wrong and why, in the command's words, for the detail line.
    pub fn said(&self) -> String {
        match &self.reason {
            Some(r) => format!("{} {r}", self.what),
            None => self.what.clone(),
        }
    }
}

/// What a change did (`change` of `raul-lod-settings`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Change {
    /// The keys it changed or would remove.
    pub keys: Vec<String>,
    /// Whether the file changed.
    pub changed: bool,
}

/// A `raul-lod-settings` document.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Doc {
    /// Every option with its value.
    #[serde(default)]
    pub options: Vec<OptionRow>,
    /// The restore point Undo goes back to.
    #[serde(default)]
    pub restore_point: Option<String>,
    /// What the settings allow but that builds less than asked.
    #[serde(default)]
    pub warnings: Vec<Warning>,
    /// A run of the Profile is going: changes apply to the next run.
    #[serde(default)]
    pub run_in_progress: bool,
    /// What the command changed.
    #[serde(default)]
    pub change: Option<Change>,
    /// Set when the command was refused.
    #[serde(default)]
    pub error: Option<Refusal>,
}

/// Reads the document a `raul lod settings ... --json` printed, also when it was refused (exit code 2).
///
/// # Errors
/// When `stdout` is not such a document.
pub fn parse(stdout: &str) -> Result<Doc, String> {
    #[derive(Deserialize)]
    struct Head {
        contract: String,
    }
    let head: Head = serde_json::from_str(stdout.trim())
        .map_err(|e| format!("not a raul-lod-settings document: {e}"))?;
    if head.contract != "raul-lod-settings" {
        return Err(format!("expected raul-lod-settings, got {}", head.contract));
    }
    serde_json::from_str(stdout.trim())
        .map_err(|e| format!("not a raul-lod-settings document: {e}"))
}

/// A value as a field shows it: a list as names separated by commas, nothing for "decided at run time".
pub fn text_of(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Array(items) => items
            .iter()
            .map(|i| i.as_str().map_or_else(|| i.to_string(), str::to_owned))
            .collect::<Vec<_>>()
            .join(", "),
        other => other.to_string(),
    }
}

/// One option in the view: what the file gives it and what is typed.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    /// The option.
    pub row: OptionRow,
    /// What the field holds now.
    pub draft: String,
    /// Its save.
    pub save: Save,
}

impl Field {
    /// The field of `row`, holding its value.
    pub fn of(row: OptionRow) -> Field {
        Field {
            draft: text_of(&row.value),
            row,
            save: Save::Idle,
        }
    }

    /// Whether the field holds something other than the option's value.
    pub fn changed(&self) -> bool {
        self.draft.trim() != text_of(&self.row.value)
    }

    /// The command that saves the draft: `unset` for an empty field (the option takes its default again), else
    /// `set` with the value, or with each name of a list.
    pub fn command(&self, instance: &str) -> Command {
        let draft = self.draft.trim();
        let key = self.row.key.as_str();
        if draft.is_empty() && !matches!(self.row.kind, Kind::Names | Kind::Records) {
            return unset_command(instance, key);
        }
        let values: Vec<String> = match self.row.kind {
            Kind::Names | Kind::Records => draft
                .split(',')
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_owned)
                .collect(),
            _ => vec![draft.to_owned()],
        };
        set_command(instance, key, &values)
    }
}

/// What the view is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Busy {
    /// Nothing.
    #[default]
    Idle,
    /// Restore defaults runs.
    Restoring,
    /// Undo restore runs.
    Undoing,
}

/// The state of the LOD settings view.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct State {
    /// The Instance the options were read from.
    pub instance: Option<String>,
    /// The options as last read.
    pub doc: Fetch<Doc>,
    /// One field per option, in the order of the command.
    pub fields: Vec<Field>,
    /// The advanced settings are shown; closed on every visit.
    pub advanced_open: bool,
    /// The person said once that they understand the advanced settings; kept in the app's own preferences.
    pub understood: bool,
    /// A bulk change runs.
    pub busy: Busy,
    /// Why the last Restore defaults or Undo restore did not work.
    pub problem: Option<String>,
}

impl State {
    /// The view after `doc` was read.
    pub fn loaded(instance: &str, doc: Doc) -> State {
        let mut s = State {
            instance: Some(instance.to_owned()),
            ..State::default()
        };
        s.take(doc);
        s
    }

    /// Takes a new reading of the options; drafts and saves start over.
    pub fn take(&mut self, doc: Doc) {
        self.fields = doc.options.iter().cloned().map(Field::of).collect();
        self.doc = Fetch::Ready(doc);
    }

    /// The field of `key`.
    pub fn field_mut(&mut self, key: &str) -> Option<&mut Field> {
        self.fields.iter_mut().find(|f| f.row.key == key)
    }

    /// The keys Restore defaults would remove: every generation option the file sets.
    pub fn restorable(&self) -> Vec<String> {
        self.fields
            .iter()
            .filter(|f| f.row.overridden && f.row.restores)
            .map(|f| f.row.key.clone())
            .collect()
    }

    /// Whether Undo restore is offered: the last Restore defaults kept a restore point and nothing changed since.
    pub fn can_undo(&self) -> bool {
        self.doc.ready().is_some_and(|d| d.restore_point.is_some())
    }

    /// Whether anything is being saved or changed.
    pub fn saving(&self) -> bool {
        self.busy != Busy::Idle || self.fields.iter().any(|f| f.save == Save::Saving)
    }

    /// Whether a run of the Profile is going.
    pub fn run_in_progress(&self) -> bool {
        self.doc.ready().is_some_and(|d| d.run_in_progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn row(key: &str, kind: Kind, value: Value, overridden: bool) -> OptionRow {
        OptionRow {
            key: key.into(),
            tier: 1,
            kind,
            value,
            default: Value::Null,
            overridden,
            restores: key != "allow_in_repo",
        }
    }

    #[test]
    fn a_document_is_read_also_when_it_was_refused() {
        let doc = parse(
            r#"{"contract":"raul-lod-settings","options":[],"restore_point":null,"warnings":[],
                "error":{"code":"out_of_range","key":"threads","what":"A LOD setting is outside the values it takes.","detail":"Nothing changed.","reason":null}}"#,
        )
        .unwrap();
        assert_eq!(doc.error.unwrap().key.as_deref(), Some("threads"));
        assert!(parse(r#"{"contract":"raul-config"}"#).is_err());
        assert!(parse("not json").is_err());
    }

    #[test]
    fn a_field_saves_its_value_and_an_emptied_field_takes_the_default_again() {
        let mut f = Field::of(row("threads", Kind::Count, json!(2), true));
        assert!(!f.changed());
        "4".clone_into(&mut f.draft);
        assert_eq!(f.command("/i").rest(), ["threads", "4", "--instance", "/i"]);
        f.draft.clear();
        assert_eq!(f.command("/i").id, "lod-settings-unset");
        let mut list = Field::of(row("worlds", Kind::Names, json!(["A", "B"]), true));
        assert_eq!(list.draft, "A, B");
        "A, C ,".clone_into(&mut list.draft);
        assert_eq!(
            list.command("/i").rest(),
            ["worlds", "A", "C", "--instance", "/i"]
        );
        list.draft.clear();
        assert_eq!(
            list.command("/i").rest(),
            ["worlds", "--instance", "/i"],
            "an empty list is a value of its own"
        );
    }

    #[test]
    fn restore_defaults_names_only_generation_options_the_file_sets() {
        let doc = Doc {
            options: vec![
                row("threads", Kind::Count, json!(2), true),
                row("memory_mib", Kind::Count, Value::Null, false),
                row("allow_in_repo", Kind::Switch, json!(true), true),
            ],
            restore_point: None,
            warnings: Vec::new(),
            run_in_progress: false,
            change: None,
            error: None,
        };
        let s = State::loaded("/i", doc);
        assert_eq!(s.restorable(), ["threads"]);
        assert!(!s.can_undo());
    }
}
