// SPDX-License-Identifier: GPL-3.0-only
//! The toolkit-free state of Home: readiness, what needs attention, and the launch panel (UI-RULES G2, G3).
//!
//! Home owns no data (UI-RULES P3): it reads the paths of Settings and the engine of Versions and shows them, each
//! row one click from the page that fixes it. Play is blocked only for a blocking problem, and the reason says
//! which. Play runs `raul run nv-rs`, which refuses a game run without the memory-capped run guard.

use super::settings::{Command, Config, Fetch, Key};
use super::versions::{Engine, EngineStatus};
use super::{PageId, Severity, View};

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
            RowId::Engine => View::Page(PageId::Versions),
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
            Block::NoEngine | Block::EngineDiffers => View::Page(PageId::Versions),
        }
    }
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

/// `raul run nv-rs --instance <dir>`: the game on nv-rs, through the memory-capped run guard.
pub fn run_command(instance: &str) -> Command {
    Command::read("run", 1, &["run", "nv-rs", "--instance", instance])
}

/// The state of Home.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct State {
    /// The paths, from Settings.
    pub config: Fetch<Config>,
    /// The engine, as Versions reads it.
    pub engine: Fetch<Engine>,
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
            ..State::default()
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

    /// Whether Play can start the game now: nothing blocks, the reads are in and no launch runs.
    pub fn can_play(&self) -> bool {
        self.block().is_none()
            && self.engine.ready().is_some()
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
        assert_eq!(Block::NoEngine.fix(), View::Page(PageId::Versions));
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
}
