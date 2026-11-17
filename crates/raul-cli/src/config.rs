// SPDX-License-Identifier: GPL-3.0-only
//! `raul config list|get|set|unset [--json]`: the paths the GUI needs (the game folders, the TTW package and the
//! storage folder). The GUI reads the file in process and writes it only through this command. With `--json` every
//! answer, also a refusal, is one `raul-config` v1 document on standard output (`contracts/README.md`).

use std::fmt::Write as _;
use std::io::Write;
use std::path::{Path, PathBuf};

use raul_core::config::{Config, ConfigDoc, ConfigError, Entry, ErrorCode, ErrorInfo, Key};

const USAGE_LINE: &str = "config takes `list`, `get <key>`, `set <key> <path>` or `unset <key>`, each with an optional --json";

/// What one command line asks for.
enum Action {
    List,
    Get(Key),
    Set(Key, PathBuf),
    Unset(Key),
}

/// The exit code of a failed command with this error code.
fn exit_code(code: ErrorCode) -> u8 {
    match code {
        ErrorCode::UnknownKey => crate::USAGE,
        ErrorCode::WriteFailed => crate::FAILED,
        ErrorCode::ReadOnly
        | ErrorCode::InvalidValue
        | ErrorCode::NewerSchema
        | ErrorCode::Unreadable
        | ErrorCode::NoConfigFolder => crate::REFUSED,
    }
}

pub(crate) fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let json = args.iter().any(|a| a == "--json");
    let words: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|a| *a != "--json")
        .collect();
    let file = raul_core::config::default_file();
    let key = |name: &str| {
        Key::parse(name).ok_or_else(|| ErrorInfo {
            code: ErrorCode::UnknownKey,
            key: Some(name.to_owned()),
            message: format!(
                "unknown key `{name}`; the keys are {}",
                Key::ALL.map(Key::name).join(", ")
            ),
        })
    };
    let action = match words.as_slice() {
        ["list"] => Ok(Action::List),
        ["get", name] => key(name).map(Action::Get),
        ["set", name, value] => key(name).map(|k| Action::Set(k, PathBuf::from(value))),
        ["unset", name] => key(name).map(Action::Unset),
        _ => return crate::usage(err, USAGE_LINE),
    };
    let result = action.and_then(|action| {
        let file = file.clone().ok_or_else(|| ErrorInfo {
            code: ErrorCode::NoConfigFolder,
            key: None,
            message: "no configuration folder: HOME, XDG_CONFIG_HOME or APPDATA is not set".into(),
        })?;
        perform(&action, &file).map_err(|e| e.info())
    });
    match result {
        Ok(entries) => {
            let text = if json {
                format!("{}\n", to_json(&ConfigDoc::new(file, entries.clone())))
            } else {
                plain(&words, &entries)
            };
            crate::write_or_fail(out, err, &text)
        }
        Err(info) => {
            let _ = writeln!(err, "raul config: {}", info.message);
            let code = exit_code(info.code);
            if json {
                let doc = ConfigDoc::failed(file, info);
                let _ = writeln!(out, "{}", to_json(&doc));
            }
            code
        }
    }
}

fn perform(action: &Action, file: &Path) -> Result<Vec<Entry>, ConfigError> {
    let mut config = Config::load(file)?;
    match action {
        Action::List => Ok(config.entries()),
        Action::Get(key) => Ok(vec![config.entry(*key)]),
        Action::Set(key, value) => {
            config.set(*key, value)?;
            config.save()?;
            Ok(vec![config.entry(*key)])
        }
        Action::Unset(key) => {
            config.unset(*key)?;
            config.save()?;
            Ok(vec![config.entry(*key)])
        }
    }
}

fn to_json(doc: &ConfigDoc) -> String {
    serde_json::to_string(doc).unwrap_or_default()
}

/// The text answer: `get` prints the bare value (an empty line when unset), the others `key = value` lines.
fn plain(words: &[&str], entries: &[Entry]) -> String {
    let mut text = String::new();
    if words.first() == Some(&"get") {
        for e in entries {
            let _ = writeln!(
                text,
                "{}",
                e.value
                    .as_deref()
                    .map_or_else(String::new, |v| v.display().to_string())
            );
        }
        return text;
    }
    for e in entries {
        let value = e
            .value
            .as_deref()
            .map_or_else(|| "(not set)".to_owned(), |v| v.display().to_string());
        let note = if e.read_only { "  (derived)" } else { "" };
        let _ = writeln!(text, "{} = {value}{note}", e.key.name());
    }
    text
}
