// SPDX-License-Identifier: GPL-3.0-only
//! The one list of checks of the app (TICKETS CORE-02).
//!
//! [`checks`] is a pure function from [`SetupState`] (plain facts) to a list of [`Check`]s. The sidebar [`badges`],
//! the Play verdict ([`play_verdict`]), Home's setup rows and the Doctor's items are views of that list, so they
//! cannot disagree. The rules follow the clickable demo (`docs/design/demo/src/core.js` `checks`, `pages.js`
//! `sideHTML`) and FUNCTIONALITY F-SHELL-05, F-HOME-04, F-DOCTOR-02, F-MODS-11, F-PLAY-01 to 03.

use crate::detect::GameCopy;
use crate::space::Space;
use serde::{Deserialize, Serialize};

/// Free space below which the Toolbox storage check warns (the demo's 20 GB).
pub const LOW_STORAGE_BYTES: u64 = 20 * 1024 * 1024 * 1024;
/// A load order with more plugins than this warns (the engine's limit is 255).
pub const PLUGIN_WARN_ABOVE: usize = 230;
/// The engine's plugin limit.
pub const PLUGIN_LIMIT: usize = 255;

/// Where the game folder stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    /// Found, and the executable is the supported version (or its version is not known).
    Found,
    /// Found, but the executable is not the supported version.
    UnsupportedVersion {
        /// The version found, when known.
        version: Option<String>,
    },
    /// No game folder.
    NotFound,
}

/// Where the engine stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineState {
    /// All engine files are as installed.
    Ok,
    /// Engine files changed.
    Damaged {
        /// How many files changed.
        files_changed: usize,
    },
}

/// Tale of Two Wastelands, from the profile's point of view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TtwState {
    /// The profile does not use it.
    NotUsed,
    /// The profile uses it but it is not added yet.
    NotAdded,
    /// Added.
    Added,
}

/// Distant land (LOD) of the profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LodState {
    /// Built from this setup.
    UpToDate,
    /// A build is running.
    Building,
    /// Never built.
    NotBuilt,
    /// Built, but the quality setting changed since.
    StaleQuality,
    /// Built, but the mods changed since.
    StaleMods,
}

/// A mod that needs attention.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModAttention {
    /// The mod's name.
    pub name: String,
    /// What is wrong.
    pub reason: ModReason,
}

/// Why a mod needs attention.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModReason {
    /// One of its plugins needs a master that is not there.
    MissingMaster {
        /// The master it needs.
        master: String,
    },
    /// Anything else, in plain words.
    Other(String),
}

/// A plugin whose master is missing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingMaster {
    /// The plugin file.
    pub plugin: String,
    /// The master it needs, when the source names it as data (the Doctor's report carries it in its message only).
    pub master: Option<String>,
}

/// The facts the checks need, as plain data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetupState {
    /// The game folder.
    pub game: GameState,
    /// Free bytes on the Toolbox storage; `None` when not known (not a problem).
    pub storage_free_bytes: Option<u64>,
    /// The engine.
    pub engine: EngineState,
    /// Tale of Two Wastelands.
    pub ttw: TtwState,
    /// Mods in the profile.
    pub mods_total: usize,
    /// Mods switched on.
    pub mods_on: usize,
    /// Mods that need attention.
    pub mods_attention: Vec<ModAttention>,
    /// Plugins whose master is missing.
    pub missing_masters: Vec<MissingMaster>,
    /// Plugins out of order.
    pub out_of_order: usize,
    /// Active plugin files.
    pub plugin_count: usize,
    /// Distant land.
    pub lod: LodState,
    /// Whether this exact setup has been stability-tested (or there is nothing to test).
    pub stable: bool,
    /// Whether a newer Toolbox is available.
    pub update_available: bool,
}

impl Default for SetupState {
    /// A healthy setup: nothing to report.
    fn default() -> Self {
        Self {
            game: GameState::Found,
            storage_free_bytes: None,
            engine: EngineState::Ok,
            ttw: TtwState::NotUsed,
            mods_total: 0,
            mods_on: 0,
            mods_attention: Vec::new(),
            missing_masters: Vec::new(),
            out_of_order: 0,
            plugin_count: 0,
            lod: LodState::UpToDate,
            stable: true,
            update_available: false,
        }
    }
}

impl SetupState {
    /// Fill the game fact from what detection found; `None` is no game.
    #[must_use]
    pub fn with_game_copy(mut self, copy: Option<&GameCopy>) -> Self {
        self.game = match copy {
            None => GameState::NotFound,
            Some(c) if c.supported_version == Some(false) => GameState::UnsupportedVersion {
                version: c.version.clone(),
            },
            Some(_) => GameState::Found,
        };
        self
    }

    /// Fill the storage fact from [`crate::space::free_space`]; an error leaves it unknown.
    #[must_use]
    pub fn with_space<E>(mut self, space: Result<&Space, E>) -> Self {
        self.storage_free_bytes = space.ok().map(|s| s.free_bytes);
        self
    }

    /// Fill the mod, missing-master and out-of-order facts from the Doctor's typed report. Reads each finding's stable
    /// `code` and `subject` (a mod or a plugin name), never its message text, and replaces those three facts.
    ///
    /// * `profile.mod-folder-missing`, `meta.modid-mismatch`: the subject mod needs attention.
    /// * `loadorder.missing-master`: the subject plugin has a missing master (the master's name is only in the
    ///   message, so it stays unknown).
    /// * `loadorder.differs-from-loot`, `loadorder.times-out-of-order`: the order is off; the report says "yes", not
    ///   how many, so each counts 1. [`Self::with_load_order_check`] gives the exact number.
    ///
    /// The report has no mod totals and its plugins are NVSE DLLs, so those facts stay as they were.
    #[must_use]
    pub fn with_doctor_report(mut self, report: &nvmod_api::doctor::Report) -> Self {
        use nvmod_api::doctor::Report;
        let findings = match report {
            Report::Base(r) | Report::Instance(r) => &r.findings,
            Report::Mo2(r) => &r.findings,
        };
        self.mods_attention.clear();
        self.missing_masters.clear();
        self.out_of_order = 0;
        for f in findings {
            match f.code {
                "profile.mod-folder-missing" => self.mods_attention.push(ModAttention {
                    name: f.subject.clone(),
                    reason: ModReason::Other("its folder is missing".into()),
                }),
                "meta.modid-mismatch" => self.mods_attention.push(ModAttention {
                    name: f.subject.clone(),
                    reason: ModReason::Other("its Nexus id does not match".into()),
                }),
                "loadorder.missing-master" => self.missing_masters.push(MissingMaster {
                    plugin: f.subject.clone(),
                    master: None,
                }),
                "loadorder.differs-from-loot" | "loadorder.times-out-of-order" => {
                    self.out_of_order += 1;
                }
                _ => {}
            }
        }
        self
    }

    /// Fill the plugin count and the out-of-order count from a sort proposal: `current` is the proposal's current
    /// order and `positions_changed` its `Proposal::positions_changed()`. Active plugins are counted.
    #[must_use]
    pub fn with_load_order_check(
        mut self,
        current: &[nvmod_stage::Slot],
        positions_changed: usize,
    ) -> Self {
        self.plugin_count = current.iter().filter(|s| s.active).count();
        self.out_of_order = positions_changed;
        self
    }

    /// Fill the LOD fact from [`raul_drivers::lod::stale::classify`]. Nothing recorded (or not tellable) counts as
    /// not built; a plugin or settings change both map to `StaleMods`, because the staleness carries its reason as
    /// text only, so a quality-only change cannot be told apart here (`StaleQuality` has no typed source).
    #[must_use]
    pub fn with_lod_staleness(mut self, staleness: &raul_drivers::lod::stale::Staleness) -> Self {
        use raul_drivers::lod::stale::State;
        self.lod = match staleness.state {
            State::Unknown => LodState::NotBuilt,
            State::Current => LodState::UpToDate,
            State::May | State::Must => LodState::StaleMods,
        };
        self
    }

    /// Fill the engine fact from the installed nv-rs against its install record. A program that is not the file
    /// the record names is one changed engine file.
    #[must_use]
    pub fn with_engine_record(mut self, nv_rs: &nvmod_api::verify::NvRs) -> Self {
        self.engine = if nv_rs.installed && !nv_rs.verified {
            EngineState::Damaged { files_changed: 1 }
        } else {
            EngineState::Ok
        };
        self
    }

    /// Fill the TTW fact: whether the profile uses it (see [`crate::game_ini::lists_ttw`]) and whether the master is
    /// in the staged `Data`.
    #[must_use]
    pub fn with_ttw(mut self, uses: bool, added: bool) -> Self {
        self.ttw = match (uses, added) {
            (false, _) => TtwState::NotUsed,
            (true, false) => TtwState::NotAdded,
            (true, true) => TtwState::Added,
        };
        self
    }
}

/// How a check stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Level {
    /// Fine (or nothing to say).
    Ok,
    /// Worth a look; Play lets the person choose.
    Warn,
    /// Blocks Play.
    Fail,
}

/// The sidebar item or page that owns a check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Area {
    /// Mods.
    Mods,
    /// Load order.
    LoadOrder,
    /// LOD.
    Lod,
    /// System.
    System,
    /// Settings.
    Settings,
    /// Doctor (badge only).
    Doctor,
}

/// Which check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CheckId {
    /// The game folder.
    Game,
    /// Toolbox storage.
    Storage,
    /// The engine.
    Engine,
    /// Tale of Two Wastelands.
    Ttw,
    /// Mods that need attention.
    Mods,
    /// Plugins whose master is missing (one shared check, F-MODS-11).
    MissingMasters,
    /// Plugins out of order.
    LoadOrder,
    /// Plugin count against the limit.
    PluginCount,
    /// Distant land.
    Lod,
    /// The untested-change flag.
    ModSetup,
    /// A Toolbox update.
    Update,
}

/// One check and how it stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Check {
    /// Which check.
    pub id: CheckId,
    /// The page that owns and fixes it.
    pub area: Area,
    /// How it stands.
    pub level: Level,
    /// How many things are wrong (0 when `Ok`; the update check is 1 when available).
    pub count: usize,
    /// Short plain text: why it is not fine, or its status when it is.
    pub reason: String,
}

fn chk(id: CheckId, area: Area, level: Level, count: usize, reason: impl Into<String>) -> Check {
    Check {
        id,
        area,
        level,
        count,
        reason: reason.into(),
    }
}

fn plural(n: usize, one: &str) -> String {
    format!("{n} {one}{}", if n == 1 { "" } else { "s" })
}

/// Every check of `state`, in the order the Doctor shows them. Pure.
#[must_use]
#[allow(clippy::too_many_lines)] // one flat list of the checks, one arm per state
pub fn checks(state: &SetupState) -> Vec<Check> {
    use CheckId as Id;
    let mut out = Vec::with_capacity(11);

    out.push(match &state.game {
        GameState::Found => chk(Id::Game, Area::Settings, Level::Ok, 0, "Found"),
        GameState::UnsupportedVersion { version } => chk(
            Id::Game,
            Area::Settings,
            Level::Warn,
            1,
            version.as_ref().map_or_else(
                || "Unsupported game version".to_string(),
                |v| format!("Unsupported game version {v}"),
            ),
        ),
        GameState::NotFound => chk(
            Id::Game,
            Area::Settings,
            Level::Fail,
            1,
            "Game folder not found",
        ),
    });

    out.push(match state.storage_free_bytes {
        Some(b) if b < LOW_STORAGE_BYTES => {
            chk(Id::Storage, Area::Settings, Level::Warn, 1, "Low on space")
        }
        _ => chk(Id::Storage, Area::Settings, Level::Ok, 0, "OK"),
    });

    out.push(match state.engine {
        EngineState::Ok => chk(Id::Engine, Area::System, Level::Ok, 0, "OK"),
        EngineState::Damaged { files_changed } => chk(
            Id::Engine,
            Area::System,
            Level::Fail,
            1,
            format!("{} changed", plural(files_changed, "engine file")),
        ),
    });

    out.push(match state.ttw {
        TtwState::NotUsed => chk(Id::Ttw, Area::Mods, Level::Ok, 0, "Not used"),
        TtwState::NotAdded => chk(Id::Ttw, Area::Mods, Level::Warn, 1, "This profile uses TTW"),
        TtwState::Added => chk(Id::Ttw, Area::Mods, Level::Ok, 0, "Added"),
    });

    let attention = state.mods_attention.len();
    let missing_master_mod = state
        .mods_attention
        .iter()
        .any(|m| matches!(m.reason, ModReason::MissingMaster { .. }));
    out.push(if attention == 0 {
        chk(
            Id::Mods,
            Area::Mods,
            Level::Ok,
            0,
            format!("{} of {} on", state.mods_on, state.mods_total),
        )
    } else {
        let first = &state.mods_attention[0];
        let why = match &first.reason {
            ModReason::MissingMaster { master } => format!("{} needs {master}", first.name),
            ModReason::Other(r) => format!("{}: {r}", first.name),
        };
        let level = if missing_master_mod {
            Level::Fail
        } else {
            Level::Warn
        };
        chk(Id::Mods, Area::Mods, level, attention, why)
    });

    out.push(match state.missing_masters.first() {
        Some(m) => chk(
            Id::MissingMasters,
            Area::LoadOrder,
            Level::Fail,
            state.missing_masters.len(),
            m.master.as_ref().map_or_else(
                || format!("{} has a missing master", m.plugin),
                |master| format!("{} needs {master}", m.plugin),
            ),
        ),
        None => chk(Id::MissingMasters, Area::LoadOrder, Level::Ok, 0, "OK"),
    });

    out.push(if state.out_of_order > 0 {
        chk(
            Id::LoadOrder,
            Area::LoadOrder,
            Level::Warn,
            state.out_of_order,
            "Not sorted",
        )
    } else {
        chk(Id::LoadOrder, Area::LoadOrder, Level::Ok, 0, "OK")
    });

    let n = state.plugin_count;
    out.push(if n > PLUGIN_WARN_ABOVE {
        chk(
            Id::PluginCount,
            Area::LoadOrder,
            Level::Warn,
            1,
            format!("{n} of {PLUGIN_LIMIT}"),
        )
    } else {
        chk(
            Id::PluginCount,
            Area::LoadOrder,
            Level::Ok,
            0,
            format!("{n} of {PLUGIN_LIMIT}"),
        )
    });

    out.push(match state.lod {
        LodState::UpToDate => chk(Id::Lod, Area::Lod, Level::Ok, 0, "Up to date"),
        LodState::Building => chk(Id::Lod, Area::Lod, Level::Ok, 0, "Building"),
        LodState::NotBuilt => chk(Id::Lod, Area::Lod, Level::Warn, 1, "Far away looks flat"),
        LodState::StaleQuality => chk(Id::Lod, Area::Lod, Level::Warn, 1, "Quality changed"),
        LodState::StaleMods => chk(
            Id::Lod,
            Area::Lod,
            Level::Warn,
            1,
            "Mods changed since the build",
        ),
    });

    out.push(if state.stable {
        chk(Id::ModSetup, Area::LoadOrder, Level::Ok, 0, "Tested stable")
    } else {
        chk(
            Id::ModSetup,
            Area::LoadOrder,
            Level::Warn,
            1,
            "This exact setup has no stability test",
        )
    });

    out.push(if state.update_available {
        chk(Id::Update, Area::System, Level::Ok, 1, "Update available")
    } else {
        chk(Id::Update, Area::System, Level::Ok, 0, "Up to date")
    });

    out
}

/// A sidebar badge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Badge {
    /// The sidebar item.
    pub area: Area,
    /// The number shown; 0 for the icon-only System badge.
    pub count: usize,
    /// Red for `Fail`, orange for `Warn`; `Ok` is the neutral (blue) update badge.
    pub level: Level,
    /// The tooltip: every reason, joined.
    pub tooltip: String,
}

/// What each check says in a tooltip (counts, not file names).
fn tip(c: &Check) -> String {
    match c.id {
        CheckId::MissingMasters => plural(c.count, "missing master"),
        CheckId::LoadOrder => format!("{} out of order", c.count),
        CheckId::ModSetup => "untested change".to_string(),
        CheckId::Lod => {
            if c.reason == "Far away looks flat" {
                "LOD not built".to_string()
            } else {
                "LOD out of date".to_string()
            }
        }
        _ => String::new(),
    }
}

/// The sidebar badges of `checks` (F-SHELL-05), in sidebar order. An area without a reason shows no badge.
///
/// * Mods: mods needing attention plus TTW not added; red with a missing master, else orange.
/// * Load order: missing masters plus plugins out of order plus the untested flag; red with a missing master, else
///   orange. The plugin count warning is not counted (it is a Doctor item).
/// * LOD: 1 when not built or out of date; orange.
/// * System: icon-only "Update available".
/// * Doctor: failures, else warnings, over all checks; only when `doctor_ran`.
#[must_use]
pub fn badges(checks: &[Check], doctor_ran: bool) -> Vec<Badge> {
    let mut out = Vec::new();
    let pick = |ids: &[CheckId]| -> Vec<&Check> {
        checks
            .iter()
            .filter(|c| ids.contains(&c.id) && c.level != Level::Ok)
            .collect()
    };

    let mods = pick(&[CheckId::Mods, CheckId::Ttw]);
    if !mods.is_empty() {
        let attention = mods
            .iter()
            .find(|c| c.id == CheckId::Mods)
            .map_or(0, |c| c.count);
        let ttw = mods.iter().any(|c| c.id == CheckId::Ttw);
        let count = attention + usize::from(ttw);
        let mut tip = format!(
            "{} {} attention",
            plural(count, "mod"),
            if count == 1 { "needs" } else { "need" }
        );
        if ttw {
            tip.push_str(", TTW not added");
        }
        out.push(badge(Area::Mods, count, &mods, tip));
    }

    let load = pick(&[
        CheckId::MissingMasters,
        CheckId::LoadOrder,
        CheckId::ModSetup,
    ]);
    if !load.is_empty() {
        let count = load.iter().map(|c| c.count).sum();
        let tooltip = load.iter().map(|c| tip(c)).collect::<Vec<_>>().join(", ");
        out.push(badge(Area::LoadOrder, count, &load, tooltip));
    }

    let far = pick(&[CheckId::Lod]);
    if let Some(c) = far.first() {
        out.push(badge(Area::Lod, 1, &far, tip(c)));
    }

    if checks
        .iter()
        .any(|c| c.id == CheckId::Update && c.count > 0)
    {
        out.push(Badge {
            area: Area::System,
            count: 0,
            level: Level::Ok,
            tooltip: "Update available".into(),
        });
    }

    if doctor_ran {
        let fails = checks.iter().filter(|c| c.level == Level::Fail).count();
        let warns = checks.iter().filter(|c| c.level == Level::Warn).count();
        if fails > 0 {
            out.push(Badge {
                area: Area::Doctor,
                count: fails,
                level: Level::Fail,
                tooltip: format!("{fails} failed"),
            });
        } else if warns > 0 {
            out.push(Badge {
                area: Area::Doctor,
                count: warns,
                level: Level::Warn,
                tooltip: plural(warns, "warning"),
            });
        }
    }
    out
}

fn badge(area: Area, count: usize, from: &[&Check], tooltip: String) -> Badge {
    Badge {
        area,
        count,
        level: from.iter().map(|c| c.level).max().unwrap_or(Level::Ok),
        tooltip,
    }
}

/// What Play does after its checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayVerdict {
    /// Nothing to say: start the game.
    Go,
    /// Warnings only: "Before you play", the person chooses (F-PLAY-02).
    Warnings(Vec<Check>),
    /// At least one failure: "Play stopped", no Play anyway (F-PLAY-03). Holds the failures.
    Blocked(Vec<Check>),
}

/// The Play verdict of `checks`: failures block, warnings let the person choose.
#[must_use]
pub fn play_verdict(checks: &[Check]) -> PlayVerdict {
    let of = |level| -> Vec<Check> {
        checks
            .iter()
            .filter(|c| c.level == level)
            .cloned()
            .collect()
    };
    let fails = of(Level::Fail);
    if !fails.is_empty() {
        return PlayVerdict::Blocked(fails);
    }
    let warns = of(Level::Warn);
    if warns.is_empty() {
        PlayVerdict::Go
    } else {
        PlayVerdict::Warnings(warns)
    }
}

/// The Doctor's groups (F-DOCTOR-02), each with its checks.
#[must_use]
pub fn doctor_groups(checks: &[Check]) -> Vec<(&'static str, Vec<&Check>)> {
    use CheckId as Id;
    let group = |name, ids: &[Id]| {
        (
            name,
            checks.iter().filter(|c| ids.contains(&c.id)).collect(),
        )
    };
    vec![
        group("Game folder and files", &[Id::Game, Id::Storage]),
        group("Engine and Toolbox", &[Id::Engine, Id::Update]),
        group(
            "Masters, load order and plugins",
            &[Id::Mods, Id::MissingMasters, Id::LoadOrder, Id::PluginCount],
        ),
        group("LOD, mod setup and TTW", &[Id::Lod, Id::ModSetup, Id::Ttw]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get(l: &[Check], id: CheckId) -> &Check {
        l.iter().find(|c| c.id == id).unwrap()
    }
    fn with(f: impl FnOnce(&mut SetupState)) -> Vec<Check> {
        let mut s = SetupState::default();
        f(&mut s);
        checks(&s)
    }
    fn mm(n: usize) -> Vec<MissingMaster> {
        (0..n)
            .map(|i| MissingMaster {
                plugin: format!("P{i}.esp"),
                master: Some("M.esm".into()),
            })
            .collect()
    }
    fn badge_of(l: &[Check], a: Area) -> Option<Badge> {
        badges(l, false).into_iter().find(|b| b.area == a)
    }

    #[test]
    fn healthy_state_is_all_ok_and_go() {
        let l = checks(&SetupState::default());
        assert!(l.iter().all(|c| c.level == Level::Ok && c.count == 0));
        assert_eq!(badges(&l, true), Vec::<Badge>::new());
        assert_eq!(play_verdict(&l), PlayVerdict::Go);
    }

    #[test]
    fn game_cases() {
        let l = with(|s| s.game = GameState::NotFound);
        assert_eq!(get(&l, CheckId::Game).level, Level::Fail);
        assert_eq!(get(&l, CheckId::Game).area, Area::Settings);
        let l = with(|s| {
            s.game = GameState::UnsupportedVersion {
                version: Some("1.0".into()),
            }
        });
        assert_eq!(get(&l, CheckId::Game).level, Level::Warn);
        assert!(get(&l, CheckId::Game).reason.contains("1.0"));
        assert_eq!(
            get(&checks(&SetupState::default()), CheckId::Game).level,
            Level::Ok
        );
    }

    #[test]
    fn storage_cases() {
        let low = with(|s| s.storage_free_bytes = Some(LOW_STORAGE_BYTES - 1));
        assert_eq!(get(&low, CheckId::Storage).level, Level::Warn);
        let ok = with(|s| s.storage_free_bytes = Some(LOW_STORAGE_BYTES));
        assert_eq!(get(&ok, CheckId::Storage).level, Level::Ok);
        let unknown = with(|s| s.storage_free_bytes = None);
        assert_eq!(get(&unknown, CheckId::Storage).level, Level::Ok);
    }

    #[test]
    fn engine_cases() {
        let l = with(|s| s.engine = EngineState::Damaged { files_changed: 3 });
        let c = get(&l, CheckId::Engine);
        assert_eq!((c.level, c.area), (Level::Fail, Area::System));
        assert_eq!(c.reason, "3 engine files changed");
        assert_eq!(
            get(&checks(&SetupState::default()), CheckId::Engine).level,
            Level::Ok
        );
    }

    #[test]
    fn ttw_cases() {
        assert_eq!(
            get(&with(|s| s.ttw = TtwState::NotUsed), CheckId::Ttw).level,
            Level::Ok
        );
        assert_eq!(
            get(&with(|s| s.ttw = TtwState::Added), CheckId::Ttw).level,
            Level::Ok
        );
        let l = with(|s| s.ttw = TtwState::NotAdded);
        assert_eq!(get(&l, CheckId::Ttw).level, Level::Warn);
        let b = badge_of(&l, Area::Mods).unwrap();
        assert_eq!((b.count, b.level), (1, Level::Warn));
    }

    #[test]
    fn mods_cases() {
        let l = with(|s| {
            s.mods_total = 5;
            s.mods_on = 3;
        });
        assert_eq!(get(&l, CheckId::Mods).reason, "3 of 5 on");
        let other = |n: &str| ModAttention {
            name: n.into(),
            reason: ModReason::Other("old".into()),
        };
        let l = with(|s| s.mods_attention = vec![other("A"), other("B")]);
        assert_eq!(get(&l, CheckId::Mods).level, Level::Warn);
        let b = badge_of(&l, Area::Mods).unwrap();
        assert_eq!((b.count, b.level), (2, Level::Warn));
        assert_eq!(b.tooltip, "2 mods need attention");
        let l = with(|s| {
            s.mods_attention = vec![ModAttention {
                name: "A".into(),
                reason: ModReason::MissingMaster {
                    master: "M.esm".into(),
                },
            }];
            s.ttw = TtwState::NotAdded;
        });
        assert_eq!(get(&l, CheckId::Mods).level, Level::Fail);
        assert_eq!(get(&l, CheckId::Mods).reason, "A needs M.esm");
        let b = badge_of(&l, Area::Mods).unwrap();
        assert_eq!((b.count, b.level), (2, Level::Fail));
    }

    #[test]
    fn load_order_cases() {
        let l = with(|s| s.missing_masters = mm(2));
        let c = get(&l, CheckId::MissingMasters);
        assert_eq!((c.level, c.count), (Level::Fail, 2));
        assert_eq!(c.reason, "P0.esp needs M.esm");
        let l = with(|s| s.out_of_order = 4);
        assert_eq!(get(&l, CheckId::LoadOrder).level, Level::Warn);
        assert_eq!(get(&l, CheckId::LoadOrder).count, 4);
        let l = with(|s| s.stable = false);
        assert_eq!(get(&l, CheckId::ModSetup).level, Level::Warn);
    }

    #[test]
    fn plugin_count_cases() {
        let at = with(|s| s.plugin_count = 230);
        assert_eq!(get(&at, CheckId::PluginCount).level, Level::Ok);
        let over = with(|s| s.plugin_count = 231);
        let c = get(&over, CheckId::PluginCount);
        assert_eq!((c.level, c.reason.as_str()), (Level::Warn, "231 of 255"));
        // not part of the Load order badge
        assert!(badge_of(&over, Area::LoadOrder).is_none());
    }

    #[test]
    fn lod_cases() {
        for (st, level, tooltip) in [
            (LodState::UpToDate, Level::Ok, ""),
            (LodState::Building, Level::Ok, ""),
            (LodState::NotBuilt, Level::Warn, "LOD not built"),
            (LodState::StaleQuality, Level::Warn, "LOD out of date"),
            (LodState::StaleMods, Level::Warn, "LOD out of date"),
        ] {
            let l = with(|s| s.lod = st);
            assert_eq!(get(&l, CheckId::Lod).level, level);
            match badge_of(&l, Area::Lod) {
                Some(b) => assert_eq!(
                    (b.count, b.level, b.tooltip.as_str()),
                    (1, Level::Warn, tooltip)
                ),
                None => assert_eq!(level, Level::Ok),
            }
        }
    }

    #[test]
    fn update_is_an_icon_only_badge() {
        let l = with(|s| s.update_available = true);
        let b = badge_of(&l, Area::System).unwrap();
        assert_eq!((b.count, b.level), (0, Level::Ok));
        assert_eq!(play_verdict(&l), PlayVerdict::Go);
        assert!(badge_of(&checks(&SetupState::default()), Area::System).is_none());
    }

    #[test]
    fn load_order_badge_is_missing_plus_out_of_order_plus_untested() {
        let l = with(|s| {
            s.missing_masters = mm(2);
            s.out_of_order = 4;
            s.stable = false;
        });
        let b = badge_of(&l, Area::LoadOrder).unwrap();
        assert_eq!(b.count, 7);
        assert_eq!(b.level, Level::Fail);
        assert_eq!(
            b.tooltip,
            "2 missing masters, 4 out of order, untested change"
        );
    }

    #[test]
    fn load_order_badge_is_orange_without_missing_master() {
        let l = with(|s| {
            s.out_of_order = 4;
            s.stable = false;
        });
        let b = badge_of(&l, Area::LoadOrder).unwrap();
        assert_eq!((b.count, b.level), (5, Level::Warn));
        assert_eq!(b.tooltip, "4 out of order, untested change");
    }

    #[test]
    fn badges_clear_when_the_cause_is_fixed() {
        let broken = SetupState {
            out_of_order: 2,
            stable: false,
            lod: LodState::NotBuilt,
            ..SetupState::default()
        };
        assert_eq!(badges(&checks(&broken), false).len(), 2);
        assert_eq!(
            badges(&checks(&SetupState::default()), false),
            Vec::<Badge>::new()
        );
    }

    #[test]
    fn doctor_badge_only_after_a_run_failures_before_warnings() {
        let warn = with(|s| s.lod = LodState::NotBuilt);
        assert!(badges(&warn, false).iter().all(|b| b.area != Area::Doctor));
        let b = badges(&warn, true)
            .into_iter()
            .find(|b| b.area == Area::Doctor)
            .unwrap();
        assert_eq!(
            (b.count, b.level, b.tooltip.as_str()),
            (1, Level::Warn, "1 warning")
        );
        let fail = with(|s| {
            s.lod = LodState::NotBuilt;
            s.engine = EngineState::Damaged { files_changed: 1 };
            s.game = GameState::NotFound;
        });
        let b = badges(&fail, true)
            .into_iter()
            .find(|b| b.area == Area::Doctor)
            .unwrap();
        assert_eq!(
            (b.count, b.level, b.tooltip.as_str()),
            (2, Level::Fail, "2 failed")
        );
    }

    #[test]
    fn play_verdicts() {
        let warn = with(|s| {
            s.lod = LodState::StaleMods;
            s.stable = false;
        });
        match play_verdict(&warn) {
            PlayVerdict::Warnings(w) => {
                assert_eq!(
                    w.iter().map(|c| c.id).collect::<Vec<_>>(),
                    [CheckId::Lod, CheckId::ModSetup]
                );
            }
            v => panic!("{v:?}"),
        }
        let fail = with(|s| {
            s.lod = LodState::StaleMods;
            s.engine = EngineState::Damaged { files_changed: 3 };
        });
        match play_verdict(&fail) {
            PlayVerdict::Blocked(f) => {
                assert_eq!(f.len(), 1);
                assert_eq!(f[0].id, CheckId::Engine);
            }
            v => panic!("{v:?}"),
        }
    }

    #[test]
    fn doctor_groups_cover_every_check_once() {
        let l = checks(&SetupState::default());
        let groups = doctor_groups(&l);
        assert_eq!(groups.len(), 4);
        assert_eq!(groups.iter().map(|(_, g)| g.len()).sum::<usize>(), l.len());
    }

    #[test]
    fn builds_from_detection_and_space() {
        let copy = GameCopy {
            game: crate::detect::Game::NewVegas,
            path: "/g".into(),
            storefront: "Steam".into(),
            version: Some("1.3".into()),
            supported_version: Some(false),
            missing_dlc: vec![],
        };
        let s = SetupState::default().with_game_copy(Some(&copy));
        assert!(matches!(s.game, GameState::UnsupportedVersion { .. }));
        assert_eq!(
            SetupState::default().with_game_copy(None).game,
            GameState::NotFound
        );
        let sp = Space {
            folder: "/s".into(),
            free_bytes: 5,
            total_bytes: 10,
        };
        assert_eq!(
            SetupState::default()
                .with_space::<()>(Ok(&sp))
                .storage_free_bytes,
            Some(5)
        );
        assert_eq!(
            SetupState::default().with_space(Err(())).storage_free_bytes,
            None
        );
    }

    mod mappings {
        use super::*;
        use nvmod_api::doctor::Report;
        use nvmod_core::{Finding, Severity, Storefront};
        use nvmod_doctor::{BaseReport, BaseSummary};
        use raul_drivers::lod::stale::{Staleness, State};

        fn base(findings: Vec<Finding>) -> BaseReport {
            BaseReport {
                schema_version: 1,
                base: "base".into(),
                storefront: Storefront::Steam,
                exe: None,
                target: None,
                nv_rs: None,
                findings,
                summary: BaseSummary::default(),
            }
        }
        fn f(code: &'static str, subject: &str) -> Finding {
            Finding::new(code, Severity::Warning, subject, "words")
        }

        #[test]
        fn doctor_findings_fill_mods_masters_and_order() {
            let r = Report::Instance(base(vec![
                f("profile.mod-folder-missing", "Gone"),
                f("meta.modid-mismatch", "Wrong"),
                f("loadorder.missing-master", "A.esp"),
                f("loadorder.missing-master", "B.esp"),
                f("loadorder.times-out-of-order", "p"),
                f("pe.damaged", "x.dll"),
            ]));
            let s = SetupState {
                out_of_order: 9,
                ..SetupState::default()
            }
            .with_doctor_report(&r);
            let names: Vec<_> = s.mods_attention.iter().map(|m| m.name.as_str()).collect();
            assert_eq!(names, ["Gone", "Wrong"]);
            assert_eq!(s.missing_masters.len(), 2);
            assert_eq!(s.missing_masters[0].plugin, "A.esp");
            assert_eq!(s.missing_masters[0].master, None);
            assert_eq!(s.out_of_order, 1);
            let l = checks(&s);
            assert_eq!(
                get(&l, CheckId::MissingMasters).reason,
                "A.esp has a missing master"
            );
        }

        #[test]
        fn a_clean_report_clears_the_facts() {
            let s = SetupState {
                missing_masters: mm(1),
                out_of_order: 3,
                ..SetupState::default()
            }
            .with_doctor_report(&Report::Base(base(vec![])));
            assert!(s.mods_attention.is_empty() && s.missing_masters.is_empty());
            assert_eq!(s.out_of_order, 0);
        }

        #[test]
        fn load_order_check_counts_active_plugins_and_changed_positions() {
            let slot = |n: &str, active| nvmod_stage::Slot {
                name: n.into(),
                active,
            };
            let s = SetupState::default().with_load_order_check(
                &[
                    slot("a.esm", true),
                    slot("b.esp", false),
                    slot("c.esp", true),
                ],
                2,
            );
            assert_eq!((s.plugin_count, s.out_of_order), (2, 2));
        }

        #[test]
        fn lod_staleness_maps_each_state() {
            let of = |state| {
                SetupState::default()
                    .with_lod_staleness(&Staleness {
                        state,
                        reason: String::new(),
                    })
                    .lod
            };
            assert_eq!(of(State::Unknown), LodState::NotBuilt);
            assert_eq!(of(State::Current), LodState::UpToDate);
            assert_eq!(of(State::May), LodState::StaleMods);
            assert_eq!(of(State::Must), LodState::StaleMods);
        }

        #[test]
        fn engine_record_and_ttw() {
            let nv = |installed, verified| nvmod_api::verify::NvRs {
                installed,
                verified,
                message: String::new(),
            };
            let e = |n| SetupState::default().with_engine_record(&n).engine;
            assert_eq!(
                e(nv(true, false)),
                EngineState::Damaged { files_changed: 1 }
            );
            assert_eq!(e(nv(true, true)), EngineState::Ok);
            assert_eq!(e(nv(false, false)), EngineState::Ok);
            let t = |u, a| SetupState::default().with_ttw(u, a).ttw;
            assert_eq!(t(false, true), TtwState::NotUsed);
            assert_eq!(t(true, false), TtwState::NotAdded);
            assert_eq!(t(true, true), TtwState::Added);
        }
    }
}
