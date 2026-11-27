// SPDX-License-Identifier: GPL-3.0-only
//! `raul nexus key set|clear|status`: the Nexus Mods API key. The key is kept in the OS keyring (or, when the person
//! asks, in a file only they can read), is read from standard input and never from the command line, and is never
//! printed. `status` asks Nexus who the key belongs to (docs/adr/0024).

use std::io::{BufRead, IsTerminal, Write};

use raul_core::context::Context;
use raul_core::nexus::curl::CurlTransport;
use raul_core::nexus::key::{self, ENV_VAR, FileStore, KeyringStore};
use raul_core::nexus::redact::redact;
use raul_core::nexus::{
    Identity, KeySource, KeyStore, NexusApi, NexusClient, NexusError, Secret, Transport,
};

/// The version raul tells Nexus it is.
const VERSION: &str = env!("CARGO_PKG_VERSION");

const USAGE: &str = "nexus takes `key set [--file]`, `key clear` or `key status [--json]`";

/// Everything a command reads from outside, so a test can give it fakes.
pub(crate) struct Deps<'a> {
    pub env: Option<String>,
    pub keyring: &'a dyn KeyStore,
    pub transport: &'a dyn Transport,
    pub input: &'a mut dyn BufRead,
    pub interactive: bool,
}

pub(crate) fn run(ctx: &Context, args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let stdin = std::io::stdin();
    let interactive = stdin.is_terminal();
    let mut input = stdin.lock();
    let mut deps = Deps {
        env: std::env::var(ENV_VAR).ok(),
        keyring: &KeyringStore,
        transport: &CurlTransport::default(),
        input: &mut input,
        interactive,
    };
    run_with(ctx, args, &mut deps, out, err)
}

pub(crate) fn run_with(
    ctx: &Context,
    args: &[String],
    deps: &mut Deps,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    let Some(folder) = ctx.config_folder() else {
        let _ = writeln!(
            err,
            "raul nexus: no configuration folder (HOME, XDG_CONFIG_HOME or APPDATA is not set)"
        );
        return crate::FOUND;
    };
    let file = FileStore::in_folder(&folder);
    let settings = folder.join("settings.toml");
    match words.as_slice() {
        ["key", "set"] => set(false, deps, &file, &settings, out, err),
        ["key", "set", "--file"] => set(true, deps, &file, &settings, out, err),
        ["key", "clear"] => clear(deps, &file, &settings, out, err),
        ["key", "status"] => status(false, deps, &file, &settings, out, err),
        ["key", "status", "--json"] => status(true, deps, &file, &settings, out, err),
        _ => crate::usage(err, USAGE),
    }
}

fn say(out: &mut dyn Write, err: &mut dyn Write, text: &str) -> u8 {
    crate::write_or_fail(out, err, &format!("{text}\n"))
}

fn set(
    to_file: bool,
    deps: &mut Deps,
    file: &FileStore,
    settings: &std::path::Path,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    if deps.interactive {
        let _ = write!(
            err,
            "Paste your Nexus API key and press Enter (it is shown as you type): "
        );
        let _ = err.flush();
    }
    let mut line = String::new();
    if deps.input.read_line(&mut line).is_err() {
        let _ = writeln!(
            err,
            "raul nexus: the key could not be read from standard input"
        );
        return crate::FOUND;
    }
    let secret = match Secret::new(&line) {
        Ok(secret) => secret,
        Err(e) => {
            let _ = writeln!(
                err,
                "raul nexus: {e}; pipe the key in, for example `raul nexus key set < keyfile`"
            );
            return crate::USAGE;
        }
    };
    let (store, other, source): (&dyn KeyStore, &dyn KeyStore, _) = if to_file {
        (file, deps.keyring, KeySource::File)
    } else {
        (deps.keyring, file, KeySource::Keyring)
    };
    if let Err(e) = store.set(&secret) {
        let _ = writeln!(err, "raul nexus: {}", redact(&e.to_string(), Some(&secret)));
        if !to_file {
            let _ = writeln!(
                err,
                "raul nexus: to keep the key in a file only you can read instead, run `raul nexus key set --file`; \
                 or set {ENV_VAR} in the environment"
            );
        }
        return crate::FOUND;
    }
    // A key left in the other place would be found first or last by accident; one place holds it.
    let _ = other.clear();
    // The identity of an earlier key does not describe this one.
    let _ = Identity::forget(settings);
    let mut text = format!("Nexus API key saved in {}.", source.describe());
    if to_file {
        text.push_str(" Anyone who can read that file can use the key.");
    }
    if deps.env.as_deref().is_some_and(|v| !v.trim().is_empty()) {
        text.push(' ');
        text.push_str(ENV_VAR);
        text.push_str(" is set and is used instead of the saved key.");
    }
    text.push_str(" Run `raul nexus key status` to check it.");
    say(out, err, &text)
}

fn clear(
    deps: &mut Deps,
    file: &FileStore,
    settings: &std::path::Path,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let mut code = crate::OK;
    for (store, name) in [
        (deps.keyring, "keyring"),
        (file as &dyn KeyStore, "key file"),
    ] {
        if let Err(e) = store.clear() {
            let _ = writeln!(err, "raul nexus: the {name} was not cleared: {e}");
            code = crate::FOUND;
        }
    }
    if let Err(e) = Identity::forget(settings) {
        let _ = writeln!(err, "raul nexus: {}: {e}", settings.display());
        code = crate::FOUND;
    }
    if code != crate::OK {
        return code;
    }
    let mut text = String::from("Nexus API key removed from the keyring and the key file.");
    if deps.env.as_deref().is_some_and(|v| !v.trim().is_empty()) {
        text.push(' ');
        text.push_str(ENV_VAR);
        text.push_str(" is still set in the environment; unset it there.");
    }
    say(out, err, &text)
}

fn status(
    json: bool,
    deps: &mut Deps,
    file: &FileStore,
    settings: &std::path::Path,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let found = match key::resolve(deps.env.as_deref(), deps.keyring, file) {
        Ok(found) => found,
        Err(e) => {
            return fail(
                json,
                "key_unreadable",
                &e.to_string(),
                crate::FOUND,
                out,
                err,
            );
        }
    };
    let Some((secret, source)) = found else {
        let doc = serde_json::json!({ "key": { "set": false } });
        let text = "No Nexus API key is set. Run `raul nexus key set` (the key is read from standard input).";
        return if json {
            let _ = writeln!(out, "{doc}");
            crate::FOUND
        } else {
            let _ = writeln!(err, "{text}");
            crate::FOUND
        };
    };
    let client = NexusClient::new(deps.transport, secret.clone(), VERSION);
    match client.validate() {
        Ok(who) => {
            let identity = Identity {
                user_name: who.name.clone(),
                is_premium: who.is_premium,
                validated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_secs()),
            };
            if let Err(e) = identity.save(settings) {
                let _ = writeln!(err, "raul nexus: the name could not be remembered: {e}");
            }
            let limit = client.rate_limit();
            let doc = serde_json::json!({
                "key": { "set": true, "source": source_name(source) },
                "valid": true,
                "user_name": who.name,
                "is_premium": who.is_premium,
                "requests_left": {
                    "hourly": limit.and_then(|l| l.hourly_remaining),
                    "daily": limit.and_then(|l| l.daily_remaining),
                },
            });
            let mut text = format!(
                "Nexus API key: valid (from {})\nuser: {}\npremium: {}",
                source.describe(),
                who.name,
                if who.is_premium { "yes" } else { "no" }
            );
            if let Some(limit) = limit {
                text.push_str("\nrequests left: ");
                text.push_str(&limit.describe());
            }
            if !who.is_premium {
                text.push_str("\nWithout premium, Nexus files are downloaded by hand.");
            }
            if json {
                say(out, err, &doc.to_string())
            } else {
                say(out, err, &text)
            }
        }
        Err(e) => {
            let code = match e {
                NexusError::Unauthorized => crate::FOUND,
                _ => crate::FAILED,
            };
            let message = redact(&e.to_string(), Some(&secret));
            fail(
                json,
                if code == crate::FOUND {
                    "key_refused"
                } else {
                    "not_validated"
                },
                &message,
                code,
                out,
                err,
            )
        }
    }
}

fn source_name(source: KeySource) -> &'static str {
    match source {
        KeySource::Env => "env",
        KeySource::Keyring => "keyring",
        KeySource::File => "file",
    }
}

fn fail(
    json: bool,
    kind: &str,
    message: &str,
    code: u8,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let _ = writeln!(err, "raul nexus: {message}");
    if json {
        let _ = writeln!(
            out,
            "{}",
            serde_json::json!({ "valid": false, "error": kind, "message": message })
        );
    }
    code
}

#[cfg(test)]
mod tests {
    use super::*;
    use raul_core::nexus::KeyError;
    use raul_core::nexus::client::{Request, Response, TransportError};
    use std::cell::RefCell;

    #[derive(Default)]
    struct Memory(RefCell<Option<Secret>>, bool);

    impl KeyStore for Memory {
        fn get(&self) -> Result<Option<Secret>, KeyError> {
            if self.1 {
                Err(KeyError::Unavailable("no secret service".into()))
            } else {
                Ok(self.0.borrow().clone())
            }
        }
        fn set(&self, s: &Secret) -> Result<(), KeyError> {
            if self.1 {
                return Err(KeyError::Unavailable("no secret service".into()));
            }
            *self.0.borrow_mut() = Some(s.clone());
            Ok(())
        }
        fn clear(&self) -> Result<(), KeyError> {
            *self.0.borrow_mut() = None;
            Ok(())
        }
    }

    struct Answer(u16, &'static str, RefCell<Vec<Request>>);

    impl Transport for Answer {
        fn send(&self, r: &Request) -> Result<Response, TransportError> {
            self.2.borrow_mut().push(r.clone());
            Ok(Response {
                status: self.0,
                headers: vec![("x-rl-daily-remaining".into(), "1999".into())],
                body: self.1.into(),
            })
        }
    }

    const KEY: &str = "synthetic-key-0123456789";
    const WHO: &str =
        r#"{"user_id":3,"name":"Courier Six","is_premium":false,"is_supporter":true}"#;

    fn folder(name: &str) -> (Context, std::path::PathBuf) {
        let dir =
            std::env::temp_dir().join(format!("raul-nexus-cli-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        (Context::new().with_config_folder(&dir), dir)
    }

    fn go(
        ctx: &Context,
        args: &[&str],
        keyring: &Memory,
        transport: &Answer,
        input: &str,
        env: Option<&str>,
    ) -> (u8, String, String) {
        let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
        let mut reader = std::io::Cursor::new(input.as_bytes().to_vec());
        let mut deps = Deps {
            env: env.map(str::to_owned),
            keyring,
            transport,
            input: &mut reader,
            interactive: false,
        };
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run_with(ctx, &args, &mut deps, &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    #[test]
    fn set_reads_the_key_from_standard_input_and_puts_it_in_the_keyring() {
        let (ctx, dir) = folder("set");
        let keyring = Memory::default();
        let t = Answer(200, WHO, RefCell::default());
        let (code, out, err) = go(
            &ctx,
            &["key", "set"],
            &keyring,
            &t,
            &format!("{KEY}\n"),
            None,
        );
        assert_eq!(code, crate::OK, "{err}");
        assert_eq!(keyring.0.borrow().as_ref().unwrap().expose(), KEY);
        assert!(!out.contains(KEY) && !err.contains(KEY));
        assert!(!dir.join("nexus-api-key").exists());
        assert!(
            !dir.join("settings.toml").exists()
                || !std::fs::read_to_string(dir.join("settings.toml"))
                    .unwrap()
                    .contains(KEY)
        );
    }

    #[test]
    fn set_without_a_keyring_says_so_and_changes_nothing_until_the_file_is_asked_for() {
        let (ctx, dir) = folder("nokeyring");
        let keyring = Memory(RefCell::default(), true);
        let t = Answer(200, WHO, RefCell::default());
        let (code, _, err) = go(
            &ctx,
            &["key", "set"],
            &keyring,
            &t,
            &format!("{KEY}\n"),
            None,
        );
        assert_eq!(code, crate::FOUND);
        assert!(err.contains("--file") && !err.contains(KEY), "{err}");
        assert!(!dir.join("nexus-api-key").exists());
        let (code, out, _) = go(
            &ctx,
            &["key", "set", "--file"],
            &keyring,
            &t,
            &format!("{KEY}\n"),
            None,
        );
        assert_eq!(code, crate::OK);
        assert!(out.contains("Anyone who can read"), "{out}");
        assert_eq!(
            std::fs::read_to_string(dir.join("nexus-api-key"))
                .unwrap()
                .trim(),
            KEY
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn set_refuses_a_key_that_cannot_be_one_and_never_echoes_it() {
        let (ctx, _) = folder("bad");
        let t = Answer(200, WHO, RefCell::default());
        let (code, _, err) = go(
            &ctx,
            &["key", "set"],
            &Memory::default(),
            &t,
            "has space in it\n",
            None,
        );
        assert_eq!(code, crate::USAGE);
        assert!(!err.contains("has space"), "{err}");
    }

    #[test]
    fn status_shows_the_user_and_premium_and_remembers_only_those() {
        let (ctx, dir) = folder("status");
        let keyring = Memory::default();
        *keyring.0.borrow_mut() = Some(Secret::new(KEY).unwrap());
        let t = Answer(200, WHO, RefCell::default());
        let (code, out, err) = go(&ctx, &["key", "status"], &keyring, &t, "", None);
        assert_eq!(code, crate::OK, "{err}");
        assert!(
            out.contains("user: Courier Six")
                && out.contains("premium: no")
                && out.contains("1999 today"),
            "{out}"
        );
        let request = t.2.borrow()[0].clone();
        assert_eq!(request.header("apikey"), Some(KEY));
        assert_eq!(request.header("Application-Name"), Some("raul"));
        let settings = std::fs::read_to_string(dir.join("settings.toml")).unwrap();
        assert!(
            settings.contains("Courier Six") && !settings.contains(KEY),
            "{settings}"
        );
        let (code, json, _) = go(&ctx, &["key", "status", "--json"], &keyring, &t, "", None);
        assert_eq!(code, crate::OK);
        let doc: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            (
                doc["valid"].clone(),
                doc["is_premium"].clone(),
                doc["key"]["source"].clone()
            ),
            (true.into(), false.into(), "keyring".into())
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn status_uses_the_environment_key_first_and_reports_a_refused_key() {
        let (ctx, dir) = folder("refused");
        let keyring = Memory::default();
        let t = Answer(
            401,
            r#"{"message":"Please provide a valid API Key"}"#,
            RefCell::default(),
        );
        let (code, _, err) = go(&ctx, &["key", "status"], &keyring, &t, "", Some(KEY));
        assert_eq!(code, crate::FOUND);
        assert!(
            err.contains("does not accept") && !err.contains(KEY),
            "{err}"
        );
        assert_eq!(t.2.borrow()[0].header("apikey"), Some(KEY));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn status_without_a_key_asks_nothing_of_nexus() {
        let (ctx, _) = folder("none");
        let t = Answer(200, WHO, RefCell::default());
        let (code, _, err) = go(&ctx, &["key", "status"], &Memory::default(), &t, "", None);
        assert_eq!(code, crate::FOUND);
        assert!(err.contains("No Nexus API key"), "{err}");
        assert!(t.2.borrow().is_empty());
    }

    #[test]
    fn clear_removes_the_key_everywhere_and_the_remembered_name() {
        let (ctx, dir) = folder("clear");
        let keyring = Memory::default();
        let t = Answer(200, WHO, RefCell::default());
        go(
            &ctx,
            &["key", "set", "--file"],
            &keyring,
            &t,
            &format!("{KEY}\n"),
            None,
        );
        *keyring.0.borrow_mut() = Some(Secret::new(KEY).unwrap());
        go(&ctx, &["key", "status"], &keyring, &t, "", None);
        let (code, out, err) = go(&ctx, &["key", "clear"], &keyring, &t, "", Some("x"));
        assert_eq!(code, crate::OK, "{err}");
        assert!(out.contains("still set"), "{out}");
        assert!(keyring.0.borrow().is_none() && !dir.join("nexus-api-key").exists());
        assert!(
            !std::fs::read_to_string(dir.join("settings.toml"))
                .unwrap()
                .contains("Courier")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_unknown_subcommand_is_a_usage_error() {
        let (ctx, _) = folder("usage");
        let t = Answer(200, WHO, RefCell::default());
        assert_eq!(
            go(&ctx, &["key", "set", KEY], &Memory::default(), &t, "", None).0,
            crate::USAGE
        );
    }
}
