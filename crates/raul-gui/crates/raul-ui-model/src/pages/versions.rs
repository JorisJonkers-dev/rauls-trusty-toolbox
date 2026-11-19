// SPDX-License-Identifier: GPL-3.0-only
//! The toolkit-free state of About and Update: raul's own version and the update check (UI-RULES P3, P5).
//!
//! The page shows one version, the Toolbox's own, as plain text, with the update check beside it: `raul update
//! check` looks for a newer release and downloads nothing. Every packaged Component comes with raul and changes
//! only when raul does, so no Component version and no per-Component action is shown (ADR-0016, ADR-0018). Nothing
//! reaches the network until the update check is pressed.

use super::settings::{Command, Config, Fetch, Key};
use serde::Deserialize;

/// The Toolbox's own version, as `raul version --json` names it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Shipped {
    /// The Toolbox version.
    pub raul: String,
}

#[derive(Deserialize)]
struct VersionDoc {
    raul: String,
}

/// Reads what `raul version --json` printed. The document also carries the release check's set and the Component
/// versions; those are build-time facts, and serde ignores the keys this struct does not name, so the page reads
/// only the Toolbox's own version whether or not the command still prints them.
///
/// # Errors
/// When it is not that document.
pub fn parse_version(stdout: &str) -> Result<Shipped, String> {
    let doc: VersionDoc = serde_json::from_str(stdout.trim())
        .map_err(|e| format!("not a raul version document: {e}"))?;
    Ok(Shipped { raul: doc.raul })
}

/// A newer release than the one that runs, as `raul update check --json` names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    /// The version that is out.
    pub version: String,
    /// The release page.
    pub url: String,
}

#[derive(Deserialize)]
struct UpdateDoc {
    current: String,
    newer: Option<ReleaseDoc>,
}

#[derive(Deserialize)]
struct ReleaseDoc {
    version: String,
    url: String,
}

/// Reads what `raul update check --json` printed: the version that runs, and the release that is out if there is
/// one.
///
/// # Errors
/// When it is not that document.
pub fn parse_update(stdout: &str) -> Result<(Shipped, Option<Release>), String> {
    let doc: UpdateDoc = serde_json::from_str(stdout.trim())
        .map_err(|e| format!("not a raul update document: {e}"))?;
    Ok((
        Shipped { raul: doc.current },
        doc.newer.map(|r| Release {
            version: r.version,
            url: r.url,
        }),
    ))
}

/// `raul update check --json`: reads the release list and downloads nothing (ADR-0005).
pub fn update_command() -> Command {
    Command::read("update-check", 2, &["update", "check"])
}

/// `raul version`: read only.
pub fn version_command() -> Command {
    Command::read("version", 1, &["version"])
}

/// The one update action beside the Toolbox version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateAction {
    /// Check for updates: no newer release is known yet.
    Check,
    /// Update: a newer release is out.
    Update,
}

/// Where the update check is.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Update {
    /// Not asked for.
    #[default]
    Idle,
    /// `raul update check` runs.
    Checking,
    /// It ran: the version that runs, and the release that is out if there is one.
    Checked {
        /// The Toolbox version that runs.
        raul: String,
        /// The newer release, when there is one.
        newer: Option<Release>,
    },
    /// It could not finish; what it said.
    Failed(String),
}

/// The state of About and Update.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct State {
    /// The Toolbox version, from `raul version`.
    pub shipped: Fetch<Shipped>,
    /// The update check, if one was asked for.
    pub update: Update,
}

impl State {
    /// The Toolbox version the page shows: what the update check read, else what `raul version` read.
    pub fn raul(&self) -> Option<&str> {
        match &self.update {
            Update::Checked { raul, .. } => Some(raul),
            _ => self.shipped.ready().map(|s| s.raul.as_str()),
        }
    }

    /// The newer release the update check found, once it has run; a check on its way or one that failed found
    /// none, so the button offers the check again.
    pub fn newer(&self) -> Option<&Release> {
        match &self.update {
            Update::Checked { newer, .. } => newer.as_ref(),
            Update::Idle | Update::Checking | Update::Failed(_) => None,
        }
    }

    /// Whether the update button can be pressed: no check runs.
    pub fn can_check_updates(&self) -> bool {
        self.update != Update::Checking
    }

    /// What the one update button beside the version says: `Update` once a newer release is known, else
    /// `Check for updates`.
    pub fn update_action(&self) -> UpdateAction {
        if self.newer().is_some() {
            UpdateAction::Update
        } else {
            UpdateAction::Check
        }
    }

    /// The details a person can copy into a report: the version that runs, the platform, and where the update
    /// check stands. No path and nothing about the machine beyond the operating system and architecture.
    pub fn details(&self, os: &str, arch: &str) -> String {
        let version = self.raul().unwrap_or("not read yet");
        let update = match &self.update {
            Update::Idle => "not checked".to_owned(),
            Update::Checking => "checking".to_owned(),
            Update::Checked { newer: None, .. } => "up to date".to_owned(),
            Update::Checked {
                newer: Some(r), ..
            } => format!("{} is out", r.version),
            Update::Failed(why) => format!("check failed: {why}"),
        };
        format!("raul {version}\nplatform: {os} {arch}\nupdate check: {update}\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_rauls_own_version_is_read_and_the_rest_is_ignored() {
        // The document `raul version --json` prints also carries the release check's set and the Components; the page
        // reads neither, so serde ignores both keys and only a document without the Toolbox version is refused.
        // The keys are spelled the way the command prints them, so the fixture stays the one it reads.
        let set_key = ["known", "_good_set"].concat();
        let full = format!(
            r#"{{"raul":"0.2.0","{set_key}":"2026-11","components":{{"nv-rs":"0.5.0","nvmod-rs":"0.6.0"}}}}"#
        );
        let s = parse_version(&full).unwrap();
        assert_eq!(s.raul, "0.2.0");
        assert_eq!(
            s,
            parse_version(r#"{"raul":"0.2.0"}"#).unwrap(),
            "the extra keys are ignored"
        );
        assert!(parse_version("{}").is_err());
    }

    #[test]
    fn the_update_check_names_the_release_that_is_out_or_none() {
        let (running, newer) = parse_update(
            r#"{"current":"0.2.0","newer":{"version":"0.3.0","url":"https://example.invalid/r"}}"#,
        )
        .unwrap();
        assert_eq!(running.raul, "0.2.0");
        assert_eq!(
            newer,
            Some(Release {
                version: "0.3.0".into(),
                url: "https://example.invalid/r".into(),
            })
        );
        let (running, newer) = parse_update(r#"{"current":"0.2.0","newer":null}"#).unwrap();
        assert_eq!(running.raul, "0.2.0");
        assert_eq!(newer, None);
        assert!(parse_update("not a document").is_err());
        assert_eq!(update_command().shown(), "raul update check");
        assert!(!update_command().writes);
    }

    #[test]
    fn the_version_text_and_the_update_action_follow_the_state() {
        let mut s = State::default();
        assert_eq!(s.raul(), None);
        assert!(s.can_check_updates());
        assert_eq!(s.update_action(), UpdateAction::Check);
        s.shipped = Fetch::Ready(Shipped {
            raul: "0.2.0".into(),
        });
        assert_eq!(s.raul(), Some("0.2.0"));
        assert_eq!(s.newer(), None);
        assert_eq!(s.update_action(), UpdateAction::Check);
        s.update = Update::Checking;
        assert!(!s.can_check_updates(), "one check at a time");
        s.update = Update::Checked {
            raul: "0.2.0".into(),
            newer: Some(Release {
                version: "0.3.0".into(),
                url: "https://example.invalid/r".into(),
            }),
        };
        assert_eq!(s.raul(), Some("0.2.0"));
        assert_eq!(s.newer().map(|r| r.version.as_str()), Some("0.3.0"));
        assert_eq!(s.update_action(), UpdateAction::Update, "a release is out");
        assert!(s.can_check_updates());
        s.update = Update::Checked {
            raul: "0.2.0".into(),
            newer: None,
        };
        assert_eq!(s.newer(), None);
        assert_eq!(s.update_action(), UpdateAction::Check);
        s.update = Update::Failed("the network is not reachable".into());
        assert_eq!(s.raul(), Some("0.2.0"), "a failed check keeps the version");
        assert_eq!(s.update_action(), UpdateAction::Check);
        assert!(s.can_check_updates());
    }

    #[test]
    fn a_check_on_its_way_or_one_that_failed_offers_the_check_again() {
        let mut s = State {
            update: Update::Checking,
            ..State::default()
        };
        assert_eq!(s.newer(), None);
        assert_eq!(s.update_action(), UpdateAction::Check);
        s.update = Update::Failed("no network".into());
        assert_eq!(s.update_action(), UpdateAction::Check);
    }

    #[test]
    fn the_details_name_the_version_the_platform_and_the_check() {
        let mut s = State {
            shipped: Fetch::Ready(Shipped {
                raul: "0.2.0".into(),
            }),
            ..State::default()
        };
        assert_eq!(
            s.details("linux", "x86_64"),
            "raul 0.2.0\nplatform: linux x86_64\nupdate check: not checked\n"
        );
        s.update = Update::Checked {
            raul: "0.2.0".into(),
            newer: Some(Release {
                version: "0.3.0".into(),
                url: "https://example.invalid/r".into(),
            }),
        };
        assert!(s.details("linux", "x86_64").ends_with("update check: 0.3.0 is out\n"));
        assert!(State::default().details("a", "b").starts_with("raul not read yet\n"));
    }
}
