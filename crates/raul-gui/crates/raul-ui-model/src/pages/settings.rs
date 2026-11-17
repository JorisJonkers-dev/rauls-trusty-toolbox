// SPDX-License-Identifier: GPL-3.0-only
//! The toolkit-free state of Settings: the paths the Toolbox uses and nothing else (UI-RULES P3, P5).
//!
//! Settings is the home of the `raul-config` keys (`contracts/README.md`). It reads them with `raul config list`
//! and writes one key at a time with `raul config set` or `raul config unset` when a field is left (UI-RULES T6).
//! The other pages of the setup path read the same document, so the [`Command`] a page runs, the [`Fetch`] of a
//! read and the parsed [`Config`] live here, at the data's home.

use serde::Deserialize;

/// A `raul` command a page runs: the id of its entry in the GUI's command registry, its words after `raul`, and
/// whether it writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// The registry id (`config-set`, `install-nv-rs`, ...).
    pub id: &'static str,
    /// Every word after `raul`, without `--json` (the service adds it where the command takes it).
    pub words: Vec<String>,
    /// How many of the words name the command (`config set` is two); the rest are its values and flags.
    pub head: usize,
    /// Whether it writes; only one writing command runs at a time.
    pub writes: bool,
}

impl Command {
    /// A command that only reads: registry id, how many words name it, and the words.
    pub fn read(id: &'static str, head: usize, words: &[&str]) -> Command {
        Command {
            id,
            words: words.iter().map(|w| (*w).to_owned()).collect(),
            head,
            writes: false,
        }
    }

    /// A command that writes.
    pub fn write(id: &'static str, head: usize, words: &[&str]) -> Command {
        Command {
            writes: true,
            ..Command::read(id, head, words)
        }
    }

    /// The values and flags after the words that name the command.
    pub fn rest(&self) -> Vec<String> {
        self.words.iter().skip(self.head).cloned().collect()
    }

    /// The command as a person types it in a terminal (UI-RULES G6): `raul` and the words, each quoted when it
    /// holds a space or a character a shell reads.
    pub fn shown(&self) -> String {
        let mut line = String::from("raul");
        for word in &self.words {
            line.push(' ');
            line.push_str(&quote(word));
        }
        line
    }

    /// `raul config list`.
    pub fn config_list() -> Command {
        Command::read("config-list", 2, &["config", "list"])
    }

    /// `raul config set <key> <value>`, or `raul config unset <key>` when `value` is empty.
    pub fn config_save(key: Key, value: &str) -> Command {
        let value = value.trim();
        if value.is_empty() {
            Command::write("config-unset", 2, &["config", "unset", key.name()])
        } else {
            Command::write("config-set", 2, &["config", "set", key.name(), value])
        }
    }
}

/// `word` quoted for a POSIX shell when it needs it.
fn quote(word: &str) -> String {
    let plain = !word.is_empty()
        && word
            .chars()
            .all(|c| c.is_alphanumeric() || "-_./:=@,+%".contains(c));
    if plain {
        word.to_owned()
    } else {
        format!("'{}'", word.replace('\'', r"'\''"))
    }
}

/// A read a page waits for.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Fetch<T> {
    /// Not asked yet.
    #[default]
    Idle,
    /// Asked; the answer is on its way.
    Loading,
    /// The answer.
    Ready(T),
    /// It could not be read; the message of the command, for the detail line.
    Failed(String),
}

impl<T> Fetch<T> {
    /// The answer, once it is there.
    pub fn ready(&self) -> Option<&T> {
        match self {
            Fetch::Ready(v) => Some(v),
            _ => None,
        }
    }

    /// Whether the read has not been asked or is still on its way.
    pub fn is_pending(&self) -> bool {
        matches!(self, Fetch::Idle | Fetch::Loading)
    }
}

/// A key of `raul config` (`contracts/README.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Key {
    /// `game.newvegas`: the Fallout: New Vegas folder.
    NewVegas,
    /// `game.fallout3`: the Fallout 3 folder, only for TTW.
    Fallout3,
    /// `ttw.package`: the TTW installer package the player downloaded.
    TtwPackage,
    /// `storage.root`: the Toolbox storage folder, which holds the Instance.
    StorageRoot,
    /// `instance`: `<storage.root>/instance`, derived and read only.
    Instance,
}

impl Key {
    /// Every key, in the order `raul config list` prints them.
    pub const ALL: [Key; 5] = [
        Key::NewVegas,
        Key::Fallout3,
        Key::TtwPackage,
        Key::StorageRoot,
        Key::Instance,
    ];

    /// The keys a person sets.
    pub const EDITABLE: [Key; 4] = [
        Key::NewVegas,
        Key::Fallout3,
        Key::TtwPackage,
        Key::StorageRoot,
    ];

    /// Its name on the command line.
    pub fn name(self) -> &'static str {
        match self {
            Key::NewVegas => "game.newvegas",
            Key::Fallout3 => "game.fallout3",
            Key::TtwPackage => "ttw.package",
            Key::StorageRoot => "storage.root",
            Key::Instance => "instance",
        }
    }

    /// The key of a name.
    pub fn parse(name: &str) -> Option<Key> {
        Key::ALL.into_iter().find(|k| k.name() == name)
    }
}

/// The values of `raul config list`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Config {
    /// The configuration file.
    pub file: String,
    values: Vec<(Key, Option<String>)>,
}

impl Config {
    /// A configuration with these values; the keys not named are not set.
    pub fn with(file: &str, values: &[(Key, &str)]) -> Config {
        Config {
            file: file.to_owned(),
            values: values
                .iter()
                .map(|(k, v)| (*k, Some((*v).to_owned())))
                .collect(),
        }
    }

    /// The value of `key`, `None` when it is not set.
    pub fn get(&self, key: Key) -> Option<&str> {
        self.values
            .iter()
            .find(|(k, _)| *k == key)
            .and_then(|(_, v)| v.as_deref())
    }

    /// Sets `key` to `value` (`None` unsets it), as a successful `set` or `unset` did.
    pub fn put(&mut self, key: Key, value: Option<String>) {
        self.values.retain(|(k, _)| *k != key);
        self.values.push((key, value));
    }
}

/// Why `raul config` refused, from the document's `error`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// The stable code (`invalid_value`, `unreadable`, ...).
    pub code: String,
    /// The key it was about.
    pub key: Option<Key>,
    /// The message for people.
    pub message: String,
}

/// What a `raul config` command answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// The entries it printed: every key for `list`, the one key for `get`, `set` and `unset`.
    Entries(Config),
    /// A refusal; nothing was written.
    Refused(Refusal),
}

#[derive(Deserialize)]
struct Doc {
    contract: String,
    #[serde(default)]
    file: String,
    #[serde(default)]
    entries: Vec<DocEntry>,
    error: Option<DocError>,
}

#[derive(Deserialize)]
struct DocEntry {
    key: String,
    value: Option<String>,
}

#[derive(Deserialize)]
struct DocError {
    code: String,
    key: Option<String>,
    message: String,
}

/// Reads the `raul-config` v1 document a `raul config ... --json` printed.
///
/// # Errors
/// When `stdout` is not such a document; the text says what was wrong with it.
pub fn parse(stdout: &str) -> Result<Answer, String> {
    let doc: Doc = serde_json::from_str(stdout.trim())
        .map_err(|e| format!("not a raul-config document: {e}"))?;
    if doc.contract != "raul-config" {
        return Err(format!("expected raul-config, got {}", doc.contract));
    }
    if let Some(e) = doc.error {
        return Ok(Answer::Refused(Refusal {
            code: e.code,
            key: e.key.as_deref().and_then(Key::parse),
            message: e.message,
        }));
    }
    Ok(Answer::Entries(Config {
        file: doc.file,
        values: doc
            .entries
            .into_iter()
            .filter_map(|e| Key::parse(&e.key).map(|k| (k, e.value)))
            .collect(),
    }))
}

/// Where a field's save is.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Save {
    /// Nothing to save.
    #[default]
    Idle,
    /// `raul config set` runs.
    Saving,
    /// Refused: the value was not written; the message of `raul config`.
    Refused(String),
    /// The command could not run; the message of the service.
    Failed(String),
}

/// One path field: what is saved and what is typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// Its key.
    pub key: Key,
    /// The value saved in the configuration.
    pub saved: Option<String>,
    /// What the field holds now.
    pub draft: String,
    /// Its save.
    pub save: Save,
}

impl Field {
    /// A field showing the saved `value`.
    pub fn new(key: Key, value: Option<&str>) -> Field {
        Field {
            key,
            saved: value.map(str::to_owned),
            draft: value.unwrap_or_default().to_owned(),
            save: Save::Idle,
        }
    }

    /// A field whose typed text differs from what is saved, with that save's outcome.
    pub fn typed(key: Key, saved: Option<&str>, draft: &str, save: Save) -> Field {
        Field {
            draft: draft.to_owned(),
            save,
            ..Field::new(key, saved)
        }
    }

    /// Whether the typed text differs from what is saved.
    pub fn changed(&self) -> bool {
        self.draft.trim() != self.saved.as_deref().unwrap_or_default()
    }

    /// The command that saves the typed text.
    pub fn command(&self) -> Command {
        Command::config_save(self.key, &self.draft)
    }

    /// The command shown next to the field: the save it runs, or its shape while the field is empty.
    pub fn shown_command(&self) -> String {
        if self.draft.trim().is_empty() && self.saved.is_none() {
            format!("raul config set {} <path>", self.key.name())
        } else {
            self.command().shown()
        }
    }

    /// Takes in the answer of its save.
    pub fn saved_as(&mut self, answer: &Answer) {
        match answer {
            Answer::Entries(config) => {
                self.saved = config.get(self.key).map(str::to_owned);
                self.draft = self.saved.clone().unwrap_or_default();
                self.save = Save::Idle;
            }
            Answer::Refused(r) => self.save = Save::Refused(r.message.clone()),
        }
    }
}

/// The state of Settings.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct State {
    /// The configuration as last read.
    pub config: Fetch<Config>,
    /// The editable fields, once the configuration is read: in [`Key::EDITABLE`] order.
    pub fields: Vec<Field>,
}

impl State {
    /// Settings showing `config`.
    pub fn loaded(config: Config) -> State {
        let fields = Key::EDITABLE
            .into_iter()
            .map(|k| Field::new(k, config.get(k)))
            .collect();
        State {
            config: Fetch::Ready(config),
            fields,
        }
    }

    /// The field of `key`.
    pub fn field(&self, key: Key) -> Option<&Field> {
        self.fields.iter().find(|f| f.key == key)
    }

    /// The field of `key`, to change.
    pub fn field_mut(&mut self, key: Key) -> Option<&mut Field> {
        self.fields.iter_mut().find(|f| f.key == key)
    }

    /// Takes in the answer of a save of `key`: the field and the configuration both learn the new value.
    pub fn saved(&mut self, key: Key, answer: &Answer) {
        if let Some(f) = self.field_mut(key) {
            f.saved_as(answer);
        }
        if let (Answer::Entries(new), Fetch::Ready(config)) = (answer, &mut self.config) {
            config.put(key, new.get(key).map(str::to_owned));
            if key == Key::StorageRoot {
                // The Instance is derived from the storage folder; the next read shows it as raul has it.
                config.put(
                    Key::Instance,
                    new.get(Key::StorageRoot)
                        .map(|root| format!("{}/instance", root.trim_end_matches(['/', '\\']))),
                );
            }
        }
    }

    /// Whether a save is running.
    pub fn saving(&self) -> bool {
        self.fields.iter().any(|f| f.save == Save::Saving)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: &str = include_str!("../../../../../../contracts/fixtures/raul-config/list.json");
    const EMPTY: &str =
        include_str!("../../../../../../contracts/fixtures/raul-config/list-empty.json");
    const REFUSED: &str =
        include_str!("../../../../../../contracts/fixtures/raul-config/set-refused.json");

    #[test]
    fn a_list_is_read_with_every_key() {
        let Ok(Answer::Entries(c)) = parse(LIST) else {
            panic!("a list")
        };
        assert_eq!(
            c.get(Key::NewVegas),
            Some("/games/steamapps/common/Fallout New Vegas")
        );
        assert_eq!(c.get(Key::TtwPackage), None);
        assert_eq!(
            c.get(Key::Instance),
            Some("/users/player/raul-storage/instance")
        );
        assert_eq!(c.file, "/users/player/config/raul/config.toml");
        let Ok(Answer::Entries(empty)) = parse(EMPTY) else {
            panic!("a list")
        };
        assert!(Key::ALL.into_iter().all(|k| empty.get(k).is_none()));
    }

    #[test]
    fn a_refusal_keeps_its_code_key_and_message() {
        let Ok(Answer::Refused(r)) = parse(REFUSED) else {
            panic!("a refusal")
        };
        assert_eq!(r.code, "invalid_value");
        assert_eq!(r.key, Some(Key::NewVegas));
        assert!(r.message.contains("not a Fallout: New Vegas folder"));
    }

    #[test]
    fn anything_else_is_not_a_config_document() {
        assert!(parse("{}").is_err());
        assert!(parse("").is_err());
        assert!(parse(r#"{"contract":"raul-events","entries":[],"error":null}"#).is_err());
    }

    #[test]
    fn commands_show_as_typed_with_paths_quoted() {
        assert_eq!(Command::config_list().shown(), "raul config list");
        assert_eq!(
            Command::config_save(Key::NewVegas, "/games/Fallout New Vegas").shown(),
            "raul config set game.newvegas '/games/Fallout New Vegas'"
        );
        assert_eq!(
            Command::config_save(Key::Fallout3, "  ").shown(),
            "raul config unset game.fallout3"
        );
        assert_eq!(
            Command::config_save(Key::StorageRoot, "/a/it's").shown(),
            r"raul config set storage.root '/a/it'\''s'"
        );
        assert!(Command::config_save(Key::StorageRoot, "/x").writes);
        assert!(!Command::config_list().writes);
        assert!(Command::config_list().rest().is_empty());
        assert_eq!(
            Command::config_save(Key::NewVegas, "/nv").rest(),
            ["game.newvegas", "/nv"]
        );
        assert_eq!(Command::config_save(Key::NewVegas, "").id, "config-unset");
    }

    #[test]
    fn a_field_saves_only_a_change_and_learns_the_answer() {
        let mut f = Field::new(Key::NewVegas, Some("/games/nv"));
        assert!(!f.changed());
        f.draft = " /games/nv ".into();
        assert!(!f.changed(), "surrounding space is not a change");
        f.draft = "/games/other".into();
        assert!(f.changed());
        let refused = parse(REFUSED).unwrap();
        f.saved_as(&refused);
        assert!(matches!(f.save, Save::Refused(_)));
        assert_eq!(
            f.saved.as_deref(),
            Some("/games/nv"),
            "a refusal changes nothing"
        );
        f.saved_as(&Answer::Entries(Config::with(
            "",
            &[(Key::NewVegas, "/games/other")],
        )));
        assert_eq!(
            (f.saved.as_deref(), f.save.clone()),
            (Some("/games/other"), Save::Idle)
        );
        assert_eq!(
            Field::new(Key::TtwPackage, None).shown_command(),
            "raul config set ttw.package <path>"
        );
    }

    #[test]
    fn saving_the_storage_folder_moves_the_instance() {
        let mut s = State::loaded(Config::with("/c", &[(Key::StorageRoot, "/old")]));
        assert_eq!(s.fields.len(), 4);
        s.saved(
            Key::StorageRoot,
            &Answer::Entries(Config::with("/c", &[(Key::StorageRoot, "/new/")])),
        );
        let config = s.config.ready().unwrap();
        assert_eq!(config.get(Key::Instance), Some("/new/instance"));
        assert_eq!(
            s.field(Key::StorageRoot).unwrap().saved.as_deref(),
            Some("/new/")
        );
        assert!(!s.saving());
    }

    #[test]
    fn a_fetch_says_whether_it_waits() {
        assert!(Fetch::<()>::Idle.is_pending());
        assert!(Fetch::<()>::Loading.is_pending());
        assert!(!Fetch::Ready(()).is_pending());
        assert_eq!(Fetch::<u8>::Failed("x".into()).ready(), None);
    }
}
