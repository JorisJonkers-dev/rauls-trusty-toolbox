// SPDX-License-Identifier: GPL-3.0-only
//! The toolkit-free state of Versions: the engine build and the Component versions (UI-RULES P3, P5).
//!
//! Versions installs the nv-rs release with `raul install-nv-rs --release` and shows the build that is active: what
//! `raul verify` says of the installed nv-rs against its install record. The Components are the Known-good set
//! this `raul` ships, from `raul version`. Nothing here holds a path; the Instance comes from Settings. Nothing
//! reaches the network until Install is pressed and confirmed.

use super::settings::{Command, Config, Fetch, Key};
use serde::Deserialize;

/// The Components this `raul` ships (`raul version --json`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Shipped {
    /// The Toolbox version.
    pub raul: String,
    /// The Known-good set.
    pub set: String,
    /// Each Component and its version, in the order printed.
    pub components: Vec<(String, String)>,
}

impl Shipped {
    /// The version of `component`, if the set has it.
    pub fn version_of(&self, component: &str) -> Option<&str> {
        self.components
            .iter()
            .find(|(c, _)| c == component)
            .map(|(_, v)| v.as_str())
    }
}

#[derive(Deserialize)]
struct VersionDoc {
    raul: String,
    known_good_set: String,
    components: serde_json::Map<String, serde_json::Value>,
}

/// Reads what `raul version --json` printed.
///
/// # Errors
/// When it is not that document.
pub fn parse_version(stdout: &str) -> Result<Shipped, String> {
    let doc: VersionDoc = serde_json::from_str(stdout.trim())
        .map_err(|e| format!("not a raul version document: {e}"))?;
    Ok(Shipped {
        raul: doc.raul,
        set: doc.known_good_set,
        components: doc
            .components
            .into_iter()
            .map(|(c, v)| (c, v.as_str().unwrap_or_default().to_owned()))
            .collect(),
    })
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

/// The state of Versions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct State {
    /// The configuration, for the Instance.
    pub config: Fetch<Config>,
    /// The installed engine.
    pub engine: Fetch<Engine>,
    /// The shipped Components.
    pub shipped: Fetch<Shipped>,
    /// The install, if one was asked for.
    pub install: Install,
}

impl State {
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
    fn the_shipped_set_is_read_in_order() {
        let s = parse_version(
            r#"{"raul":"0.2.0","known_good_set":"2026-11","components":{"nv-rs":"0.5.0","nvmod-rs":"0.6.0"}}"#,
        )
        .unwrap();
        assert_eq!(s.raul, "0.2.0");
        assert_eq!(s.set, "2026-11");
        assert_eq!(s.version_of("nv-rs"), Some("0.5.0"));
        assert_eq!(s.version_of("loot-rs"), None);
        assert!(parse_version("{}").is_err());
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
    }
}
