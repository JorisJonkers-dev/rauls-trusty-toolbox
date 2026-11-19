// SPDX-License-Identifier: GPL-3.0-only
//! The toolkit-free state of Home: readiness, what needs attention, and the launch panel (UI-RULES G2, G3).
//!
//! Home owns no data (UI-RULES P3): it reads the paths of Settings and the engine and shows them, each
//! row one click from the page that fixes it. Play is blocked only for a blocking problem, and the reason says
//! which. Play runs `raul run nv-rs`.
//!
//! [`State::readiness`] is the one definition of "ready to play": the game folder is found, the storage folder is
//! set, the Instance folder exists, the engine is installed and matches its install record, and the load order is
//! valid. Nothing that Home has not read counts as read: while a read is on its way Home is checking, and when a
//! read cannot be asked for at all it has stopped checking and names what is missing.

use super::load_order;
use super::settings::{Command, Config, Fetch, Key};
use super::{PageId, Severity, View};
use serde::Deserialize;

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

/// A readiness row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RowId {
    /// The New Vegas folder.
    NewVegas,
    /// The Toolbox storage folder, which holds the Instance.
    Storage,
    /// The engine build.
    Engine,
    /// Tale of Two Wastelands: optional, set up in its own flow.
    Ttw,
    /// The plugin order, on Load order.
    LoadOrder,
    /// Distant land, on LOD.
    Lod,
}

impl RowId {
    /// Every row, in the order Home shows them.
    pub const ALL: [RowId; 6] = [
        RowId::NewVegas,
        RowId::Storage,
        RowId::Engine,
        RowId::Ttw,
        RowId::LoadOrder,
        RowId::Lod,
    ];

    /// The view that owns this row's data and fixes it.
    pub fn home(self) -> View {
        match self {
            RowId::NewVegas | RowId::Storage => View::Page(PageId::Settings),
            RowId::Engine => View::Flow(super::Flow::Setup),
            RowId::Ttw => View::Flow(super::Flow::Ttw),
            RowId::LoadOrder => View::Page(PageId::LoadOrder),
            RowId::Lod => View::Page(PageId::Lod),
        }
    }
}

/// How a row stands (UI-RULES C3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Level {
    /// Fine.
    Ok,
    /// Worth a look; does not block.
    Warning,
    /// Blocks Play.
    Problem,
    /// Not set up, and that is allowed.
    NotSet,
    /// Home does not check it; its page does.
    Elsewhere,
    /// Being read.
    Checking,
}

/// What a row shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Which row.
    pub id: RowId,
    /// How it stands.
    pub level: Level,
    /// Why, for the detail line.
    pub why: Why,
}

/// Why a row stands as it does; the page turns it into words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Why {
    /// A path that is set.
    Path(String),
    /// The value is not set.
    NotSet,
    /// Still being read.
    Checking,
    /// The read failed; its message.
    Unreadable(String),
    /// The engine matches its install record.
    EngineActive,
    /// The engine differs from its install record; what verify said.
    EngineDiffers(String),
    /// No engine is installed.
    EngineMissing,
    /// TTW: the Fallout 3 folder and the installer are both set.
    TtwReady,
    /// TTW: something it needs is not set.
    TtwNeedsSources,
    /// Shown and fixed on its own page.
    OnItsPage,
}

/// Why Play is blocked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block {
    /// The settings could not be read.
    Unreadable,
    /// No New Vegas folder.
    NoGame,
    /// No storage folder, so no Instance.
    NoStorage,
    /// No engine installed.
    NoEngine,
    /// The engine differs from its install record.
    EngineDiffers,
}

impl Block {
    /// The page whose control fixes it.
    pub fn fix(self) -> View {
        match self {
            Block::Unreadable | Block::NoGame | Block::NoStorage => View::Page(PageId::Settings),
            Block::NoEngine | Block::EngineDiffers => View::Flow(super::Flow::Setup),
        }
    }
}

/// The first thing that stands between the person and a game that starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// No New Vegas folder.
    Game,
    /// No storage folder, so no Instance.
    Storage,
    /// The Instance folder does not exist yet.
    Instance,
    /// No engine installed, or it differs from its install record.
    Engine,
    /// The load order cannot be read or is not valid.
    LoadOrder,
}

impl Step {
    /// The page that fixes it.
    pub fn fix(self) -> View {
        match self {
            Step::Game | Step::Storage => View::Page(PageId::Settings),
            Step::Instance | Step::Engine => View::Flow(super::Flow::Setup),
            Step::LoadOrder => View::Page(PageId::LoadOrder),
        }
    }
}

/// Whether the game can start, defined in one place: [`State::readiness`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Readiness {
    /// Still reading; nothing is claimed.
    Checking,
    /// Everything holds.
    Ready,
    /// The first missing step.
    Next(Step),
}

/// Where a launch is.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Launch {
    /// Not asked for.
    #[default]
    Idle,
    /// `raul run` runs.
    Starting,
    /// `raul run` started the game and returned.
    Started,
    /// It refused or failed; what it said.
    Failed(String),
}

/// `raul run nv-rs --instance <dir>`: the game on the nv-rs engine, through nvmod's run command, which stages the
/// Profile first.
pub fn run_command(instance: &str) -> Command {
    Command::write("run", 1, &["run", "nv-rs", "--instance", instance])
}

/// `raul plugins list`: Home asks it to know whether the load order is valid.
pub fn load_order_command(instance: &str) -> Command {
    load_order::list_command(instance)
}

/// The state of Home.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct State {
    /// The paths, from Settings.
    pub config: Fetch<Config>,
    /// The engine, as Versions reads it.
    pub engine: Fetch<Engine>,
    /// Whether the Instance folder exists; `None` until known.
    pub instance_exists: Option<bool>,
    /// Whether the Profile's load order can be listed; `Failed` carries why not.
    pub load_order: Fetch<()>,
    /// The launch.
    pub launch: Launch,
    /// Whether Home already opened the setup wizard this session, so it does so once.
    pub offered_setup: bool,
}

impl State {
    /// Home showing `config` and `engine`.
    pub fn ready(config: Config, engine: Engine) -> State {
        State {
            config: Fetch::Ready(config),
            engine: Fetch::Ready(engine),
            instance_exists: Some(true),
            load_order: Fetch::Ready(()),
            ..State::default()
        }
    }

    /// Whether any read is still on its way.
    ///
    /// A settings file that could not be read ends the reading: every other read runs against the paths it holds,
    /// so those reads cannot even be asked for. Waiting for them anyway would leave Home saying "Checking" with no
    /// read running, its Check again disabled forever.
    pub fn reading(&self) -> bool {
        if matches!(self.config, Fetch::Failed(_)) {
            return false;
        }
        if self.config.is_pending() {
            return true;
        }
        // The engine, the Instance folder and the load order are read only when the settings name an Instance;
        // without one they are never asked for, so they are not on their way either.
        let has_instance = self
            .config
            .ready()
            .is_some_and(|c| c.get(Key::Instance).is_some());
        has_instance
            && (self.engine.is_pending()
                || self.instance_exists.is_none()
                || self.load_order.is_pending())
    }

    /// The one definition of "ready to play": the game is found, the storage folder is set, the Instance exists,
    /// the engine is installed and matches its record, and the load order is valid. Otherwise the first missing
    /// step. A step whose read has not been asked for, has not come back, or came back unread is not met.
    pub fn readiness(&self) -> Readiness {
        if self.reading() {
            // A known missing step is still worth naming while other reads run.
            return match self.first_missing() {
                Some(step) => Readiness::Next(step),
                None => Readiness::Checking,
            };
        }
        self.first_missing()
            .map_or(Readiness::Ready, Readiness::Next)
    }

    fn first_missing(&self) -> Option<Step> {
        match &self.config {
            Fetch::Ready(c) => {
                if c.get(Key::NewVegas).is_none() {
                    return Some(Step::Game);
                }
                if c.get(Key::Instance).is_none() {
                    return Some(Step::Storage);
                }
            }
            Fetch::Failed(_) => return Some(Step::Storage),
            _ => return None,
        }
        if self.instance_exists == Some(false) {
            return Some(Step::Instance);
        }
        match &self.engine {
            Fetch::Ready(e) if e.status() != EngineStatus::Active => return Some(Step::Engine),
            Fetch::Failed(_) => return Some(Step::Engine),
            _ => {}
        }
        match self.load_order {
            Fetch::Failed(_) => Some(Step::LoadOrder),
            _ => None,
        }
    }

    fn path_row(&self, id: RowId, key: Key) -> Row {
        let (level, why) = match &self.config {
            Fetch::Idle | Fetch::Loading => (Level::Checking, Why::Checking),
            Fetch::Failed(m) => (Level::Warning, Why::Unreadable(m.clone())),
            Fetch::Ready(c) => match c.get(key) {
                Some(p) => (Level::Ok, Why::Path(p.to_owned())),
                None => (Level::Problem, Why::NotSet),
            },
        };
        Row { id, level, why }
    }

    fn engine_row(&self) -> Row {
        let (level, why) = match &self.engine {
            Fetch::Idle | Fetch::Loading => (Level::Checking, Why::Checking),
            Fetch::Failed(m) => (Level::Warning, Why::Unreadable(m.clone())),
            Fetch::Ready(e) => match e.status() {
                EngineStatus::Active => (Level::Ok, Why::EngineActive),
                EngineStatus::Differs => (Level::Problem, Why::EngineDiffers(e.message.clone())),
                EngineStatus::Missing => (Level::Problem, Why::EngineMissing),
            },
        };
        Row {
            id: RowId::Engine,
            level,
            why,
        }
    }

    fn ttw_row(&self) -> Row {
        let (level, why) = match &self.config {
            Fetch::Idle | Fetch::Loading => (Level::Checking, Why::Checking),
            Fetch::Failed(m) => (Level::Warning, Why::Unreadable(m.clone())),
            Fetch::Ready(c) => {
                if c.get(Key::Fallout3).is_some() && c.get(Key::TtwPackage).is_some() {
                    (Level::NotSet, Why::TtwReady)
                } else {
                    (Level::NotSet, Why::TtwNeedsSources)
                }
            }
        };
        Row {
            id: RowId::Ttw,
            level,
            why,
        }
    }

    /// The readiness rows, in [`RowId::ALL`] order.
    pub fn rows(&self) -> Vec<Row> {
        let elsewhere = |id| Row {
            id,
            level: Level::Elsewhere,
            why: Why::OnItsPage,
        };
        vec![
            self.path_row(RowId::NewVegas, Key::NewVegas),
            self.path_row(RowId::Storage, Key::StorageRoot),
            self.engine_row(),
            self.ttw_row(),
            elsewhere(RowId::LoadOrder),
            elsewhere(RowId::Lod),
        ]
    }

    /// The rows that need attention: problems first, then warnings (UI-RULES G2).
    pub fn attention(&self) -> Vec<Row> {
        let mut rows: Vec<Row> = self
            .rows()
            .into_iter()
            .filter(|r| matches!(r.level, Level::Problem | Level::Warning))
            .collect();
        rows.sort_by_key(|r| r.level != Level::Problem);
        rows
    }

    /// The sidebar chip: how many rows need attention and how urgent the worst is.
    pub fn chip(&self) -> Option<(Severity, u32)> {
        let rows = self.attention();
        let worst = rows.first()?;
        let severity = if worst.level == Level::Problem {
            Severity::Problem
        } else {
            Severity::Warning
        };
        Some((severity, u32::try_from(rows.len()).unwrap_or(u32::MAX)))
    }

    /// Why Play is blocked, if it is; `None` also while Home is still reading (UI-RULES G3: only a known blocking
    /// problem blocks).
    ///
    /// What the game needs to start is [`State::readiness`]'s business: a step no row marks as a problem, a
    /// missing Instance folder or a load order that cannot be read, still keeps Play from starting anything.
    pub fn block(&self) -> Option<Block> {
        match &self.config {
            Fetch::Failed(_) => return Some(Block::Unreadable),
            Fetch::Ready(c) if c.get(Key::NewVegas).is_none() => return Some(Block::NoGame),
            Fetch::Ready(c) if c.get(Key::Instance).is_none() => return Some(Block::NoStorage),
            _ => {}
        }
        match self.engine.ready().map(Engine::status) {
            Some(EngineStatus::Missing) => Some(Block::NoEngine),
            Some(EngineStatus::Differs) => Some(Block::EngineDiffers),
            _ => None,
        }
    }

    /// The command Play runs, once the Instance is known.
    pub fn run_command(&self) -> Option<Command> {
        self.config
            .ready()
            .and_then(|c| c.get(Key::Instance))
            .map(run_command)
    }

    /// Whether Play can start the game now: readiness holds in full, nothing blocks and no launch runs. Play
    /// never starts a game that readiness says cannot start.
    pub fn can_play(&self) -> bool {
        self.block().is_none()
            && self.readiness() == Readiness::Ready
            && self.launch != Launch::Starting
            && self.run_command().is_some()
    }

    /// Whether this is a first run Home should hand to the setup wizard: the settings were read, no New Vegas
    /// folder is set, and the wizard was not offered yet this session.
    pub fn wants_setup(&self) -> bool {
        !self.offered_setup
            && self
                .config
                .ready()
                .is_some_and(|c| c.get(Key::NewVegas).is_none())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_set() -> Config {
        Config::with(
            "/c",
            &[
                (Key::NewVegas, "/games/nv"),
                (Key::StorageRoot, "/s"),
                (Key::Instance, "/s/instance"),
            ],
        )
    }

    #[test]
    fn a_ready_setup_blocks_nothing_and_needs_no_attention() {
        let s = State::ready(all_set(), Engine::active());
        assert_eq!(s.block(), None);
        assert!(s.can_play());
        assert!(s.attention().is_empty());
        assert_eq!(s.chip(), None);
        assert_eq!(
            s.run_command().unwrap().shown(),
            "raul run nv-rs --instance /s/instance"
        );
        assert_eq!(s.rows().len(), RowId::ALL.len());
        assert_eq!(
            run_command("/i").rest(),
            ["nv-rs", "--instance", "/i"],
            "the target is the first value after `raul run`"
        );
    }

    #[test]
    fn each_blocking_problem_names_its_fix() {
        let none = State::ready(Config::with("/c", &[]), Engine::missing());
        assert_eq!(none.block(), Some(Block::NoGame));
        assert_eq!(Block::NoGame.fix(), View::Page(PageId::Settings));
        let no_storage = State::ready(
            Config::with("/c", &[(Key::NewVegas, "/nv")]),
            Engine::missing(),
        );
        assert_eq!(no_storage.block(), Some(Block::NoStorage));
        let no_engine = State::ready(all_set(), Engine::missing());
        assert_eq!(no_engine.block(), Some(Block::NoEngine));
        assert_eq!(Block::NoEngine.fix(), View::Flow(super::super::Flow::Setup));
        let differs = State::ready(
            all_set(),
            Engine {
                installed: true,
                verified: false,
                message: "differs".into(),
            },
        );
        assert_eq!(differs.block(), Some(Block::EngineDiffers));
        assert!(!differs.can_play());
        let unreadable = State {
            config: Fetch::Failed("bad".into()),
            ..State::default()
        };
        assert_eq!(unreadable.block(), Some(Block::Unreadable));
    }

    #[test]
    fn nothing_blocks_while_home_is_still_reading_but_play_waits() {
        let s = State::default();
        assert_eq!(s.block(), None);
        assert!(!s.can_play());
        assert!(s.rows()[..4].iter().all(|r| r.level == Level::Checking));
    }

    #[test]
    fn problems_come_before_warnings_and_set_the_chip() {
        let s = State {
            config: Fetch::Ready(Config::with("/c", &[(Key::StorageRoot, "/s")])),
            engine: Fetch::Failed("verify failed".into()),
            ..State::default()
        };
        let ids: Vec<RowId> = s.attention().iter().map(|r| r.id).collect();
        assert_eq!(ids, [RowId::NewVegas, RowId::Engine]);
        assert_eq!(s.chip(), Some((Severity::Problem, 2)));
    }

    #[test]
    fn ttw_is_optional_and_never_needs_attention() {
        let mut c = all_set();
        let s = State::ready(c.clone(), Engine::active());
        assert_eq!(s.rows()[3].why, Why::TtwNeedsSources);
        c.put(Key::Fallout3, Some("/games/fo3".into()));
        c.put(Key::TtwPackage, Some("/d/ttw.mpi".into()));
        let s = State::ready(c, Engine::active());
        assert_eq!(s.rows()[3].why, Why::TtwReady);
        assert!(s.attention().is_empty());
        assert_eq!(RowId::Ttw.home(), View::Flow(super::super::Flow::Ttw));
    }

    #[test]
    fn the_wizard_is_offered_once_on_a_first_run() {
        let mut s = State::ready(Config::with("/c", &[]), Engine::missing());
        assert!(s.wants_setup());
        s.offered_setup = true;
        assert!(!s.wants_setup());
        assert!(!State::default().wants_setup(), "not before the read");
        assert!(!State::ready(all_set(), Engine::active()).wants_setup());
    }

    #[test]
    fn a_running_launch_is_not_started_twice() {
        let mut s = State::ready(all_set(), Engine::active());
        s.launch = Launch::Starting;
        assert!(!s.can_play());
    }

    #[test]
    fn readiness_is_ready_only_when_every_condition_holds() {
        let s = State::ready(all_set(), Engine::active());
        assert_eq!(s.readiness(), Readiness::Ready);
    }

    #[test]
    fn readiness_names_the_first_missing_step() {
        let ready = || State::ready(all_set(), Engine::active());
        let no_game = State::ready(Config::with("/c", &[]), Engine::active());
        assert_eq!(no_game.readiness(), Readiness::Next(Step::Game));
        let no_storage = State::ready(
            Config::with("/c", &[(Key::NewVegas, "/nv")]),
            Engine::active(),
        );
        assert_eq!(no_storage.readiness(), Readiness::Next(Step::Storage));
        let no_instance = State {
            instance_exists: Some(false),
            ..ready()
        };
        assert_eq!(no_instance.readiness(), Readiness::Next(Step::Instance));
        assert_eq!(Step::Instance.fix(), View::Flow(super::super::Flow::Setup));
        let no_engine = State::ready(all_set(), Engine::missing());
        assert_eq!(no_engine.readiness(), Readiness::Next(Step::Engine));
        let bad_order = State {
            load_order: Fetch::Failed("no profile".into()),
            ..ready()
        };
        assert_eq!(bad_order.readiness(), Readiness::Next(Step::LoadOrder));
        assert_eq!(Step::LoadOrder.fix(), View::Page(PageId::LoadOrder));
        let checking = State {
            load_order: Fetch::Loading,
            ..ready()
        };
        assert_eq!(checking.readiness(), Readiness::Checking);
        assert_eq!(State::default().readiness(), Readiness::Checking);
    }

    /// Everything readiness asks of a setup, so a test can break exactly one condition.
    fn sound() -> State {
        State::ready(all_set(), Engine::active())
    }

    #[test]
    fn not_ready_without_a_game_folder_and_play_does_not_run() {
        let no_game = State::ready(
            Config::with("/c", &[(Key::Instance, "/s/instance")]),
            Engine::active(),
        );
        assert_eq!(no_game.readiness(), Readiness::Next(Step::Game));
        assert_eq!(no_game.first_missing(), Some(Step::Game));
        assert_eq!(no_game.block(), Some(Block::NoGame));
        assert!(!no_game.can_play());
    }

    #[test]
    fn not_ready_without_a_storage_folder_and_play_does_not_run() {
        let no_storage = State::ready(
            Config::with("/c", &[(Key::NewVegas, "/nv")]),
            Engine::active(),
        );
        assert_eq!(no_storage.readiness(), Readiness::Next(Step::Storage));
        assert_eq!(no_storage.first_missing(), Some(Step::Storage));
        assert_eq!(no_storage.block(), Some(Block::NoStorage));
        assert!(!no_storage.can_play());
    }

    #[test]
    fn not_ready_when_the_instance_folder_is_missing_and_play_does_not_run() {
        // The one condition no row marks as a problem, so only readiness can keep Play from starting a game that
        // cannot start. It used to run it anyway.
        let mut no_instance = sound();
        no_instance.instance_exists = Some(false);
        assert_eq!(no_instance.readiness(), Readiness::Next(Step::Instance));
        assert_eq!(no_instance.block(), None);
        assert!(no_instance.attention().is_empty());
        assert!(!no_instance.can_play());
    }

    #[test]
    fn not_ready_when_the_engine_is_not_installed_or_does_not_match_and_play_does_not_run() {
        assert_eq!(sound().readiness(), Readiness::Ready);
        let mut not_installed = sound();
        not_installed.engine = Fetch::Ready(Engine::missing());
        assert_eq!(not_installed.readiness(), Readiness::Next(Step::Engine));
        assert!(!not_installed.can_play());
        let differs = State::ready(
            all_set(),
            Engine {
                installed: true,
                verified: false,
                message: "differs".into(),
            },
        );
        assert_eq!(differs.readiness(), Readiness::Next(Step::Engine));
        assert!(!differs.can_play());
    }

    #[test]
    fn not_ready_when_the_engine_could_not_be_read_and_play_does_not_run() {
        let mut unread_engine = sound();
        unread_engine.engine = Fetch::Failed("the install record could not be read".into());
        assert_eq!(unread_engine.readiness(), Readiness::Next(Step::Engine));
        assert!(!unread_engine.can_play());
    }

    #[test]
    fn not_ready_when_the_load_order_cannot_be_read_and_play_does_not_run() {
        let mut unread_order = sound();
        unread_order.load_order = Fetch::Failed("the load order could not be read".into());
        assert_eq!(unread_order.readiness(), Readiness::Next(Step::LoadOrder));
        assert_eq!(unread_order.block(), None);
        assert!(!unread_order.can_play());
    }

    #[test]
    fn not_ready_while_the_load_order_has_not_been_read_and_play_does_not_run() {
        // An order that was never asked for, or is still on its way, is not an order that is valid. Play used to
        // start the game on the strength of the engine alone.
        for order in [Fetch::Idle, Fetch::Loading] {
            let mut unread = sound();
            unread.load_order = order;
            assert!(unread.reading());
            assert_eq!(unread.readiness(), Readiness::Checking);
            assert!(!unread.can_play());
        }
    }

    #[test]
    fn home_stops_reading_when_the_settings_could_not_be_read() {
        // The paths every other read runs against live in the settings, so without them nothing is on its way
        // any more. Home used to wait forever: "Checking" with no read running and Check again disabled.
        let mut unreadable = sound();
        unreadable.config = Fetch::Failed("the settings file could not be read".into());
        unreadable.engine = Fetch::Idle;
        unreadable.load_order = Fetch::Idle;
        assert!(!unreadable.reading());
        assert_eq!(unreadable.readiness(), Readiness::Next(Step::Storage));
        assert_eq!(unreadable.block(), Some(Block::Unreadable));
        assert!(!unreadable.can_play());
    }

    #[test]
    fn home_stops_reading_when_the_settings_hold_no_paths() {
        // A settings answer that read as empty is a read that came back; the other reads were never asked for.
        let mut no_paths = sound();
        no_paths.config = Fetch::Ready(Config::with("/c", &[]));
        no_paths.engine = Fetch::Idle;
        no_paths.load_order = Fetch::Idle;
        assert!(!no_paths.reading());
        assert_eq!(no_paths.readiness(), Readiness::Next(Step::Game));
        assert!(!no_paths.can_play());
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
}
