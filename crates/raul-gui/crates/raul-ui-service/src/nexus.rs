// SPDX-License-Identifier: GPL-3.0-only
//! The Nexus API key as a job of the window (docs/adr/0024): save, clear, check and read its status through
//! `raul_core::nexus`, in this process.
//!
//! The key never travels as an argument: a [`NexusOp::Save`] holds it as a [`Secret`] (no `Display`, a `Debug` that
//! hides it) inside the job, and every message that leaves here passes through `raul_core::nexus::redact`. The
//! documents the jobs print hold no key, only where it is kept and who it belongs to.

use crate::direct::DirectJob;
use nvmod_api::{ErrorKind, JobError, Outcome};
use raul_core::context::Context;
use raul_core::nexus::curl::CurlTransport;
use raul_core::nexus::key::{self, ENV_VAR, FileStore, KeyringStore};
use raul_core::nexus::redact::redact;
use raul_core::nexus::{
    Identity, KeySource, KeyStore, NexusApi, NexusClient, NexusError, Secret, Transport,
};
use std::sync::Arc;

/// The version raul tells Nexus it is.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// What a job reads from outside, so a test can give it fakes.
#[derive(Clone)]
pub struct NexusDeps {
    /// The operating system's keyring.
    pub keyring: Arc<dyn KeyStore + Send + Sync>,
    /// How requests reach Nexus.
    pub transport: Arc<dyn Transport + Send + Sync>,
    /// The `NEXUS_API_KEY` variable, if it is set.
    pub env: Option<String>,
}

impl std::fmt::Debug for NexusDeps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The variable holds a key: only whether it is set is shown.
        f.debug_struct("NexusDeps")
            .field("env_set", &self.env.is_some())
            .finish_non_exhaustive()
    }
}

impl NexusDeps {
    /// The real keyring, `curl` and the process's environment.
    pub fn real() -> NexusDeps {
        NexusDeps {
            keyring: Arc::new(KeyringStore),
            transport: Arc::new(CurlTransport::default()),
            env: std::env::var(ENV_VAR).ok(),
        }
    }
}

/// What a job does with the key.
#[derive(Clone)]
pub enum NexusOp {
    /// Reads where the key is kept and what is remembered about it; asks no one.
    Status,
    /// Stores the key: in the keyring, or in the key file when `to_file`.
    Save {
        /// The key.
        secret: Secret,
        /// Keep it in the key file instead of the keyring.
        to_file: bool,
    },
    /// Removes the stored key from the keyring and the key file.
    Clear,
    /// Asks Nexus who the key belongs to, and remembers the answer.
    Check,
}

impl std::fmt::Debug for NexusOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            NexusOp::Status => "Status",
            NexusOp::Save { .. } => "Save([hidden])",
            NexusOp::Clear => "Clear",
            NexusOp::Check => "Check",
        })
    }
}

/// The job that does `op`: its document is the status after it (see `raul_ui_model::pages::nexus::KeyStatus`).
pub fn job(op: NexusOp, ctx: &Context, deps: NexusDeps) -> DirectJob {
    let ctx = ctx.clone();
    DirectJob::reading(
        move |_, _| -> Result<Outcome<serde_json::Value>, JobError> {
            run(&op, &ctx, &deps)
                .map(Outcome::Done)
                .map_err(|why| JobError::new(ErrorKind::Refused, "nexus.key", why))
        },
    )
}

fn files(ctx: &Context) -> Result<(FileStore, std::path::PathBuf), String> {
    let folder = ctx.config_folder().ok_or_else(|| {
        "there is no configuration folder (HOME, XDG_CONFIG_HOME or APPDATA is not set)".to_owned()
    })?;
    Ok((FileStore::in_folder(&folder), folder.join("settings.toml")))
}

/// Runs `op`; the error is one plain line without the key.
///
/// # Errors
/// Why the key could not be stored, removed or checked.
pub fn run(op: &NexusOp, ctx: &Context, deps: &NexusDeps) -> Result<serde_json::Value, String> {
    let (file, settings) = files(ctx)?;
    match op {
        NexusOp::Status => {}
        NexusOp::Save { secret, to_file } => save(secret, *to_file, deps, &file, &settings)?,
        NexusOp::Clear => clear(&file, &settings, deps)?,
        NexusOp::Check => check(deps, &file, &settings)?,
    }
    Ok(status(deps, &file, &settings))
}

fn save(
    secret: &Secret,
    to_file: bool,
    deps: &NexusDeps,
    file: &FileStore,
    settings: &std::path::Path,
) -> Result<(), String> {
    let (store, other): (&dyn KeyStore, &dyn KeyStore) = if to_file {
        (file, deps.keyring.as_ref())
    } else {
        (deps.keyring.as_ref(), file)
    };
    store
        .set(secret)
        .map_err(|e| redact(&e.to_string(), Some(secret)))?;
    // One place holds the key, and the identity of an earlier key does not describe this one.
    let _ = other.clear();
    let _ = Identity::forget(settings);
    Ok(())
}

fn clear(file: &FileStore, settings: &std::path::Path, deps: &NexusDeps) -> Result<(), String> {
    let mut failed = Vec::new();
    if let Err(e) = deps.keyring.clear() {
        failed.push(format!("the keyring was not cleared: {e}"));
    }
    if let Err(e) = file.clear() {
        failed.push(format!("the key file was not removed: {e}"));
    }
    if let Err(e) = Identity::forget(settings) {
        failed.push(format!("{}: {e}", settings.display()));
    }
    if failed.is_empty() {
        Ok(())
    } else {
        Err(failed.join("; "))
    }
}

fn check(deps: &NexusDeps, file: &FileStore, settings: &std::path::Path) -> Result<(), String> {
    let (secret, _) = key::resolve(deps.env.as_deref(), deps.keyring.as_ref(), file)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No Nexus API key is set.".to_owned())?;
    let client = NexusClient::new(deps.transport.as_ref(), secret.clone(), VERSION);
    let who = client
        .validate()
        .map_err(|e: NexusError| redact(&e.to_string(), Some(&secret)))?;
    let identity = Identity {
        user_name: who.name,
        is_premium: who.is_premium,
        validated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
    };
    identity
        .save(settings)
        .map_err(|e| format!("the account name could not be remembered: {e}"))
}

/// The status document: where the key is kept and what is remembered about its account. Nothing here asks Nexus.
fn status(deps: &NexusDeps, file: &FileStore, settings: &std::path::Path) -> serde_json::Value {
    let keyring_available = deps.keyring.get().is_ok();
    let env_set = deps.env.as_deref().is_some_and(|v| !v.trim().is_empty());
    let (source, problem) = match key::resolve(deps.env.as_deref(), deps.keyring.as_ref(), file) {
        Ok(Some((_, source))) => (
            Some(match source {
                KeySource::Env => "env",
                KeySource::Keyring => "keyring",
                KeySource::File => "file",
            }),
            None,
        ),
        Ok(None) => (None, None),
        Err(e) => (None, Some(e.to_string())),
    };
    let identity = source.and_then(|_| Identity::load(settings));
    serde_json::json!({
        "source": source,
        "keyring_available": keyring_available,
        "env_set": env_set,
        "problem": problem,
        "user_name": identity.as_ref().map(|i| i.user_name.clone()),
        "is_premium": identity.as_ref().map(|i| i.is_premium),
        "checked_at": identity.as_ref().map(|i| i.validated_at),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use raul_core::nexus::{Request, Response, TransportError};
    use std::sync::Mutex;

    #[derive(Default)]
    struct Memory {
        value: Mutex<Option<String>>,
        broken: bool,
    }

    impl KeyStore for Memory {
        fn get(&self) -> Result<Option<Secret>, key::KeyError> {
            if self.broken {
                return Err(key::KeyError::Unavailable("no Secret Service".into()));
            }
            let v = self.value.lock().unwrap().clone();
            v.map(|t| Secret::new(&t)).transpose()
        }
        fn set(&self, secret: &Secret) -> Result<(), key::KeyError> {
            if self.broken {
                return Err(key::KeyError::Unavailable(format!(
                    "no Secret Service for {}",
                    secret.expose()
                )));
            }
            *self.value.lock().unwrap() = Some(secret.expose().to_owned());
            Ok(())
        }
        fn clear(&self) -> Result<(), key::KeyError> {
            if self.broken {
                return Err(key::KeyError::Unavailable("no Secret Service".into()));
            }
            *self.value.lock().unwrap() = None;
            Ok(())
        }
    }

    struct Nexus {
        status: u16,
        body: &'static str,
    }

    impl Transport for Nexus {
        fn send(&self, _: &Request) -> Result<Response, TransportError> {
            Ok(Response {
                status: self.status,
                headers: Vec::new(),
                body: self.body.to_owned(),
            })
        }
    }

    const WHO: &str = r#"{"user_id":7,"name":"Courier6","is_premium":true,"is_supporter":false}"#;

    fn deps(keyring: Memory, env: Option<&str>, status: u16, body: &'static str) -> NexusDeps {
        NexusDeps {
            keyring: Arc::new(keyring),
            transport: Arc::new(Nexus { status, body }),
            env: env.map(str::to_owned),
        }
    }

    fn folder(name: &str) -> (Context, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("raul-ui-nexus-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        (Context::new().with_config_folder(&dir), dir)
    }

    fn secret(text: &str) -> Secret {
        Secret::new(text).unwrap()
    }

    #[test]
    fn save_check_and_clear_go_through_the_keyring_and_never_show_the_key() {
        let (ctx, dir) = folder("flow");
        let d = deps(Memory::default(), None, 200, WHO);
        let empty = run(&NexusOp::Status, &ctx, &d).unwrap();
        assert_eq!(empty["source"], serde_json::Value::Null);
        assert_eq!(empty["keyring_available"], true);

        let saved = run(
            &NexusOp::Save {
                secret: secret("sUpErSeCrEtKey123"),
                to_file: false,
            },
            &ctx,
            &d,
        )
        .unwrap();
        assert_eq!(saved["source"], "keyring");
        assert_eq!(saved["user_name"], serde_json::Value::Null);

        let checked = run(&NexusOp::Check, &ctx, &d).unwrap();
        assert_eq!(checked["user_name"], "Courier6");
        assert_eq!(checked["is_premium"], true);
        assert!(checked["checked_at"].as_u64().unwrap() > 1_700_000_000);
        assert!(!checked.to_string().contains("sUpErSeCrEtKey123"));
        assert!(
            !dir.join("settings.toml").exists()
                || !std::fs::read_to_string(dir.join("settings.toml"))
                    .unwrap()
                    .contains("sUpErSeCrEt")
        );

        let cleared = run(&NexusOp::Clear, &ctx, &d).unwrap();
        assert_eq!(cleared["source"], serde_json::Value::Null);
        assert_eq!(cleared["user_name"], serde_json::Value::Null);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_new_key_forgets_the_account_of_the_old_one() {
        let (ctx, dir) = folder("forget");
        let d = deps(Memory::default(), None, 200, WHO);
        run(
            &NexusOp::Save {
                secret: secret("firstKey1"),
                to_file: false,
            },
            &ctx,
            &d,
        )
        .unwrap();
        run(&NexusOp::Check, &ctx, &d).unwrap();
        let again = run(
            &NexusOp::Save {
                secret: secret("secondKey2"),
                to_file: false,
            },
            &ctx,
            &d,
        )
        .unwrap();
        assert_eq!(again["user_name"], serde_json::Value::Null);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn without_a_keyring_the_key_goes_to_a_file_only_the_person_reads() {
        let (ctx, dir) = folder("file");
        let d = deps(
            Memory {
                broken: true,
                ..Memory::default()
            },
            None,
            200,
            WHO,
        );
        let status = run(&NexusOp::Status, &ctx, &d).unwrap();
        assert_eq!(status["keyring_available"], false);
        let why = run(
            &NexusOp::Save {
                secret: secret("fileKeyValue9"),
                to_file: false,
            },
            &ctx,
            &d,
        )
        .unwrap_err();
        assert!(!why.contains("fileKeyValue9"), "{why}");
        // The broken keyring cannot be cleared either, which the save of a file does not mind.
        run(
            &NexusOp::Save {
                secret: secret("fileKeyValue9"),
                to_file: true,
            },
            &ctx,
            &d,
        )
        .unwrap();
        let file = dir.join("nexus-api-key");
        assert_eq!(
            std::fs::read_to_string(&file).unwrap().trim(),
            "fileKeyValue9"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&file).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        let status = run(&NexusOp::Status, &ctx, &d).unwrap();
        assert_eq!(status["source"], "file");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_variable_is_used_before_a_saved_key_and_is_said_to_be() {
        let (ctx, dir) = folder("env");
        let d = deps(Memory::default(), Some("fromTheEnvironment1"), 200, WHO);
        let status = run(&NexusOp::Status, &ctx, &d).unwrap();
        assert_eq!(
            (status["source"].as_str(), status["env_set"].as_bool()),
            (Some("env"), Some(true))
        );
        let bad = deps(Memory::default(), Some("has a space"), 200, WHO);
        let status = run(&NexusOp::Status, &ctx, &bad).unwrap();
        assert!(status["problem"].as_str().unwrap().contains(ENV_VAR));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_check_says_why_it_failed_without_the_key() {
        let (ctx, dir) = folder("fail");
        let d = deps(
            Memory::default(),
            None,
            401,
            r#"{"message":"Please provide a valid API Key"}"#,
        );
        assert_eq!(
            run(&NexusOp::Check, &ctx, &d).unwrap_err(),
            "No Nexus API key is set."
        );
        run(
            &NexusOp::Save {
                secret: secret("rejectedKey77"),
                to_file: false,
            },
            &ctx,
            &d,
        )
        .unwrap();
        let why = run(&NexusOp::Check, &ctx, &d).unwrap_err();
        assert!(why.contains("does not accept"), "{why}");
        assert!(!why.contains("rejectedKey77"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn an_operation_never_prints_its_key() {
        let op = NexusOp::Save {
            secret: secret("printedKey55"),
            to_file: false,
        };
        assert!(!format!("{op:?}").contains("printedKey55"));
        let d = deps(Memory::default(), Some("envKeyValue88"), 200, WHO);
        assert!(!format!("{d:?}").contains("envKeyValue88"));
    }
}
