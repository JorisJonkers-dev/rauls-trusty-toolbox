// SPDX-License-Identifier: GPL-3.0-only
//! Where the Nexus API key lives, in the order it is looked up: the `NEXUS_API_KEY` environment variable, the OS
//! keyring (Secret Service on Linux, Credential Manager on Windows, Keychain on macOS), then a file in the
//! configuration folder that only the person can read. `settings.toml` never holds it.

use std::fmt;
use std::path::{Path, PathBuf};

/// The environment variable that overrides everything else; for CI and machines without a keyring.
pub const ENV_VAR: &str = "NEXUS_API_KEY";

/// The service name the keyring entry is stored under.
pub const KEYRING_SERVICE: &str = "dev.jorisjonkers.raul";

/// The keyring entry's user name.
pub const KEYRING_USER: &str = "nexus-api-key";

/// The fallback file's name in the configuration folder.
pub const FILE_NAME: &str = "nexus-api-key";

const MAX_LEN: usize = 512;

/// The API key. It has no `Display`, its `Debug` hides it, and it is compared and stored only through
/// [`Secret::expose`], so a stray `{:?}` or `{}` cannot leak it.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    /// A key from `text` (surrounding white space is trimmed).
    ///
    /// # Errors
    /// [`KeyError::Invalid`] when it is empty, too long, or holds a character that is not printable ASCII, or a
    /// quote or backslash: such a character cannot be part of a Nexus key and could break the header it is sent in.
    pub fn new(text: &str) -> Result<Secret, KeyError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(KeyError::Invalid("the key is empty".into()));
        }
        if text.len() > MAX_LEN {
            return Err(KeyError::Invalid("the key is too long".into()));
        }
        if !text
            .bytes()
            .all(|b| b.is_ascii_graphic() && !matches!(b, b'"' | b'\'' | b'\\'))
        {
            return Err(KeyError::Invalid(
                "the key holds a character a Nexus key does not (a space, quote, backslash or non-ASCII character)"
                    .into(),
            ));
        }
        Ok(Secret(text.to_owned()))
    }

    /// The key itself. Use it only to hand it to the transport or a store.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret([redacted])")
    }
}

/// Why the key could not be read or stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyError {
    /// The store cannot be used here (no Secret Service running, a locked keychain, a file that cannot be written).
    Unavailable(String),
    /// The key text is not a usable key.
    Invalid(String),
}

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyError::Unavailable(why) | KeyError::Invalid(why) => f.write_str(why),
        }
    }
}

impl std::error::Error for KeyError {}

/// Where a key was found, or stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySource {
    /// The `NEXUS_API_KEY` environment variable.
    Env,
    /// The OS keyring.
    Keyring,
    /// The fallback file.
    File,
}

impl KeySource {
    /// A phrase for the person.
    #[must_use]
    pub fn describe(self) -> &'static str {
        match self {
            KeySource::Env => "the NEXUS_API_KEY environment variable",
            KeySource::Keyring => "the operating system's keyring",
            KeySource::File => "the key file in the configuration folder",
        }
    }
}

/// One place a key can be kept.
pub trait KeyStore {
    /// The stored key, or `None` when there is none.
    ///
    /// # Errors
    /// [`KeyError::Unavailable`] when the store cannot be reached or its content is not a key.
    fn get(&self) -> Result<Option<Secret>, KeyError>;
    /// Stores `secret`, replacing any earlier one.
    ///
    /// # Errors
    /// [`KeyError::Unavailable`] when the store cannot be written.
    fn set(&self, secret: &Secret) -> Result<(), KeyError>;
    /// Removes the stored key; having none is fine.
    ///
    /// # Errors
    /// [`KeyError::Unavailable`] when the store cannot be reached.
    fn clear(&self) -> Result<(), KeyError>;
}

/// The OS keyring.
#[derive(Debug, Default, Clone, Copy)]
pub struct KeyringStore;

impl KeyringStore {
    fn entry() -> Result<keyring::Entry, KeyError> {
        keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
            .map_err(|e| KeyError::Unavailable(format!("the keyring cannot be used: {e}")))
    }
}

impl KeyStore for KeyringStore {
    fn get(&self) -> Result<Option<Secret>, KeyError> {
        match Self::entry()?.get_password() {
            Ok(text) => Secret::new(&text).map(Some),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(KeyError::Unavailable(format!(
                "the keyring cannot be read: {e}"
            ))),
        }
    }

    fn set(&self, secret: &Secret) -> Result<(), KeyError> {
        Self::entry()?
            .set_password(secret.expose())
            .map_err(|e| KeyError::Unavailable(format!("the keyring cannot be written: {e}")))
    }

    fn clear(&self) -> Result<(), KeyError> {
        match Self::entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(KeyError::Unavailable(format!(
                "the keyring cannot be cleared: {e}"
            ))),
        }
    }
}

/// The fallback file: readable and writable by the person only (mode 0600 on Unix; on Windows it lives in the
/// person's own `%APPDATA%` folder, which other accounts cannot read).
#[derive(Debug, Clone)]
pub struct FileStore {
    path: PathBuf,
}

impl FileStore {
    /// The key file at `path`.
    #[must_use]
    pub fn at(path: impl Into<PathBuf>) -> FileStore {
        FileStore { path: path.into() }
    }

    /// The key file in the configuration folder `folder`.
    #[must_use]
    pub fn in_folder(folder: &Path) -> FileStore {
        FileStore::at(folder.join(FILE_NAME))
    }

    /// Where the file is.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn io(&self, what: &str, e: &std::io::Error) -> KeyError {
        KeyError::Unavailable(format!("{what} {}: {e}", self.path.display()))
    }
}

impl KeyStore for FileStore {
    fn get(&self) -> Result<Option<Secret>, KeyError> {
        match std::fs::read_to_string(&self.path) {
            Ok(text) => Secret::new(&text).map(Some),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(self.io("cannot read", &e)),
        }
    }

    fn set(&self, secret: &Secret) -> Result<(), KeyError> {
        use std::io::Write;
        let dir = self.path.parent().unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(dir).map_err(|e| self.io("cannot create the folder of", &e))?;
        // Written under a temporary name that already has the right mode, then renamed over the file: the key is
        // never on disk with wider permissions than the person's own.
        let temp = self.path.with_extension("tmp");
        let _ = std::fs::remove_file(&temp);
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        let mut file = options
            .open(&temp)
            .map_err(|e| self.io("cannot write", &e))?;
        file.write_all(secret.expose().as_bytes())
            .and_then(|()| file.write_all(b"\n"))
            .and_then(|()| file.sync_all())
            .map_err(|e| self.io("cannot write", &e))?;
        drop(file);
        std::fs::rename(&temp, &self.path).map_err(|e| {
            let _ = std::fs::remove_file(&temp);
            self.io("cannot write", &e)
        })
    }

    fn clear(&self) -> Result<(), KeyError> {
        match std::fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(self.io("cannot remove", &e)),
        }
    }
}

/// The key to use: the environment variable `env` when it is set and not blank, else the keyring, else the file.
/// A keyring that cannot be reached is skipped (that is what the file is for); an unreadable file is an error.
///
/// # Errors
/// [`KeyError::Invalid`] when the environment variable holds something that is not a key, and
/// [`KeyError::Unavailable`] when the file cannot be read.
pub fn resolve(
    env: Option<&str>,
    keyring: &dyn KeyStore,
    file: &dyn KeyStore,
) -> Result<Option<(Secret, KeySource)>, KeyError> {
    if let Some(text) = env.filter(|t| !t.trim().is_empty()) {
        return Secret::new(text)
            .map(|s| Some((s, KeySource::Env)))
            .map_err(|e| KeyError::Invalid(format!("{ENV_VAR}: {e}")));
    }
    if let Ok(Some(secret)) = keyring.get() {
        return Ok(Some((secret, KeySource::Keyring)));
    }
    Ok(file.get()?.map(|s| (s, KeySource::File)))
}

#[cfg(test)]
pub(crate) mod fake {
    use super::*;
    use std::cell::RefCell;

    /// A store in memory; `broken` makes every call fail like a keyring with no Secret Service.
    #[derive(Default)]
    pub struct MemoryStore {
        pub value: RefCell<Option<Secret>>,
        pub broken: bool,
    }

    impl KeyStore for MemoryStore {
        fn get(&self) -> Result<Option<Secret>, KeyError> {
            if self.broken {
                return Err(KeyError::Unavailable("no secret service".into()));
            }
            Ok(self.value.borrow().clone())
        }
        fn set(&self, secret: &Secret) -> Result<(), KeyError> {
            if self.broken {
                return Err(KeyError::Unavailable("no secret service".into()));
            }
            *self.value.borrow_mut() = Some(secret.clone());
            Ok(())
        }
        fn clear(&self) -> Result<(), KeyError> {
            if self.broken {
                return Err(KeyError::Unavailable("no secret service".into()));
            }
            *self.value.borrow_mut() = None;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fake::MemoryStore;
    use super::*;

    fn secret(text: &str) -> Secret {
        Secret::new(text).unwrap()
    }

    #[test]
    fn a_secret_never_prints_itself() {
        let s = secret("abc123==");
        assert!(!format!("{s:?}").contains("abc123"));
    }

    #[test]
    fn text_that_cannot_be_a_key_is_refused() {
        for bad in [
            "",
            "  ",
            "a b",
            "a\"b",
            "a\\b",
            "a\nb",
            "kéy",
            &"x".repeat(600),
        ] {
            assert!(Secret::new(bad).is_err(), "{bad:?}");
        }
        assert_eq!(
            Secret::new("  key-1_2+3/4=\n").unwrap().expose(),
            "key-1_2+3/4="
        );
    }

    #[test]
    fn the_environment_wins_over_the_keyring_and_the_keyring_over_the_file() {
        let keyring = MemoryStore::default();
        let file = MemoryStore::default();
        *keyring.value.borrow_mut() = Some(secret("fromkeyring"));
        *file.value.borrow_mut() = Some(secret("fromfile"));
        let (s, source) = resolve(Some("fromenv"), &keyring, &file).unwrap().unwrap();
        assert_eq!((s.expose(), source), ("fromenv", KeySource::Env));
        let (s, source) = resolve(None, &keyring, &file).unwrap().unwrap();
        assert_eq!((s.expose(), source), ("fromkeyring", KeySource::Keyring));
        *keyring.value.borrow_mut() = None;
        let (s, source) = resolve(Some("  "), &keyring, &file).unwrap().unwrap();
        assert_eq!((s.expose(), source), ("fromfile", KeySource::File));
    }

    #[test]
    fn a_keyring_that_cannot_be_reached_falls_back_to_the_file() {
        let keyring = MemoryStore {
            broken: true,
            ..MemoryStore::default()
        };
        let file = MemoryStore::default();
        assert_eq!(resolve(None, &keyring, &file).unwrap(), None);
        *file.value.borrow_mut() = Some(secret("fromfile"));
        assert_eq!(
            resolve(None, &keyring, &file).unwrap().unwrap().1,
            KeySource::File
        );
    }

    #[test]
    fn a_blank_or_invalid_environment_variable_is_not_silently_replaced() {
        let none = MemoryStore::default();
        assert!(resolve(Some("has space"), &none, &none).is_err());
    }

    #[test]
    fn the_key_file_is_written_for_the_person_only_and_read_back() {
        let dir = std::env::temp_dir().join(format!("raul-keyfile-{}", std::process::id()));
        let store = FileStore::in_folder(&dir);
        assert_eq!(store.get().unwrap(), None);
        store.set(&secret("first")).unwrap();
        store.set(&secret("second")).unwrap();
        assert_eq!(store.get().unwrap().unwrap().expose(), "second");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(store.path())
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        store.clear().unwrap();
        store.clear().unwrap();
        assert_eq!(store.get().unwrap(), None);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
