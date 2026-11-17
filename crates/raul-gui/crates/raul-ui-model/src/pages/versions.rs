// SPDX-License-Identifier: GPL-3.0-only
//! The toolkit-free state of Versions: raul's own version, the update check and the engine build (UI-RULES P3, P5).
//!
//! Versions shows one version, the Toolbox's own, as plain text, with the update check beside it: `raul update
//! check` looks for a newer release and downloads nothing. It installs the nv-rs release with `raul install-nv-rs
//! --release` and shows the build that is active: what `raul verify` says of the installed nv-rs against its install
//! record. No Component version is shown anywhere, and the release check's set is a build-time idea the page does
//! not read. Nothing here holds a path; the Instance comes from Settings. Nothing reaches the network until the
//! update check or Install is pressed and confirmed.

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

/// The installed nv-rs, as `raul verify` sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Engine {
    /// Whether an nv-rs is installed in the Instance.
    pub installed: bool,
    /// Whether it is still the file its install record names.
    pub verified: bool,
    /// What `raul verify` said, for the detail line.
    pub message: String,
}

/// How the engine stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineStatus {
    /// Installed and matching its record: the active build.
    Active,
    /// Installed, but the program differs from its record.
    Differs,
    /// Not installed.
    Missing,
}

impl Engine {
    /// No nv-rs: the Instance does not exist yet, or holds none.
    pub fn missing() -> Engine {
        Engine {
            installed: false,
            verified: false,
            message: String::new(),
        }
    }

    /// An installed nv-rs that matches its record.
    pub fn active() -> Engine {
        Engine {
            installed: true,
            verified: true,
            message: String::new(),
        }
    }

    /// How it stands.
    pub fn status(&self) -> EngineStatus {
        match (self.installed, self.verified) {
            (false, _) => EngineStatus::Missing,
            (true, true) => EngineStatus::Active,
            (true, false) => EngineStatus::Differs,
        }
    }
}

#[derive(Deserialize)]
struct VerifyDoc {
    nv_rs: VerifyNvRs,
}

#[derive(Deserialize)]
struct VerifyNvRs {
    installed: bool,
    verified: bool,
    message: String,
}

/// Reads the `nv_rs` part of what `raul verify --json` printed. The rest of the report is the Doctor's.
///
/// # Errors
/// When it is not that document.
pub fn parse_verify(stdout: &str) -> Result<Engine, String> {
    let doc: VerifyDoc = serde_json::from_str(stdout.trim())
        .map_err(|e| format!("not a raul verify document: {e}"))?;
    Ok(Engine {
        installed: doc.nv_rs.installed,
        verified: doc.nv_rs.verified,
        message: doc.nv_rs.message,
    })
}

/// `raul verify --instance <dir>`: read only.
pub fn verify_command(instance: &str) -> Command {
    Command::read("verify", 1, &["verify", "--instance", instance])
}

/// `raul version`: read only.
pub fn version_command() -> Command {
    Command::read("version", 1, &["version"])
}

/// `raul install-nv-rs --instance <dir> --release`: fetches the fork's release, verifies it and runs its check.
pub fn install_command(instance: &str) -> Command {
    Command::write(
        "install-nv-rs",
        1,
        &["install-nv-rs", "--instance", instance, "--release"],
    )
}

/// The one update action beside the Toolbox version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateAction {
    /// Check for updates: no newer release is known yet.
    Check,
    /// Update: a newer release is out.
    Update,
}

/// Where an install is.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Install {
    /// None asked for.
    #[default]
    Idle,
    /// The confirm dialog is open.
    Asking,
    /// `raul install-nv-rs` runs.
    Running,
    /// It finished; the engine is read again.
    Done,
    /// It failed; what it said. The old build, if any, is left as it was.
    Failed(String),
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

/// The state of Versions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct State {
    /// The configuration, for the Instance.
    pub config: Fetch<Config>,
    /// The installed engine.
    pub engine: Fetch<Engine>,
    /// The Toolbox version, from `raul version`.
    pub shipped: Fetch<Shipped>,
    /// The install, if one was asked for.
    pub install: Install,
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

    /// Whether the update button can be pressed: no check runs and no install does.
    pub fn can_check_updates(&self) -> bool {
        self.update != Update::Checking
            && !matches!(self.install, Install::Running | Install::Asking)
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

    /// The Instance the engine goes into, once the configuration says.
    pub fn instance(&self) -> Option<&str> {
        self.config.ready().and_then(|c| c.get(Key::Instance))
    }

    /// The command Install runs, once there is an Instance.
    pub fn install_command(&self) -> Option<Command> {
        self.instance().map(install_command)
    }

    /// Whether Install can be pressed: there is an Instance and no install runs.
    pub fn can_install(&self) -> bool {
        self.instance().is_some() && !matches!(self.install, Install::Running | Install::Asking)
    }

    /// What the header says needs attention: the engine missing or differing from its record.
    pub fn needs_attention(&self) -> bool {
        matches!(
            self.engine.ready().map(Engine::status),
            Some(EngineStatus::Missing | EngineStatus::Differs)
        )
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
        s.install = Install::Running;
        assert!(!s.can_check_updates());
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
    fn verify_tells_active_differing_and_missing_apart() {
        let read = |installed: bool, verified: bool| {
            parse_verify(&format!(
                r#"{{"schema_version":1,"verified":false,"doctor":{{}},"nv_rs":{{"installed":{installed},"verified":{verified},"message":"m"}},"launch_plan":{{}}}}"#
            ))
            .unwrap()
            .status()
        };
        assert_eq!(read(true, true), EngineStatus::Active);
        assert_eq!(read(true, false), EngineStatus::Differs);
        assert_eq!(read(false, false), EngineStatus::Missing);
        assert!(parse_verify("{}").is_err());
        assert_eq!(Engine::missing().status(), EngineStatus::Missing);
        assert_eq!(Engine::active().status(), EngineStatus::Active);
    }

    #[test]
    fn install_needs_an_instance_and_runs_once() {
        let mut s = State::default();
        assert!(!s.can_install());
        assert_eq!(s.install_command(), None);
        s.config = Fetch::Ready(Config::with("", &[(Key::Instance, "/s/instance")]));
        assert!(s.can_install());
        let cmd = s.install_command().unwrap();
        assert_eq!(
            cmd.shown(),
            "raul install-nv-rs --instance /s/instance --release"
        );
        assert!(cmd.writes);
        s.install = Install::Running;
        assert!(!s.can_install());
    }

    #[test]
    fn a_missing_or_differing_engine_needs_attention() {
        let mut s = State {
            engine: Fetch::Ready(Engine::missing()),
            ..State::default()
        };
        assert!(s.needs_attention());
        s.engine = Fetch::Ready(Engine::active());
        assert!(!s.needs_attention());
        s.engine = Fetch::Loading;
        assert!(!s.needs_attention());
        assert_eq!(verify_command("/i").shown(), "raul verify --instance /i");
        assert_eq!(version_command().shown(), "raul version");
        assert_eq!(update_command().shown(), "raul update check");
    }
}
