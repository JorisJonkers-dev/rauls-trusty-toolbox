// SPDX-License-Identifier: GPL-3.0-only
//! The toolkit-free state of the Nexus API key on Settings (docs/adr/0024).
//!
//! The page never holds the stored key: it only learns where a key is kept, who it belongs to and when it was last
//! checked, from a document the in-process job prints. What the person types is held in a [`Draft`], whose `Debug`
//! hides it, and is handed to the job once and then dropped.

use crate::pages::settings::Fetch;
use serde::Deserialize;

/// Where the key in use is kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    /// The `NEXUS_API_KEY` environment variable.
    Env,
    /// The operating system's keyring.
    Keyring,
    /// The key file in the configuration folder.
    File,
}

/// What is known about the key without asking Nexus, and what the last check found.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct KeyStatus {
    /// Where the key in use is kept; `None` when there is no key.
    pub source: Option<Source>,
    /// Whether the operating system's keyring can be used here.
    pub keyring_available: bool,
    /// Whether `NEXUS_API_KEY` is set (it is used before a saved key).
    pub env_set: bool,
    /// What kept the key from being read, when something did.
    #[serde(default)]
    pub problem: Option<String>,
    /// The account name the last check found.
    #[serde(default)]
    pub user_name: Option<String>,
    /// Whether that account is premium.
    #[serde(default)]
    pub is_premium: Option<bool>,
    /// When the key was last checked, in seconds since 1970 UTC.
    #[serde(default)]
    pub checked_at: Option<u64>,
}

impl KeyStatus {
    /// Whether a key is set.
    pub fn key_set(&self) -> bool {
        self.source.is_some()
    }

    /// Whether Save keeps the key in a file for want of a keyring (and no variable decides it).
    pub fn saves_to_file(&self) -> bool {
        !self.keyring_available
    }
}

/// Reads the document the status job prints.
///
/// # Errors
/// When `stdout` is not such a document.
pub fn parse(stdout: &str) -> Result<KeyStatus, String> {
    serde_json::from_str(stdout.trim()).map_err(|e| format!("not a key status: {e}"))
}

/// What the person typed: the key. Its `Debug` hides it, so a state that is printed or logged never shows it.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct Draft(pub String);

impl std::fmt::Debug for Draft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.0.is_empty() {
            "Draft(empty)"
        } else {
            "Draft([hidden])"
        })
    }
}

impl Draft {
    /// Whether nothing is typed.
    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }

    /// Takes the typed key out, leaving the field empty.
    pub fn take(&mut self) -> String {
        std::mem::take(&mut self.0)
    }
}

/// What is running on the key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Op {
    /// Nothing.
    #[default]
    Idle,
    /// The typed key is being stored.
    Saving,
    /// The stored key is being removed.
    Clearing,
    /// Nexus is being asked who the key belongs to.
    Checking,
}

/// The state of the key field and its status.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct State {
    /// The status, as last read.
    pub status: Fetch<KeyStatus>,
    /// The typed key.
    pub draft: Draft,
    /// What runs.
    pub op: Op,
    /// Why the last Save, Clear or Check did not go through; the plain line of the job.
    pub failed: Option<String>,
}

impl State {
    /// A key section showing `status`.
    pub fn loaded(status: KeyStatus) -> State {
        State {
            status: Fetch::Ready(status),
            ..State::default()
        }
    }

    /// Whether a Save, Clear or Check runs.
    pub fn busy(&self) -> bool {
        self.op != Op::Idle
    }
}

/// A UTC time as `2026-11-27 14:03 UTC`, from seconds since 1970.
pub fn utc_text(seconds: u64) -> String {
    let days = i64::try_from(seconds / 86_400).unwrap_or(0);
    let minute_of_day = (seconds % 86_400) / 60;
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02} UTC",
        minute_of_day / 60,
        minute_of_day % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_status_document_is_read() {
        let s = parse(
            r#"{"source":"keyring","keyring_available":true,"env_set":false,"user_name":"Courier6","is_premium":true,"checked_at":1795788180}"#,
        )
        .unwrap();
        assert_eq!(s.source, Some(Source::Keyring));
        assert!(s.key_set() && !s.saves_to_file());
        assert_eq!(s.user_name.as_deref(), Some("Courier6"));
        let none = parse(r#"{"source":null,"keyring_available":false,"env_set":false}"#).unwrap();
        assert!(!none.key_set() && none.saves_to_file());
        assert!(parse("{}").is_err());
        assert!(parse("").is_err());
    }

    #[test]
    fn a_draft_never_prints_its_key() {
        let d = Draft("abc123secret".into());
        assert!(!format!("{d:?}").contains("abc123"));
        let state = State {
            draft: d,
            ..State::default()
        };
        assert!(!format!("{state:?}").contains("abc123"));
        let mut d = Draft("k".into());
        assert_eq!(d.take(), "k");
        assert!(d.is_empty());
    }

    #[test]
    fn times_are_shown_in_utc() {
        assert_eq!(utc_text(0), "1970-01-01 00:00 UTC");
        assert_eq!(utc_text(1_700_000_000), "2023-11-14 22:13 UTC");
    }
}
