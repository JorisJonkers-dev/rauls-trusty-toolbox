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

/// What a `config` command line ended with: the entries it answers, or the refusal.
type Answer = Result<Vec<Entry>, ErrorInfo>;

/// Runs the command `words` (without `--json`) against the config file of `ctx`. `None` is a usage error: the
/// words are not one of the four forms.
fn evaluate(ctx: &raul_core::context::Context, words: &[&str]) -> Option<Answer> {
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
    let action = match words {
        ["list"] => Ok(Action::List),
        ["get", name] => key(name).map(Action::Get),
        ["set", name, value] => key(name).map(|k| Action::Set(k, PathBuf::from(value))),
        ["unset", name] => key(name).map(Action::Unset),
        _ => return None,
    };
    Some(action.and_then(|action| {
        let file = ctx.config_file().ok_or_else(|| ErrorInfo {
            code: ErrorCode::NoConfigFolder,
            key: None,
            message: "no configuration folder: HOME, XDG_CONFIG_HOME or APPDATA is not set".into(),
        })?;
        perform(&action, &file).map_err(|e| e.info())
    }))
}

/// The `raul-config` document `raul config <words> --json` prints, and the exit code it ends with; `None` for a usage
/// error. The window's direct job answers with this.
#[must_use]
pub fn json_answer(
    ctx: &raul_core::context::Context,
    words: &[&str],
) -> Option<(serde_json::Value, u8)> {
    let file = ctx.config_file();
    let (doc, code) = match evaluate(ctx, words)? {
        Ok(entries) => (ConfigDoc::new(file, entries), crate::OK),
        Err(info) => {
            let code = exit_code(info.code);
            (ConfigDoc::failed(file, info), code)
        }
    };
    serde_json::to_value(&doc).ok().map(|json| (json, code))
}

pub(crate) fn run(
    ctx: &raul_core::context::Context,
    args: &[String],
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let json = args.iter().any(|a| a == "--json");
    let words: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|a| *a != "--json")
        .collect();
    let file = ctx.config_file();
    let Some(result) = evaluate(ctx, &words) else {
        return crate::usage(err, USAGE_LINE);
    };
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

#[cfg(test)]
mod tests {
    use super::*;

    fn context(label: &str) -> (raul_core::context::Context, PathBuf) {
        let dir = std::env::temp_dir().join(format!("raul-config-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        (
            raul_core::context::Context::new()
                .with_config_folder(&dir)
                .without_run_log(),
            dir,
        )
    }

    #[test]
    fn the_json_answer_lists_sets_gets_and_unsets_and_refuses_in_the_commands_words() {
        let (ctx, dir) = context("answer");
        let folder = dir.to_string_lossy().into_owned();
        let (list, code) = json_answer(&ctx, &["list"]).unwrap();
        assert_eq!(code, crate::OK);
        assert!(list["entries"].is_array() || list.is_object(), "{list}");
        let (set, code) = json_answer(&ctx, &["set", "storage.root", &folder]).unwrap();
        assert_eq!(code, crate::OK, "{set}");
        let (got, _) = json_answer(&ctx, &["get", "storage.root"]).unwrap();
        assert!(got.to_string().contains("storage.root"), "{got}");
        let (_, code) = json_answer(&ctx, &["unset", "storage.root"]).unwrap();
        assert_eq!(code, crate::OK);
        let (refused, code) = json_answer(&ctx, &["get", "no.such.key"]).unwrap();
        assert_eq!(code, crate::USAGE, "{refused}");
        assert!(json_answer(&ctx, &["set", "storage.root"]).is_none());
        assert!(json_answer(&ctx, &[]).is_none());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
