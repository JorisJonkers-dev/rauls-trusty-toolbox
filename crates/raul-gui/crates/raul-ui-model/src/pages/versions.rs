// SPDX-License-Identifier: GPL-3.0-only
//! The toolkit-free state of About and Update: raul's own version and the update check (UI-RULES P3, P5).
//!
//! The page shows one version, the Toolbox's own, as plain text, with the update check beside it: `raul update
//! check` looks for a newer release and downloads nothing. Every packaged Component comes with raul and changes
//! only when raul does, so no Component version and no per-Component action is shown (ADR-0016, ADR-0018). Nothing
//! reaches the network until the update check is pressed.

use super::settings::{Command, Fetch};
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

/// What this install can do about updates, as `raul update status --json` names it. Nothing here reaches the network.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
pub struct UpdateStatus {
    /// Whether this install can be updated from inside the window.
    pub updatable: bool,
    /// When it cannot: why, in plain words.
    #[serde(default)]
    pub reason: Option<String>,
    /// A version that was downloaded and is ready to install.
    #[serde(default)]
    pub staged: Option<String>,
    /// The version before the last update, which going back restores.
    #[serde(default)]
    pub previous: Option<String>,
}

#[derive(Deserialize)]
struct StatusDoc {
    status: UpdateStatus,
}

/// Reads what `raul update status --json` printed.
///
/// # Errors
/// When it is not that document.
pub fn parse_status(stdout: &str) -> Result<UpdateStatus, String> {
    serde_json::from_str::<StatusDoc>(stdout.trim())
        .map(|d| d.status)
        .map_err(|e| format!("not an update status document: {e}"))
}

/// Reads the version a finished download (`staged`) or install (`installed`) names in its `--json` answer.
///
/// # Errors
/// When the answer is not that document.
pub fn parse_applied(stdout: &str, key: &str) -> Result<String, String> {
    let doc: serde_json::Value =
        serde_json::from_str(stdout.trim()).map_err(|e| format!("not an update document: {e}"))?;
    // A download or an install names the version as `{"to": ...}`, a rollback as a plain string.
    doc[key]["to"]
        .as_str()
        .or_else(|| doc[key].as_str())
        .map(str::to_owned)
        .ok_or_else(|| format!("the update document names no {key} version"))
}

/// `raul update status`: what this install can do about updates; local, nothing is fetched.
pub fn status_command() -> Command {
    Command::read("update-status", 2, &["update", "status"])
}

/// `raul update download --tag v<version>`: downloads that release, checks it and gets it ready. The one step that
/// downloads, and only on a press.
pub fn download_command(version: &str) -> Command {
    Command::write(
        "update-download",
        2,
        &["update", "download", "--tag", &format!("v{version}")],
    )
}

/// `raul update install`: installs what was downloaded and checks the new version, going back if it does not work.
pub fn install_update_command() -> Command {
    Command::write("update-install", 2, &["update", "install"])
}

/// `raul update rollback`: goes back to the version before the last update.
pub fn rollback_command() -> Command {
    Command::write("update-rollback", 2, &["update", "rollback"])
}

/// Where applying an update is.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Apply {
    /// Nothing is under way.
    #[default]
    Idle,
    /// A version is being downloaded and checked.
    Downloading,
    /// A version is downloaded and checked, ready to install.
    Ready(String),
    /// The downloaded version is being installed and checked.
    Installing,
    /// The version is installed; starting raul again uses it.
    Installed(String),
    /// The version before is being restored.
    RollingBack,
    /// The version before is back; starting raul again uses it.
    WentBack(String),
    /// A step did not work; what it said. Nothing is half done.
    Failed(String),
}

/// The step the page offers next beside the version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// Download this version.
    Download(String),
    /// Install the downloaded version.
    Install,
    /// Start raul again.
    Restart,
}

/// The state of About and Update.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct State {
    /// The Toolbox version, from `raul version`.
    pub shipped: Fetch<Shipped>,
    /// The update check, if one was asked for.
    pub update: Update,
    /// What this install can do about updates, read when the page opens.
    pub status: Fetch<UpdateStatus>,
    /// Where downloading, installing and going back are.
    pub apply: Apply,
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

    /// Whether a download, an install or a rollback is running.
    pub fn busy(&self) -> bool {
        matches!(
            self.apply,
            Apply::Downloading | Apply::Installing | Apply::RollingBack
        )
    }

    /// Why this install cannot be updated from the window, when that is known.
    pub fn not_updatable(&self) -> Option<&str> {
        match &self.status {
            Fetch::Ready(s) if !s.updatable => Some(s.reason.as_deref().unwrap_or("")),
            _ => None,
        }
    }

    fn updatable(&self) -> bool {
        matches!(&self.status, Fetch::Ready(s) if s.updatable)
    }

    /// The step offered next: install what is downloaded, start raul again after an install or a rollback, or download
    /// the newer release the check found. `None` while a step runs, when this install cannot update itself, and when
    /// there is nothing to do.
    pub fn next_step(&self) -> Option<Step> {
        if self.busy() || !self.updatable() {
            return None;
        }
        match &self.apply {
            Apply::Ready(_) => Some(Step::Install),
            Apply::Installed(_) | Apply::WentBack(_) => Some(Step::Restart),
            Apply::Idle | Apply::Failed(_) => {
                self.newer().map(|r| Step::Download(r.version.clone()))
            }
            Apply::Downloading | Apply::Installing | Apply::RollingBack => None,
        }
    }

    /// The version a person may go back to: the one before the last update, offered unless a step runs, the new version
    /// was only just installed (restart first) or the old one was only just restored.
    pub fn go_back_to(&self) -> Option<&str> {
        if self.busy()
            || !self.updatable()
            || matches!(self.apply, Apply::Installed(_) | Apply::WentBack(_))
        {
            return None;
        }
        match &self.status {
            Fetch::Ready(s) => s.previous.as_deref(),
            _ => None,
        }
    }

    /// Settle `apply` from what the status says on first read: a download that is already there is ready to install.
    pub fn settle_from_status(&mut self) {
        if let (Apply::Idle, Fetch::Ready(s)) = (&self.apply, &self.status)
            && let Some(v) = &s.staged
        {
            self.apply = Apply::Ready(v.clone());
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
            Update::Checked { newer: Some(r), .. } => format!("{} is out", r.version),
            Update::Failed(why) => format!("check failed: {why}"),
        };
        format!("Toolbox version: {version}\nPlatform: {os} {arch}\nUpdate check: {update}\n")
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
            "Toolbox version: 0.2.0\nPlatform: linux x86_64\nUpdate check: not checked\n"
        );
        s.update = Update::Checked {
            raul: "0.2.0".into(),
            newer: Some(Release {
                version: "0.3.0".into(),
                url: "https://example.invalid/r".into(),
            }),
        };
        assert!(
            s.details("linux", "x86_64")
                .ends_with("Update check: 0.3.0 is out\n")
        );
        assert!(
            State::default()
                .details("a", "b")
                .starts_with("Toolbox version: not read yet\n")
        );
    }

    #[test]
    fn the_status_and_the_answers_of_each_step_are_read() {
        let s = parse_status(
            r#"{"current":"0.2.0","status":{"updatable":true,"reason":null,"staged":"0.3.0","previous":"0.1.0","confirmed":false}}"#,
        )
        .unwrap();
        assert_eq!(
            (s.updatable, s.staged.as_deref(), s.previous.as_deref()),
            (true, Some("0.3.0"), Some("0.1.0"))
        );
        let s =
            parse_status(r#"{"status":{"updatable":false,"reason":"Nix updates it."}}"#).unwrap();
        assert_eq!(
            (s.updatable, s.reason.as_deref()),
            (false, Some("Nix updates it."))
        );
        assert!(parse_status("nope").is_err());
        assert_eq!(
            parse_applied(r#"{"ok":true,"staged":{"to":"0.3.0"}}"#, "staged").unwrap(),
            "0.3.0"
        );
        assert_eq!(
            parse_applied(r#"{"ok":true,"installed":{"to":"0.3.0"}}"#, "installed").unwrap(),
            "0.3.0"
        );
        assert_eq!(
            parse_applied(r#"{"ok":true,"back_to":"0.1.0"}"#, "back_to").unwrap(),
            "0.1.0"
        );
        assert!(parse_applied(r#"{"ok":false}"#, "staged").is_err());
    }

    #[test]
    fn the_commands_of_each_step_are_named_and_only_download_names_a_release() {
        assert_eq!(
            download_command("0.3.0").shown(),
            "raul update download --tag v0.3.0"
        );
        assert!(download_command("0.3.0").writes);
        assert_eq!(download_command("0.3.0").rest(), ["--tag", "v0.3.0"]);
        assert_eq!(install_update_command().shown(), "raul update install");
        assert_eq!(rollback_command().shown(), "raul update rollback");
        assert!(
            !status_command().writes
                && install_update_command().writes
                && rollback_command().writes
        );
    }

    fn updatable(previous: Option<&str>, staged: Option<&str>) -> Fetch<UpdateStatus> {
        Fetch::Ready(UpdateStatus {
            updatable: true,
            reason: None,
            staged: staged.map(str::to_owned),
            previous: previous.map(str::to_owned),
        })
    }

    fn found() -> Update {
        Update::Checked {
            raul: "0.2.0".into(),
            newer: Some(Release {
                version: "0.3.0".into(),
                url: "u".into(),
            }),
        }
    }

    #[test]
    fn the_steps_follow_each_other_one_press_at_a_time() {
        let mut s = State {
            update: found(),
            status: updatable(None, None),
            ..State::default()
        };
        assert_eq!(s.next_step(), Some(Step::Download("0.3.0".into())));
        s.apply = Apply::Downloading;
        assert!(s.busy() && s.next_step().is_none() && s.go_back_to().is_none());
        s.apply = Apply::Ready("0.3.0".into());
        assert_eq!(s.next_step(), Some(Step::Install));
        s.apply = Apply::Installing;
        assert!(s.next_step().is_none());
        s.apply = Apply::Installed("0.3.0".into());
        assert_eq!(s.next_step(), Some(Step::Restart));
        s.apply = Apply::Failed("did not work".into());
        assert_eq!(
            s.next_step(),
            Some(Step::Download("0.3.0".into())),
            "a failed step can be tried again"
        );
    }

    #[test]
    fn nothing_is_offered_when_this_install_cannot_update_itself_or_nothing_is_newer() {
        let mut s = State {
            update: found(),
            status: Fetch::Ready(UpdateStatus {
                updatable: false,
                reason: Some("Nix updates it.".into()),
                ..UpdateStatus::default()
            }),
            ..State::default()
        };
        assert_eq!(s.next_step(), None);
        assert_eq!(s.not_updatable(), Some("Nix updates it."));
        s.status = Fetch::Loading;
        assert_eq!(
            (s.next_step(), s.not_updatable()),
            (None, None),
            "nothing is offered before the status is known"
        );
        s.status = updatable(None, None);
        s.update = Update::Checked {
            raul: "0.2.0".into(),
            newer: None,
        };
        assert_eq!(s.next_step(), None);
    }

    #[test]
    fn going_back_is_offered_while_a_previous_version_is_kept_and_nothing_runs() {
        let mut s = State {
            status: updatable(Some("0.1.0"), None),
            ..State::default()
        };
        assert_eq!(s.go_back_to(), Some("0.1.0"));
        s.apply = Apply::Installed("0.2.0".into());
        assert_eq!(s.go_back_to(), None, "restart first");
        s.apply = Apply::RollingBack;
        assert_eq!(s.go_back_to(), None);
        s.apply = Apply::WentBack("0.1.0".into());
        assert_eq!((s.go_back_to(), s.next_step()), (None, Some(Step::Restart)));
        s.apply = Apply::Idle;
        s.status = updatable(None, None);
        assert_eq!(s.go_back_to(), None, "nothing is kept");
    }

    #[test]
    fn a_download_that_is_already_there_is_ready_to_install() {
        let mut s = State {
            status: updatable(None, Some("0.3.0")),
            ..State::default()
        };
        s.settle_from_status();
        assert_eq!(s.apply, Apply::Ready("0.3.0".into()));
        assert_eq!(s.next_step(), Some(Step::Install));
        let mut s = State {
            apply: Apply::Downloading,
            status: updatable(None, Some("0.3.0")),
            ..State::default()
        };
        s.settle_from_status();
        assert_eq!(
            s.apply,
            Apply::Downloading,
            "a step that runs is not overruled"
        );
    }
}
